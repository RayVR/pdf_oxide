use napi_derive::napi;

/// PDF page sizes
#[napi]
#[derive(Clone, Copy, Debug)]
pub enum PageSize {
  /// Letter: 8.5" × 11"
  Letter,
  /// A4: 210mm × 297mm
  A4,
  /// A3: 297mm × 420mm
  A3,
  /// Legal: 8.5" × 14"
  Legal,
  /// Ledger: 11" × 17"
  Ledger,
  /// Tabloid: 11" × 17"
  Tabloid,
}

/// Geometry - Rectangle
#[napi]
#[derive(Clone, Copy, Debug)]
pub struct Rect {
  /// Left position
  pub x: f32,
  /// Top position
  pub y: f32,
  /// Width
  pub width: f32,
  /// Height
  pub height: f32,
}

/// Geometry - Point
#[napi]
#[derive(Clone, Copy, Debug)]
pub struct Point {
  /// X coordinate
  pub x: f32,
  /// Y coordinate
  pub y: f32,
}

/// Geometry - Color (RGBA)
#[napi]
#[derive(Clone, Copy, Debug)]
pub struct Color {
  /// Red component (0-255)
  pub r: u8,
  /// Green component (0-255)
  pub g: u8,
  /// Blue component (0-255)
  pub b: u8,
  /// Alpha component (0-255, 255 = opaque)
  pub a: u8,
}

/// PDF creation configuration
#[napi]
#[derive(Clone, Debug)]
pub struct PdfConfig {
  /// PDF title
  pub title: Option<String>,
  /// PDF author
  pub author: Option<String>,
  /// PDF subject
  pub subject: Option<String>,
  /// Page size (default: A4)
  pub page_size: Option<String>,
  /// Top margin in points
  pub margin_top: Option<f32>,
  /// Right margin in points
  pub margin_right: Option<f32>,
  /// Bottom margin in points
  pub margin_bottom: Option<f32>,
  /// Left margin in points
  pub margin_left: Option<f32>,
}

/// Text and format conversion options
#[napi]
#[derive(Clone, Debug)]
pub struct ConversionOptions {
  /// Detect headings from font size
  pub detect_headings: Option<bool>,
  /// Preserve visual layout
  pub preserve_layout: Option<bool>,
  /// Include images in output
  pub include_images: Option<bool>,
  /// Directory to save extracted images
  pub image_output_dir: Option<String>,
  /// Embed images as base64 data URIs
  pub embed_images: Option<bool>,
}

/// Search options for text search
#[napi]
#[derive(Clone, Debug)]
pub struct SearchOptions {
  /// Case-sensitive search
  pub case_sensitive: Option<bool>,
  /// Whole words only
  pub whole_words: Option<bool>,
  /// Use regular expressions
  pub regex: Option<bool>,
}

/// Search result with position information
#[napi]
#[derive(Clone, Debug)]
pub struct SearchResult {
  /// Matched text
  pub text: String,
  /// Page index
  pub page_index: i32,
  /// Bounding box
  pub bbox: Rect,
  /// Confidence (0.0 - 1.0)
  pub confidence: Option<f32>,
}

/// Element content for adding new elements
#[napi]
#[derive(Clone, Debug)]
pub struct ElementContent {
  /// Element type: "text", "image", "path", "table"
  pub element_type: String,
  /// Element data (JSON encoded)
  pub data: String,
}

/// Annotation content for adding new annotations
#[napi]
#[derive(Clone, Debug)]
pub struct AnnotationContent {
  /// Annotation type: "text", "highlight", "link", etc.
  pub annotation_type: String,
  /// Annotation data (JSON encoded)
  pub data: String,
}
