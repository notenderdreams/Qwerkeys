//! Low-level FFI bindings for macOS IOKit and CoreFoundation HID APIs.
//!
//! # Architecture Overview
//! macOS manages hardware input devices through the IOKit HID (Human Interface Device)
//! subsystem. Standard user-space event APIs (such as NSEvent) aggregate keystrokes into
//! a single global stream, obscuring physical device origin.
//!
//! This module binds directly to `IOKit.framework` symbols to enable:
//! 1. Direct hardware discovery and enumeration of keyboard devices.
//! 2. Interception of raw USB HID reports before OS-level window server processing.
//! 3. Extraction of vendor, product, and hardware bus attributes for per-device routing.

use core_foundation::base::CFIndex;
use core_foundation::string::CFStringRef;
use std::ffi::{c_int, c_void};

// Opaque type handles: zero-sized or opaque structs representing pointers to
// CoreFoundation and IOKit objects managed by macOS kernel/frameworks.
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

// Status codes and option flags.
pub type IOReturn = c_int;
pub type IOOptionBits = u32;

pub const K_IOHID_OPTIONS_TYPE_NONE: IOOptionBits = 0;
pub const K_IO_RETURN_SUCCESS: IOReturn = 0;

// USB HID specification constants used to filter and decode keyboard inputs.
// - Page 0x01 (Generic Desktop): defines device classes like keyboards and mice.
// - Usage 0x06: specific keyboard usage ID on generic desktop page.
// - Page 0x07 (Keyboard/Keypad): defines individual key scan codes.
pub const HID_USAGE_PAGE_GENERIC_DESKTOP: u32 = 0x01;
pub const HID_USAGE_GENERIC_DESKTOP_KEYBOARD: u32 = 0x06;
pub const HID_USAGE_PAGE_KEYBOARD_OR_KEYPAD: u32 = 0x07;

// Function pointer invoked by IOHIDManager on input state changes.
pub type IOHIDValueCallback = extern "C" fn(
    context: *mut c_void,
    result: IOReturn,
    sender: *mut c_void,
    value: IOHIDValueRef,
);

// Dynamic linker declarations for macOS IOKit.framework.
// Callers must adhere to CoreFoundation Get/Copy memory management rules.
#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    // IOHIDManager lifecycle and run loop scheduling.
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

    // Device property introspection.
    pub fn IOHIDDeviceGetProperty(device: IOHIDDeviceRef, key: CFStringRef) -> *const c_void;

    // HID value and element parsing.
    pub fn IOHIDValueGetElement(value: IOHIDValueRef) -> IOHIDElementRef;
    pub fn IOHIDElementGetUsagePage(element: IOHIDElementRef) -> u32;
    pub fn IOHIDElementGetUsage(element: IOHIDElementRef) -> u32;
    pub fn IOHIDValueGetIntegerValue(value: IOHIDValueRef) -> CFIndex;

    // CoreFoundation primitives.
    pub fn CFSetGetCount(the_set: CFSetRef) -> CFIndex;
    pub fn CFSetGetValues(the_set: CFSetRef, values: *mut *const c_void);
    pub fn CFRetain(cf: *const c_void) -> *const c_void;
    pub fn CFRelease(cf: *const c_void);

    pub fn CFNumberGetValue(number: *const c_void, the_type: c_int, value_ptr: *mut c_void)
    -> bool;
}
