//! Java Native Interface (JNI) bindings for Java/Kotlin integration.
//!
//! This module provides JNI bindings that expose the pdf_oxide Rust API to Java/Kotlin
//! applications via Java Native Interface. All bindings are gated behind the `java` feature flag.
//!
//! # Architecture
//!
//! - **Memory Management**: Native pointers wrapped in Java NativeHandle with Cleaner API
//! - **Exception Handling**: Rust errors mapped to Java checked exceptions
//! - **Thread Safety**: All JNI calls validate thread attachment to JVM
//! - **Type Mapping**: Rust types converted to idiomatic Java equivalents
//!
//! # Submodules
//!
//! - `exceptions` - Error mapping from Rust Result to Java exceptions
//! - `utils` - JNI utility functions (string conversion, array handling, etc.)
//! - `pdf_document` - PdfDocument read API bindings
//! - `pdf` - Universal Pdf API bindings (read/create/edit)
//! - `dom` - DOM element navigation bindings
//! - `annotations` - Annotation creation and management bindings
//! - `forms` - Form field creation and management bindings
//! - `search` - Text search bindings
//! - `compliance` - PDF/A compliance validation bindings
//! - `signatures` - Digital signature bindings
//! - `document_editor` - DocumentEditor edit API bindings (future)
//! - `geometry` - Geometry types (Rect, Point, Color) (future)
//! - `conversion` - Format conversion options (future)
//!
//! # JNI Method Naming Convention
//!
//! All native methods follow the pattern: `native_<rust_api_method_name>`
//! For example: `PdfDocument.open()` → `nativeOpen(String path)`

pub mod annotations;
pub mod compliance;
pub mod dom;
pub mod exceptions;
pub mod forms;
pub mod pdf;
pub mod pdf_document;
pub mod search;
pub mod signatures;
pub mod utils;

// Future modules (placeholders for Phase 1-8 implementation)
// pub mod document_editor;
// pub mod geometry;
// pub mod conversion;

use jni::objects::JClass;
use jni::JNIEnv;

/// JNI version check - ensures minimum JNI version compatibility
#[no_mangle]
pub extern "system" fn JNI_OnLoad(
    vm: *mut jni::sys::JavaVM,
    _reserved: *mut std::ffi::c_void,
) -> jni::sys::jint {
    // Initialize JNI runtime
    // SAFETY: This is called by the JVM with a valid JavaVM pointer
    match unsafe { jni::JavaVM::from_raw(vm) } {
        Ok(_jvm) => {
            // Future: Initialize thread-local JVM handle
            log::info!("pdf_oxide JNI initialized successfully");
            jni::sys::JNI_VERSION_1_8
        },
        Err(e) => {
            eprintln!("Failed to initialize pdf_oxide JNI: {}", e);
            jni::sys::JNI_ERR
        },
    }
}

/// JNI unload handler - cleanup resources
#[no_mangle]
pub extern "system" fn JNI_OnUnload(_vm: *mut jni::sys::JavaVM, _reserved: *mut std::ffi::c_void) {
    log::info!("pdf_oxide JNI unloading");
    // Future: Cleanup thread-local resources
}

/// Returns the native library version
/// Java signature: `public static native String nativeGetVersion()`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_util_NativeLibraryLoader_nativeGetVersion(
    mut env: JNIEnv,
    _class: JClass,
) -> jni::sys::jobject {
    let version = crate::VERSION;
    match env.new_string(version) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Returns the native library name
/// Java signature: `public static native String nativeGetLibraryName()`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_util_NativeLibraryLoader_nativeGetLibraryName(
    mut env: JNIEnv,
    _class: JClass,
) -> jni::sys::jobject {
    let name = crate::NAME;
    match env.new_string(name) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Checks if an optional feature is available at runtime
/// Java signature: `public static native boolean nativeHasFeature(String feature)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_util_FeatureDetection_nativeHasFeature(
    mut env: JNIEnv,
    _class: JClass,
    feature: jni::objects::JString,
) -> jni::sys::jboolean {
    let feature_name: String = match env.get_string(&feature) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return 0,
        },
        Err(_) => return 0,
    };

    let has_feature = match feature_name.as_str() {
        #[cfg(feature = "ocr")]
        "ocr" => true,
        #[cfg(feature = "rendering")]
        "rendering" => true,
        #[cfg(feature = "signatures")]
        "signatures" => true,
        #[cfg(feature = "ml")]
        "ml" => true,
        #[cfg(feature = "gpu")]
        "gpu" => true,
        #[cfg(feature = "barcodes")]
        "barcodes" => true,
        #[cfg(feature = "office")]
        "office" => true,
        _ => false,
    };

    if has_feature {
        1
    } else {
        0
    }
}
