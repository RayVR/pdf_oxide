use crate::types::Rect;
use napi_derive::napi;

/// Annotation types - All 27 PDF annotation subtypes (ISO 32000-1:2008, Section 12.5)
#[napi]
#[derive(Clone, Debug)]
pub enum AnnotationType {
    /// Text annotation (sticky note)
    Text,
    /// Link annotation (hyperlink)
    Link,
    /// FreeText annotation (text box)
    FreeText,
    /// Line annotation (single straight line)
    Line,
    /// Square annotation (rectangle)
    Square,
    /// Circle annotation (ellipse)
    Circle,
    /// Polygon annotation (closed polygon)
    Polygon,
    /// PolyLine annotation (open polyline)
    PolyLine,
    /// Highlight annotation (text marking)
    Highlight,
    /// Underline annotation (text underline)
    Underline,
    /// Squiggly annotation (wavy underline)
    Squiggly,
    /// StrikeOut annotation (strikethrough text)
    StrikeOut,
    /// Stamp annotation (rubber stamp)
    Stamp,
    /// Caret annotation (text insertion marker)
    Caret,
    /// Ink annotation (freehand drawing)
    Ink,
    /// Popup annotation (pop-up window)
    Popup,
    /// FileAttachment annotation (embedded file)
    FileAttachment,
    /// Sound annotation (audio playback)
    Sound,
    /// Movie annotation (legacy video)
    Movie,
    /// Screen annotation (multimedia container)
    Screen,
    /// Widget annotation (form field)
    Widget,
    /// PrinterMark annotation (printer's mark)
    PrinterMark,
    /// TrapNet annotation (trap network)
    TrapNet,
    /// Watermark annotation (background)
    Watermark,
    /// Redact annotation (content removal)
    Redact,
    /// ThreeD annotation (3D model)
    ThreeD,
    /// RichMedia annotation (interactive content)
    RichMedia,
}

/// Text annotation (sticky note) - Section 12.5.6.4
#[napi]
#[derive(Clone, Debug)]
pub struct TextAnnotation {
    /// Unique identifier
    pub id: String,
    /// Position on page
    pub rect: Rect,
    /// Comment text content
    pub contents: Option<String>,
    /// Creator's name
    pub author: Option<String>,
    /// Subject line
    pub subject: Option<String>,
    /// Icon name: Comment, Key, Note, Help, NewParagraph, Paragraph, Insert
    pub icon_name: Option<String>,
    /// Color (RGB)
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    /// Whether popup is initially open
    pub open: Option<bool>,
}

/// Link annotation (hyperlink) - Section 12.5.6.5
#[napi]
#[derive(Clone, Debug)]
pub struct LinkAnnotation {
    /// Unique identifier
    pub id: String,
    /// Click region
    pub rect: Rect,
    /// External URI
    pub uri: Option<String>,
    /// Internal page destination (0-indexed)
    pub destination_page: Option<i32>,
    /// Open in new window/tab
    pub target_blank: Option<bool>,
}

/// FreeText annotation (text box) - Section 12.5.6.6
#[napi]
#[derive(Clone, Debug)]
pub struct FreeTextAnnotation {
    /// Unique identifier
    pub id: String,
    /// Text box region
    pub rect: Rect,
    /// Text content
    pub contents: String,
    /// Font name (Helvetica, Times-Roman, Courier, etc.)
    pub font_name: Option<String>,
    /// Font size in points
    pub font_size: f32,
    /// Text color (RGB)
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    /// Background color (RGB) - optional
    pub background_color_r: Option<u8>,
    pub background_color_g: Option<u8>,
    pub background_color_b: Option<u8>,
    /// Border style: solid, dashed, beveled, inset, underline
    pub border_style: Option<String>,
}

/// Line annotation (single straight line) - Section 12.5.6.7
#[napi]
#[derive(Clone, Debug)]
pub struct LineAnnotation {
    /// Unique identifier
    pub id: String,
    /// Bounding box
    pub rect: Rect,
    /// Line start point
    pub start_x: f32,
    pub start_y: f32,
    /// Line end point
    pub end_x: f32,
    pub end_y: f32,
    /// Stroke color (RGB)
    pub stroke_color_r: u8,
    pub stroke_color_g: u8,
    pub stroke_color_b: u8,
    /// Stroke width in points
    pub stroke_width: f32,
    /// Line start style: None, Square, Circle, Diamond, OpenArrow, ClosedArrow, Butt, ROpenArrow, RClosedArrow, Slash
    pub start_style: Option<String>,
    /// Line end style
    pub end_style: Option<String>,
}

