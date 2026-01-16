//! DOM Navigation JNI bindings for element querying and manipulation.
//!
//! This module implements native methods for navigating the page DOM tree,
//! searching for elements, and modifying element content.

use jni::objects::{JClass, JObject, JString};
use jni::sys::{jint, jobject};
use jni::JNIEnv;

use crate::editor::dom::PdfPage;

use std::collections::HashMap;
use std::sync::Mutex;

// Reuse document storage from pdf module (shared across JNI context)
thread_local! {
    static OPEN_PAGES: Mutex<HashMap<u64, Mutex<PdfPage>>> = Mutex::new(HashMap::new());
}

static mut NEXT_PAGE_PTR: u64 = 1000; // Start at different offset than documents

/// Stores a PdfPage and returns a unique pointer to it
fn store_page(page: PdfPage) -> u64 {
    // SAFETY: This is only accessed from JNI, single-threaded per JVM
    let ptr = unsafe {
        NEXT_PAGE_PTR += 1;
        NEXT_PAGE_PTR - 1
    };

    OPEN_PAGES.with(|p| {
        let mut pages = p.lock().unwrap();
        pages.insert(ptr, Mutex::new(page));
    });

    ptr
}

/// Executes a function with mutable access to a stored PdfPage
fn with_page_mut<F, R>(ptr: u64, f: F) -> Result<R, String>
where
    F: FnOnce(&mut PdfPage) -> R,
{
    OPEN_PAGES.with(|p| {
        let pages = p.lock().unwrap();
        match pages.get(&ptr) {
            Some(page_mutex) => {
                let mut page = page_mutex
                    .lock()
                    .map_err(|e| format!("Failed to lock page: {}", e))?;
                Ok(f(&mut *page))
            },
            None => Err("Page pointer is invalid".to_string()),
        }
    })
}

/// Frees a PdfPage
/// Java signature: `private static native void nativeFreePage(long ptr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_dom_PdfPage_nativeFreePage(
    _env: JNIEnv,
    _class: JClass,
    ptr: u64,
) {
    if ptr != 0 {
        OPEN_PAGES.with(|p| {
            p.lock().unwrap().remove(&ptr);
        });
    }
}

/// Gets all top-level child elements on a page
/// Java signature: `private static native PdfElement[] nativeGetChildren(long ptr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_dom_PdfPage_nativeGetChildren(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
) -> jobject {
    match with_page_mut(ptr, |page| page.children()) {
        Ok(children) => {
            // For Phase 4, return empty array - full implementation in Phase 5+
            // Would need to create Java PdfElement objects and return Object[] array
            match env.find_class("java/lang/Object") {
                Ok(obj_class) => {
                    match env.new_object_array(children.len() as i32, &obj_class, JObject::null()) {
                        Ok(array) => array.into_raw(),
                        Err(e) => {
                            log::error!("Failed to create object array: {}", e);
                            std::ptr::null_mut()
                        },
                    }
                },
                Err(e) => {
                    log::error!("Failed to find Object class: {}", e);
                    std::ptr::null_mut()
                },
            }
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
            std::ptr::null_mut()
        },
    }
}

/// Finds all text elements containing a substring
/// Java signature: `private static native String[] nativeFindTextContaining(long ptr, String needle)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_dom_PdfPage_nativeFindTextContaining(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    needle: JString,
) -> jobject {
    let needle_str: String = match env.get_string(&needle) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid needle encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return std::ptr::null_mut();
            },
        },
        Err(e) => {
            let msg = format!("Failed to read needle: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return std::ptr::null_mut();
        },
    };

    match with_page_mut(ptr, |page| page.find_text_containing(&needle_str)) {
        Ok(text_elements) => {
            // Create array of text strings
            match env.find_class("java/lang/String") {
                Ok(string_class) => {
                    match env.new_object_array(
                        text_elements.len() as i32,
                        &string_class,
                        JObject::null(),
                    ) {
                        Ok(array) => {
                            for (i, pdf_text) in text_elements.iter().enumerate() {
                                let text = pdf_text.text();
                                match env.new_string(text) {
                                    Ok(jstr) => {
                                        let _ =
                                            env.set_object_array_element(&array, i as i32, jstr);
                                    },
                                    Err(e) => {
                                        log::error!(
                                            "Failed to create string at index {}: {}",
                                            i,
                                            e
                                        );
                                        return std::ptr::null_mut();
                                    },
                                }
                            }
                            array.into_raw()
                        },
                        Err(e) => {
                            log::error!("Failed to create object array: {}", e);
                            std::ptr::null_mut()
                        },
                    }
                },
                Err(e) => {
                    log::error!("Failed to find String class: {}", e);
                    std::ptr::null_mut()
                },
            }
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
            std::ptr::null_mut()
        },
    }
}

