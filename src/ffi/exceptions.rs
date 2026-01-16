//! Error code definitions for C# exception mapping
//!
//! Maps Rust errors to standardized error codes that C# can translate to typed exceptions.

use crate::Error;

/// Standard error codes for C# exception mapping
///
/// # Code Meanings
/// - 0: Success (no error)
/// - 1: I/O error (file not found, permission denied, etc.)
/// - 2: Parse error (invalid PDF structure, malformed content)
/// - 3: Encryption error (incorrect password, unsupported cipher)
/// - 4: Invalid state error (operation not allowed in current state)
/// - 5: Unsupported feature: rendering
/// - 6: Unsupported feature: OCR
/// - 100+: Generic/internal error
#[repr(i32)]
pub enum ErrorCode {
    /// No error occurred
    Success = 0,

    /// I/O error: File not found, permission denied, or read/write failed
    IoError = 1,

    /// Parse error: Invalid PDF structure or content stream
    ParseError = 2,

    /// Encryption error: Incorrect password or unsupported encryption
    EncryptionError = 3,

    /// Invalid state: Operation not allowed in current document state
    InvalidStateError = 4,

    /// Feature not enabled: Rendering support not compiled in
    RenderingUnsupported = 5,

    /// Feature not enabled: OCR support not compiled in
    OcrUnsupported = 6,

    /// Generic/unknown error
    InternalError = 100,
}

impl ErrorCode {
    /// Convert a value to ErrorCode, defaulting to InternalError for unknown codes
    pub fn from_i32(code: i32) -> Self {
        match code {
            0 => ErrorCode::Success,
            1 => ErrorCode::IoError,
            2 => ErrorCode::ParseError,
            3 => ErrorCode::EncryptionError,
            4 => ErrorCode::InvalidStateError,
            5 => ErrorCode::RenderingUnsupported,
            6 => ErrorCode::OcrUnsupported,
            _ => ErrorCode::InternalError,
        }
    }
}

/// Convert a Rust Error to a C-compatible error code
pub fn pdf_error_to_code(err: &Error) -> i32 {
    match err {
        Error::Io(_) => ErrorCode::IoError as i32,
        Error::ParseError { .. }
        | Error::InvalidPdf(_)
        | Error::InvalidHeader(_)
        | Error::InvalidXref
        | Error::InvalidObjectType { .. } => ErrorCode::ParseError as i32,
        Error::Unsupported(msg) => {
            if msg.contains("rendering") {
                ErrorCode::RenderingUnsupported as i32
            } else if msg.contains("ocr") {
                ErrorCode::OcrUnsupported as i32
            } else {
                ErrorCode::InternalError as i32
            }
        },
        Error::InvalidOperation(_) => ErrorCode::InvalidStateError as i32,
        _ => ErrorCode::InternalError as i32,
    }
}
