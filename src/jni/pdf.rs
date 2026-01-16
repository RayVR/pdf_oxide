//! Pdf (Universal API) JNI bindings for read/create/edit operations.
//!
//! This module implements native methods for the universal Pdf API that combines
//! reading, creation, and editing capabilities in a single interface.

use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jobject};
use jni::JNIEnv;

use crate::api::Pdf;

use std::collections::HashMap;
use std::sync::Mutex;

thread_local! {
    static OPEN_DOCUMENTS: Mutex<HashMap<u64, Mutex<Pdf>>> = Mutex::new(HashMap::new());
}

static mut NEXT_PTR: u64 = 1;

/// Stores a Pdf instance and returns a unique pointer to it
fn store_document(doc: Pdf) -> u64 {
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

/// Executes a function with mutable access to a stored Pdf instance
fn with_document_mut<F, R>(ptr: u64, f: F) -> Result<R, String>
where
    F: FnOnce(&mut Pdf) -> R,
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

/// Creates a new blank PDF document
/// Java signature: `public static native long nativeCreate()`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeCreate(
    _env: JNIEnv,
    _class: JClass,
) -> u64 {
    let doc = Pdf::new();
    store_document(doc)
}

/// Opens an existing PDF document
/// Java signature: `public static native long nativeOpen(String path)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeOpen(
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

    match Pdf::open(&path_str) {
        Ok(doc) => store_document(doc),
        Err(e) => {
            log::error!("Failed to open PDF: {}", e);
            crate::jni::exceptions::throw_exception(env, e);
            0
        },
    }
}

/// Creates a PDF from Markdown source
/// Java signature: `public static native long nativeFromMarkdown(String markdown)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeFromMarkdown(
    mut env: JNIEnv,
    _class: JClass,
    markdown: JString,
) -> u64 {
    let markdown_str: String = match env.get_string(&markdown) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid markdown encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return 0;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read markdown: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return 0;
        },
    };

    match Pdf::from_markdown(&markdown_str) {
        Ok(doc) => store_document(doc),
        Err(e) => {
            log::error!("Failed to create PDF from markdown: {}", e);
            crate::jni::exceptions::throw_exception(env, e);
            0
        },
    }
}

/// Creates a PDF from HTML source
/// Java signature: `public static native long nativeFromHtml(String html)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeFromHtml(
    mut env: JNIEnv,
    _class: JClass,
    html: JString,
) -> u64 {
    let html_str: String = match env.get_string(&html) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid HTML encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return 0;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read HTML: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return 0;
        },
    };

    match Pdf::from_html(&html_str) {
        Ok(doc) => store_document(doc),
        Err(e) => {
            log::error!("Failed to create PDF from HTML: {}", e);
            crate::jni::exceptions::throw_exception(env, e);
            0
        },
    }
}

/// Creates a PDF from plain text
/// Java signature: `public static native long nativeFromText(String text)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeFromText(
    mut env: JNIEnv,
    _class: JClass,
    text: JString,
) -> u64 {
    let text_str: String = match env.get_string(&text) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid text encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return 0;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read text: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return 0;
        },
    };

    match Pdf::from_text(&text_str) {
        Ok(doc) => store_document(doc),
        Err(e) => {
            log::error!("Failed to create PDF from text: {}", e);
            crate::jni::exceptions::throw_exception(env, e);
            0
        },
    }
}

/// Creates a PDF from a single image file
/// Java signature: `public static native long nativeFromImage(String imagePath)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeFromImage(
    mut env: JNIEnv,
    _class: JClass,
    image_path: JString,
) -> u64 {
    let path_str: String = match env.get_string(&image_path) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid image path encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return 0;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read image path: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return 0;
        },
    };

    match Pdf::from_image(&path_str) {
        Ok(doc) => store_document(doc),
        Err(e) => {
            log::error!("Failed to create PDF from image: {}", e);
            crate::jni::exceptions::throw_exception(env, e);
            0
        },
    }
}

/// Creates a PDF from multiple image files
/// Java signature: `public static native long nativeFromImages(String[] imagePaths)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeFromImages(
    mut env: JNIEnv,
    _class: JClass,
    image_paths: jni::objects::JObjectArray,
) -> u64 {
    let len = match env.get_array_length(&image_paths) {
        Ok(len) => len as usize,
        Err(e) => {
            log::error!("Failed to get array length: {}", e);
            return 0;
        },
    };

    let mut paths = Vec::with_capacity(len);
    for i in 0..len {
        match env.get_object_array_element(&image_paths, i as i32) {
            Ok(obj) => match env.get_string((&obj).into()) {
                Ok(s) => match s.to_str() {
                    Ok(s) => paths.push(s.to_string()),
                    Err(e) => {
                        log::error!("Invalid path encoding at index {}: {}", i, e);
                        return 0;
                    },
                },
                Err(e) => {
                    log::error!("Failed to read path at index {}: {}", i, e);
                    return 0;
                },
            },
            Err(e) => {
                log::error!("Failed to get array element at index {}: {}", i, e);
                return 0;
            },
        }
    }

    match Pdf::from_images(&paths) {
        Ok(doc) => store_document(doc),
        Err(e) => {
            log::error!("Failed to create PDF from images: {}", e);
            crate::jni::exceptions::throw_exception(env, e);
            0
        },
    }
}

/// Frees a PDF document
/// Java signature: `private static native void nativeFree(long ptr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeFree(
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

/// Gets the page count
/// Java signature: `private static native int nativeGetPageCount(long ptr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeGetPageCount(
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

