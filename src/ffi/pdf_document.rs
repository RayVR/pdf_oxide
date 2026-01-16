//! PdfDocument C API for reading and text extraction
//!
//! Provides FFI functions for:
//! - Opening PDF files
//! - Extracting text from pages
//! - Converting to various formats (Markdown, HTML, PlainText)
//! - Accessing PDF metadata and structure
//!
//! **Note**: PdfDocument methods require mutable references, which makes them
//! not thread-safe without additional synchronization. Future versions will
//! wrap this with Arc<Mutex<>> for thread-safe access.

use crate::PdfDocument as RustPdfDocument;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

use super::exceptions::{pdf_error_to_code, ErrorCode};
use super::utils::rust_string_to_c;

// Opaque handle for PdfDocument
pub struct PdfDocumentHandle(RustPdfDocument);

/// Open a PDF document from file path
///
/// # Arguments
/// * `path` - UTF-8 null-terminated file path
/// * `error_code` - Output parameter for error code (0 = success)
///
/// # Returns
/// Opaque handle to PdfDocument, or null on error (check error_code)
#[no_mangle]
pub unsafe extern "C" fn pdf_document_open(
    path: *const c_char,
    error_code: *mut i32,
) -> *mut PdfDocumentHandle {
    // Validate error_code pointer
    if error_code.is_null() {
        return ptr::null_mut();
    }

    // Parse path
    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => {
            *error_code = ErrorCode::ParseError as i32;
            return ptr::null_mut();
        },
    };

    // Open document
    match RustPdfDocument::open(path_str) {
        Ok(doc) => {
            *error_code = ErrorCode::Success as i32;
            Box::into_raw(Box::new(PdfDocumentHandle(doc)))
        },
        Err(e) => {
            *error_code = pdf_error_to_code(&e);
            ptr::null_mut()
        },
    }
}

/// Free a PdfDocument handle
///
/// # Safety
/// The handle must be valid and not used after this call.
#[no_mangle]
pub unsafe extern "C" fn pdf_document_free(handle: *mut PdfDocumentHandle) {
    if !handle.is_null() {
        let _ = Box::from_raw(handle);
    }
}

/// Get PDF version as (major, minor)
#[no_mangle]
pub unsafe extern "C" fn pdf_document_get_version(
    handle: *const PdfDocumentHandle,
    major: *mut u8,
    minor: *mut u8,
) {
    if handle.is_null() || major.is_null() || minor.is_null() {
        return;
    }

    let doc = &(*handle).0;
    let version = doc.version();
    *major = version.0;
    *minor = version.1;
}

/// Get the number of pages in the document
///
/// **Note**: Requires mutable access to document
#[no_mangle]
pub unsafe extern "C" fn pdf_document_get_page_count(
    handle: *mut PdfDocumentHandle,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() || error_code.is_null() {
        if !error_code.is_null() {
            *error_code = ErrorCode::InternalError as i32;
        }
        return -1;
    }

    let doc = &mut (*handle).0;
    match doc.page_count() {
        Ok(count) => {
            *error_code = ErrorCode::Success as i32;
            count as i32
        },
        Err(e) => {
            *error_code = pdf_error_to_code(&e);
            -1
        },
    }
}

/// Check if document has a structure tree (Tagged PDF)
///
/// **Note**: Requires mutable access to document
#[no_mangle]
pub unsafe extern "C" fn pdf_document_has_structure_tree(handle: *mut PdfDocumentHandle) -> bool {
    if handle.is_null() {
        return false;
    }

    let doc = &mut (*handle).0;
    doc.structure_tree().ok().is_some()
}

/// Extract text from a page
///
/// # Returns
/// UTF-8 null-terminated string pointer. Must be freed with free_string().
/// Returns null on error.
///
/// **Note**: Requires mutable access to document
#[no_mangle]
pub unsafe extern "C" fn pdf_document_extract_text(
    handle: *mut PdfDocumentHandle,
    page_index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() || error_code.is_null() {
        if !error_code.is_null() {
            *error_code = ErrorCode::InternalError as i32;
        }
        return ptr::null_mut();
    }

    let doc = &mut (*handle).0;
    match doc.extract_text(page_index as usize) {
        Ok(text) => {
            *error_code = ErrorCode::Success as i32;
            rust_string_to_c(text)
        },
        Err(e) => {
            *error_code = pdf_error_to_code(&e);
            ptr::null_mut()
        },
    }
}

/// Convert page to Markdown format
///
/// # Returns
/// UTF-8 null-terminated string pointer. Must be freed with free_string().
///
/// **Note**: Requires mutable access to document
#[no_mangle]
pub unsafe extern "C" fn pdf_document_to_markdown(
    handle: *mut PdfDocumentHandle,
    page_index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() || error_code.is_null() {
        if !error_code.is_null() {
            *error_code = ErrorCode::InternalError as i32;
        }
        return ptr::null_mut();
    }

    let doc = &mut (*handle).0;
    // Create default conversion options
    let options = crate::converters::ConversionOptions::default();

    match doc.to_markdown(page_index as usize, &options) {
        Ok(markdown) => {
            *error_code = ErrorCode::Success as i32;
            rust_string_to_c(markdown)
        },
        Err(e) => {
            *error_code = pdf_error_to_code(&e);
            ptr::null_mut()
        },
    }
}

/// Convert all pages to Markdown
///
/// **Note**: Requires mutable access to document
#[no_mangle]
pub unsafe extern "C" fn pdf_document_to_markdown_all(
    handle: *mut PdfDocumentHandle,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() || error_code.is_null() {
        if !error_code.is_null() {
            *error_code = ErrorCode::InternalError as i32;
        }
        return ptr::null_mut();
    }

    let doc = &mut (*handle).0;
    let options = crate::converters::ConversionOptions::default();

    match doc.to_markdown_all(&options) {
        Ok(markdown) => {
            *error_code = ErrorCode::Success as i32;
            rust_string_to_c(markdown)
        },
        Err(e) => {
            *error_code = pdf_error_to_code(&e);
            ptr::null_mut()
        },
    }
}

/// Convert page to HTML format
///
/// **Note**: Requires mutable access to document
#[no_mangle]
pub unsafe extern "C" fn pdf_document_to_html(
    handle: *mut PdfDocumentHandle,
    page_index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() || error_code.is_null() {
        if !error_code.is_null() {
            *error_code = ErrorCode::InternalError as i32;
        }
        return ptr::null_mut();
    }

    let doc = &mut (*handle).0;
    let options = crate::converters::ConversionOptions::default();

    match doc.to_html(page_index as usize, &options) {
        Ok(html) => {
            *error_code = ErrorCode::Success as i32;
            rust_string_to_c(html)
        },
        Err(e) => {
            *error_code = pdf_error_to_code(&e);
            ptr::null_mut()
        },
    }
}

/// Convert page to plain text format
///
/// **Note**: Requires mutable access to document
#[no_mangle]
pub unsafe extern "C" fn pdf_document_to_plain_text(
    handle: *mut PdfDocumentHandle,
    page_index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() || error_code.is_null() {
        if !error_code.is_null() {
            *error_code = ErrorCode::InternalError as i32;
        }
        return ptr::null_mut();
    }

    let doc = &mut (*handle).0;
    let options = crate::converters::ConversionOptions::default();

    match doc.to_plain_text(page_index as usize, &options) {
        Ok(text) => {
            *error_code = ErrorCode::Success as i32;
            rust_string_to_c(text)
        },
        Err(e) => {
            *error_code = pdf_error_to_code(&e);
            ptr::null_mut()
        },
    }
}
