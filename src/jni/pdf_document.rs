//! PdfDocument JNI bindings for the read API.
//!
//! This module implements native methods for reading and extracting text from PDF documents.

use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jobject};
use jni::JNIEnv;

use crate::document::PdfDocument;

use std::collections::HashMap;
use std::sync::Mutex;

thread_local! {
    static OPEN_DOCUMENTS: Mutex<HashMap<u64, Mutex<PdfDocument>>> = Mutex::new(HashMap::new());
}

static mut NEXT_PTR: u64 = 1;

/// Stores a PdfDocument and returns a unique pointer to it
fn store_document(doc: PdfDocument) -> u64 {
    // SAFETY: This is only accessed from JNI, single-threaded per JVM
    let ptr = unsafe {
        NEXT_PTR += 1;
        NEXT_PTR - 1
    };

    OPEN_DOCUMENTS.with(|d| {
        let mut docs = d.lock().unwrap();
        docs.insert(ptr, Mutex::new(doc));
    });

    ptr
}

/// Executes a function with mutable access to a stored PdfDocument
fn with_document_mut<F, R>(ptr: u64, f: F) -> Result<R, String>
where
    F: FnOnce(&mut PdfDocument) -> R,
{
    OPEN_DOCUMENTS.with(|d| {
        let docs = d.lock().unwrap();
        match docs.get(&ptr) {
            Some(doc_mutex) => {
                let mut doc = doc_mutex
                    .lock()
                    .map_err(|e| format!("Failed to lock document: {}", e))?;
                Ok(f(&mut *doc))
            },
            None => Err("Document pointer is invalid".to_string()),
        }
    })
}

/// Opens a PDF document from a file path
/// Java signature: `public static native long nativeOpen(String path)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_PdfDocument_nativeOpen(
    mut env: JNIEnv,
    _class: JClass,
    path: JString,
) -> u64 {
    let path_str: String = match env.get_string(&path) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid path encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return 0;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read path: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return 0;
        },
    };

    match PdfDocument::open(&path_str) {
        Ok(doc) => store_document(doc),
        Err(e) => {
            log::error!("Failed to open PDF: {}", e);
            crate::jni::exceptions::throw_exception(env, e);
            0
        },
    }
}

/// Opens an encrypted PDF with a password
/// Java signature: `public static native long nativeOpenWithPassword(String path, String password)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_PdfDocument_nativeOpenWithPassword(
    mut env: JNIEnv,
    _class: JClass,
    path: JString,
    _password: JString,
) -> u64 {
    let path_str: String = match env.get_string(&path) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid path encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return 0;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read path: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return 0;
        },
    };

    // TODO: Implement password support in a future phase
    match PdfDocument::open(&path_str) {
        Ok(doc) => store_document(doc),
        Err(e) => {
            log::error!("Failed to open PDF: {}", e);
            crate::jni::exceptions::throw_exception(env, e);
            0
        },
    }
}

/// Frees a PDF document
/// Java signature: `private static native void nativeFree(long ptr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_PdfDocument_nativeFree(
    _env: JNIEnv,
    _class: JClass,
    ptr: u64,
) {
    if ptr != 0 {
        OPEN_DOCUMENTS.with(|d| {
            d.lock().unwrap().remove(&ptr);
        });
    }
}

/// Gets the PDF version
/// Java signature: `private static native int[] nativeGetVersion(long ptr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_PdfDocument_nativeGetVersion(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
) -> jobject {
    match with_document_mut(ptr, |doc| {
        let (major, minor) = doc.version();
        (major, minor)
    }) {
        Ok((major, minor)) => match env.new_int_array(2) {
            Ok(array) => {
                let versions = [major as i32, minor as i32];
                match env.set_int_array_region(&array, 0, &versions) {
                    Ok(()) => array.into_raw(),
                    Err(e) => {
                        log::error!("Failed to set array region: {}", e);
                        std::ptr::null_mut()
                    },
                }
            },
            Err(e) => {
                log::error!("Failed to create int array: {}", e);
                std::ptr::null_mut()
            },
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
            std::ptr::null_mut()
        },
    }
}

/// Gets the page count
/// Java signature: `private static native int nativeGetPageCount(long ptr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_PdfDocument_nativeGetPageCount(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
) -> jint {
    match with_document_mut(ptr, |doc| doc.page_count()) {
        Ok(result) => match result {
            Ok(count) => count as jint,
            Err(e) => {
                log::error!("Failed to get page count: {}", e);
                crate::jni::exceptions::throw_exception(env, e);
                -1
            },
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
            -1
        },
    }
}

/// Extracts text from a page
/// Java signature: `private static native String nativeExtractText(long ptr, int pageIndex)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_PdfDocument_nativeExtractText(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    page_index: jint,
) -> jobject {
    match with_document_mut(ptr, |doc| doc.extract_text(page_index as usize)) {
        Ok(result) => match result {
            Ok(text) => match env.new_string(&text) {
                Ok(jstring) => jstring.into_raw(),
                Err(e) => {
                    log::error!("Failed to create Java string: {}", e);
                    let msg = format!("Failed to create string: {}", e);
                    crate::jni::exceptions::throw_pdf_exception(env, &msg);
                    std::ptr::null_mut()
                },
            },
            Err(e) => {
                log::error!("Failed to extract text: {}", e);
                crate::jni::exceptions::throw_exception(env, e);
                std::ptr::null_mut()
            },
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
            std::ptr::null_mut()
        },
    }
}

