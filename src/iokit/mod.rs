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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceTargetMode {
    Omni,
    ExternalOnly,
    Specific {
        name: String,
        vid: Option<u32>,
        pid: Option<u32>,
    },
}

type SenderCache = HashMap<usize, SenderDeviceInfo>;

static GLOBAL_KEY_HANDLER: OnceLock<KeyHandler> = OnceLock::new();
static GLOBAL_MGR: AtomicUsize = AtomicUsize::new(0);
static TARGET_MODE: Mutex<DeviceTargetMode> = Mutex::new(DeviceTargetMode::Omni);
static SENDER_CACHE: Mutex<Option<SenderCache>> = Mutex::new(None);
static SUPPRESSION_ENABLED: AtomicBool = AtomicBool::new(true);
static SWALLOW_KEYCODES: OnceLock<Mutex<HashSet<i64>>> = OnceLock::new();
static RUN_LOOP_REF: AtomicUsize = AtomicUsize::new(0);

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

#[allow(dead_code)]
pub fn get_target_mode() -> DeviceTargetMode {
    TARGET_MODE
        .lock()
        .map(|m| m.clone())
        .unwrap_or(DeviceTargetMode::Omni)
}

#[allow(dead_code)]
pub fn set_target_device_ptr(ptr: usize) {
    if ptr == 0 {
        set_target_mode(DeviceTargetMode::Omni);
    }
}

pub fn set_suppression_enabled(enabled: bool) {
    SUPPRESSION_ENABLED.store(enabled, Ordering::SeqCst);
    if !enabled
        && let Some(swallow) = SWALLOW_KEYCODES.get()
        && let Ok(mut set) = swallow.lock()
    {
        set.clear();
    }
}

pub fn refresh_keyboards() -> Vec<KeyboardDevice> {
    let mgr_ptr = GLOBAL_MGR.load(Ordering::SeqCst);
    if mgr_ptr != 0 {
        enumerate_keyboards(mgr_ptr as IOHIDManagerRef)
    } else {
        Vec::new()
    }
}

pub fn stop_hid_system() {
    let rloop = RUN_LOOP_REF.load(Ordering::SeqCst);
    if rloop != 0 {
        unsafe {
            let cf_loop = rloop as core_foundation::runloop::CFRunLoopRef;
            core_foundation::runloop::CFRunLoopStop(cf_loop);
        }
    }
}

pub struct HidSystem {
    mgr: IOHIDManagerRef,
}

unsafe impl Send for HidSystem {}

impl HidSystem {
    pub fn new() -> Result<Self, &'static str> {
        let mgr = unsafe { IOHIDManagerCreate(ptr::null(), K_IOHID_OPTIONS_TYPE_NONE) };
        if mgr.is_null() {
            return Err("Failed to create IOHIDManager");
        }
        GLOBAL_MGR.store(mgr as usize, Ordering::SeqCst);
        Ok(Self { mgr })
    }

    pub fn enumerate_keyboards(&self) -> Vec<KeyboardDevice> {
        device::enumerate_keyboards(self.mgr)
    }

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