/// Square annotation (rectangle) - Section 12.5.6.8
#[napi]
#[derive(Clone, Debug)]
pub struct SquareAnnotation {
    /// Unique identifier
    pub id: String,
    /// Rectangle bounds
    pub rect: Rect,
    /// Stroke color (RGB)
    pub stroke_color_r: u8,
    pub stroke_color_g: u8,
    pub stroke_color_b: u8,
    /// Fill color (RGB) - optional
    pub fill_color_r: Option<u8>,
    pub fill_color_g: Option<u8>,
    pub fill_color_b: Option<u8>,
    /// Stroke width
    pub stroke_width: f32,
}

/// Circle annotation (ellipse) - Section 12.5.6.8
#[napi]
#[derive(Clone, Debug)]
pub struct CircleAnnotation {
    /// Unique identifier
    pub id: String,
    /// Bounding box of ellipse
    pub rect: Rect,
    /// Stroke color (RGB)
    pub stroke_color_r: u8,
    pub stroke_color_g: u8,
    pub stroke_color_b: u8,
    /// Fill color (RGB) - optional
    pub fill_color_r: Option<u8>,
    pub fill_color_g: Option<u8>,
    pub fill_color_b: Option<u8>,
    /// Stroke width
    pub stroke_width: f32,
}

/// Polygon annotation (closed polygon) - Section 12.5.6.9
#[napi]
#[derive(Clone, Debug)]
pub struct PolygonAnnotation {
    /// Unique identifier
    pub id: String,
    /// Bounding box
    pub rect: Rect,
    /// Stroke color (RGB)
    pub stroke_color_r: u8,
    pub stroke_color_g: u8,
    pub stroke_color_b: u8,
    /// Fill color (RGB) - optional
    pub fill_color_r: Option<u8>,
    pub fill_color_g: Option<u8>,
    pub fill_color_b: Option<u8>,
    /// Stroke width
    pub stroke_width: f32,
    /// Vertices as JSON array of [x, y] points
    pub vertices: Option<String>,
}

/// PolyLine annotation (open polyline) - Section 12.5.6.9
#[napi]
#[derive(Clone, Debug)]
pub struct PolyLineAnnotation {
    /// Unique identifier
    pub id: String,
    /// Bounding box
    pub rect: Rect,
    /// Stroke color (RGB)
    pub stroke_color_r: u8,
    pub stroke_color_g: u8,
    pub stroke_color_b: u8,
    /// Stroke width
    pub stroke_width: f32,
    /// Start style
    pub start_style: Option<String>,
    /// End style
    pub end_style: Option<String>,
    /// Vertices as JSON array
    pub vertices: Option<String>,
}

/// Highlight annotation (text marking) - Section 12.5.6.10
#[napi]
#[derive(Clone, Debug)]
pub struct HighlightAnnotation {
    /// Unique identifier
    pub id: String,
    /// Annotation area
    pub rect: Rect,
    /// Color (RGB) - yellow by default
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    /// Quad points (text areas) as JSON
    pub quad_points: Option<String>,
}

/// Underline annotation (underline text) - Section 12.5.6.10
#[napi]
#[derive(Clone, Debug)]
pub struct UnderlineAnnotation {
    /// Unique identifier
    pub id: String,
    /// Annotation area
    pub rect: Rect,
    /// Color (RGB) - red by default
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    /// Quad points as JSON
    pub quad_points: Option<String>,
}

/// Squiggly annotation (wavy underline) - Section 12.5.6.10
#[napi]
#[derive(Clone, Debug)]
pub struct SquigglyAnnotation {
    /// Unique identifier
    pub id: String,
    /// Annotation area
    pub rect: Rect,
    /// Color (RGB) - orange by default
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    /// Quad points as JSON
    pub quad_points: Option<String>,
}

/// StrikeOut annotation (strikethrough text) - Section 12.5.6.10
#[napi]
#[derive(Clone, Debug)]
pub struct StrikeOutAnnotation {
    /// Unique identifier
    pub id: String,
    /// Annotation area
    pub rect: Rect,
    /// Color (RGB) - red by default
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    /// Quad points as JSON
    pub quad_points: Option<String>,
}

/// Stamp annotation (rubber stamp) - Section 12.5.6.12
#[napi]
#[derive(Clone, Debug)]
pub struct StampAnnotation {
    /// Unique identifier
    pub id: String,
    /// Stamp location
    pub rect: Rect,
    /// Stamp name: Approved, Draft, Confidential, Final, Expired, Sold, TopSecret, etc.
    pub name: String,
    /// Color tint (RGB)
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
}

