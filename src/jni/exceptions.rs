//! Exception mapping from Rust errors to Java exceptions.
//!
//! This module provides utilities to throw appropriate Java exceptions from Rust code
//! when errors occur during JNI operations.

use crate::Error;
use jni::JNIEnv;

/// Exception class paths in Java
const PDFEXCEPTION: &str = "com/pdfoxide/exceptions/PdfException";
const IOEXCEPTION: &str = "com/pdfoxide/exceptions/IoException";
const PARSEEXCEPTION: &str = "com/pdfoxide/exceptions/ParseException";
const INVALIDSTATEEXCEPTION: &str = "com/pdfoxide/exceptions/InvalidStateException";
const ENCRYPTIONEXCEPTION: &str = "com/pdfoxide/exceptions/EncryptionException";
const UNSUPPORTEDFEATUREEXCEPTION: &str = "com/pdfoxide/exceptions/UnsupportedFeatureException";

/// Determines the appropriate Java exception class and message for a pdf_oxide error
fn categorize_error(error: &Error) -> (&'static str, String) {
    let error_msg = error.to_string();

    // Categorize based on error message patterns
    if error_msg.contains("encrypt")
        || error_msg.contains("password")
        || error_msg.contains("decrypt")
    {
        (ENCRYPTIONEXCEPTION, error_msg)
    } else if error_msg.contains("parse")
        || error_msg.contains("invalid pdf")
        || error_msg.contains("unexpected token")
    {
        (PARSEEXCEPTION, error_msg)
    } else if error_msg.contains("io error")
        || error_msg.contains("file not found")
        || error_msg.contains("permission denied")
    {
        (IOEXCEPTION, error_msg)
    } else if error_msg.contains("closed") || error_msg.contains("already") {
        (INVALIDSTATEEXCEPTION, error_msg)
    } else if error_msg.contains("not available") || error_msg.contains("feature") {
        (UNSUPPORTEDFEATUREEXCEPTION, error_msg)
    } else {
        (PDFEXCEPTION, error_msg)
    }
}

/// Throws a Java exception from a Rust error
///
/// # Safety
/// This function must only be called from JNI methods with a valid JNIEnv
pub fn throw_exception(mut env: JNIEnv, error: Error) -> jni::sys::jint {
    let (exception_class, message) = categorize_error(&error);

    if let Err(e) = env.throw_new(exception_class, message) {
        eprintln!("Failed to throw Java exception: {}", e);
        return jni::sys::JNI_ERR;
    }

    jni::sys::JNI_OK
}

/// Throws a PdfException with a custom message
pub fn throw_pdf_exception(mut env: JNIEnv, message: &str) -> jni::sys::jint {
    if let Err(e) = env.throw_new(PDFEXCEPTION, message) {
        eprintln!("Failed to throw PdfException: {}", e);
        return jni::sys::JNI_ERR;
    }
    jni::sys::JNI_OK
}

/// Throws an IoException with a custom message
pub fn throw_io_exception(mut env: JNIEnv, message: &str) -> jni::sys::jint {
    if let Err(e) = env.throw_new(IOEXCEPTION, message) {
        eprintln!("Failed to throw IoException: {}", e);
        return jni::sys::JNI_ERR;
    }
    jni::sys::JNI_OK
}

/// Throws a ParseException with a custom message
pub fn throw_parse_exception(mut env: JNIEnv, message: &str) -> jni::sys::jint {
    if let Err(e) = env.throw_new(PARSEEXCEPTION, message) {
        eprintln!("Failed to throw ParseException: {}", e);
        return jni::sys::JNI_ERR;
    }
    jni::sys::JNI_OK
}

/// Throws an InvalidStateException with a custom message
pub fn throw_invalid_state_exception(mut env: JNIEnv, message: &str) -> jni::sys::jint {
    if let Err(e) = env.throw_new(INVALIDSTATEEXCEPTION, message) {
        eprintln!("Failed to throw InvalidStateException: {}", e);
        return jni::sys::JNI_ERR;
    }
    jni::sys::JNI_OK
}

/// Throws an EncryptionException with a custom message
pub fn throw_encryption_exception(mut env: JNIEnv, message: &str) -> jni::sys::jint {
    if let Err(e) = env.throw_new(ENCRYPTIONEXCEPTION, message) {
        eprintln!("Failed to throw EncryptionException: {}", e);
        return jni::sys::JNI_ERR;
    }
    jni::sys::JNI_OK
}

/// Throws an UnsupportedFeatureException for a missing feature
pub fn throw_unsupported_feature_exception(mut env: JNIEnv, feature: &str) -> jni::sys::jint {
    let message =
        format!("Feature not available: {}. Rebuild with appropriate feature flag.", feature);
    if let Err(e) = env.throw_new(UNSUPPORTEDFEATUREEXCEPTION, message) {
        eprintln!("Failed to throw UnsupportedFeatureException: {}", e);
        return jni::sys::JNI_ERR;
    }
    jni::sys::JNI_OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categorization() {
        let encryption_error = Error::Parsing("encryption failed".to_string());
        let (class, _msg) = categorize_error(&encryption_error);
        // Should categorize as either encryption or parse error
        assert!(class == PARSEEXCEPTION || class == ENCRYPTIONEXCEPTION);

        let io_error = Error::Parsing("file not found".to_string());
        let (class, _msg) = categorize_error(&io_error);
        assert_eq!(class, IOEXCEPTION);
    }
}