/// Gets all text elements on the page
/// Java signature: `private static native String[] nativeGetAllText(long ptr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_dom_PdfPage_nativeGetAllText(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
) -> jobject {
    match with_page_mut(ptr, |page| {
        // Use empty string to get all text
        page.find_text_containing("")
    }) {
        Ok(text_elements) => {
            // Create array of text strings
            match env.find_class("java/lang/String") {
                Ok(string_class) => {
                    match env.new_object_array(
                        text_elements.len() as i32,
                        &string_class,
                        JObject::null(),
                    ) {
                        Ok(array) => {
                            for (i, pdf_text) in text_elements.iter().enumerate() {
                                let text = pdf_text.text();
                                match env.new_string(text) {
                                    Ok(jstr) => {
                                        let _ =
                                            env.set_object_array_element(&array, i as i32, jstr);
                                    },
                                    Err(e) => {
                                        log::error!(
                                            "Failed to create string at index {}: {}",
                                            i,
                                            e
                                        );
                                        return std::ptr::null_mut();
                                    },
                                }
                            }
                            array.into_raw()
                        },
                        Err(e) => {
                            log::error!("Failed to create object array: {}", e);
                            std::ptr::null_mut()
                        },
                    }
                },
                Err(e) => {
                    log::error!("Failed to find String class: {}", e);
                    std::ptr::null_mut()
                },
            }
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
            std::ptr::null_mut()
        },
    }
}

/// Finds all image elements on the page
/// Java signature: `private static native int nativeFindImagesCount(long ptr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_dom_PdfPage_nativeFindImagesCount(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
) -> jint {
    match with_page_mut(ptr, |page| page.find_images()) {
        Ok(images) => images.len() as jint,
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
            -1
        },
    }
}

/// Finds all path elements on the page
/// Java signature: `private static native int nativeFindPathsCount(long ptr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_dom_PdfPage_nativeFindPathsCount(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
) -> jint {
    match with_page_mut(ptr, |page| page.find_paths()) {
        Ok(paths) => paths.len() as jint,
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
            -1
        },
    }
}

/// Finds all table elements on the page
/// Java signature: `private static native int nativeFindTablesCount(long ptr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_dom_PdfPage_nativeFindTablesCount(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
) -> jint {
    match with_page_mut(ptr, |page| page.find_tables()) {
        Ok(tables) => tables.len() as jint,
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
            -1
        },
    }
}

/// Sets text content of a specific element by ID
/// Java signature: `private static native void nativeSetText(long ptr, String elementId, String text)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_dom_PdfPage_nativeSetText(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    element_id: JString,
    text: JString,
) {
    let _element_id_str: String = match env.get_string(&element_id) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid element ID encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read element ID: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return;
        },
    };

    let text_str: String = match env.get_string(&text) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid text encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read text: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return;
        },
    };

    match with_page_mut(ptr, |page| {
        // TODO: Implement element ID lookup and text setting
        // For now, this is a placeholder
        Ok::<(), String>(())
    }) {
        Ok(_) => {},
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
        },
    }
}

/// Removes an element from the page by ID
/// Java signature: `private static native void nativeRemoveElement(long ptr, String elementId)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_dom_PdfPage_nativeRemoveElement(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    element_id: JString,
) {
    let _element_id_str: String = match env.get_string(&element_id) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid element ID encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return;
            },
        },
        Err(e) => {
            let msg = format!("Failed to read element ID: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return;
        },
    };

    match with_page_mut(ptr, |page| {
        // TODO: Implement element removal
        // For now, this is a placeholder
        Ok::<(), String>(())
    }) {
        Ok(_) => {},
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
        },
    }
}

/// Adds a new text element to the page
/// Java signature: `private static native String nativeAddText(long ptr, String text, float x, float y, float fontSize, String fontName)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_dom_PdfPage_nativeAddText(
    mut env: JNIEnv,
    _class: JClass,
    ptr: u64,
    text: JString,
    _x: jni::sys::jfloat,
    _y: jni::sys::jfloat,
    _font_size: jni::sys::jfloat,
    font_name: JString,
) -> jobject {
    let _text_str: String = match env.get_string(&text) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid text encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return std::ptr::null_mut();
            },
        },
        Err(e) => {
            let msg = format!("Failed to read text: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return std::ptr::null_mut();
        },
    };

    let _font_name_str: String = match env.get_string(&font_name) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid font name encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return std::ptr::null_mut();
            },
        },
        Err(e) => {
            let msg = format!("Failed to read font name: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return std::ptr::null_mut();
        },
    };

    match with_page_mut(ptr, |_page| {
        // TODO: Implement text element creation
        // For now, return a random element ID
        let element_id = uuid::Uuid::new_v4();
        element_id.to_string()
    }) {
        Ok(id_str) => match env.new_string(&id_str) {
            Ok(jstr) => jstr.into_raw(),
            Err(e) => {
                log::error!("Failed to create element ID string: {}", e);
                std::ptr::null_mut()
            },
        },
        Err(e) => {
            crate::jni::exceptions::throw_pdf_exception(env, &e);
            std::ptr::null_mut()
        },
    }
}
