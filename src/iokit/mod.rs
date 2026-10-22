//! macOS IOKit HID & Keystroke Suppression Subsystem.
//!
//! # System Architecture: Dual-Subsystem Pipeline
//!
//! Connecting a regular computer keyboard to serve as a MIDI controller on macOS introduces
//! two fundamental technical hurdles:
//!
//! 1. **Hardware Attribution (Per-Device Routing):**
//!    Standard macOS APIs (`NSEvent`, `CGEventTap`) aggregate all connected keyboards into a
//!    single unified input stream. An application cannot determine which physical keyboard
//!    generated an incoming event.
//!    Solved via `IOHIDManager`, which provides explicit `IOHIDDeviceRef` hardware handles
//!    for each incoming USB/Bluetooth report.
//!
//! 2. **Keystroke Suppression (Event Swallowing):**
//!    `IOHIDManager` is purely observational: it reports raw hardware state changes but cannot
//!    prevent macOS from forwarding keystrokes to the active window (e.g. typing random letters
//!    into terminal or text editors while playing music).
//!    Solved via a Quartz `CGEventTap` placed at `HeadInsertEventTap` in the HID event tap
//!    chain.
//!
//! ```text
//! [ Physical Keyboard ]
//!         │
//!         ├───> [ IOHIDManager (Raw HID Report) ]
//!         │            │
//!         │            ├──> Check Device Filter (Omni / External / Specific)
//!         │            ├──> Trigger MIDI Note Handler
//!         │            └──> If Suppression Enabled: Add Mac Keycode to SWALLOW_KEYCODES
//!         │
//!         └───> [ macOS Window Server / Quartz Event Pipeline ]
//!                      │
//!                      └───> [ CGEventTap @ HeadInsertEventTap ]
//!                                   │
//!                                   ├── If keycode in SWALLOW_KEYCODES: DROP
//!                                   └── Else: KEEP (forward to active app)
//! ```

pub mod device;
pub mod ffi;

use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use core_foundation::base::TCFType;
use core_foundation::runloop::{CFRunLoop, kCFRunLoopDefaultMode};
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    CallbackResult, EventField,
};

use crate::mapping::hid_usage_to_mac_keycode;
pub use device::*;
use ffi::*;

type KeyHandler = Box<dyn Fn(u32, bool) + Send + Sync + 'static>;

/// Device targeting strategy for routing keyboard inputs to MIDI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceTargetMode {
    /// Listen to all keyboards connected to the system.
    Omni,
    /// Ignore built-in MacBook keyboards; only route external USB/Bluetooth keyboards.
    ExternalOnly,
    /// Route only a specific keyboard matched by product name or Vendor/Product IDs.
    Specific {
        name: String,
        vid: Option<u32>,
        pid: Option<u32>,
    },
}

type SenderCache = HashMap<usize, SenderDeviceInfo>;

// Global synchronization state:
// C function pointer callbacks (`IOHIDValueCallback`, `CGEventTapCallback`) cannot
// capture Rust closures or environment contexts. Shared runtime state is coordinated
// through thread-safe global statics.
static GLOBAL_KEY_HANDLER: OnceLock<KeyHandler> = OnceLock::new();
static GLOBAL_MGR: AtomicUsize = AtomicUsize::new(0);
static TARGET_MODE: Mutex<DeviceTargetMode> = Mutex::new(DeviceTargetMode::Omni);
static SENDER_CACHE: Mutex<Option<SenderCache>> = Mutex::new(None);
static SUPPRESSION_ENABLED: AtomicBool = AtomicBool::new(true);
static SWALLOW_KEYCODES: OnceLock<Mutex<HashSet<i64>>> = OnceLock::new();
static RUN_LOOP_REF: AtomicUsize = AtomicUsize::new(0);

/// Updates the keyboard filter mode. Flushes the sender device cache and any
/// currently held suppression keys to prevent stale state.
pub fn set_target_mode(mode: DeviceTargetMode) {
    if let Ok(mut lock) = TARGET_MODE.lock() {
        *lock = mode;
    }
    if let Ok(mut cache) = SENDER_CACHE.lock() {
        *cache = Some(HashMap::new());
    }
    if let Some(swallow) = SWALLOW_KEYCODES.get()
        && let Ok(mut set) = swallow.lock()
    {
        set.clear();
    }
}

