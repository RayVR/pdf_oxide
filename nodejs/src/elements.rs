use napi_derive::napi;
use crate::types::Rect;

/// PDF element types (discriminated union)
#[napi]
#[derive(Clone, Debug)]
pub enum PdfElement {
  Text,
  Image,
  Path,
  Table,
  Structure,
}

/// Text element for content on a PDF page
///
/// Represents textual content with formatting information.
#[napi]
#[derive(Clone, Debug)]
pub struct PdfText {
  /// Unique element identifier
  pub id: String,
  /// Text content
  pub text: String,
  /// Position and size
  pub bbox: Rect,
  /// Font size in points
  pub font_size: f32,
  /// Font name
  pub font: String,
  /// RGB color values (0-255 each)
  pub color_r: u8,
  pub color_g: u8,
  pub color_b: u8,
}

/// Image element for raster content on a PDF page
///
/// Represents images (JPEG, PNG, etc.) embedded in the PDF.
#[napi]
#[derive(Clone, Debug)]
pub struct PdfImage {
  /// Unique element identifier
  pub id: String,
  /// Position and size
  pub bbox: Rect,
  /// Image width in pixels
  pub width: i32,
  /// Image height in pixels
  pub height: i32,
  /// Image data format (JPEG, PNG, etc.)
  pub format: String,
  /// Approximate file size of embedded image
  pub size: i32,
}

/// Path element for vector graphics on a PDF page
///
/// Represents drawn paths, shapes, and other vector content.
#[napi]
#[derive(Clone, Debug)]
pub struct PdfPath {
  /// Unique element identifier
  pub id: String,
  /// Bounding box of the path
  pub bbox: Rect,
  /// Stroke color RGB (0-255 each)
  pub stroke_r: u8,
  pub stroke_g: u8,
  pub stroke_b: u8,
  /// Fill color RGB (0-255 each)
  pub fill_r: u8,
  pub fill_g: u8,
  pub fill_b: u8,
  /// Stroke width in points
  pub stroke_width: f32,
}

/// Table element for tabular data
///
/// Represents structured tables within a PDF page.
#[napi]
#[derive(Clone, Debug)]
pub struct PdfTable {
  /// Unique element identifier
  pub id: String,
  /// Position and size
  pub bbox: Rect,
  /// Number of rows
  pub rows: i32,
  /// Number of columns
  pub cols: i32,
  /// Cell border color RGB (0-255 each)
  pub border_r: u8,
  pub border_g: u8,
  pub border_b: u8,
}

/// Structure element for tagged PDF hierarchy
///
/// Represents semantic structure (headings, paragraphs, etc.) in tagged PDFs.
#[napi]
#[derive(Clone, Debug)]
pub struct PdfStructure {
  /// Unique element identifier
  pub id: String,
  /// Structure type (H1, H2, P, DIV, SPAN, etc.)
  pub structure_type: String,
  /// Position if applicable
  pub bbox: Option<Rect>,
  /// Parent structure ID
  pub parent_id: Option<String>,
  /// Human-readable label
  pub label: Option<String>,
}

/// Implements common operations on PdfText
#[napi]
impl PdfText {
  /// Gets the text content
  #[napi]
  pub fn text(&self) -> String {
    self.text.clone()
  }

  /// Gets the bounding box
  #[napi]
  pub fn bounding_box(&self) -> Rect {
    self.bbox
  }

  /// Gets the font name
  #[napi]
  pub fn font_name(&self) -> String {
    self.font.clone()
  }

  /// Gets the font size
  #[napi]
  pub fn font_size_points(&self) -> f32 {
    self.font_size
  }
}

/// Implements common operations on PdfImage
#[napi]
impl PdfImage {
  /// Gets the image dimensions
  #[napi]
  pub fn dimensions(&self) -> (i32, i32) {
    (self.width, self.height)
  }

  /// Gets the bounding box
  #[napi]
  pub fn bounding_box(&self) -> Rect {
    self.bbox
  }

  /// Gets the image format
  #[napi]
  pub fn format(&self) -> String {
    self.format.clone()
  }
}

/// Implements common operations on PdfPath
#[napi]
impl PdfPath {
  /// Gets the bounding box
  #[napi]
  pub fn bounding_box(&self) -> Rect {
    self.bbox
  }

  /// Gets the stroke width
  #[napi]
  pub fn stroke_width_points(&self) -> f32 {
    self.stroke_width
  }
}

/// Implements common operations on PdfTable
#[napi]
impl PdfTable {
  /// Gets the table dimensions (rows x cols)
  #[napi]
  pub fn dimensions(&self) -> (i32, i32) {
    (self.rows, self.cols)
  }

  /// Gets the bounding box
  #[napi]
  pub fn bounding_box(&self) -> Rect {
    self.bbox
  }
}

/// Implements common operations on PdfStructure
#[napi]
impl PdfStructure {
  /// Gets the structure type
  #[napi]
  pub fn struct_type(&self) -> String {
    self.structure_type.clone()
  }

  /// Gets the bounding box if available
  #[napi]
  pub fn bounding_box(&self) -> Option<Rect> {
    self.bbox
  }
}
