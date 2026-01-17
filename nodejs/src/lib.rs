#![allow(non_snake_case)]

use napi_derive::napi;

mod document;
mod pdf;
mod builder;
mod page;
mod elements;
mod annotations;
mod search;
mod types;
mod errors;
mod utils;
mod dom;
mod forms;
mod metadata;

// Re-export main classes for napi
pub use document::PdfDocument;
pub use pdf::Pdf;
pub use builder::PdfBuilder;
pub use page::PdfPage;
pub use errors::{PdfError, PdfIoError, PdfParseError};
pub use dom::AnnotationData;
pub use annotations::{
  AnnotationType,
  TextAnnotation,
  LinkAnnotation,
  FreeTextAnnotation,
  LineAnnotation,
  SquareAnnotation,
  CircleAnnotation,
  PolygonAnnotation,
  PolyLineAnnotation,
  HighlightAnnotation,
  UnderlineAnnotation,
  SquigglyAnnotation,
  StrikeOutAnnotation,
  StampAnnotation,
  CaretAnnotation,
  InkAnnotation,
  PopupAnnotation,
  FileAttachmentAnnotation,
  SoundAnnotation,
  RedactAnnotation,
  WidgetAnnotation,
  ScreenAnnotation,
  ThreeDAnnotation,
  WatermarkAnnotation,
};
pub use search::{TextSearcher, TextSearchResult};
pub use forms::{
  FormFieldType,
  FormField,
  TextFormField,
  CheckboxField,
  RadioButtonField,
  ListField,
  ButtonField,
  SignatureField,
  AcroForm,
  XFAForm,
  FormSubmission,
  FormReset,
};
pub use metadata::{
  XMPMetadata,
  PageLabel,
  EmbeddedFile,
  DocumentInfo,
};

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
