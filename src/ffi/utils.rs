//! Utility functions for FFI: String marshaling, memory management
//!
//! Provides C-compatible functions for:
//! - Converting Rust strings to C-compatible UTF-8 pointers
//! - Freeing strings allocated in Rust
//! - Converting between Rust and C representations

use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

/// Allocate a Rust string as a C-compatible UTF-8 null-terminated string
///
/// # Safety
/// The returned pointer must be freed using `free_string()`.
/// The C# code must call `FreeString()` after reading the string.
#[no_mangle]
pub unsafe extern "C" fn alloc_string(s: *const c_char) -> *mut c_char {
    if s.is_null() {
        return ptr::null_mut();
    }

    match CStr::from_ptr(s).to_str() {
        Ok(s) => {
            let boxed: Box<str> = s.into();
            Box::into_raw(Box::new(boxed)) as *mut c_char
        },
        Err(_) => ptr::null_mut(),
    }
}

/// Free a string allocated by Rust
///
/// # Safety
/// The pointer must have been returned by a Rust FFI function (e.g., from `alloc_string()`).
/// Calling this twice on the same pointer is undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }

    // Convert the C string pointer to a Rust string and let it drop
    let _ = CStr::from_ptr(ptr).to_string_lossy();
    let _ = Box::from_raw(ptr);
}

/// Free a byte buffer allocated by Rust
///
/// # Safety
/// The pointer must have been returned by a Rust FFI function.
/// The size must match the actual allocated size.
#[no_mangle]
pub unsafe extern "C" fn free_bytes(ptr: *mut u8, _len: usize) {
    if ptr.is_null() {
        return;
    }

    // Convert to Box to trigger deallocation
    let _ = Box::from_raw(ptr);
}

/// Convert a Rust String to a C-compatible pointer
///
/// # Returns
/// A pointer to UTF-8 null-terminated string. Must be freed with `free_string()`.
pub fn rust_string_to_c(s: String) -> *mut c_char {
    let mut bytes = s.into_bytes();
    bytes.push(0); // null terminator

    Box::into_raw(bytes.into_boxed_slice()) as *mut c_char
}

/// Convert a C string pointer to a Rust String
///
/// # Safety
/// The pointer must be a valid null-terminated UTF-8 string.
pub unsafe fn c_string_to_rust(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_roundtrip() {
        let original = "Hello, World!";
        let c_string = std::ffi::CString::new(original).unwrap();

        unsafe {
            let allocated = alloc_string(c_string.as_ptr());
            assert!(!allocated.is_null());

            let result = CStr::from_ptr(allocated).to_str().unwrap();
            assert_eq!(result, original);

            free_string(allocated);
        }
    }

    #[test]
    fn test_null_string() {
        unsafe {
            let allocated = alloc_string(ptr::null());
            assert!(allocated.is_null());

            free_string(ptr::null_mut());
        }
    }
}
