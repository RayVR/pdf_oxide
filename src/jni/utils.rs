//! JNI utility functions for common operations.
//!
//! This module provides helper functions for:
//! - String conversion between Rust and Java
//! - Array marshaling
//! - Type conversions
//! - Memory management utilities

use jni::JNIEnv;

/// Converts a Java String to a Rust String
///
/// # Errors
/// Returns an error if the JString is invalid or cannot be converted to UTF-8
pub fn java_string_to_rust(
    mut env: JNIEnv,
    jstring: jni::objects::JString,
) -> Result<String, Box<dyn std::error::Error>> {
    let java_string = env.get_string(&jstring)?;
    let s = java_string.to_str()?.to_string();
    Ok(s)
}

/// Converts a Rust &str to a Java String
///
/// # Errors
/// Returns an error if the string cannot be encoded or JNI operation fails
pub fn rust_string_to_java(
    mut env: JNIEnv,
    s: &str,
) -> Result<jni::sys::jobject, jni::errors::Error> {
    match env.new_string(s) {
        Ok(jstring) => Ok(jstring.into_raw()),
        Err(e) => Err(e),
    }
}

/// Helper macro to unwrap Results with exception handling
/// If Result is Err, throws the exception and returns the specified value
#[macro_export]
macro_rules! jni_try {
    ($env:expr, $result:expr, $return_value:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => {
                $crate::jni::exceptions::throw_exception($env, e);
                return $return_value;
            },
        }
    };
}

/// Helper macro to unwrap Option with exception handling
#[macro_export]
macro_rules! jni_some_or_else {
    ($env:expr, $option:expr, $return_value:expr, $error_msg:expr) => {
        match $option {
            Some(v) => v,
            None => {
                $crate::jni::exceptions::throw_pdf_exception($env, $error_msg);
                return $return_value;
            },
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_conversion() {
        // These tests would need a JNI environment to run
        // They're more for documentation purposes
    }
}