/// Enables or disables keystroke suppression via the Quartz event tap.
pub fn set_suppression_enabled(enabled: bool) {
    SUPPRESSION_ENABLED.store(enabled, Ordering::SeqCst);
    if !enabled
        && let Some(swallow) = SWALLOW_KEYCODES.get()
        && let Ok(mut set) = swallow.lock()
    {
        set.clear();
    }
}

/// Queries connected keyboards using the existing global IOHIDManager instance.
pub fn refresh_keyboards() -> Vec<KeyboardDevice> {
    let mgr_ptr = GLOBAL_MGR.load(Ordering::SeqCst);
    if mgr_ptr != 0 {
        enumerate_keyboards(mgr_ptr as IOHIDManagerRef)
    } else {
        Vec::new()
    }
}

/// Signals the background CFRunLoop to terminate.
pub fn stop_hid_system() {
    let rloop = RUN_LOOP_REF.load(Ordering::SeqCst);
    if rloop != 0 {
        unsafe {
            let cf_loop = rloop as core_foundation::runloop::CFRunLoopRef;
            core_foundation::runloop::CFRunLoopStop(cf_loop);
        }
    }
}

/// Manages lifecycle, device routing, and run loop execution for IOKit input listening.
pub struct HidSystem {
    mgr: IOHIDManagerRef,
}

unsafe impl Send for HidSystem {}

