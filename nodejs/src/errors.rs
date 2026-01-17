use napi::{Error as NapiError, Status};
use pdf_oxide::error::Error as PdfError;

/// Base PDF error class
#[napi]
pub struct PdfError {
    pub code: String,
    pub message: String,
}

/// I/O errors (file not found, cannot be opened)
#[napi]
pub struct PdfIoError {
    pub code: String,
    pub message: String,
}

/// Parse errors (invalid PDF structure)
#[napi]
pub struct PdfParseError {
    pub code: String,
    pub message: String,
    pub offset: Option<u32>,
}

/// Encryption errors (password incorrect)
#[napi]
pub struct PdfEncryptionError {
    pub code: String,
    pub message: String,
}

/// Unsupported feature errors
#[napi]
pub struct PdfUnsupportedError {
    pub code: String,
    pub message: String,
}

/// Invalid state errors
#[napi]
pub struct PdfInvalidStateError {
    pub code: String,
    pub message: String,
}

/// Decode errors
#[napi]
pub struct PdfDecodeError {
    pub code: String,
    pub message: String,
}

/// Encode errors
#[napi]
pub struct PdfEncodeError {
    pub code: String,
    pub message: String,
}

/// Font errors
#[napi]
pub struct PdfFontError {
    pub code: String,
    pub message: String,
}

/// Image errors
#[napi]
pub struct PdfImageError {
    pub code: String,
    pub message: String,
}

/// Circular reference errors
#[napi]
pub struct PdfCircularReferenceError {
    pub code: String,
    pub message: String,
}

/// Recursion limit errors
#[napi]
pub struct PdfRecursionLimitError {
    pub code: String,
    pub message: String,
}

/// OCR errors (optional feature)
#[napi]
pub struct PdfOcrError {
    pub code: String,
    pub message: String,
}

/// ML errors (optional feature)
#[napi]
pub struct PdfMlError {
    pub code: String,
    pub message: String,
}

/// Barcode errors
#[napi]
pub struct PdfBarcodeError {
    pub code: String,
    pub message: String,
}

/// Map Rust pdf_oxide errors to napi errors
pub fn map_error(err: PdfError) -> NapiError {
    let (code, message) = match err {
        PdfError::InvalidHeader(header) => (
            "INVALID_HEADER",
            format!("Invalid PDF header: expected '%PDF-', found '{}'", header),
        ),
        PdfError::UnsupportedVersion(version) => {
            ("UNSUPPORTED_VERSION", format!("Unsupported PDF version: {}", version))
        },
        PdfError::ParseError { offset, reason } => {
            ("PARSE_ERROR", format!("Failed to parse object at byte {}: {}", offset, reason))
        },
        PdfError::ParseWarning {
            offset,
            message: msg,
        } => ("PARSE_WARNING", format!("Parse warning at byte {}: {}", offset, msg)),
        PdfError::InvalidXref => ("INVALID_XREF", "Invalid cross-reference table".to_string()),
        PdfError::ObjectNotFound(obj, gen) => {
            ("OBJECT_NOT_FOUND", format!("Object not found: {} {} R", obj, gen))
        },
        PdfError::InvalidObjectType { expected, found } => (
            "INVALID_OBJECT_TYPE",
            format!("Invalid object type: expected {}, found {}", expected, found),
        ),
        PdfError::UnexpectedEof => {
            ("UNEXPECTED_EOF", "End of file reached unexpectedly".to_string())
        },
        PdfError::Io(io_err) => ("IO_ERROR", format!("IO error: {}", io_err)),
        PdfError::Utf8Error(utf8_err) => {
            ("UTF8_ERROR", format!("UTF-8 decoding error: {}", utf8_err))
        },
        PdfError::Unsupported(feature) => {
            ("UNSUPPORTED", format!("Unsupported feature: {}", feature))
        },
        PdfError::InvalidPdf(reason) => ("INVALID_PDF", format!("Invalid PDF: {}", reason)),
        PdfError::Decode(reason) => ("DECODE_ERROR", format!("Stream decoding error: {}", reason)),
        PdfError::Encode(reason) => ("ENCODE_ERROR", format!("Encoding error: {}", reason)),
        PdfError::UnsupportedFilter(filter) => {
            ("UNSUPPORTED_FILTER", format!("Unsupported filter: {}", filter))
        },
        PdfError::Font(reason) => ("FONT_ERROR", format!("Font error: {}", reason)),
        PdfError::Image(reason) => ("IMAGE_ERROR", format!("Image error: {}", reason)),
        #[cfg(feature = "ml")]
        PdfError::Ml(reason) => ("ML_ERROR", format!("ML error: {}", reason)),
        #[cfg(feature = "ocr")]
        PdfError::Ocr(reason) => ("OCR_ERROR", format!("OCR error: {}", reason)),
        PdfError::CircularReference(obj_ref) => {
            ("CIRCULAR_REFERENCE", format!("Circular reference detected: {:?}", obj_ref))
        },
        PdfError::RecursionLimitExceeded(limit) => (
            "RECURSION_LIMIT_EXCEEDED",
            format!("Recursion depth limit exceeded (max: {})", limit),
        ),
        PdfError::InvalidOperation(reason) => {
            ("INVALID_OPERATION", format!("Invalid operation: {}", reason))
        },
        PdfError::Barcode(reason) => ("BARCODE_ERROR", format!("Barcode error: {}", reason)),
    };

    NapiError::new(Status::GenericFailure, format!("[{}] {}", code, message))
}