/// Caret annotation (text insertion marker) - Section 12.5.6.11
#[napi]
#[derive(Clone, Debug)]
pub struct CaretAnnotation {
    /// Unique identifier
    pub id: String,
    /// Caret location
    pub rect: Rect,
    /// Symbol: "P" for paragraph, none for insertion point
    pub symbol: Option<String>,
    /// Color (RGB)
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
}

/// Ink annotation (freehand drawing/scribbles) - Section 12.5.6.13
#[napi]
#[derive(Clone, Debug)]
pub struct InkAnnotation {
    /// Unique identifier
    pub id: String,
    /// Bounding box
    pub rect: Rect,
    /// Stroke color (RGB)
    pub stroke_color_r: u8,
    pub stroke_color_g: u8,
    pub stroke_color_b: u8,
    /// Stroke width
    pub stroke_width: f32,
    /// Ink list (pen strokes) as JSON array of paths
    pub ink_list: Option<String>,
}

/// Popup annotation (pop-up window) - Section 12.5.6.14
#[napi]
#[derive(Clone, Debug)]
pub struct PopupAnnotation {
    /// Unique identifier
    pub id: String,
    /// Popup window region
    pub rect: Rect,
    /// Parent annotation ID (for replies)
    pub parent_annotation_id: Option<String>,
    /// Initially open
    pub is_open: Option<bool>,
}

/// FileAttachment annotation (embedded file) - Section 12.5.6.15
#[napi]
#[derive(Clone, Debug)]
pub struct FileAttachmentAnnotation {
    /// Unique identifier
    pub id: String,
    /// Attachment icon location
    pub rect: Rect,
    /// File name
    pub filename: String,
    /// File description
    pub description: Option<String>,
    /// Icon: Graph, Paperclip, PushPin, Tag
    pub icon_name: Option<String>,
}

/// Sound annotation (audio playback) - Section 12.5.6.16
#[napi]
#[derive(Clone, Debug)]
pub struct SoundAnnotation {
    /// Unique identifier
    pub id: String,
    /// Speaker icon location
    pub rect: Rect,
    /// Audio file name
    pub filename: String,
    /// Sampling rate in Hz
    pub sampling_rate: Option<i32>,
    /// Number of audio channels
    pub channels: Option<i32>,
    /// Bits per sample
    pub bits_per_sample: Option<i32>,
}

/// Redact annotation (content removal/blackout) - Section 12.5.6.23
#[napi]
#[derive(Clone, Debug)]
pub struct RedactAnnotation {
    /// Unique identifier
    pub id: String,
    /// Redaction area
    pub rect: Rect,
    /// Text to show over redaction
    pub overlay_text: Option<String>,
    /// Quad points to redact
    pub quad_points: Option<String>,
    /// Redaction color (RGB)
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
}

/// Widget annotation (form field) - Section 12.5.6.19
#[napi]
#[derive(Clone, Debug)]
pub struct WidgetAnnotation {
    /// Unique identifier
    pub id: String,
    /// Field region
    pub rect: Rect,
    /// Field name (unique identifier)
    pub field_name: String,
    /// Field type: Text, Checkbox, Radio, Button, Choice, Signature
    pub field_type: String,
    /// Current value
    pub field_value: Option<String>,
    /// Default value
    pub default_value: Option<String>,
    /// Read-only field
    pub read_only: bool,
    /// Required field
    pub required: bool,
}

/// Screen annotation (multimedia container) - Section 12.5.6.18
#[napi]
#[derive(Clone, Debug)]
pub struct ScreenAnnotation {
    /// Unique identifier
    pub id: String,
    /// Media display region
    pub rect: Rect,
    /// MIME type
    pub media_type: String,
    /// Media data (base64 or reference)
    pub media_data: Option<String>,
}

/// ThreeD annotation (3D model) - Section 12.5.6.24
#[napi]
#[derive(Clone, Debug)]
pub struct ThreeDAnnotation {
    /// Unique identifier
    pub id: String,
    /// 3D display region
    pub rect: Rect,
    /// Format: U3D or PRC
    pub model_format: String,
    /// Model data (base64 or reference)
    pub model_data: Option<String>,
}

/// Watermark annotation (background watermark) - Section 12.5.6.22
#[napi]
#[derive(Clone, Debug)]
pub struct WatermarkAnnotation {
    /// Unique identifier
    pub id: String,
    /// Watermark region
    pub rect: Rect,
    /// Watermark text
    pub text: String,
    /// Opacity (0.0 - 1.0)
    pub opacity: f32,
    /// Rotation in degrees
    pub rotation_degrees: f32,
    /// Font size
    pub font_size: f32,
    /// Text color (RGB)
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
}