impl HidSystem {
    /// Creates and initializes a new IOKit HID manager instance.
    pub fn new() -> Result<Self, &'static str> {
        let mgr = unsafe { IOHIDManagerCreate(ptr::null(), K_IOHID_OPTIONS_TYPE_NONE) };
        if mgr.is_null() {
            return Err("Failed to create IOHIDManager");
        }
        GLOBAL_MGR.store(mgr as usize, Ordering::SeqCst);
        Ok(Self { mgr })
    }

    /// Enumerates currently attached keyboards.
    pub fn enumerate_keyboards(&self) -> Vec<KeyboardDevice> {
        device::enumerate_keyboards(self.mgr)
    }

    /// Spawns the main IOKit event loop and event tap.
    ///
    /// # Operation:
    /// 1. Registers the raw HID input callback with `IOHIDManager`.
    /// 2. Attaches the manager to the current thread's `CFRunLoop`.
    /// 3. Installs a `CGEventTap` at `HeadInsertEventTap` to drop suppressed keys.
    /// 4. Runs `CFRunLoop::run_current()` (blocks until `stop_hid_system` is called).
    /// 5. Cleans up run loop sources and releases CoreFoundation references on exit.
    pub fn start_listener<F>(&self, initial_mode: DeviceTargetMode, on_key: F)
    where
        F: Fn(u32, bool) + Send + Sync + 'static,
    {
        set_target_mode(initial_mode);
        GLOBAL_KEY_HANDLER.set(Box::new(on_key)).ok();
        SWALLOW_KEYCODES.set(Mutex::new(HashSet::new())).ok();

        unsafe {
            IOHIDManagerRegisterInputValueCallback(
                self.mgr,
                Some(hid_input_callback),
                ptr::null_mut(),
            );

            let run_loop = CFRunLoop::get_current();
            RUN_LOOP_REF.store(run_loop.as_concrete_TypeRef() as usize, Ordering::SeqCst);

            IOHIDManagerScheduleWithRunLoop(
                self.mgr,
                run_loop.as_concrete_TypeRef() as *mut c_void,
                kCFRunLoopDefaultMode,
            );
            IOHIDManagerOpen(self.mgr, K_IOHID_OPTIONS_TYPE_NONE);

            let _tap = {
                let tap = CGEventTap::new(
                    CGEventTapLocation::HID,
                    CGEventTapPlacement::HeadInsertEventTap,
                    CGEventTapOptions::Default,
                    vec![CGEventType::KeyDown, CGEventType::KeyUp],
                    |_proxy, event_type, event| {
                        let keycode =
                            event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                        if let Some(swallow) = SWALLOW_KEYCODES.get()
                            && let Ok(mut set) = swallow.lock()
                        {
                            if matches!(event_type, CGEventType::KeyUp) {
                                if set.remove(&keycode) {
                                    return CallbackResult::Drop;
                                }
                            } else if set.contains(&keycode) {
                                return CallbackResult::Drop;
                            }
                        }
                        CallbackResult::Keep
                    },
                );

                if let Ok(tap) = tap {
                    if let Ok(source) = tap.mach_port().create_runloop_source(0) {
                        run_loop.add_source(&source, kCFRunLoopDefaultMode);
                        Some((tap, source))
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            CFRunLoop::run_current();

            let _ = IOHIDManagerClose(self.mgr, K_IOHID_OPTIONS_TYPE_NONE);
            CFRelease(self.mgr.cast());
        }
    }
}

// IOKit input callback:
// Invoked by macOS on run loop thread on matched HID element state change.
//
// Sequence:
// 1. Sender Validation: Reads raw device pointer, inspects or looks up cached
//    device metadata (`SenderDeviceInfo`).
// 2. Target Mode Filter: Validates whether sending device matches criteria
//    (Omni, ExternalOnly, or Specific VID/PID/Name). Unmatched inputs ignored.
// 3. Usage Extraction: Confirms report is HID Usage Page 0x07 (Keyboard/Keypad).
//    Extracts HID key usage ID and integer value (1=down, 0=up).
// 4. Suppression Handoff: Maps HID usage to macOS virtual keycode. On KeyDown,
//    registers keycode in `SWALLOW_KEYCODES` so CGEventTap drops the event.
// 5. MIDI Dispatch: Forwards HID usage and state to `GLOBAL_KEY_HANDLER`.
extern "C" fn hid_input_callback(
    _context: *mut c_void,
    result: IOReturn,
    sender: *mut c_void,
    value: IOHIDValueRef,
) {
    if result != K_IO_RETURN_SUCCESS || sender.is_null() || value.is_null() {
        return;
    }

    let sender_ptr = sender as usize;
    let target_mode = if let Ok(lock) = TARGET_MODE.lock() {
        lock.clone()
    } else {
        DeviceTargetMode::Omni
    };

    let is_match = match &target_mode {
        DeviceTargetMode::Omni => true,
        DeviceTargetMode::ExternalOnly => {
            let is_internal = {
                let mut cache_lock = SENDER_CACHE.lock().unwrap();
                let cache = cache_lock.get_or_insert_with(HashMap::new);
                if let Some(info) = cache.get(&sender_ptr) {
                    info.is_internal
                } else {
                    let info = inspect_device(sender as IOHIDDeviceRef);
                    let res = info.is_internal;
                    cache.insert(sender_ptr, info);
                    res
                }
            };
            !is_internal
        }
        DeviceTargetMode::Specific { name, vid, pid } => {
            let info = {
                let mut cache_lock = SENDER_CACHE.lock().unwrap();
                let cache = cache_lock.get_or_insert_with(HashMap::new);
                if let Some(info) = cache.get(&sender_ptr) {
                    info.clone()
                } else {
                    let info = inspect_device(sender as IOHIDDeviceRef);
                    cache.insert(sender_ptr, info.clone());
                    info
                }
            };

            let vid_match = vid.is_some() && *vid == info.vendor_id && *pid == info.product_id;
            let name_match = !name.is_empty()
                && (info.name.eq_ignore_ascii_case(name)
                    || info.name.to_lowercase().contains(&name.to_lowercase()));
            vid_match || name_match
        }
    };

    if !is_match {
        return;
    }

    unsafe {
        let element = IOHIDValueGetElement(value);
        if element.is_null() {
            return;
        }

        let page = IOHIDElementGetUsagePage(element);
        let usage = IOHIDElementGetUsage(element);
        let val = IOHIDValueGetIntegerValue(value);

        if page == HID_USAGE_PAGE_KEYBOARD_OR_KEYPAD {
            let is_down = val == 1;
            let is_up = val == 0;

            if SUPPRESSION_ENABLED.load(Ordering::SeqCst)
                && let Some(mac_code) = hid_usage_to_mac_keycode(usage)
                && let Some(swallow) = SWALLOW_KEYCODES.get()
                && let Ok(mut set) = swallow.lock()
                && is_down
            {
                set.insert(mac_code);
            }

            if let Some(handler) = GLOBAL_KEY_HANDLER.get()
                && (is_down || is_up)
            {
                handler(usage, is_down);
            }
        }
    }
}