/// Checks if document has structure tree
/// Java signature: `private static native boolean nativeHasStructureTree(long ptr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_PdfDocument_nativeHasStructureTree(
    _env: JNIEnv,
    _class: JClass,
    ptr: u64,
) -> jboolean {
    match with_document_mut(ptr, |doc| match doc.structure_tree() {
        Ok(tree) => tree.is_some(),
        Err(_) => false,
    }) {
        Ok(has_tree) => {
            if has_tree {
                1
            } else {
                0
            }
        },
        Err(_) => 0,
    }
}

/// Creates conversion options (stored on Rust side)
/// Java signature: `private static native long nativeCreateConversionOptions(...)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_PdfDocument_nativeCreateConversionOptions(
    _env: JNIEnv,
    _class: JClass,
    _detect_headings: jboolean,
    _preserve_layout: jboolean,
    _extract_images: jboolean,
    _extract_tables: jboolean,
    _max_line_length: jint,
    _language_hints: JString,
) -> u64 {
    // For Phase 2, return a dummy value
    // Full implementation in Phase 3
    1 as u64
}

/// Frees conversion options
/// Java signature: `private static native void nativeFreeConversionOptions(long optionsPtr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_PdfDocument_nativeFreeConversionOptions(
    _env: JNIEnv,
    _class: JClass,
    _options_ptr: u64,
) {
    // Phase 2 placeholder
}

/// Converts page to Markdown
/// Java signature: `private static native String nativeToMarkdown(long ptr, int pageIndex, long optionsPtr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_PdfDocument_nativeToMarkdown(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    page_index: jint,
    _options_ptr: u64,
) -> jobject {
    match with_document_mut(ptr, |doc| {
        doc.to_markdown(page_index as usize, &crate::converters::ConversionOptions::default())
    }) {
        Ok(result) => match result {
            Ok(markdown) => match env.new_string(&markdown) {
                Ok(jstring) => jstring.into_raw(),
                Err(e) => {
                    log::error!("Failed to create Java string: {}", e);
                    std::ptr::null_mut()
                },
            },
            Err(e) => {
                log::error!("Failed to convert to Markdown: {}", e);
                crate::jni::exceptions::throw_exception(env, e);
                std::ptr::null_mut()
            },
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
            std::ptr::null_mut()
        },
    }
}

/// Converts all pages to Markdown
/// Java signature: `private static native String nativeToMarkdownAll(long ptr, long optionsPtr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_PdfDocument_nativeToMarkdownAll(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    _options_ptr: u64,
) -> jobject {
    match with_document_mut(ptr, |doc| {
        doc.to_markdown_all(&crate::converters::ConversionOptions::default())
    }) {
        Ok(result) => match result {
            Ok(markdown) => match env.new_string(&markdown) {
                Ok(jstring) => jstring.into_raw(),
                Err(e) => {
                    log::error!("Failed to create Java string: {}", e);
                    std::ptr::null_mut()
                },
            },
            Err(e) => {
                log::error!("Failed to convert to Markdown: {}", e);
                crate::jni::exceptions::throw_exception(env, e);
                std::ptr::null_mut()
            },
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
            std::ptr::null_mut()
        },
    }
}

/// Converts page to HTML
/// Java signature: `private static native String nativeToHtml(long ptr, int pageIndex, long optionsPtr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_PdfDocument_nativeToHtml(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    page_index: jint,
    _options_ptr: u64,
) -> jobject {
    match with_document_mut(ptr, |doc| {
        doc.to_html(page_index as usize, &crate::converters::ConversionOptions::default())
    }) {
        Ok(result) => match result {
            Ok(html) => match env.new_string(&html) {
                Ok(jstring) => jstring.into_raw(),
                Err(e) => {
                    log::error!("Failed to create Java string: {}", e);
                    std::ptr::null_mut()
                },
            },
            Err(e) => {
                log::error!("Failed to convert to HTML: {}", e);
                crate::jni::exceptions::throw_exception(env, e);
                std::ptr::null_mut()
            },
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
            std::ptr::null_mut()
        },
    }
}

/// Converts all pages to HTML
/// Java signature: `private static native String nativeToHtmlAll(long ptr, long optionsPtr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_PdfDocument_nativeToHtmlAll(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    _options_ptr: u64,
) -> jobject {
    match with_document_mut(ptr, |doc| {
        doc.to_html_all(&crate::converters::ConversionOptions::default())
    }) {
        Ok(result) => match result {
            Ok(html) => match env.new_string(&html) {
                Ok(jstring) => jstring.into_raw(),
                Err(e) => {
                    log::error!("Failed to create Java string: {}", e);
                    std::ptr::null_mut()
                },
            },
            Err(e) => {
                log::error!("Failed to convert to HTML: {}", e);
                crate::jni::exceptions::throw_exception(env, e);
                std::ptr::null_mut()
            },
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
            std::ptr::null_mut()
        },
    }
}

/// Converts page to plain text
/// Java signature: `private static native String nativeToPlainText(long ptr, int pageIndex, long optionsPtr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_PdfDocument_nativeToPlainText(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    page_index: jint,
    _options_ptr: u64,
) -> jobject {
    match with_document_mut(ptr, |doc| {
        doc.to_plain_text(page_index as usize, &crate::converters::ConversionOptions::default())
    }) {
        Ok(result) => match result {
            Ok(text) => match env.new_string(&text) {
                Ok(jstring) => jstring.into_raw(),
                Err(e) => {
                    log::error!("Failed to create Java string: {}", e);
                    std::ptr::null_mut()
                },
            },
            Err(e) => {
                log::error!("Failed to convert to plain text: {}", e);
                crate::jni::exceptions::throw_exception(env, e);
                std::ptr::null_mut()
            },
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
            std::ptr::null_mut()
        },
    }
}
