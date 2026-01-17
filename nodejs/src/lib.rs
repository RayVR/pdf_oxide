#![allow(non_snake_case)]

use napi_derive::napi;

mod annotations;
mod builder;
mod document;
mod dom;
mod elements;
mod errors;
mod forms;
mod metadata;
mod page;
mod pdf;
mod search;
mod types;
mod utils;

// Re-export main classes for napi
pub use annotations::{
    AnnotationType, CaretAnnotation, CircleAnnotation, FileAttachmentAnnotation,
    FreeTextAnnotation, HighlightAnnotation, InkAnnotation, LineAnnotation, LinkAnnotation,
    PolyLineAnnotation, PolygonAnnotation, PopupAnnotation, RedactAnnotation, ScreenAnnotation,
    SoundAnnotation, SquareAnnotation, SquigglyAnnotation, StampAnnotation, StrikeOutAnnotation,
    TextAnnotation, ThreeDAnnotation, UnderlineAnnotation, WatermarkAnnotation, WidgetAnnotation,
};
pub use builder::PdfBuilder;
pub use document::PdfDocument;
pub use dom::AnnotationData;
pub use errors::{PdfError, PdfIoError, PdfParseError};
pub use forms::{
    AcroForm, ButtonField, CheckboxField, FormField, FormFieldType, FormReset, FormSubmission,
    ListField, RadioButtonField, SignatureField, TextFormField, XFAForm,
};
pub use metadata::{DocumentInfo, EmbeddedFile, PageLabel, XMPMetadata};
pub use page::PdfPage;
pub use pdf::Pdf;
pub use search::{TextSearchResult, TextSearcher};

/// pdf_oxide Node.js bindings
///
/// Complete Node.js/TypeScript bindings for the pdf_oxide Rust library.
/// Exposes all 4 interfaces: read (PdfDocument), create (Pdf), edit (Pdf), universal (PdfBuilder)
#[napi]
pub fn get_version() -> String {
    "1.0.0".to_string()
}

#[napi]
pub fn get_pdf_oxide_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
