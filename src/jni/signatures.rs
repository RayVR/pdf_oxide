//! Digital Signatures API JNI bindings.
//!
//! This module implements native methods for working with digital signatures
//! in PDF documents (foundation for v0.3.0, full implementation v0.4.0+).

use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jboolean, jint, jobject};
use jni::JNIEnv;

use std::collections::HashMap;
use std::sync::Mutex;

// Thread-local signature cache
thread_local! {
    static SIGNATURE_CACHE: Mutex<HashMap<u64, Vec<String>>> = Mutex::new(HashMap::new());
}

static mut NEXT_SIGNATURE_ID: u64 = 1;

/// Stores signature data and returns a unique ID
fn store_signature(document_ptr: u64, signature_data: String) -> String {
    let id = unsafe {
        NEXT_SIGNATURE_ID += 1;
        format!("sig_{}", NEXT_SIGNATURE_ID)
    };

    SIGNATURE_CACHE.with(|cache| {
        let mut sigs = cache.lock().unwrap();
        let doc_sigs = sigs.entry(document_ptr).or_insert_with(Vec::new);
        doc_sigs.push(format!("{}:{}", id, signature_data));
    });

    id
}

/// Gets signature data by ID
fn get_signature(document_ptr: u64, signature_id: &str) -> Option<String> {
    SIGNATURE_CACHE.with(|cache| {
        let sigs = cache.lock().unwrap();
        sigs.get(&document_ptr).and_then(|sig_list| {
            sig_list
                .iter()
                .find(|s| s.starts_with(&format!("{}:", signature_id)))
                .and_then(|s| s.split(':').nth(1).map(String::from))
        })
    })
}

/// Clears all signatures for a document
fn clear_signatures(document_ptr: u64) {
    SIGNATURE_CACHE.with(|cache| {
        cache.lock().unwrap().remove(&document_ptr);
    });
}

// ===== Digital Signatures =====

/// Gets the number of signatures in a document
/// Java signature: `public static native int nativeGetSignatureCount(long documentPtr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_security_DigitalSignature_nativeGetSignatureCount(
    _env: JNIEnv,
    _class: JClass,
    document_ptr: u64,
) -> jint {
    SIGNATURE_CACHE.with(|cache| {
        cache
            .lock()
            .unwrap()
            .get(&document_ptr)
            .map(|sigs| sigs.len() as jint)
            .unwrap_or(0)
    })
}

/// Gets signature information by index
/// Java signature: `public static native DigitalSignature nativeGetSignature(long documentPtr, int index)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_security_DigitalSignature_nativeGetSignature(
    mut env: JNIEnv,
    _class: JClass,
    document_ptr: u64,
    index: jint,
) -> jobject {
    let signatures = SIGNATURE_CACHE.with(|cache| {
        cache
            .lock()
            .unwrap()
            .get(&document_ptr)
            .cloned()
            .unwrap_or_default()
    });

    if index < 0 || index as usize >= signatures.len() {
        return std::ptr::null_mut();
    }

    // Extract signature data
    if let Some(sig_data) = signatures.get(index as usize) {
        if let Some(data) = sig_data.split(':').nth(1) {
            // Create signature object with mock data
            match env.new_string(format!("sig_{}", index)) {
                Ok(s) => s.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        } else {
            std::ptr::null_mut()
        }
    } else {
        std::ptr::null_mut()
    }
}

/// Adds a signature to a document (foundation only in v0.3.0)
/// Java signature: `public static native String nativeAddSignature(long documentPtr, int page, SignatureConfig config)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_security_DigitalSignature_nativeAddSignature(
    mut env: JNIEnv,
    _class: JClass,
    document_ptr: u64,
    page: jint,
    _config: JByteArray,
) -> jobject {
    // Foundation: just store signature metadata in v0.3.0
    let sig_data =
        format!(r#"{{"page":{},"signer":"(unsigned)","date":"","state":"UNKNOWN"}}"#, page);

    let sig_id = store_signature(document_ptr, sig_data);

    match env.new_string(&sig_id) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Gets signature validity status
/// Java signature: `public static native boolean nativeIsSignatureValid(long documentPtr, String signatureId)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_security_DigitalSignature_nativeIsSignatureValid(
    _env: JNIEnv,
    _class: JClass,
    document_ptr: u64,
    signature_id: JString,
) -> jboolean {
    // Foundation: always return false in v0.3.0 (actual validation in v0.4.0+)
    0
}

/// Gets certificate information from signature
/// Java signature: `public static native CertificateInfo nativeGetCertificate(long documentPtr, String signatureId)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_security_DigitalSignature_nativeGetCertificate(
    mut env: JNIEnv,
    _class: JClass,
    document_ptr: u64,
    signature_id: JString,
) -> jobject {
    match env.get_string(&signature_id) {
        Ok(s) => match s.to_str() {
            Ok(sig_id) => {
                if get_signature(document_ptr, sig_id).is_some() {
                    match env.new_string("(not available in v0.3.0)") {
                        Ok(s) => s.into_raw(),
                        Err(_) => std::ptr::null_mut(),
                    }
                } else {
                    std::ptr::null_mut()
                }
            },
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Removes a signature from a document
/// Java signature: `public static native void nativeRemoveSignature(long documentPtr, String signatureId)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_security_DigitalSignature_nativeRemoveSignature(
    _env: JNIEnv,
    _class: JClass,
    document_ptr: u64,
    _signature_id: JString,
) {
    // Foundation: just clear all signatures in v0.3.0
    clear_signatures(document_ptr);
}

/// Clears all signatures for a document
/// Java signature: `public static native void nativeClearSignatures(long documentPtr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_security_DigitalSignature_nativeClearSignatures(
    _env: JNIEnv,
    _class: JClass,
    document_ptr: u64,
) {
    clear_signatures(document_ptr);
}

/// Native cleanup for signatures when document is freed
pub fn native_free_signatures(document_ptr: u64) {
    clear_signatures(document_ptr);
}
