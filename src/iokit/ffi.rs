use core_foundation::base::CFIndex;
use core_foundation::string::CFStringRef;
use std::ffi::{c_int, c_void};

#[repr(C)]
pub struct __IOHIDManager(c_void);
pub type IOHIDManagerRef = *mut __IOHIDManager;

#[repr(C)]
pub struct __IOHIDDevice(c_void);
pub type IOHIDDeviceRef = *mut __IOHIDDevice;

#[repr(C)]
pub struct __IOHIDValue(c_void);
pub type IOHIDValueRef = *mut __IOHIDValue;

#[repr(C)]
pub struct __IOHIDElement(c_void);
pub type IOHIDElementRef = *mut __IOHIDElement;

#[repr(C)]
pub struct __CFSet(c_void);
pub type CFSetRef = *const __CFSet;

pub type IOReturn = c_int;
pub type IOOptionBits = u32;

pub const K_IOHID_OPTIONS_TYPE_NONE: IOOptionBits = 0;
#[allow(dead_code)]
pub const K_IOHID_OPTIONS_TYPE_SEIZE_DEVICE: IOOptionBits = 1;
pub const K_IO_RETURN_SUCCESS: IOReturn = 0;

pub const HID_USAGE_PAGE_GENERIC_DESKTOP: u32 = 0x01;
pub const HID_USAGE_GENERIC_DESKTOP_KEYBOARD: u32 = 0x06;
pub const HID_USAGE_PAGE_KEYBOARD_OR_KEYPAD: u32 = 0x07;

pub type IOHIDValueCallback = extern "C" fn(
    context: *mut c_void,
    result: IOReturn,
    sender: *mut c_void,
    value: IOHIDValueRef,
);

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    pub fn IOHIDManagerCreate(allocator: *const c_void, options: IOOptionBits) -> IOHIDManagerRef;
    pub fn IOHIDManagerSetDeviceMatching(manager: IOHIDManagerRef, matching: *const c_void);
    pub fn IOHIDManagerCopyDevices(manager: IOHIDManagerRef) -> CFSetRef;
    pub fn IOHIDManagerOpen(manager: IOHIDManagerRef, options: IOOptionBits) -> IOReturn;
    pub fn IOHIDManagerClose(manager: IOHIDManagerRef, options: IOOptionBits) -> IOReturn;
    pub fn IOHIDManagerScheduleWithRunLoop(
        manager: IOHIDManagerRef,
        run_loop: *mut c_void,
        run_loop_mode: CFStringRef,
    );
    pub fn IOHIDManagerRegisterInputValueCallback(
        manager: IOHIDManagerRef,
        callback: Option<IOHIDValueCallback>,
        context: *mut c_void,
    );

    #[allow(dead_code)]
    pub fn IOHIDDeviceOpen(device: IOHIDDeviceRef, options: IOOptionBits) -> IOReturn;
    #[allow(dead_code)]
    pub fn IOHIDDeviceClose(device: IOHIDDeviceRef, options: IOOptionBits) -> IOReturn;
    pub fn IOHIDDeviceGetProperty(device: IOHIDDeviceRef, key: CFStringRef) -> *const c_void;

    pub fn IOHIDValueGetElement(value: IOHIDValueRef) -> IOHIDElementRef;
    pub fn IOHIDElementGetUsagePage(element: IOHIDElementRef) -> u32;
    pub fn IOHIDElementGetUsage(element: IOHIDElementRef) -> u32;
    pub fn IOHIDValueGetIntegerValue(value: IOHIDValueRef) -> CFIndex;

    pub fn CFSetGetCount(the_set: CFSetRef) -> CFIndex;
    pub fn CFSetGetValues(the_set: CFSetRef, values: *mut *const c_void);
    pub fn CFRetain(cf: *const c_void) -> *const c_void;
    pub fn CFRelease(cf: *const c_void);

    pub fn CFNumberGetValue(number: *const c_void, the_type: c_int, value_ptr: *mut c_void)
    -> bool;
}
