pub mod device;
pub mod ffi;

use std::collections::HashSet;
use std::ffi::c_void;
use std::ptr;
use std::sync::{Mutex, OnceLock};

use core_foundation::base::TCFType;
use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    CallbackResult, EventField,
};

pub use device::*;
use ffi::*;
use crate::mapping::hid_usage_to_mac_keycode;

type KeyHandler = Box<dyn Fn(u32, bool) + Send + Sync + 'static>;

static GLOBAL_KEY_HANDLER: OnceLock<KeyHandler> = OnceLock::new();
static TARGET_DEVICE_PTR: OnceLock<usize> = OnceLock::new();
static SWALLOW_KEYCODES: OnceLock<Mutex<HashSet<i64>>> = OnceLock::new();

pub struct HidSystem {
    mgr: IOHIDManagerRef,
}

impl HidSystem {
    pub fn new() -> Result<Self, &'static str> {
        let mgr = unsafe { IOHIDManagerCreate(ptr::null(), K_IOHID_OPTIONS_TYPE_NONE) };
        if mgr.is_null() {
            return Err("Failed to create IOHIDManager");
        }
        Ok(Self { mgr })
    }

    pub fn enumerate_keyboards(&self) -> Vec<KeyboardDevice> {
        device::enumerate_keyboards(self.mgr)
    }

    pub fn start_listener<F>(
        &self,
        target_device_ptr: usize,
        is_external: bool,
        is_hardware_seized: bool,
        on_key: F,
    ) where
        F: Fn(u32, bool) + Send + Sync + 'static,
    {
        TARGET_DEVICE_PTR.set(target_device_ptr).ok();
        GLOBAL_KEY_HANDLER.set(Box::new(on_key)).ok();
        SWALLOW_KEYCODES.set(Mutex::new(HashSet::new())).ok();

        unsafe {
            IOHIDManagerRegisterInputValueCallback(
                self.mgr,
                Some(hid_input_callback),
                ptr::null_mut(),
            );

            let run_loop = CFRunLoop::get_current();
            IOHIDManagerScheduleWithRunLoop(
                self.mgr,
                run_loop.as_concrete_TypeRef() as *mut c_void,
                kCFRunLoopDefaultMode,
            );
            IOHIDManagerOpen(self.mgr, K_IOHID_OPTIONS_TYPE_NONE);

            let _tap = if !is_hardware_seized && target_device_ptr != 0 && is_external {
                let tap = CGEventTap::new(
                    CGEventTapLocation::HID,
                    CGEventTapPlacement::HeadInsertEventTap,
                    CGEventTapOptions::Default,
                    vec![CGEventType::KeyDown, CGEventType::KeyUp],
                    |_proxy, event_type, event| {
                        let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                        if let Some(swallow) = SWALLOW_KEYCODES.get() {
                            if let Ok(mut set) = swallow.lock() {
                                if matches!(event_type, CGEventType::KeyUp) {
                                    if set.remove(&keycode) {
                                        return CallbackResult::Drop;
                                    }
                                } else if set.contains(&keycode) {
                                    return CallbackResult::Drop;
                                }
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
            } else {
                None
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

    if let Some(&target) = TARGET_DEVICE_PTR.get() {
        if target != 0 && (sender as usize) != target {
            return;
        }
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

            if let Some(mac_code) = hid_usage_to_mac_keycode(usage) {
                if let Some(swallow) = SWALLOW_KEYCODES.get() {
                    if let Ok(mut set) = swallow.lock() {
                        if is_down {
                            set.insert(mac_code);
                        }
                    }
                }
            }

            if let Some(handler) = GLOBAL_KEY_HANDLER.get() {
                if is_down || is_up {
                    handler(usage, is_down);
                }
            }
        }
    }
}
