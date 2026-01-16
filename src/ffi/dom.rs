//! DOM C API for page and element access
//!
//! Provides FFI functions for:
//! - Page information and properties
//! - Finding and accessing page elements (text, images, paths)
//! - Basic element property access

use crate::editor::dom::PdfPage;
use std::ptr;

use super::exceptions::ErrorCode;

/// Opaque handle for PdfPage
pub struct PdfPageHandle(PdfPage);

/// Get page width
#[no_mangle]
pub unsafe extern "C" fn pdf_page_get_width(handle: *const PdfPageHandle) -> f32 {
    if handle.is_null() {
        return 0.0;
    }

    let page = &(*handle).0;
    page.width
}

/// Get page height
#[no_mangle]
pub unsafe extern "C" fn pdf_page_get_height(handle: *const PdfPageHandle) -> f32 {
    if handle.is_null() {
        return 0.0;
    }

    let page = &(*handle).0;
    page.height
}

/// Get page index
#[no_mangle]
pub unsafe extern "C" fn pdf_page_get_index(handle: *const PdfPageHandle) -> i32 {
    if handle.is_null() {
        return -1;
    }

    let page = &(*handle).0;
    page.page_index as i32
}

/// Get page dimensions as (width, height)
///
/// # Arguments
/// * `handle` - The page handle
/// * `width_out` - Output parameter for width
/// * `height_out` - Output parameter for height
#[no_mangle]
pub unsafe extern "C" fn pdf_page_get_dimensions(
    handle: *const PdfPageHandle,
    width_out: *mut f32,
    height_out: *mut f32,
) {
    if handle.is_null() || width_out.is_null() || height_out.is_null() {
        return;
    }

    let page = &(*handle).0;
    *width_out = page.width;
    *height_out = page.height;
}

/// Free a PdfPage handle
///
/// # Safety
/// The handle must be valid and not used after this call.
#[no_mangle]
pub unsafe extern "C" fn pdf_page_free(handle: *mut PdfPageHandle) {
    if !handle.is_null() {
        let _ = Box::from_raw(handle);
    }
}
