use std::ffi::c_void;
use std::ptr;

use core_foundation::base::TCFType;
use core_foundation::string::CFString;

use super::ffi::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardDevice {
    pub index: usize,
    pub name: String,
    pub manufacturer: String,
    pub vendor_id: Option<u32>,
    pub product_id: Option<u32>,
    pub transport: String,
    pub is_internal: bool,
    pub device_ptr: usize,
}

pub fn enumerate_keyboards(mgr: IOHIDManagerRef) -> Vec<KeyboardDevice> {
    let mut result = Vec::new();
    if mgr.is_null() {
        return result;
    }

    unsafe {
        IOHIDManagerSetDeviceMatching(mgr, ptr::null());
        let device_set = IOHIDManagerCopyDevices(mgr);
        if device_set.is_null() {
            return result;
        }

        let count = CFSetGetCount(device_set);
        if count <= 0 {
            CFRelease(device_set.cast());
            return result;
        }

        let mut devices: Vec<*const c_void> = vec![ptr::null(); count as usize];
        CFSetGetValues(device_set, devices.as_mut_ptr());

        for raw_ptr in devices {
            if raw_ptr.is_null() {
                continue;
            }
            let dev = raw_ptr as IOHIDDeviceRef;

            let usage_page = get_u32_property(dev, "PrimaryUsagePage").unwrap_or(0);
            let usage = get_u32_property(dev, "PrimaryUsage").unwrap_or(0);

            let is_keyboard = (usage_page == HID_USAGE_PAGE_GENERIC_DESKTOP && usage == HID_USAGE_GENERIC_DESKTOP_KEYBOARD)
                || (usage_page == HID_USAGE_PAGE_KEYBOARD_OR_KEYPAD && usage == HID_USAGE_GENERIC_DESKTOP_KEYBOARD);

            if !is_keyboard {
                continue;
            }

            let name = get_string_property(dev, "Product");
            let mfg = get_string_property(dev, "Manufacturer");
            let transport = get_string_property(dev, "Transport");
            let vid = get_u32_property(dev, "VendorID");
            let pid = get_u32_property(dev, "ProductID");

            let display_name = if !name.is_empty() {
                name
            } else if !mfg.is_empty() {
                format!("{mfg} Keyboard")
            } else {
                "Unnamed Keyboard".to_string()
            };

            let is_internal = display_name.to_lowercase().contains("internal")
                || display_name.to_lowercase().contains("apple internal")
                || transport.eq_ignore_ascii_case("fifo")
                || transport.eq_ignore_ascii_case("spu")
                || transport.eq_ignore_ascii_case("spi");

            CFRetain(dev.cast());

            result.push(KeyboardDevice {
                index: result.len(),
                name: display_name,
                manufacturer: if mfg.is_empty() {
                    "Unknown".to_string()
                } else {
                    mfg
                },
                vendor_id: vid,
                product_id: pid,
                transport: if transport.is_empty() {
                    "Unknown".to_string()
                } else {
                    transport
                },
                is_internal,
                device_ptr: dev as usize,
            });
        }

        CFRelease(device_set.cast());
    }

    result
}

pub fn try_seize_device(device_ptr: usize) -> bool {
    if device_ptr == 0 {
        return false;
    }
    unsafe {
        let dev = device_ptr as IOHIDDeviceRef;
        let res = IOHIDDeviceOpen(dev, K_IOHID_OPTIONS_TYPE_SEIZE_DEVICE);
        res == K_IO_RETURN_SUCCESS
    }
}

pub fn unseize_device(device_ptr: usize) {
    if device_ptr == 0 {
        return;
    }
    unsafe {
        let dev = device_ptr as IOHIDDeviceRef;
        let _ = IOHIDDeviceClose(dev, K_IOHID_OPTIONS_TYPE_SEIZE_DEVICE);
    }
}

fn get_string_property(device: IOHIDDeviceRef, key_name: &str) -> String {
    let cf_key = CFString::new(key_name);
    unsafe {
        let prop = IOHIDDeviceGetProperty(device, cf_key.as_concrete_TypeRef());
        if prop.is_null() {
            return String::new();
        }
        let cf_str = CFString::wrap_under_get_rule(prop as core_foundation::string::CFStringRef);
        cf_str.to_string().trim().to_string()
    }
}

fn get_u32_property(device: IOHIDDeviceRef, key_name: &str) -> Option<u32> {
    let cf_key = CFString::new(key_name);
    unsafe {
        let prop = IOHIDDeviceGetProperty(device, cf_key.as_concrete_TypeRef());
        if prop.is_null() {
            return None;
        }
        let mut val: u32 = 0;
        if CFNumberGetValue(prop, 3, &mut val as *mut u32 as *mut c_void) {
            Some(val)
        } else {
            None
        }
    }
}