/// Saves the PDF to a file
/// Java signature: `private static native void nativeSave(long ptr, String path)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeSave(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    path: JString,
) {
    let path_str: String = match env.get_string(&path) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid path encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read path: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return;
        },
    };

    match with_document_mut(ptr, |doc| doc.save(&path_str)) {
        Ok(result) => {
            if let Err(e) = result {
                log::error!("Failed to save PDF: {}", e);
                crate::jni::exceptions::throw_exception(env, e);
            }
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
        },
    }
}

/// Saves the PDF with encryption
/// Java signature: `private static native void nativeSaveEncrypted(long ptr, String path, String userPassword, String ownerPassword)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeSaveEncrypted(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    path: JString,
    user_password: JString,
    owner_password: JString,
) {
    let path_str: String = match env.get_string(&path) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid path encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read path: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return;
        },
    };

    let user_pwd_str: String = match env.get_string(&user_password) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid user password encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read user password: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return;
        },
    };

    let owner_pwd_str: String = match env.get_string(&owner_password) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid owner password encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read owner password: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return;
        },
    };

    match with_document_mut(ptr, |doc| doc.save_encrypted(&path_str, &user_pwd_str, &owner_pwd_str))
    {
        Ok(result) => {
            if let Err(e) = result {
                log::error!("Failed to save encrypted PDF: {}", e);
                crate::jni::exceptions::throw_exception(env, e);
            }
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
        },
    }
}

/// Sets the document title
/// Java signature: `private static native void nativeSetTitle(long ptr, String title)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeSetTitle(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    title: JString,
) {
    let title_str: String = match env.get_string(&title) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid title encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read title: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return;
        },
    };

    match with_document_mut(ptr, |doc| doc.set_title(&title_str)) {
        Ok(result) => {
            if let Err(e) = result {
                log::error!("Failed to set title: {}", e);
                crate::jni::exceptions::throw_exception(env, e);
            }
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
        },
    }
}

/// Sets the document author
/// Java signature: `private static native void nativeSetAuthor(long ptr, String author)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeSetAuthor(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    author: JString,
) {
    let author_str: String = match env.get_string(&author) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid author encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read author: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return;
        },
    };

    match with_document_mut(ptr, |doc| doc.set_author(&author_str)) {
        Ok(result) => {
            if let Err(e) = result {
                log::error!("Failed to set author: {}", e);
                crate::jni::exceptions::throw_exception(env, e);
            }
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
        },
    }
}

/// Sets the document subject
/// Java signature: `private static native void nativeSetSubject(long ptr, String subject)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeSetSubject(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    subject: JString,
) {
    let subject_str: String = match env.get_string(&subject) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid subject encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read subject: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return;
        },
    };

    match with_document_mut(ptr, |doc| doc.set_subject(&subject_str)) {
        Ok(result) => {
            if let Err(e) = result {
                log::error!("Failed to set subject: {}", e);
                crate::jni::exceptions::throw_exception(env, e);
            }
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
        },
    }
}

/// Sets document keywords
/// Java signature: `private static native void nativeSetKeywords(long ptr, String keywords)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeSetKeywords(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    keywords: JString,
) {
    let keywords_str: String = match env.get_string(&keywords) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid keywords encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read keywords: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return;
        },
    };

    match with_document_mut(ptr, |doc| doc.set_keywords(&keywords_str)) {
        Ok(result) => {
            if let Err(e) = result {
                log::error!("Failed to set keywords: {}", e);
                crate::jni::exceptions::throw_exception(env, e);
            }
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
        },
    }
}

/// Gets document information (placeholder for Phase 4)
/// Java signature: `private static native DocumentInfo nativeGetInfo(long ptr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeGetInfo(
    _env: JNIEnv,
    _class: JClass,
    _ptr: u64,
) -> jobject {
    // TODO: Phase 4 - Implement DocumentInfo binding
    // For now, return null as DocumentInfo binding is not yet implemented
    std::ptr::null_mut()
}

/// Converts a page to Markdown
/// Java signature: `private static native String nativeToMarkdown(long ptr, int pageIndex)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeToMarkdown(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    page_index: jint,
) -> jobject {
    match with_document_mut(ptr, |doc| doc.to_markdown(page_index as usize)) {
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

/// Converts a page to HTML
/// Java signature: `private static native String nativeToHtml(long ptr, int pageIndex)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeToHtml(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    page_index: jint,
) -> jobject {
    match with_document_mut(ptr, |doc| doc.to_html(page_index as usize)) {
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

/// Converts a page to plain text
/// Java signature: `private static native String nativeToText(long ptr, int pageIndex)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeToText(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    page_index: jint,
) -> jobject {
    match with_document_mut(ptr, |doc| doc.to_text(page_index as usize)) {
        Ok(result) => match result {
            Ok(text) => match env.new_string(&text) {
                Ok(jstring) => jstring.into_raw(),
                Err(e) => {
                    log::error!("Failed to create Java string: {}", e);
                    std::ptr::null_mut()
                },
            },
            Err(e) => {
                log::error!("Failed to convert to text: {}", e);
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

/// Checks if the document has been modified
/// Java signature: `private static native boolean nativeIsModified(long ptr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_core_Pdf_nativeIsModified(
    _env: JNIEnv,
    _class: JClass,
    ptr: u64,
) -> jboolean {
    match with_document_mut(ptr, |doc| doc.is_modified()) {
        Ok(is_modified) => {
            if is_modified {
                1
            } else {
                0
            }
        },
        Err(_) => 0,
    }
}
