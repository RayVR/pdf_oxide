//! Page renderer using tiny-skia.
//!
//! This module implements the core PDF rendering logic, converting
//! PDF operators into tiny-skia drawing commands.
#![allow(
    clippy::manual_div_ceil,
    clippy::field_reassign_with_default,
    clippy::collapsible_if,
    clippy::needless_borrow,
    clippy::get_first,
    clippy::if_same_then_else,
    clippy::needless_return_with_question_mark,
    clippy::ptr_arg
)]

use crate::content::graphics_state::{GraphicsState, GraphicsStateStack, Matrix};
use crate::content::operators::Operator;
use crate::content::parser::parse_content_stream;
use crate::document::PdfDocument;
use crate::error::{Error, Result};
use crate::object::{Object, ObjectRef};
use crate::rendering::ext_gstate::{parse_ext_g_state_inner, ParsedExtGState};
use crate::rendering::path_rasterizer::PathRasterizer;
use crate::rendering::resolution::{
    DeviceColor, LogicalColor, PaintIntent, PaintKind, PaintSide, ResolutionContext,
    ResolutionPipeline, ResolvedColor,
};
use crate::rendering::text_rasterizer::TextRasterizer;

use crate::fonts::FontInfo;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tiny_skia::{Color, PathBuilder, Pixmap, PixmapPaint, Transform};

/// Which path-paint side(s) [`PageRenderer::pipeline_resolve_paint_gs`]
/// should resolve for the current operator.
///
/// Text operators (`Tj` / `TJ` / `'` / `"`) use the sibling
/// [`PageRenderer::pipeline_resolve_text_colors`] instead — it returns
/// `Option<ResolvedColors>` rather than `Option<GraphicsState>` so the
/// text rasteriser's internal `current_gs` clone (the one that advances
/// `text_matrix` per glyph or per `TJ` element) is the only
/// `GraphicsState` allocation on the toggle-on text path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PipelinePaintKind {
    /// `f`, `F`, `f*` — path-fill only.
    PathFill,
    /// `S` — path-stroke only.
    PathStroke,
    /// `B`, `b`, `B*`, `b*` — fill then stroke (one spliced clone covers
    /// both passes; the fill pass reads `fill_*` fields, the stroke pass
    /// reads `stroke_*` fields).
    PathFillStroke,
    /// `Do` with `/Subtype /Image` and `/ImageMask true` — stencil mask
    /// painted with the current fill colour. Behaviourally identical to
    /// [`PipelinePaintKind::PathFill`] inside the helper (one fill-side
    /// resolve, splice into `fill_color_rgb` / `fill_alpha`), but kept as
    /// a distinct variant so the call site reads as "image-mask intent"
    /// rather than "secretly a path fill" — and so a future wave that
    /// needs image-mask-specific routing (e.g. per-pixel overprint
    /// against an image mask painted with a spot colour) can branch on
    /// this without changing the path-fill arms.
    ImageMask,
}

/// Resolved RGBA colours destined for the text rasteriser, side by side.
///
/// The operator arm picks the colours from
/// [`PageRenderer::pipeline_resolve_text_colors`] and hands them to
/// `render_text` / `render_tj_array`. The rasteriser already clones the
/// `GraphicsState` to advance `text_matrix` per glyph or per `TJ`
/// element, so it splices the overrides into that clone — no
/// operator-arm-side allocation happens on the toggle-on text path.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ResolvedColors {
    /// Fill RGBA, populated when `gs.render_mode` selects the fill side
    /// (Tr ∈ {0, 2, 4, 6}) and the pipeline produced an RGBA result.
    pub(crate) fill: Option<(f32, f32, f32, f32)>,
    /// Stroke RGBA, populated when `gs.render_mode` selects the stroke
    /// side (Tr ∈ {1, 2, 5, 6}) and the pipeline produced an RGBA
    /// result.
    pub(crate) stroke: Option<(f32, f32, f32, f32)>,
}

impl ResolvedColors {
    /// `true` when neither side carries an override.
    pub(crate) fn is_empty(&self) -> bool {
        self.fill.is_none() && self.stroke.is_none()
    }
}

/// Image output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// Portable Network Graphics
    Png,
    /// Joint Photographic Experts Group
    Jpeg,
    /// Raw premultiplied RGBA8888 pixels, row-major, top-left origin.
    /// `data.len() == width * height * 4`. No encoding overhead; callers
    /// that need straight (un-premultiplied) alpha must convert themselves.
    RawRgba8,
}

/// Options for page rendering.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Resolution in dots per inch (default: 150)
    pub dpi: u32,
    /// Output image format (default: PNG)
    pub format: ImageFormat,
    /// Background color (RGBA, default: white)
    pub background: Option<[f32; 4]>,
    /// Whether to render annotations (default: true)
    pub render_annotations: bool,
    /// JPEG quality (1-100, default: 85)
    pub jpeg_quality: u8,
    /// Optional Content Group (layer) names to exclude from rendering.
    ///
    /// When a BDC operator with tag "OC" references an OCG whose /Name matches
    /// one of these entries, all graphical content within that marked content
    /// scope is suppressed (not painted). Empty means render everything.
    pub excluded_layers: HashSet<String>,
    /// Explicit float scale factor set by `render_page_fit`.
    /// When `Some`, bypasses integer-DPI quantization so fit dimensions are
    /// exact (issue #480). Not part of the public API; set via
    /// `render_page_fit` only.
    pub(crate) scale_override: Option<f32>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            dpi: 150,
            format: ImageFormat::Png,
            background: Some([1.0, 1.0, 1.0, 1.0]), // White background
            render_annotations: true,
            jpeg_quality: 85,
            excluded_layers: HashSet::new(),
            scale_override: None,
        }
    }
}

impl RenderOptions {
    /// Set a transparent background (no background fill).
    pub fn with_transparent_background(mut self) -> Self {
        self.background = None;
        self
    }
}

impl RenderOptions {
    /// Create options with specified DPI.
    pub fn with_dpi(dpi: u32) -> Self {
        Self {
            dpi,
            ..Default::default()
        }
    }

    /// Set format to JPEG with quality (clamped to 1-100).
    pub fn as_jpeg(mut self, quality: u8) -> Self {
        self.format = ImageFormat::Jpeg;
        self.jpeg_quality = quality.clamp(1, 100);
        self
    }

    /// Set format to raw premultiplied RGBA8888 (no encoding overhead).
    pub fn as_raw(mut self) -> Self {
        self.format = ImageFormat::RawRgba8;
        self
    }
}

/// A rendered page image.
pub struct RenderedImage {
    /// Raw image data
    pub data: Vec<u8>,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Format of the image data
    pub format: ImageFormat,
}

impl RenderedImage {
    /// Save the image to a file.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        std::fs::write(path, &self.data)
            .map_err(|e| Error::InvalidPdf(format!("Failed to write image: {}", e)))
    }

    /// Get the image data as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

/// Page renderer that converts PDF pages to raster images.
pub struct PageRenderer {
    options: RenderOptions,
    path_rasterizer: PathRasterizer,
    text_rasterizer: TextRasterizer,
    /// Font cache (name -> FontInfo) for current context
    fonts: HashMap<String, Arc<FontInfo>>,
    /// Color space cache (name -> Object) for current context
    color_spaces: HashMap<String, Object>,
    /// Snapshot of `options.excluded_layers` wrapped in an `Arc` so that every
    /// recursive `execute_operators` call holds a cheap reference instead of
    /// deep-cloning the set per nested Form XObject. Recomputed on the first
    /// access per `render_page` invocation. Stays `None` (no allocation) when
    /// the set is empty — the common case.
    excluded_layers_snapshot: Option<Arc<HashSet<String>>>,
    /// Cached truth value of the `PDF_OXIDE_RESOLUTION_PIPELINE` env var,
    /// refreshed once per `render_page_with_options` call. The dispatcher
    /// fires paint operators many thousands of times per page; reading the
    /// env var on every `pipeline_resolve_rgba` call is well under a
    /// microsecond but still measurable, and the toggle never changes
    /// mid-render. Tests that flip the env var around `render_page` calls
    /// pick up the new value on the next call because the cache is
    /// recomputed there.
    pipeline_enabled_cache: bool,
}

impl PageRenderer {
    /// Create a new page renderer with the specified options.
    pub fn new(options: RenderOptions) -> Self {
        Self {
            options,
            path_rasterizer: PathRasterizer::new(),
            text_rasterizer: TextRasterizer::new(),
            fonts: HashMap::new(),
            color_spaces: HashMap::new(),
            excluded_layers_snapshot: None,
            pipeline_enabled_cache: false,
        }
    }

    /// Render a page to a raster image.
    pub fn render_page(&mut self, doc: &PdfDocument, page_num: usize) -> Result<RenderedImage> {
        self.render_page_with_options(page_num, doc)
    }

    /// Render a page with specific options.
    pub fn render_page_with_options(
        &mut self,
        page_num: usize,
        doc: &PdfDocument,
    ) -> Result<RenderedImage> {
        // Clear caches for new page
        self.fonts.clear();
        self.color_spaces.clear();

        // Refresh the pipeline-toggle cache once per render. Tests that
        // flip `PDF_OXIDE_RESOLUTION_PIPELINE` between renders pick up
        // the new value here; the dispatcher reads the cached bool to
        // avoid an env-var lookup per paint operator.
        self.pipeline_enabled_cache = read_pipeline_env();

        // Refresh the excluded-layers snapshot once per page. The effective
        // set combines (a) the PDF's default-off OCGs per /OCProperties/D
        // (BaseState, /ON, /OFF) — ISO 32000-1 §8.11.4 — with (b) the caller's
        // explicit excluded_layers. This makes the renderer respect the PDF's
        // default visibility configuration, matching a viewer's initial state.
        let default_off = crate::optional_content::compute_default_off_ocgs(doc);
        let effective: HashSet<String> = default_off
            .into_iter()
            .chain(self.options.excluded_layers.iter().cloned())
            .collect();
        self.excluded_layers_snapshot = if effective.is_empty() {
            None
        } else {
            Some(Arc::new(effective))
        };

        // Get page info
        let page_info = doc.get_page_info(page_num)?;
        let media_box = page_info.media_box;

        // Calculate output dimensions, accounting for page rotation
        let rotation = page_info.rotation % 360;
        let (page_w, page_h) = if rotation == 90 || rotation == 270 {
            (media_box.height, media_box.width) // Swap for landscape
        } else {
            (media_box.width, media_box.height)
        };
        let scale = self
            .options
            .scale_override
            .unwrap_or(self.options.dpi as f32 / 72.0);
        let (width, height) = if self.options.scale_override.is_some() {
            // Float scale path: round to avoid off-by-one from exact fractional pixels.
            // Clamp to 1 so extreme aspect ratios never produce a 0-sized pixmap.
            (
                ((page_w * scale).round() as u32).max(1),
                ((page_h * scale).round() as u32).max(1),
            )
        } else {
            ((page_w * scale).ceil() as u32, (page_h * scale).ceil() as u32)
        };

        // Create pixmap
        let mut pixmap = Pixmap::new(width, height)
            .ok_or_else(|| Error::InvalidPdf("Failed to create pixmap".to_string()))?;

        // Fill background
        if let Some(bg) = self.options.background {
            let [r, g, b, a] = bg;
            pixmap.fill(Color::from_rgba(r, g, b, a).unwrap_or(Color::WHITE));
        }

        // Create base transform: PDF coordinates to pixel coordinates
        // PDF origin is bottom-left; we flip Y and apply page rotation.
        // Per PDF spec §8.3.2.3, /Rotate specifies clockwise rotation.
        // The approach: first map PDF coords to an unrotated pixel space,
        // then rotate the entire result.
        let transform = match rotation {
            90 => {
                // 90° CW rotation: portrait PDF → landscape display
                // PDF y-up (x,y) → screen y-down: screen_x = y*s, screen_y = x*s
                Transform::from_translate(-media_box.x, -media_box.y)
                    .post_concat(Transform::from_row(0.0, scale, scale, 0.0, 0.0, 0.0))
            },
            180 => Transform::from_translate(-media_box.x, -media_box.y)
                .post_scale(-scale, scale)
                .post_translate(media_box.width * scale, 0.0),
            270 => Transform::from_translate(-media_box.x, -media_box.y).post_concat(
                Transform::from_row(0.0, scale, -scale, 0.0, media_box.height * scale, 0.0),
            ),
            _ => {
                // No rotation (0°)
                Transform::from_translate(-media_box.x, -media_box.y)
                    .post_scale(scale, -scale)
                    .post_translate(0.0, page_h * scale)
            },
        };

        // Get page resources
        let resources = doc.get_page_resources(page_num)?;

        // Pre-load resources (v0.3.18 synchronization)
        self.load_resources(doc, &resources)?;

        // Get page content stream
        let content_data = doc.get_page_content_data(page_num)?;

        // Parse content stream
        let operators = match parse_content_stream(&content_data) {
            Ok(ops) => ops,
            Err(e) => {
                return Err(e);
            },
        };

        // Execute operators
        self.execute_operators(&mut pixmap, transform, &operators, doc, page_num, &resources)?;

        // Render annotations (if requested and present)
        if self.options.render_annotations {
            self.render_annotations(&mut pixmap, transform, doc, page_num)?;
        }

        // Encode to output format
        let data = match self.options.format {
            ImageFormat::Png => encode_png(&pixmap)?,
            ImageFormat::Jpeg => self.encode_jpeg(&pixmap)?,
            ImageFormat::RawRgba8 => pixmap.data().to_vec(),
        };

        Ok(RenderedImage {
            data,
            width,
            height,
            format: self.options.format,
        })
    }

    /// Load resources (fonts, color spaces) into local cache.
    fn load_resources(&mut self, doc: &PdfDocument, resources: &Object) -> Result<()> {
        if let Object::Dictionary(res_dict) = resources {
            log::debug!("Loading resources, keys: {:?}", res_dict.keys());
            // Fonts
            if let Some(font_obj) = res_dict.get("Font") {
                log::debug!("Found Font resource");
                let font_dict_obj = doc.resolve_object(font_obj)?;
                if let Some(font_dict) = font_dict_obj.as_dict() {
                    for (name, f_obj) in font_dict {
                        match doc.get_or_load_font_for_rendering(f_obj) {
                            Ok(info) => {
                                log::debug!("Resolved font '{}': subtype={}, encoding={:?}, has_to_unicode={}, has_embedded={}",
                                    info.base_font, info.subtype, info.encoding, info.to_unicode.is_some(), info.embedded_font_data.is_some());
                                self.fonts.insert(name.clone(), info);
                            },
                            Err(e) => {
                                log::warn!(
                                    "Failed to parse font '{}': {}. Text using this font may render incorrectly.",
                                    name, e
                                );
                            },
                        }
                    }
                }
            }

            // Color Spaces
            if let Some(cs_obj) = res_dict.get("ColorSpace") {
                log::debug!("Found ColorSpace resource");
                let cs_dict_obj = doc.resolve_object(cs_obj)?;
                if let Some(cs_dict) = cs_dict_obj.as_dict() {
                    for (name, o) in cs_dict {
                        if let Ok(resolved_cs) = doc.resolve_object(o) {
                            log::debug!("Resolved color space '{}': {:?}", name, resolved_cs);
                            self.color_spaces.insert(name.clone(), resolved_cs);
                        }
                    }
                }
            }

            // XObjects
            if let Some(xobj_obj) = res_dict.get("XObject") {
                let xobj_dict_obj = doc.resolve_object(xobj_obj)?;
                if let Some(xobj_dict) = xobj_dict_obj.as_dict() {
                    log::debug!("XObject dict keys: {:?}", xobj_dict.keys());
                }
            }
        }

        // Share TrueType CMaps between matching fonts (essential for CID fonts with missing ToUnicode)
        self.share_truetype_cmaps();
        Ok(())
    }

    /// Share TrueType cmap tables between fonts with matching base font names.
    fn share_truetype_cmaps(&mut self) {
        let mut base_font_to_cmap = HashMap::new();

        // First pass: collect available cmaps
        for font in self.fonts.values() {
            if let Some(cmap) = font.truetype_cmap() {
                // Get base font name without subset prefix (e.g. ABCDEF+Arial -> Arial)
                let base_name = if let Some(plus_idx) = font.base_font.find('+') {
                    &font.base_font[plus_idx + 1..]
                } else {
                    &font.base_font
                };
                base_font_to_cmap.insert(base_name.to_string(), cmap.clone());
            }
        }

        // Second pass: apply cmaps to fonts missing them
        for font in self.fonts.values() {
            if font.subtype == "Type0" && font.truetype_cmap().is_none() {
                let base_name = if let Some(plus_idx) = font.base_font.find('+') {
                    &font.base_font[plus_idx + 1..]
                } else {
                    &font.base_font
                };
                if let Some(shared_cmap) = base_font_to_cmap.get(base_name) {
                    font.truetype_cmap.set(Some(shared_cmap.clone())).ok();
                }
            }
        }
    }

    /// Execute PDF operators to render content.
    ///
    /// OCG layer exclusion is sourced from `self.options.excluded_layers`;
    /// BDC/EMC operators referencing matching layers cause graphical operators
    /// inside that scope to be silently dropped.
    fn execute_operators(
        &mut self,
        pixmap: &mut Pixmap,
        base_transform: Transform,
        operators: &[Operator],
        doc: &PdfDocument,
        page_num: usize,
        resources: &Object,
    ) -> Result<()> {
        // Per-render snapshot lives on `self.excluded_layers_snapshot` (filled
        // by `render_page_with_options`). Recursive calls into this function
        // reuse the same `Arc` without any allocation. We snapshot it as a
        // local `Arc::clone` (cheap pointer copy) so the operator loop below
        // can hold a `&HashSet` reference while still calling `&mut self`
        // methods through the inner match arms.
        let snapshot: Option<Arc<HashSet<String>>> = self.excluded_layers_snapshot.clone();
        static EMPTY: std::sync::OnceLock<HashSet<String>> = std::sync::OnceLock::new();
        let empty_ref: &HashSet<String> = EMPTY.get_or_init(HashSet::new);
        let excluded_layers: &HashSet<String> = snapshot.as_deref().unwrap_or(empty_ref);
        let mut gs_stack = GraphicsStateStack::new();

        // PDF default: DeviceGray, black
        {
            let gs = gs_stack.current_mut();
            gs.fill_color_space = "DeviceGray".to_string();
            gs.stroke_color_space = "DeviceGray".to_string();
            gs.fill_color_rgb = (0.0, 0.0, 0.0);
            gs.stroke_color_rgb = (0.0, 0.0, 0.0);
        }

        let mut in_text_object = false;
        let mut current_path = PathBuilder::new();
        let mut pending_clip: Option<(tiny_skia::Path, tiny_skia::FillRule)> = None;
        let mut clip_stack: Vec<Option<tiny_skia::Mask>> = vec![None]; // Start with no clip at depth 0

        // OCG layer exclusion tracking.
        // `excluded_layer_depth` counts how many nested BDC/OC scopes we are
        // inside that match an excluded layer. >0 means content is suppressed.
        // `marked_content_depth` tracks total BDC/BMC nesting so EMC correctly
        // decrements only when it pops an excluded-layer entry.
        let mut excluded_layer_depth: u32 = 0;
        let mut marked_content_is_excluded: Vec<bool> = Vec::new();

        // Per-`execute_operators` resolved ExtGState resource dictionary. PDF
        // content streams often invoke `gs<N>` thousands of times per page
        // (vector scatter / contour plots emit one `gs` per marker — a
        // dense plot page can have ~10 000 such calls per Form XObject with
        // ~10 000 unique names because each marker carries its own alpha).
        // Without this hoist, every `gs` op called `doc.resolve_object(...)`
        // which deep-clones the *entire* per-form ExtGState dict (10 000+
        // entries) — that single clone dominated render time. Resolving the
        // resource dict once at the top of the operator loop and keeping a
        // borrow into it collapses the per-`gs` work to a small `get` +
        // resolve of just the inner state dict.
        let ext_g_state_resolved: Option<Object> = match resources {
            Object::Dictionary(rd) => rd.get("ExtGState").and_then(|o| doc.resolve_object(o).ok()),
            _ => None,
        };
        let ext_g_states: Option<&std::collections::HashMap<String, Object>> =
            ext_g_state_resolved.as_ref().and_then(|o| o.as_dict());
        // Cache parsed state per `dict_name` so the inner-dict resolve happens
        // at most once per unique name in scope.
        let mut ext_g_state_cache: std::collections::HashMap<String, ParsedExtGState> =
            std::collections::HashMap::new();
        for op in operators {
            match op {
                // Graphics state operators
                Operator::SaveState => {
                    gs_stack.save();
                    // Clone current clip for the new graphics state level
                    // This allows the current level to modify its clip without affecting parents
                    let current_clip = clip_stack.last().cloned().flatten();
                    clip_stack.push(current_clip);
                    log::debug!(
                        "q (SaveState), depth={}, clip_stack depth={}",
                        gs_stack.depth(),
                        clip_stack.len()
                    );
                },
                Operator::RestoreState => {
                    gs_stack.restore();
                    // Restore previous clipping region by popping current level
                    if clip_stack.len() > 1 {
                        clip_stack.pop();
                    }
                    log::debug!(
                        "Q (RestoreState), depth={}, clip_stack depth={}",
                        gs_stack.depth(),
                        clip_stack.len()
                    );
                },
                Operator::Cm { a, b, c, d, e, f } => {
                    let matrix = Matrix {
                        a: *a,
                        b: *b,
                        c: *c,
                        d: *d,
                        e: *e,
                        f: *f,
                    };
                    let current = gs_stack.current_mut();
                    // PDF spec ISO 32000-1:2008 §8.3.4: cm concatenates as M_cm × CTM
                    current.ctm = matrix.multiply(&current.ctm);
                    log::debug!(
                        "cm: [{}, {}, {}, {}, {}, {}], CTM now: {:?}",
                        a,
                        b,
                        c,
                        d,
                        e,
                        f,
                        current.ctm
                    );
                },

                // Color operators
                Operator::SetFillRgb { r, g, b } => {
                    let gs = gs_stack.current_mut();
                    gs.fill_color_rgb = (*r, *g, *b);
                    gs.fill_color_space = "DeviceRGB".to_string();
                    gs.fill_color_components.clear();
                    gs.fill_color_components.extend_from_slice(&[*r, *g, *b]);
                    log::debug!("SetFillRgb: [{}, {}, {}]", r, g, b);
                },
                Operator::SetStrokeRgb { r, g, b } => {
                    let gs = gs_stack.current_mut();
                    gs.stroke_color_rgb = (*r, *g, *b);
                    gs.stroke_color_space = "DeviceRGB".to_string();
                    gs.stroke_color_components.clear();
                    gs.stroke_color_components.extend_from_slice(&[*r, *g, *b]);
                    log::debug!("SetStrokeRgb: [{}, {}, {}]", r, g, b);
                },
                Operator::SetFillGray { gray } => {
                    let g = *gray;
                    let gs = gs_stack.current_mut();
                    gs.fill_color_rgb = (g, g, g);
                    gs.fill_color_space = "DeviceGray".to_string();
                    gs.fill_color_components.clear();
                    gs.fill_color_components.push(g);
                    log::debug!("SetFillGray: {}", g);
                },
                Operator::SetStrokeGray { gray } => {
                    let g = *gray;
                    let gs = gs_stack.current_mut();
                    gs.stroke_color_rgb = (g, g, g);
                    gs.stroke_color_space = "DeviceGray".to_string();
                    gs.stroke_color_components.clear();
                    gs.stroke_color_components.push(g);
                    log::debug!("SetStrokeGray: {}", g);
                },
                Operator::SetFillCmyk { c, m, y, k } => {
                    // Convert CMYK to RGB
                    let (r, g, b) = cmyk_to_rgb(*c, *m, *y, *k);
                    let gs = gs_stack.current_mut();
                    gs.fill_color_rgb = (r, g, b);
                    gs.fill_color_cmyk = Some((*c, *m, *y, *k));
                    gs.fill_color_space = "DeviceCMYK".to_string();
                    gs.fill_color_components.clear();
                    gs.fill_color_components
                        .extend_from_slice(&[*c, *m, *y, *k]);
                    log::debug!("SetFillCmyk: [{}, {}, {}, {}] -> {:?}", c, m, y, k, (r, g, b));
                },
                Operator::SetStrokeCmyk { c, m, y, k } => {
                    let (r, g, b) = cmyk_to_rgb(*c, *m, *y, *k);
                    let gs = gs_stack.current_mut();
                    gs.stroke_color_rgb = (r, g, b);
                    gs.stroke_color_cmyk = Some((*c, *m, *y, *k));
                    gs.stroke_color_space = "DeviceCMYK".to_string();
                    gs.stroke_color_components.clear();
                    gs.stroke_color_components
                        .extend_from_slice(&[*c, *m, *y, *k]);
                    log::debug!("SetStrokeCmyk: [{}, {}, {}, {}] -> {:?}", c, m, y, k, (r, g, b));
                },

                // Color space operators
                Operator::SetFillColorSpace { name } => {
                    gs_stack.current_mut().fill_color_space = name.clone();
                    log::debug!("SetFillColorSpace: {}", name);
                },
                Operator::SetStrokeColorSpace { name } => {
                    gs_stack.current_mut().stroke_color_space = name.clone();
                },
                Operator::SetFillColor { components } => {
                    let gs = gs_stack.current_mut();
                    let space_name = gs.fill_color_space.clone();
                    let resolved_space = self.color_spaces.get(&space_name);
                    gs.fill_color_components.clear();
                    gs.fill_color_components.extend_from_slice(components);

                    match space_name.as_str() {
                        "DeviceGray" | "G" if !components.is_empty() => {
                            let g = components[0];
                            gs.fill_color_rgb = (g, g, g);
                        },
                        "DeviceRGB" | "RGB" if components.len() >= 3 => {
                            gs.fill_color_rgb = (components[0], components[1], components[2]);
                        },
                        "DeviceCMYK" | "CMYK" if components.len() >= 4 => {
                            gs.fill_color_rgb = cmyk_to_rgb(
                                components[0],
                                components[1],
                                components[2],
                                components[3],
                            );
                        },
                        _ => {
                            let mut handled = false;
                            if let Some(rs) = resolved_space {
                                if let Some(arr) = rs.as_array() {
                                    if let Some(type_name) = arr.first().and_then(|o| o.as_name()) {
                                        match type_name {
                                            "ICCBased" if arr.len() > 1 => {
                                                if let Ok(dict_obj) = doc.resolve_object(&arr[1]) {
                                                    if let Some(dict) = dict_obj.as_dict() {
                                                        let n = dict
                                                            .get("N")
                                                            .and_then(|o| o.as_integer())
                                                            .unwrap_or(3);
                                                        match n {
                                                            1 if !components.is_empty() => {
                                                                let g = components[0];
                                                                gs.fill_color_rgb = (g, g, g);
                                                                handled = true;
                                                            },
                                                            3 if components.len() >= 3 => {
                                                                gs.fill_color_rgb = (
                                                                    components[0],
                                                                    components[1],
                                                                    components[2],
                                                                );
                                                                handled = true;
                                                            },
                                                            4 if components.len() >= 4 => {
                                                                gs.fill_color_rgb = cmyk_to_rgb(
                                                                    components[0],
                                                                    components[1],
                                                                    components[2],
                                                                    components[3],
                                                                );
                                                                handled = true;
                                                            },
                                                            _ => {},
                                                        }
                                                    }
                                                }
                                            },
                                            "Separation" | "DeviceN" => {
                                                // Per PDF spec, Separation = [/Separation name altCS tintTransform]
                                                // Evaluate tint transform against alternate color space
                                                if !components.is_empty() {
                                                    let tint = components[0];
                                                    let alt_cs = arr
                                                        .get(2)
                                                        .and_then(|o| o.as_name())
                                                        .unwrap_or("");
                                                    if alt_cs == "DeviceCMYK" && arr.len() >= 4 {
                                                        if let Some(func_obj) = arr.get(3) {
                                                            if let Ok(func_res) =
                                                                doc.resolve_object(func_obj)
                                                            {
                                                                if let Some(fd) = func_res.as_dict()
                                                                {
                                                                    if fd
                                                                        .get("FunctionType")
                                                                        .and_then(|o| {
                                                                            o.as_integer()
                                                                        })
                                                                        == Some(2)
                                                                    {
                                                                        let c0 =
                                                                            fd.get("C0").and_then(
                                                                                |o| o.as_array(),
                                                                            );
                                                                        let c1 =
                                                                            fd.get("C1").and_then(
                                                                                |o| o.as_array(),
                                                                            );
                                                                        let get_f = |arr: Option<&Vec<Object>>, i: usize, def: f32| -> f32 {
                                                                            arr.and_then(|a| a.get(i)).map(|o| match o { Object::Real(v) => *v as f32, Object::Integer(v) => *v as f32, _ => def }).unwrap_or(def)
                                                                        };
                                                                        let c = get_f(c0, 0, 0.0)
                                                                            + tint
                                                                                * (get_f(
                                                                                    c1, 0, 0.0,
                                                                                ) - get_f(
                                                                                    c0, 0, 0.0,
                                                                                ));
                                                                        let m = get_f(c0, 1, 0.0)
                                                                            + tint
                                                                                * (get_f(
                                                                                    c1, 1, 0.0,
                                                                                ) - get_f(
                                                                                    c0, 1, 0.0,
                                                                                ));
                                                                        let y = get_f(c0, 2, 0.0)
                                                                            + tint
                                                                                * (get_f(
                                                                                    c1, 2, 0.0,
                                                                                ) - get_f(
                                                                                    c0, 2, 0.0,
                                                                                ));
                                                                        let k = get_f(c0, 3, 0.0)
                                                                            + tint
                                                                                * (get_f(
                                                                                    c1, 3, 1.0,
                                                                                ) - get_f(
                                                                                    c0, 3, 0.0,
                                                                                ));
                                                                        gs.fill_color_rgb =
                                                                            cmyk_to_rgb(c, m, y, k);
                                                                        handled = true;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    if !handled {
                                                        let g = 1.0 - tint;
                                                        gs.fill_color_rgb = (g, g, g);
                                                    }
                                                    handled = true;
                                                }
                                            },
                                            "Indexed" => {
                                                if !components.is_empty() {
                                                    let g = components[0] / 255.0;
                                                    gs.fill_color_rgb = (g, g, g);
                                                    handled = true;
                                                }
                                            },
                                            _ => {},
                                        }
                                    }
                                }
                            }

                            if !handled && !components.is_empty() {
                                let g = components[0];
                                gs.fill_color_rgb = (g, g, g);
                            }
                        },
                    }
                    log::debug!(
                        "SetFillColor: {} {:?} -> {:?}",
                        space_name,
                        components,
                        gs.fill_color_rgb
                    );
                },
                Operator::SetStrokeColor { components } => {
                    let gs = gs_stack.current_mut();
                    let space_name = gs.stroke_color_space.clone();
                    let resolved_space = self.color_spaces.get(&space_name);
                    gs.stroke_color_components.clear();
                    gs.stroke_color_components.extend_from_slice(components);

                    match space_name.as_str() {
                        "DeviceGray" | "G" if !components.is_empty() => {
                            let g = components[0];
                            gs.stroke_color_rgb = (g, g, g);
                        },
                        "DeviceRGB" | "RGB" if components.len() >= 3 => {
                            gs.stroke_color_rgb = (components[0], components[1], components[2]);
                        },
                        "DeviceCMYK" | "CMYK" if components.len() >= 4 => {
                            gs.stroke_color_rgb = cmyk_to_rgb(
                                components[0],
                                components[1],
                                components[2],
                                components[3],
                            );
                        },
                        _ => {
                            let mut handled = false;
                            if let Some(rs) = resolved_space {
                                if let Some(arr) = rs.as_array() {
                                    if let Some(type_name) = arr.first().and_then(|o| o.as_name()) {
                                        match type_name {
                                            "ICCBased" if arr.len() > 1 => {
                                                if let Ok(dict_obj) = doc.resolve_object(&arr[1]) {
                                                    if let Some(dict) = dict_obj.as_dict() {
                                                        let n = dict
                                                            .get("N")
                                                            .and_then(|o| o.as_integer())
                                                            .unwrap_or(3);
                                                        match n {
                                                            1 if !components.is_empty() => {
                                                                let g = components[0];
                                                                gs.stroke_color_rgb = (g, g, g);
                                                                handled = true;
                                                            },
                                                            3 if components.len() >= 3 => {
                                                                gs.stroke_color_rgb = (
                                                                    components[0],
                                                                    components[1],
                                                                    components[2],
                                                                );
                                                                handled = true;
                                                            },
                                                            4 if components.len() >= 4 => {
                                                                gs.stroke_color_rgb = cmyk_to_rgb(
                                                                    components[0],
                                                                    components[1],
                                                                    components[2],
                                                                    components[3],
                                                                );
                                                                handled = true;
                                                            },
                                                            _ => {},
                                                        }
                                                    }
                                                }
                                            },
                                            _ => {},
                                        }
                                    }
                                }
                            }
                            if !handled && !components.is_empty() {
                                let g = components[0];
                                gs.stroke_color_rgb = (g, g, g);
                            }
                        },
                    }
                    log::debug!(
                        "SetStrokeColor: {} {:?} -> {:?}",
                        space_name,
                        components,
                        gs.stroke_color_rgb
                    );
                },
                Operator::SetFillColorN { components, .. } => {
                    let gs = gs_stack.current_mut();
                    let space_name = gs.fill_color_space.clone();
                    let resolved_space = self.color_spaces.get(&space_name);
                    gs.fill_color_components.clear();
                    gs.fill_color_components.extend_from_slice(components);

                    match space_name.as_str() {
                        "DeviceGray" | "G" if !components.is_empty() => {
                            let g = components[0];
                            gs.fill_color_rgb = (g, g, g);
                        },
                        "DeviceRGB" | "RGB" if components.len() >= 3 => {
                            gs.fill_color_rgb = (components[0], components[1], components[2]);
                        },
                        "DeviceCMYK" | "CMYK" if components.len() >= 4 => {
                            gs.fill_color_rgb = cmyk_to_rgb(
                                components[0],
                                components[1],
                                components[2],
                                components[3],
                            );
                        },
                        _ => {
                            let mut handled = false;
                            if let Some(rs) = resolved_space {
                                if let Some(arr) = rs.as_array() {
                                    if let Some(type_name) = arr.first().and_then(|o| o.as_name()) {
                                        match type_name {
                                            "ICCBased" if arr.len() > 1 => {
                                                if let Ok(dict_obj) = doc.resolve_object(&arr[1]) {
                                                    if let Some(dict) = dict_obj.as_dict() {
                                                        let n = dict
                                                            .get("N")
                                                            .and_then(|o| o.as_integer())
                                                            .unwrap_or(3);
                                                        match n {
                                                            1 if !components.is_empty() => {
                                                                let g = components[0];
                                                                gs.fill_color_rgb = (g, g, g);
                                                                handled = true;
                                                            },
                                                            3 if components.len() >= 3 => {
                                                                gs.fill_color_rgb = (
                                                                    components[0],
                                                                    components[1],
                                                                    components[2],
                                                                );
                                                                handled = true;
                                                            },
                                                            4 if components.len() >= 4 => {
                                                                gs.fill_color_rgb = cmyk_to_rgb(
                                                                    components[0],
                                                                    components[1],
                                                                    components[2],
                                                                    components[3],
                                                                );
                                                                handled = true;
                                                            },
                                                            _ => {},
                                                        }
                                                    }
                                                }
                                            },
                                            "Separation" | "DeviceN" => {
                                                if !components.is_empty() {
                                                    let g = 1.0 - components[0];
                                                    gs.fill_color_rgb = (g, g, g);
                                                    handled = true;
                                                }
                                            },
                                            "Indexed" => {
                                                // Indexed: the component is
                                                // a palette index 0..hival.
                                                // Until the full palette
                                                // lookup is wired (planned
                                                // alongside the pipeline's
                                                // resolve_indexed at
                                                // src/rendering/resolution/
                                                // color.rs:237), match the
                                                // pipeline's index/255 gray
                                                // fallback so the off/on
                                                // toggle agrees. Without
                                                // this branch the generic
                                                // fallback below would
                                                // forward the raw index
                                                // (e.g. 1.0) as the gray
                                                // value, producing white
                                                // where the pipeline gives
                                                // near-black.
                                                if !components.is_empty() {
                                                    let g = (components[0] / 255.0).clamp(0.0, 1.0);
                                                    gs.fill_color_rgb = (g, g, g);
                                                    handled = true;
                                                }
                                            },
                                            _ => {},
                                        }
                                    }
                                }
                            }
                            if !handled && !components.is_empty() {
                                let g = components[0];
                                gs.fill_color_rgb = (g, g, g);
                            }
                        },
                    }
                    log::debug!(
                        "SetFillColorN: {} {:?} -> {:?}",
                        space_name,
                        components,
                        gs.fill_color_rgb
                    );
                },
                Operator::SetStrokeColorN { components, .. } => {
                    let gs = gs_stack.current_mut();
                    let space_name = gs.stroke_color_space.clone();
                    let resolved_space = self.color_spaces.get(&space_name);
                    gs.stroke_color_components.clear();
                    gs.stroke_color_components.extend_from_slice(components);
                    match space_name.as_str() {
                        "DeviceGray" | "G" if !components.is_empty() => {
                            let g = components[0];
                            gs.stroke_color_rgb = (g, g, g);
                        },
                        "DeviceRGB" | "RGB" if components.len() >= 3 => {
                            gs.stroke_color_rgb = (components[0], components[1], components[2]);
                        },
                        "DeviceCMYK" | "CMYK" if components.len() >= 4 => {
                            gs.stroke_color_rgb = cmyk_to_rgb(
                                components[0],
                                components[1],
                                components[2],
                                components[3],
                            );
                        },
                        _ => {
                            let mut handled = false;
                            if let Some(rs) = resolved_space {
                                if let Some(arr) = rs.as_array() {
                                    if let Some(type_name) = arr.first().and_then(|o| o.as_name()) {
                                        match type_name {
                                            "ICCBased" if arr.len() > 1 => {
                                                if let Ok(dict_obj) = doc.resolve_object(&arr[1]) {
                                                    if let Some(dict) = dict_obj.as_dict() {
                                                        let n = dict
                                                            .get("N")
                                                            .and_then(|o| o.as_integer())
                                                            .unwrap_or(3);
                                                        match n {
                                                            1 if !components.is_empty() => {
                                                                let g = components[0];
                                                                gs.stroke_color_rgb = (g, g, g);
                                                                handled = true;
                                                            },
                                                            3 if components.len() >= 3 => {
                                                                gs.stroke_color_rgb = (
                                                                    components[0],
                                                                    components[1],
                                                                    components[2],
                                                                );
                                                                handled = true;
                                                            },
                                                            4 if components.len() >= 4 => {
                                                                gs.stroke_color_rgb = cmyk_to_rgb(
                                                                    components[0],
                                                                    components[1],
                                                                    components[2],
                                                                    components[3],
                                                                );
                                                                handled = true;
                                                            },
                                                            _ => {},
                                                        }
                                                    }
                                                }
                                            },
                                            "Separation" | "DeviceN" => {
                                                // Mirror the fill-side
                                                // `1.0 - tint` fallback so
                                                // off-vs-on parity holds
                                                // for legacy spot inks the
                                                // pipeline hasn't fully
                                                // wired yet. The toggled
                                                // pipeline path still
                                                // honours Type 2/4 tint
                                                // transforms when present.
                                                if !components.is_empty() {
                                                    let g = 1.0 - components[0];
                                                    gs.stroke_color_rgb = (g, g, g);
                                                    handled = true;
                                                }
                                            },
                                            "Indexed" => {
                                                // Indexed stroke: match the
                                                // pipeline's index/255 gray
                                                // fallback. See the matching
                                                // fill-side branch for the
                                                // rationale.
                                                if !components.is_empty() {
                                                    let g = (components[0] / 255.0).clamp(0.0, 1.0);
                                                    gs.stroke_color_rgb = (g, g, g);
                                                    handled = true;
                                                }
                                            },
                                            _ => {},
                                        }
                                    }
                                }
                            }
                            if !handled && !components.is_empty() {
                                let g = components[0];
                                gs.stroke_color_rgb = (g, g, g);
                            }
                        },
                    }
                    log::debug!(
                        "SetStrokeColorN: {} {:?} -> {:?}",
                        space_name,
                        components,
                        gs.stroke_color_rgb
                    );
                },

                // Line style operators
                Operator::SetLineWidth { width } => {
                    gs_stack.current_mut().line_width = *width;
                },
                Operator::SetLineCap { cap_style } => {
                    gs_stack.current_mut().line_cap = *cap_style;
                },
                Operator::SetLineJoin { join_style } => {
                    gs_stack.current_mut().line_join = *join_style;
                },
                Operator::SetMiterLimit { limit } => {
                    gs_stack.current_mut().miter_limit = *limit;
                },
                Operator::SetDash { array, phase } => {
                    gs_stack.current_mut().dash_pattern = (array.clone(), *phase);
                },

                // Path construction
                Operator::MoveTo { x, y } => {
                    current_path.move_to(*x, *y);
                },
                Operator::LineTo { x, y } => {
                    current_path.line_to(*x, *y);
                },
                Operator::CurveTo {
                    x1,
                    y1,
                    x2,
                    y2,
                    x3,
                    y3,
                } => {
                    current_path.cubic_to(*x1, *y1, *x2, *y2, *x3, *y3);
                },
                Operator::CurveToV { x2, y2, x3, y3 } => {
                    if let Some(last) = current_path.last_point() {
                        current_path.cubic_to(last.x, last.y, *x2, *y2, *x3, *y3);
                    }
                },
                Operator::CurveToY { x1, y1, x3, y3 } => {
                    current_path.cubic_to(*x1, *y1, *x3, *y3, *x3, *y3);
                },
                Operator::Rectangle {
                    x,
                    y,
                    width,
                    height,
                } => {
                    // Normalize negative width/height per PDF spec:
                    // re with negative dimensions means the rect extends in the opposite direction
                    let (nx, nw) = if *width < 0.0 {
                        (x + width, -width)
                    } else {
                        (*x, *width)
                    };
                    let (ny, nh) = if *height < 0.0 {
                        (y + height, -height)
                    } else {
                        (*y, *height)
                    };
                    if let Some(rect) = tiny_skia::Rect::from_xywh(nx, ny, nw, nh) {
                        current_path.push_rect(rect);
                    }
                },
                Operator::ClosePath => {
                    current_path.close();
                },

                // Path painting — suppressed when inside an excluded OCG layer
                Operator::Stroke => {
                    if excluded_layer_depth == 0 {
                        apply_pending_clip(
                            &mut pending_clip,
                            &mut clip_stack,
                            pixmap,
                            base_transform,
                            &gs_stack,
                        );
                        let clip = clip_stack.last().and_then(|c| c.as_ref());
                        if let Some(path) = current_path.finish() {
                            let gs = gs_stack.current();
                            // Pilot routing: stroke side mirrors the path-fill
                            // routing — toggle on routes through the pipeline
                            // so Type 4 Separation strokes resolve correctly;
                            // toggle off keeps the existing behaviour
                            // byte-for-byte. Line width / cap / join / dash
                            // come from the cloned `gs` unchanged, so the
                            // stroke geometry is identical either way.
                            let spliced = self.pipeline_resolve_paint_gs(
                                doc,
                                gs,
                                PipelinePaintKind::PathStroke,
                            );
                            let render_gs: &GraphicsState = spliced.as_ref().unwrap_or(gs);
                            let transform = combine_transforms(base_transform, &gs.ctm);
                            self.path_rasterizer
                                .stroke_path_clipped(pixmap, &path, transform, render_gs, clip);
                        }
                    } else {
                        let _ = current_path.finish();
                    }
                    current_path = PathBuilder::new();
                },
                Operator::Fill => {
                    if excluded_layer_depth == 0 {
                        apply_pending_clip(
                            &mut pending_clip,
                            &mut clip_stack,
                            pixmap,
                            base_transform,
                            &gs_stack,
                        );
                        let clip = clip_stack.last().and_then(|c| c.as_ref());
                        if let Some(path) = current_path.finish() {
                            let gs = gs_stack.current();
                            // Pilot routing: when the resolution-pipeline toggle
                            // is on, evaluate the active fill colour through the
                            // pipeline (which handles Type 4 tint transforms) and
                            // splice the resulting RGBA into a transient
                            // GraphicsState copy that the rasteriser consumes.
                            // Off → no behaviour change.
                            let spliced = self.pipeline_resolve_paint_gs(
                                doc,
                                gs,
                                PipelinePaintKind::PathFill,
                            );
                            let render_gs: &GraphicsState = spliced.as_ref().unwrap_or(gs);
                            let transform = combine_transforms(base_transform, &gs.ctm);
                            self.path_rasterizer.fill_path_clipped(
                                pixmap,
                                &path,
                                transform,
                                render_gs,
                                tiny_skia::FillRule::Winding,
                                clip,
                            );
                        }
                    } else {
                        let _ = current_path.finish();
                    }
                    current_path = PathBuilder::new();
                },
                Operator::FillStroke
                | Operator::CloseFillStroke
                | Operator::CloseFillStrokeEvenOdd => {
                    if excluded_layer_depth == 0 {
                        apply_pending_clip(
                            &mut pending_clip,
                            &mut clip_stack,
                            pixmap,
                            base_transform,
                            &gs_stack,
                        );
                        let clip = clip_stack.last().and_then(|c| c.as_ref());
                        // ISO 32000-1 §8.5.3.1 Table 60: `b` and `b*` close
                        // the path before fill+stroke. The parser does not
                        // decompose them (unlike `s`, which is emitted as
                        // `ClosePath` + `Stroke`), so the dispatcher must
                        // perform the close itself or the final segment of
                        // an open subpath will not be painted by the stroke.
                        if matches!(
                            op,
                            Operator::CloseFillStroke | Operator::CloseFillStrokeEvenOdd
                        ) {
                            current_path.close();
                        }
                        if let Some(path) = current_path.finish() {
                            let gs = gs_stack.current();
                            let transform = combine_transforms(base_transform, &gs.ctm);
                            let fill_rule = if matches!(op, Operator::CloseFillStrokeEvenOdd) {
                                tiny_skia::FillRule::EvenOdd
                            } else {
                                tiny_skia::FillRule::Winding
                            };
                            // Combos resolve fill and stroke independently
                            // through the pipeline (two `PaintIntent`s per
                            // operator). Each side falls back to the inline
                            // path if its colour can't be resolved to RGBA,
                            // so a Type 4 Separation on the fill side and a
                            // plain DeviceRGB on the stroke side route
                            // correctly without entangling the two.
                            //
                            // Off-toggle keeps `gs` borrowed directly (no
                            // clone) so the parity invariant is preserved
                            // and the off-path stays alloc-free.
                            // Single splice for both sides — the rasteriser
                            // reads fill fields for the fill pass and stroke
                            // fields for the stroke pass, so one clone with
                            // both sides written is equivalent to two
                            // single-side clones.
                            let spliced = self.pipeline_resolve_paint_gs(
                                doc,
                                gs,
                                PipelinePaintKind::PathFillStroke,
                            );
                            let render_gs: &GraphicsState = spliced.as_ref().unwrap_or(gs);
                            self.path_rasterizer.fill_path_clipped(
                                pixmap, &path, transform, render_gs, fill_rule, clip,
                            );
                            self.path_rasterizer
                                .stroke_path_clipped(pixmap, &path, transform, render_gs, clip);
                        }
                    } else {
                        let _ = current_path.finish();
                    }
                    current_path = PathBuilder::new();
                },
                Operator::FillEvenOdd | Operator::FillStrokeEvenOdd => {
                    if excluded_layer_depth == 0 {
                        apply_pending_clip(
                            &mut pending_clip,
                            &mut clip_stack,
                            pixmap,
                            base_transform,
                            &gs_stack,
                        );
                        let clip = clip_stack.last().and_then(|c| c.as_ref());
                        if let Some(path) = current_path.finish() {
                            let gs = gs_stack.current();
                            let transform = combine_transforms(base_transform, &gs.ctm);
                            // One unified resolve covers both fill and the
                            // optional stroke pass — for plain `f*` the
                            // helper produces a fill-only splice; for
                            // `B*`/`b*` both sides are spliced into the
                            // same clone. Either way, the rasteriser reads
                            // the side it needs from `render_gs`.
                            let kind = if matches!(op, Operator::FillStrokeEvenOdd) {
                                PipelinePaintKind::PathFillStroke
                            } else {
                                PipelinePaintKind::PathFill
                            };
                            let spliced = self.pipeline_resolve_paint_gs(doc, gs, kind);
                            let render_gs: &GraphicsState = spliced.as_ref().unwrap_or(gs);
                            self.path_rasterizer.fill_path_clipped(
                                pixmap,
                                &path,
                                transform,
                                render_gs,
                                tiny_skia::FillRule::EvenOdd,
                                clip,
                            );
                            if matches!(op, Operator::FillStrokeEvenOdd) {
                                // Stroke side: Type 4 Separation on the
                                // stroke colour is honoured when the pilot
                                // toggle is on; the spliced `render_gs`
                                // carries the resolved stroke fields.
                                self.path_rasterizer
                                    .stroke_path_clipped(pixmap, &path, transform, render_gs, clip);
                            }
                        }
                    } else {
                        let _ = current_path.finish();
                    }
                    current_path = PathBuilder::new();
                },

                // Clipping — suppressed inside an excluded OCG scope. Per PDF
                // spec the clip is a graphics-state side-effect; without
                // gating it, a `W n` issued inside an excluded BDC scope that
                // is not bracketed by `q/Q` would silently restrict subsequent
                // visible content.
                Operator::ClipNonZero => {
                    if excluded_layer_depth == 0 {
                        if let Some(path) = current_path.clone().finish() {
                            pending_clip = Some((path, tiny_skia::FillRule::Winding));
                        }
                    }
                },
                Operator::ClipEvenOdd => {
                    if excluded_layer_depth == 0 {
                        if let Some(path) = current_path.clone().finish() {
                            pending_clip = Some((path, tiny_skia::FillRule::EvenOdd));
                        }
                    }
                },

                // Text object operators
                Operator::BeginText => {
                    in_text_object = true;
                    let gs = gs_stack.current_mut();
                    gs.text_matrix = Matrix::identity();
                    gs.text_line_matrix = Matrix::identity();
                    log::debug!("BT (BeginText)");
                },
                Operator::EndText => {
                    in_text_object = false;
                },

                // Text state operators
                Operator::Tc { char_space } => {
                    gs_stack.current_mut().char_space = *char_space;
                },
                Operator::Tw { word_space } => {
                    gs_stack.current_mut().word_space = *word_space;
                },
                Operator::Tz { scale } => {
                    gs_stack.current_mut().horizontal_scaling = *scale;
                },
                Operator::TL { leading } => {
                    gs_stack.current_mut().leading = *leading;
                },
                Operator::Ts { rise } => {
                    gs_stack.current_mut().text_rise = *rise;
                },
                Operator::Tr { render } => {
                    gs_stack.current_mut().render_mode = *render;
                },

                // Text showing — glyphs suppressed inside an excluded OCG layer,
                // but the text matrix still advances so that subsequent visible
                // text inside the same BT/ET paints at the correct X position.
                Operator::Tj { text } => {
                    if in_text_object {
                        let gs = gs_stack.current();
                        let advance = if excluded_layer_depth == 0 {
                            let clip = clip_stack.last().and_then(|c| c.as_ref());
                            let transform = combine_transforms(base_transform, &gs.ctm);
                            // Pilot routing for text-showing operators: when
                            // the resolution-pipeline toggle is on, resolve the
                            // fill (and/or stroke per Tr mode) once for the
                            // whole `Tj` call and hand the resolved RGBA to
                            // the rasteriser. The rasteriser already clones
                            // `gs` to advance `text_matrix` per element, so it
                            // splices the override into that clone — no
                            // operator-arm-side clone needed. Off → helper
                            // returns None, gs borrowed directly.
                            let colors = self.pipeline_resolve_text_colors(doc, gs);
                            self.text_rasterizer.render_text(
                                pixmap,
                                text,
                                transform,
                                gs,
                                colors.as_ref(),
                                resources,
                                doc,
                                clip,
                                &self.fonts,
                            )?
                        } else {
                            self.text_rasterizer.measure_text(text, gs, &self.fonts)
                        };

                        let gs_mut = gs_stack.current_mut();
                        let advance_matrix = Matrix::translation(advance, 0.0);
                        gs_mut.text_matrix = advance_matrix.multiply(&gs_mut.text_matrix);
                    }
                },
                Operator::Quote { text } => {
                    if in_text_object {
                        // Quote (') is T* followed by Tj — always advance line
                        let gs_mut = gs_stack.current_mut();
                        let leading = gs_mut.leading;
                        let translation = Matrix::translation(0.0, -leading);
                        gs_mut.text_line_matrix = translation.multiply(&gs_mut.text_line_matrix);
                        gs_mut.text_matrix = gs_mut.text_line_matrix;

                        let gs = gs_stack.current();
                        let advance = if excluded_layer_depth == 0 {
                            let clip = clip_stack.last().and_then(|c| c.as_ref());
                            let transform = combine_transforms(base_transform, &gs.ctm);
                            log::debug!(
                                "' (Quote): rendering text at Tm=[{}, {}, {}, {}, {}, {}]",
                                gs.text_matrix.a,
                                gs.text_matrix.b,
                                gs.text_matrix.c,
                                gs.text_matrix.d,
                                gs.text_matrix.e,
                                gs.text_matrix.f
                            );
                            // Pilot routing — same shape as `Tj`. `'` is
                            // `T* Tj` per ISO 32000-1; the resolved colour
                            // depends only on the prior colour-setting ops,
                            // so the resolve happens here, not inside `T*`.
                            let colors = self.pipeline_resolve_text_colors(doc, gs);
                            self.text_rasterizer.render_text(
                                pixmap,
                                text,
                                transform,
                                gs,
                                colors.as_ref(),
                                resources,
                                doc,
                                clip,
                                &self.fonts,
                            )?
                        } else {
                            self.text_rasterizer.measure_text(text, gs, &self.fonts)
                        };

                        let gs_mut = gs_stack.current_mut();
                        let advance_matrix = Matrix::translation(advance, 0.0);
                        gs_mut.text_matrix = advance_matrix.multiply(&gs_mut.text_matrix);
                    }
                },
                Operator::TJ { array } => {
                    if in_text_object {
                        let gs = gs_stack.current();
                        let advance = if excluded_layer_depth == 0 {
                            let clip = clip_stack.last().and_then(|c| c.as_ref());
                            let transform = combine_transforms(base_transform, &gs.ctm);
                            log::debug!(
                                "TJ: rendering array at Tm=[{}, {}, {}, {}, {}, {}]",
                                gs.text_matrix.a,
                                gs.text_matrix.b,
                                gs.text_matrix.c,
                                gs.text_matrix.d,
                                gs.text_matrix.e,
                                gs.text_matrix.f
                            );
                            // Pilot routing for `TJ`. Resolve once for the
                            // whole array — the numeric offsets inside `array`
                            // only adjust positioning; they cannot alter the
                            // active colour mid-string. The rasteriser threads
                            // the override into the per-element `render_text`
                            // calls so the colour propagates without an
                            // operator-arm-side clone of `gs`.
                            let colors = self.pipeline_resolve_text_colors(doc, gs);
                            self.text_rasterizer.render_tj_array(
                                pixmap,
                                array,
                                transform,
                                gs,
                                colors.as_ref(),
                                resources,
                                doc,
                                clip,
                                &self.fonts,
                            )?
                        } else {
                            self.text_rasterizer
                                .measure_tj_array(array, gs, &self.fonts)
                        };

                        let gs_mut = gs_stack.current_mut();
                        let advance_matrix = Matrix::translation(advance, 0.0);
                        gs_mut.text_matrix = advance_matrix.multiply(&gs_mut.text_matrix);
                    }
                },
                Operator::DoubleQuote {
                    word_space,
                    char_space,
                    text,
                } => {
                    if in_text_object {
                        // Double Quote (") always updates state
                        let gs_mut = gs_stack.current_mut();
                        gs_mut.word_space = *word_space;
                        gs_mut.char_space = *char_space;

                        let leading = gs_mut.leading;
                        let translation = Matrix::translation(0.0, -leading);
                        gs_mut.text_line_matrix = translation.multiply(&gs_mut.text_line_matrix);
                        gs_mut.text_matrix = gs_mut.text_line_matrix;

                        let gs = gs_stack.current();
                        let advance = if excluded_layer_depth == 0 {
                            let clip = clip_stack.last().and_then(|c| c.as_ref());
                            let transform = combine_transforms(base_transform, &gs.ctm);
                            log::debug!(
                                "\" (DoubleQuote): rendering text at Tm=[{}, {}, {}, {}, {}, {}]",
                                gs.text_matrix.a,
                                gs.text_matrix.b,
                                gs.text_matrix.c,
                                gs.text_matrix.d,
                                gs.text_matrix.e,
                                gs.text_matrix.f
                            );
                            // Pilot routing — `"` is equivalent to setting Tw,
                            // Tc, then `T* Tj`. Tw/Tc are state-only and don't
                            // influence the resolved colour, so the resolve
                            // happens immediately before painting just like in
                            // `Tj` / `'`.
                            let colors = self.pipeline_resolve_text_colors(doc, gs);
                            self.text_rasterizer.render_text(
                                pixmap,
                                text,
                                transform,
                                gs,
                                colors.as_ref(),
                                resources,
                                doc,
                                clip,
                                &self.fonts,
                            )?
                        } else {
                            self.text_rasterizer.measure_text(text, gs, &self.fonts)
                        };

                        let gs_mut = gs_stack.current_mut();
                        let advance_matrix = Matrix::translation(advance, 0.0);
                        gs_mut.text_matrix = advance_matrix.multiply(&gs_mut.text_matrix);
                    }
                },

                // XObject (images) — suppressed when inside an excluded OCG layer
                Operator::Do { name } => {
                    if excluded_layer_depth == 0 {
                        let gs = gs_stack.current();
                        let transform = combine_transforms(base_transform, &gs.ctm);
                        let clip = clip_stack.last().and_then(|c| c.as_ref());
                        log::debug!("Do: rendering XObject '{}'", name);
                        self.render_xobject(
                            pixmap, name, transform, gs, resources, doc, page_num, clip,
                        )?;
                    }
                },

                // Text positioning
                Operator::Td { tx, ty } => {
                    if in_text_object {
                        let gs = gs_stack.current_mut();
                        let translation = Matrix::translation(*tx, *ty);
                        gs.text_line_matrix = translation.multiply(&gs.text_line_matrix);
                        gs.text_matrix = gs.text_line_matrix;
                        log::debug!("Td: [{}, {}], text_matrix now: {:?}", tx, ty, gs.text_matrix);
                    }
                },
                Operator::TD { tx, ty } => {
                    if in_text_object {
                        let gs = gs_stack.current_mut();
                        gs.leading = -(*ty);
                        let translation = Matrix::translation(*tx, *ty);
                        gs.text_line_matrix = translation.multiply(&gs.text_line_matrix);
                        gs.text_matrix = gs.text_line_matrix;
                        log::debug!("TD: [{}, {}], text_matrix now: {:?}", tx, ty, gs.text_matrix);
                    }
                },
                Operator::Tm { a, b, c, d, e, f } => {
                    if in_text_object {
                        let gs = gs_stack.current_mut();
                        gs.text_matrix = Matrix {
                            a: *a,
                            b: *b,
                            c: *c,
                            d: *d,
                            e: *e,
                            f: *f,
                        };
                        gs.text_line_matrix = gs.text_matrix;
                        log::debug!(
                            "Tm: [{}, {}, {}, {}, {}, {}], text_matrix now: {:?}",
                            a,
                            b,
                            c,
                            d,
                            e,
                            f,
                            gs.text_matrix
                        );
                    }
                },
                Operator::TStar => {
                    if in_text_object {
                        let gs = gs_stack.current_mut();
                        let leading = gs.leading;
                        let translation = Matrix::translation(0.0, -leading);
                        gs.text_line_matrix = translation.multiply(&gs.text_line_matrix);
                        gs.text_matrix = gs.text_line_matrix;
                        log::debug!("T*: text_matrix now: {:?}", gs.text_matrix);
                    }
                },
                Operator::Tf { font, size } => {
                    let gs = gs_stack.current_mut();
                    gs.font_name = Some(font.clone());
                    gs.font_size = *size;
                },

                // Extended graphics state
                Operator::SetExtGState { dict_name } => {
                    // Fast path: resource dict is already resolved (see top of
                    // this function), so the per-`gs` cost is one HashMap
                    // lookup + one resolve of the small inner state dict.
                    let entry = ext_g_state_cache
                        .entry(dict_name.clone())
                        .or_insert_with(|| {
                            if let Some(states) = ext_g_states {
                                if let Some(state_obj) = states.get(dict_name) {
                                    return parse_ext_g_state_inner(state_obj, doc)
                                        .unwrap_or_default();
                                }
                            }
                            ParsedExtGState::default()
                        });
                    entry.apply(gs_stack.current_mut());
                },

                // EndPath (n operator): discard current path without painting,
                // but apply any pending clip. Per PDF spec, W n is the standard
                // way to set a clipping path without filling or stroking.
                // Suppress the clip application inside an excluded OCG scope so
                // the clip doesn't leak past EMC into visible content.
                Operator::EndPath => {
                    if excluded_layer_depth == 0 {
                        apply_pending_clip(
                            &mut pending_clip,
                            &mut clip_stack,
                            pixmap,
                            base_transform,
                            &gs_stack,
                        );
                    } else {
                        // Drop any pending clip without applying it.
                        let _ = pending_clip.take();
                    }
                    current_path = PathBuilder::new();
                },

                // Shading (gradient) operator — suppressed when inside excluded layer
                Operator::PaintShading { name } => {
                    if excluded_layer_depth == 0 {
                        let gs = gs_stack.current();
                        let transform = combine_transforms(base_transform, &gs.ctm);
                        let clip = clip_stack.last().and_then(|c| c.as_ref());
                        self.render_shading(pixmap, name, transform, gs, resources, doc, clip)?;
                    }
                },

                // Marked content operators — track OCG layer exclusion
                Operator::BeginMarkedContent { .. } => {
                    marked_content_is_excluded.push(false);
                },
                Operator::BeginMarkedContentDict { tag, properties } => {
                    let mut is_excluded = false;
                    // Tag "OC" scopes can hide content even with empty excluded_layers
                    // when the OCMD uses /VE /Not or /P /AllOff/AnyOff (the
                    // expression evaluates with all OCGs on by default). We can
                    // only short-circuit cheaply for simple OCG refs, which the
                    // optional_content module handles internally.
                    if tag == "OC" {
                        is_excluded = crate::optional_content::resolve_and_check_ocg_excluded(
                            properties,
                            Some(resources),
                            Some(doc),
                            excluded_layers,
                        );
                    }
                    if is_excluded {
                        excluded_layer_depth += 1;
                    }
                    marked_content_is_excluded.push(is_excluded);
                },
                Operator::EndMarkedContent => {
                    if let Some(was_excluded) = marked_content_is_excluded.pop() {
                        if was_excluded && excluded_layer_depth > 0 {
                            excluded_layer_depth -= 1;
                        }
                    }
                },

                _ => {},
            }
        }

        Ok(())
    }

    /// Render a shading pattern (gradient).
    fn render_shading(
        &self,
        pixmap: &mut Pixmap,
        name: &str,
        transform: Transform,
        gs: &GraphicsState,
        resources: &Object,
        doc: &PdfDocument,
        clip_mask: Option<&tiny_skia::Mask>,
    ) -> Result<()> {
        // Look up shading resource
        let shading_dict = if let Object::Dictionary(res_dict) = resources {
            if let Some(shading_res) = res_dict.get("Shading") {
                let resolved = doc.resolve_object(shading_res)?;
                if let Some(shadings) = resolved.as_dict() {
                    if let Some(sh_obj) = shadings.get(name) {
                        let sh = doc.resolve_object(sh_obj)?;
                        sh.as_dict().cloned()
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let shading = match shading_dict {
            Some(d) => d,
            None => {
                log::debug!("Shading '{}' not found in resources", name);
                return Ok(());
            },
        };

        let shading_type = shading
            .get("ShadingType")
            .and_then(|o| o.as_integer())
            .unwrap_or(0);

        // Pre-resolve gradient endpoint colours through the resolution
        // pipeline when the toggle is on and the shading type is one we
        // migrate (axial=2, radial=3). For both types the endpoint
        // colours live in the shading's `/Function` (Type 2 exponential
        // interpolation puts the endpoints directly in `/C0` and
        // `/C1`; Type 3 stitching wraps a sub-function whose first /
        // last sub-functions carry them). The current inline path reads
        // `/C0` and `/C1` raw and treats them as already-RGB, which
        // silently truncates DeviceCMYK to its first three components
        // and drops Separation tint-transform evaluation entirely. The
        // pipeline-resolved endpoints respect the shading dict's
        // `/ColorSpace`, so a Type 4 Separation `/C0` becomes the
        // function's actual output rather than a `1 - tint` fall-back.
        //
        // Types 1 (function-based) and 4-7 (mesh) carry per-point /
        // per-vertex colours, not endpoints; this wave does NOT migrate
        // them. They fall straight through to the existing inline path,
        // unmodified.
        let resolved_endpoints =
            if self.pipeline_enabled_cache && (shading_type == 2 || shading_type == 3) {
                self.pipeline_resolve_shading_endpoints(&shading, gs, doc)
            } else {
                None
            };

        match shading_type {
            2 => self.render_axial_shading(
                pixmap,
                &shading,
                transform,
                gs,
                doc,
                clip_mask,
                resolved_endpoints,
            ),
            3 => self.render_radial_shading(
                pixmap,
                &shading,
                transform,
                gs,
                doc,
                clip_mask,
                resolved_endpoints,
            ),
            _ => {
                log::debug!("Unsupported shading type {} for '{}'", shading_type, name);
                Ok(())
            },
        }
    }

    /// Resolve a Type 2 / Type 3 shading dictionary's `/C0` and `/C1`
    /// endpoint colours through the resolution pipeline. The shading
    /// dict's `/ColorSpace` selects the colour space; `/Function` (a
    /// Type 2 exponential or a Type 3 stitching wrapper) carries the
    /// endpoint component arrays. Returns `None` when either endpoint
    /// can't be resolved (toggle off, missing `/Function`, unsupported
    /// sub-function type, non-RGBA resolver output, etc.) — the caller
    /// falls back to the existing inline behaviour in that case.
    ///
    /// This is the wave-4 hook into the resolution pipeline: it splits
    /// the "what colour" decision (now pipeline-resolved) from the
    /// "how to interpolate" decision (still owned by the gradient
    /// backend). The interpolation math is untouched — only the two
    /// fixed endpoint colours are routed through the pipeline.
    fn pipeline_resolve_shading_endpoints(
        &self,
        shading: &std::collections::HashMap<String, Object>,
        gs: &GraphicsState,
        doc: &PdfDocument,
    ) -> Option<((f32, f32, f32, f32), (f32, f32, f32, f32))> {
        // The shading dict's `/ColorSpace` can be a Name (DeviceRGB,
        // CS1, ...) or an inline Array ([/Separation ... funcRef]).
        // Resolve indirect references so the helper sees the final
        // shape.
        let cs_obj = shading.get("ColorSpace")?;
        let resolved_cs = doc.resolve_object(cs_obj).ok()?;

        // Per ISO 32000-1 §8.7.4.5.3, axial/radial shadings carry a
        // `/Domain` array on the shading dict (default `[0 1]`) that
        // names the parameter range mapped to the gradient axis.
        // Geometric `t=0` evaluates the function at `Domain[0]` and
        // `t=1` evaluates it at `Domain[1]` — the endpoints aren't
        // necessarily `f(0)` and `f(1)`.
        let (domain0, domain1) = shading
            .get("Domain")
            .and_then(|o| o.as_array())
            .and_then(|arr| {
                let d0 = arr.first()?;
                let d1 = arr.get(1)?;
                let parse = |o: &Object| -> Option<f32> {
                    match o {
                        Object::Real(v) => Some(*v as f32),
                        Object::Integer(v) => Some(*v as f32),
                        _ => None,
                    }
                };
                Some((parse(d0)?, parse(d1)?))
            })
            .unwrap_or((0.0, 1.0));

        // Extract endpoint component arrays from `/Function`. Handles
        // Type 2 (exponential) — where the endpoints are evaluated by
        // applying the shading's `/Domain` to the function's
        // exponential interpolation — and Type 3 (stitching) — where
        // the first sub-function's `/C0` and the last sub-function's
        // `/C1` are taken at face value. Type 3 with non-trivial
        // `/Encode` is not honoured (same gap as the inline
        // `evaluate_shading_function` path); see the body comment
        // below.
        let func_obj = shading.get("Function")?;
        let resolved_func = doc.resolve_object(func_obj).ok()?;
        let func_dict = resolved_func.as_dict()?;
        let func_type = func_dict.get("FunctionType").and_then(|o| o.as_integer())?;
        let to_components = |arr: &[Object]| -> Vec<f32> {
            arr.iter()
                .map(|o| match o {
                    Object::Real(v) => *v as f32,
                    Object::Integer(v) => *v as f32,
                    _ => 0.0,
                })
                .collect()
        };
        let (c0_comps, c1_comps) = match func_type {
            2 => {
                // Type 2: exponential interpolation
                // f(x) = C0 + x^N * (C1 - C0).
                // The shading's geometric `t=0` evaluates `f(Domain[0])`
                // and `t=1` evaluates `f(Domain[1])`, so when /Domain
                // is non-default the endpoint colours are NOT raw /C0
                // and /C1.
                let c0 = to_components(func_dict.get("C0").and_then(|o| o.as_array())?);
                let c1 = to_components(func_dict.get("C1").and_then(|o| o.as_array())?);
                let n = func_dict
                    .get("N")
                    .and_then(|o| match o {
                        Object::Real(v) => Some(*v as f32),
                        Object::Integer(v) => Some(*v as f32),
                        _ => None,
                    })
                    .unwrap_or(1.0);
                let eval = |x: f32| -> Vec<f32> {
                    let p = x.abs().powf(n) * x.signum();
                    c0.iter()
                        .zip(c1.iter())
                        .map(|(a, b)| *a + p * (*b - *a))
                        .collect()
                };
                (eval(domain0), eval(domain1))
            },
            3 => {
                // Type 3: stitching. The shading's `/Domain` maps to a
                // sub-function via stitching `/Bounds` and `/Encode`
                // arrays. The current path takes the first
                // sub-function's `/C0` and the last sub-function's
                // `/C1` at face value — correct for the default
                // `Domain [0 1]` with natural `Encode`, but ignores
                // `Encode`-driven sub-domain remapping. Documented
                // gap, same as `evaluate_shading_function`.
                let funcs = func_dict.get("Functions").and_then(|o| o.as_array())?;
                let first = funcs.first()?;
                let last = funcs.last().unwrap_or(first);
                let first_resolved = doc.resolve_object(first).ok()?;
                let last_resolved = doc.resolve_object(last).ok()?;
                let first_dict = first_resolved.as_dict()?;
                let last_dict = last_resolved.as_dict()?;
                let c0 = first_dict.get("C0").and_then(|o| o.as_array())?;
                let c1 = last_dict.get("C1").and_then(|o| o.as_array())?;
                (to_components(c0), to_components(c1))
            },
            // Function types 0 (sampled) and 4 (PostScript Type 4
            // calculator) used as the shading's own /Function are
            // out-of-scope for endpoint pre-resolution — they produce
            // colours at intermediate domain points, not at two fixed
            // /C0 / /C1 arrays. Caller falls back to inline.
            _ => return None,
        };

        // Fold in `gs.fill_alpha` here — it's the alpha the inline
        // code path multiplies into each gradient stop's RGBA when
        // building the tiny-skia LinearGradient / RadialGradient.
        let c0 = self.pipeline_resolve_components(
            doc,
            &self.color_spaces,
            &resolved_cs,
            &c0_comps,
            gs.fill_alpha,
        )?;
        let c1 = self.pipeline_resolve_components(
            doc,
            &self.color_spaces,
            &resolved_cs,
            &c1_comps,
            gs.fill_alpha,
        )?;
        Some((c0, c1))
    }

    /// Render axial (linear) gradient shading (Type 2).
    ///
    /// `resolved_endpoints`, when `Some`, supplies pre-resolved RGBA
    /// values for the two gradient stops with `gs.fill_alpha` already
    /// folded in — the resolution-pipeline route produced by
    /// [`Self::pipeline_resolve_shading_endpoints`]. When `None`, the
    /// function falls back to the legacy
    /// [`Self::evaluate_shading_function`] path, which reads `/C0` and
    /// `/C1` raw and assumes they are already RGB triples.
    fn render_axial_shading(
        &self,
        pixmap: &mut Pixmap,
        shading: &std::collections::HashMap<String, Object>,
        transform: Transform,
        gs: &GraphicsState,
        doc: &PdfDocument,
        clip_mask: Option<&tiny_skia::Mask>,
        resolved_endpoints: Option<((f32, f32, f32, f32), (f32, f32, f32, f32))>,
    ) -> Result<()> {
        // Parse Coords [x0 y0 x1 y1]
        let coords = shading.get("Coords").and_then(|o| o.as_array());
        let coords = match coords {
            Some(c) if c.len() >= 4 => c,
            _ => return Ok(()),
        };
        let get_f = |i: usize| -> f32 {
            match &coords[i] {
                Object::Real(v) => *v as f32,
                Object::Integer(v) => *v as f32,
                _ => 0.0,
            }
        };
        let (x0, y0, x1, y1) = (get_f(0), get_f(1), get_f(2), get_f(3));

        // Parse Extend [bool bool]
        let extend = shading.get("Extend").and_then(|o| o.as_array());
        let (extend_start, extend_end) = if let Some(ext) = extend {
            let e0 = ext
                .get(0)
                .map(|o| matches!(o, Object::Boolean(true)))
                .unwrap_or(false);
            let e1 = ext
                .get(1)
                .map(|o| matches!(o, Object::Boolean(true)))
                .unwrap_or(false);
            (e0, e1)
        } else {
            (false, false)
        };

        // Build the two gradient-stop RGBAs. When the resolution
        // pipeline pre-resolved the endpoints, use those directly
        // (alpha already folded). Otherwise fall back to the legacy
        // raw-`/C0`-as-RGB read; in that case alpha is folded here.
        let (stop0, stop1) = if let Some(((r0, g0, b0, a0), (r1, g1, b1, a1))) = resolved_endpoints
        {
            ((r0, g0, b0, a0), (r1, g1, b1, a1))
        } else {
            let (c0, c1) = self.evaluate_shading_function(shading, doc)?;
            ((c0.0, c0.1, c0.2, gs.fill_alpha), (c1.0, c1.1, c1.2, gs.fill_alpha))
        };

        // Transform gradient endpoints
        let mut p0 = tiny_skia::Point { x: x0, y: y0 };
        let mut p1 = tiny_skia::Point { x: x1, y: y1 };
        transform.map_point(&mut p0);
        transform.map_point(&mut p1);

        // Create gradient
        let spread = if extend_start && extend_end {
            tiny_skia::SpreadMode::Pad
        } else {
            tiny_skia::SpreadMode::Pad // tiny-skia default
        };

        let gradient = tiny_skia::LinearGradient::new(
            tiny_skia::Point { x: p0.x, y: p0.y },
            tiny_skia::Point { x: p1.x, y: p1.y },
            vec![
                tiny_skia::GradientStop::new(
                    0.0,
                    tiny_skia::Color::from_rgba(stop0.0, stop0.1, stop0.2, stop0.3)
                        .unwrap_or(tiny_skia::Color::BLACK),
                ),
                tiny_skia::GradientStop::new(
                    1.0,
                    tiny_skia::Color::from_rgba(stop1.0, stop1.1, stop1.2, stop1.3)
                        .unwrap_or(tiny_skia::Color::BLACK),
                ),
            ],
            spread,
            Transform::identity(),
        );

        if let Some(shader) = gradient {
            let mut paint = tiny_skia::Paint::default();
            paint.shader = shader;
            paint.anti_alias = true;

            // Fill entire pixmap with gradient (clipped by clip_mask)
            let rect =
                tiny_skia::Rect::from_xywh(0.0, 0.0, pixmap.width() as f32, pixmap.height() as f32)
                    .unwrap();
            let path = PathBuilder::from_rect(rect);
            pixmap.fill_path(
                &path,
                &paint,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                clip_mask,
            );
            log::debug!(
                "Rendered axial gradient from ({:.1},{:.1}) to ({:.1},{:.1})",
                p0.x,
                p0.y,
                p1.x,
                p1.y
            );
        }

        Ok(())
    }

    /// Render radial gradient shading (Type 3).
    ///
    /// `resolved_endpoints`, when `Some`, supplies pre-resolved RGBA
    /// values for the two gradient stops with `gs.fill_alpha` already
    /// folded in — the resolution-pipeline route produced by
    /// [`Self::pipeline_resolve_shading_endpoints`]. When `None`, the
    /// function falls back to the legacy
    /// [`Self::evaluate_shading_function`] path, which reads `/C0` and
    /// `/C1` raw and assumes they are already RGB triples.
    fn render_radial_shading(
        &self,
        pixmap: &mut Pixmap,
        shading: &std::collections::HashMap<String, Object>,
        transform: Transform,
        gs: &GraphicsState,
        doc: &PdfDocument,
        clip_mask: Option<&tiny_skia::Mask>,
        resolved_endpoints: Option<((f32, f32, f32, f32), (f32, f32, f32, f32))>,
    ) -> Result<()> {
        // Parse Coords [x0 y0 r0 x1 y1 r1]
        let coords = shading.get("Coords").and_then(|o| o.as_array());
        let coords = match coords {
            Some(c) if c.len() >= 6 => c,
            _ => return Ok(()),
        };
        let get_f = |i: usize| -> f32 {
            match &coords[i] {
                Object::Real(v) => *v as f32,
                Object::Integer(v) => *v as f32,
                _ => 0.0,
            }
        };
        let (x0, y0, r0, x1, y1, r1) = (get_f(0), get_f(1), get_f(2), get_f(3), get_f(4), get_f(5));

        // Same pipeline-or-fallback dispatch as `render_axial_shading`
        // — see its docs for the rationale.
        let (stop0, stop1) =
            if let Some(((r0c, g0, b0, a0), (r1c, g1, b1, a1))) = resolved_endpoints {
                ((r0c, g0, b0, a0), (r1c, g1, b1, a1))
            } else {
                let (c0, c1) = self.evaluate_shading_function(shading, doc)?;
                ((c0.0, c0.1, c0.2, gs.fill_alpha), (c1.0, c1.1, c1.2, gs.fill_alpha))
            };

        // Per ISO 32000-1 §8.7.4.5.4, the radial gradient interpolates
        // between two circles `(x0, y0, r0)` (the inner / start circle,
        // mapped to the function value at the gradient's `Domain[0]`)
        // and `(x1, y1, r1)` (the outer / end circle, mapped to
        // `Domain[1]`). When `(x0, y0) == (x1, y1)` and `r0 == 0` the
        // result is a familiar centred radial; non-concentric inputs
        // produce off-centre / cone gradients that real PDFs use for
        // highlight, spotlight, and lens effects.
        let mut center0 = tiny_skia::Point { x: x0, y: y0 };
        let mut edge0 = tiny_skia::Point { x: x0 + r0, y: y0 };
        let mut center1 = tiny_skia::Point { x: x1, y: y1 };
        let mut edge1 = tiny_skia::Point { x: x1 + r1, y: y1 };
        transform.map_point(&mut center0);
        transform.map_point(&mut edge0);
        transform.map_point(&mut center1);
        transform.map_point(&mut edge1);
        let radius0 = ((edge0.x - center0.x).powi(2) + (edge0.y - center0.y).powi(2)).sqrt();
        let radius1 = ((edge1.x - center1.x).powi(2) + (edge1.y - center1.y).powi(2)).sqrt();

        let gradient = tiny_skia::RadialGradient::new(
            tiny_skia::Point {
                x: center0.x,
                y: center0.y,
            },
            radius0, // start_radius (inner circle, in device space)
            tiny_skia::Point {
                x: center1.x,
                y: center1.y,
            },
            radius1, // end_radius (outer circle, in device space)
            vec![
                tiny_skia::GradientStop::new(
                    0.0,
                    tiny_skia::Color::from_rgba(stop0.0, stop0.1, stop0.2, stop0.3)
                        .unwrap_or(tiny_skia::Color::BLACK),
                ),
                tiny_skia::GradientStop::new(
                    1.0,
                    tiny_skia::Color::from_rgba(stop1.0, stop1.1, stop1.2, stop1.3)
                        .unwrap_or(tiny_skia::Color::BLACK),
                ),
            ],
            tiny_skia::SpreadMode::Pad,
            Transform::identity(),
        );

        if let Some(shader) = gradient {
            let mut paint = tiny_skia::Paint::default();
            paint.shader = shader;
            paint.anti_alias = true;
            let rect =
                tiny_skia::Rect::from_xywh(0.0, 0.0, pixmap.width() as f32, pixmap.height() as f32)
                    .unwrap();
            let path = PathBuilder::from_rect(rect);
            pixmap.fill_path(
                &path,
                &paint,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                clip_mask,
            );
            log::debug!(
                "Rendered radial gradient from ({:.1},{:.1}) r={:.1} to ({:.1},{:.1}) r={:.1}",
                center0.x,
                center0.y,
                radius0,
                center1.x,
                center1.y,
                radius1,
            );
        }

        Ok(())
    }

    /// Evaluate a shading function at the shading's `/Domain` endpoints
    /// to get start/end colors. Per ISO 32000-1 §8.7.4.5.3 the gradient
    /// parameter `t ∈ [0, 1]` maps to function input `x ∈ [Domain[0],
    /// Domain[1]]` (default `[0, 1]`), so the start colour is
    /// `f(Domain[0])` and the end colour is `f(Domain[1])`.
    fn evaluate_shading_function(
        &self,
        shading: &std::collections::HashMap<String, Object>,
        doc: &PdfDocument,
    ) -> Result<((f32, f32, f32), (f32, f32, f32))> {
        let (domain0, domain1) = shading
            .get("Domain")
            .and_then(|o| o.as_array())
            .and_then(|arr| {
                let parse = |o: &Object| -> Option<f32> {
                    match o {
                        Object::Real(v) => Some(*v as f32),
                        Object::Integer(v) => Some(*v as f32),
                        _ => None,
                    }
                };
                Some((parse(arr.first()?)?, parse(arr.get(1)?)?))
            })
            .unwrap_or((0.0, 1.0));

        // Try to parse a simple Type 2 (exponential interpolation) or Type 0 (sampled) function
        let func_obj = shading.get("Function");
        if let Some(func) = func_obj {
            let resolved = doc.resolve_object(func)?;
            if let Some(func_dict) = resolved.as_dict() {
                let func_type = func_dict
                    .get("FunctionType")
                    .and_then(|o| o.as_integer())
                    .unwrap_or(-1);

                if func_type == 2 {
                    // Type 2: Exponential interpolation f(x) = C0 + x^N * (C1 - C0).
                    // Evaluate at the shading's /Domain endpoints, not
                    // raw /C0 and /C1.
                    let c0 = func_dict
                        .get("C0")
                        .and_then(|o| o.as_array())
                        .map(|arr| Self::parse_color_array(arr))
                        .unwrap_or((0.0, 0.0, 0.0));
                    let c1 = func_dict
                        .get("C1")
                        .and_then(|o| o.as_array())
                        .map(|arr| Self::parse_color_array(arr))
                        .unwrap_or((1.0, 1.0, 1.0));
                    let n = func_dict
                        .get("N")
                        .and_then(|o| match o {
                            Object::Real(v) => Some(*v as f32),
                            Object::Integer(v) => Some(*v as f32),
                            _ => None,
                        })
                        .unwrap_or(1.0);
                    let lerp = |x: f32| -> (f32, f32, f32) {
                        let p = x.abs().powf(n) * x.signum();
                        (
                            c0.0 + p * (c1.0 - c0.0),
                            c0.1 + p * (c1.1 - c0.1),
                            c0.2 + p * (c1.2 - c0.2),
                        )
                    };
                    return Ok((lerp(domain0), lerp(domain1)));
                } else if func_type == 3 {
                    // Type 3: Stitching function — wraps multiple
                    // sub-functions. For gradient endpoints, read the
                    // first sub-function's `/C0` and the last
                    // sub-function's `/C1` at face value. Per ISO
                    // 32000-1 §7.10.4, the stitching `/Domain` is
                    // partitioned by `/Bounds` and remapped per
                    // sub-function via `/Encode`. The path below
                    // honours neither — correct for the default
                    // `Domain [0 1]` with natural `Encode` arrays, but
                    // misrepresents non-default `Encode`-driven
                    // sub-domain remapping. Documented gap.
                    if let Some(funcs) = func_dict.get("Functions").and_then(|o| o.as_array()) {
                        if let Some(first_func) = funcs.first() {
                            let sub_resolved = doc.resolve_object(first_func)?;
                            if let Some(sub_dict) = sub_resolved.as_dict() {
                                let sub_type = sub_dict
                                    .get("FunctionType")
                                    .and_then(|o| o.as_integer())
                                    .unwrap_or(-1);
                                if sub_type == 2 {
                                    let c0 = sub_dict
                                        .get("C0")
                                        .and_then(|o| o.as_array())
                                        .map(|arr| Self::parse_color_array(arr))
                                        .unwrap_or((0.0, 0.0, 0.0));
                                    // For last color, check last sub-function if multiple
                                    let last_func_obj = funcs.last().unwrap_or(first_func);
                                    let last_resolved = doc.resolve_object(last_func_obj)?;
                                    let c1 = last_resolved
                                        .as_dict()
                                        .and_then(|d| d.get("C1"))
                                        .and_then(|o| o.as_array())
                                        .map(|arr| Self::parse_color_array(arr))
                                        .unwrap_or((1.0, 1.0, 1.0));
                                    return Ok((c0, c1));
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(((0.0, 0.0, 0.0), (1.0, 1.0, 1.0)))
    }

    fn parse_color_array(arr: &[Object]) -> (f32, f32, f32) {
        let get = |i: usize| -> f32 {
            arr.get(i)
                .map(|o| match o {
                    Object::Real(v) => *v as f32,
                    Object::Integer(v) => *v as f32,
                    _ => 0.0,
                })
                .unwrap_or(0.0)
        };
        if arr.len() >= 3 {
            (get(0), get(1), get(2))
        } else if arr.len() == 1 {
            let g = get(0);
            (g, g, g) // Grayscale
        } else {
            (0.0, 0.0, 0.0)
        }
    }

    /// Render an XObject (image or form).
    fn render_xobject(
        &mut self,
        pixmap: &mut Pixmap,
        name: &str,
        transform: Transform,
        gs: &GraphicsState,
        resources: &Object,
        doc: &PdfDocument,
        page_num: usize,
        clip_mask: Option<&tiny_skia::Mask>,
    ) -> Result<()> {
        // Get XObject from resources
        if let Object::Dictionary(res_dict) = resources {
            // PDF spec uses "XObject" (singular)
            if let Some(xobj_entry) = res_dict.get("XObject") {
                let xobjects_obj = doc.resolve_object(xobj_entry)?;
                if let Some(xobjects) = xobjects_obj.as_dict() {
                    if let Some(xobj_ref_obj) = xobjects.get(name) {
                        // Resolve reference if needed
                        let xobj = doc.resolve_object(xobj_ref_obj)?;
                        let xobj_ref = xobj_ref_obj.as_reference();
                        log::debug!("Resolved XObject '{}' type: {:?}", name, xobj);

                        if let Object::Stream { ref dict, .. } = xobj {
                            if let Some(smask) = dict.get("SMask") {
                                log::debug!("Image has SMask: {:?}", smask);
                            }
                            if let Some(mask) = dict.get("Mask") {
                                log::debug!("Image has Mask: {:?}", mask);
                            }
                            if let Some(imask) = dict.get("ImageMask") {
                                log::debug!("Image is ImageMask: {:?}", imask);
                            }
                            // Check subtype
                            if let Some(subtype) = dict.get("Subtype").and_then(|o| o.as_name()) {
                                match subtype {
                                    "Image" => {
                                        // ImageMask XObjects (1-bit stencil painted with
                                        // the current fill colour) take their fill from
                                        // graphics state, not from the pixel data. Route
                                        // that fill through the resolution pipeline
                                        // (toggle-on) so a Type 4 Separation fill paints
                                        // the mask with the function-evaluated tint
                                        // rather than the inline `1 - tint` fallback.
                                        //
                                        // Standard images (`/ImageMask` absent or false)
                                        // carry their colour in the pixel data and do
                                        // not interact with the pipeline; they pass
                                        // straight through to `render_image`.
                                        let is_image_mask = dict
                                            .get("ImageMask")
                                            .map(|o| matches!(o, Object::Boolean(true)))
                                            .unwrap_or(false);
                                        if is_image_mask {
                                            let spliced = self.pipeline_resolve_paint_gs(
                                                doc,
                                                gs,
                                                PipelinePaintKind::ImageMask,
                                            );
                                            let render_gs: &GraphicsState =
                                                spliced.as_ref().unwrap_or(gs);
                                            if let Err(e) = self.render_image_mask(
                                                pixmap, &xobj, xobj_ref, transform, doc, clip_mask,
                                                render_gs,
                                            ) {
                                                log::warn!(
                                                    "Skipping unrenderable ImageMask XObject '{}': {}",
                                                    name,
                                                    e
                                                );
                                            }
                                        } else {
                                            let smask = dict.get("SMask").cloned();
                                            let mask = dict.get("Mask").cloned();
                                            if let Err(e) = self.render_image(
                                                pixmap, &xobj, xobj_ref, transform, doc, clip_mask,
                                                smask, mask, gs,
                                            ) {
                                                log::warn!(
                                                    "Skipping unrenderable image XObject '{}': {}",
                                                    name,
                                                    e
                                                );
                                            }
                                        }
                                    },
                                    "Form" => {
                                        log::debug!("XObject '{}' is a Form", name);
                                        // Decoded stream data
                                        let stream_data = if let Some(r) = xobj_ref {
                                            doc.decode_stream_with_encryption(&xobj, r)?
                                        } else {
                                            xobj.decode_stream_data()?
                                        };

                                        // Form XObjects can have their own Resources dictionary.
                                        let form_resources =
                                            dict.get("Resources").unwrap_or(resources);

                                        // Save current fonts and load form-specific fonts
                                        let old_fonts = self.fonts.clone();
                                        let old_cs = self.color_spaces.clone();
                                        self.load_resources(doc, form_resources)?;

                                        if let Err(e) = self.render_form_xobject(
                                            pixmap,
                                            &dict,
                                            &stream_data,
                                            transform,
                                            doc,
                                            page_num,
                                            form_resources,
                                        ) {
                                            log::warn!(
                                                "Skipping malformed Form XObject '{}': {}",
                                                name,
                                                e
                                            );
                                        }

                                        // Restore caches
                                        self.fonts = old_fonts;
                                        self.color_spaces = old_cs;
                                    },
                                    _ => {},
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Render an image XObject.
    fn render_image(
        &mut self,
        pixmap: &mut Pixmap,
        xobject: &Object,
        obj_ref: Option<ObjectRef>,
        transform: Transform,
        doc: &PdfDocument,
        clip_mask: Option<&tiny_skia::Mask>,
        smask_obj: Option<Object>,
        mask_obj: Option<Object>,
        gs: &GraphicsState,
    ) -> Result<()> {
        use crate::extractors::images::extract_image_from_xobject;

        // Use robust image extractor to handle various formats and color spaces
        let color_space_map = self.color_spaces.clone();
        let pdf_image =
            extract_image_from_xobject(Some(doc), xobject, obj_ref, Some(&color_space_map))?;
        let dynamic_image = pdf_image.to_dynamic_image()?;
        let mut rgba_image = dynamic_image.to_rgba8();

        // Handle /Mask (stencil mask image) — PDF spec section 8.9.6.2
        // The mask is a separate image whose samples define opacity (1=opaque, 0=transparent)
        if let Some(mask_ref) = mask_obj {
            if let Some(ref_obj) = mask_ref.as_reference() {
                if let Ok(mask_stream) = doc.load_object(ref_obj) {
                    // Try to decode the mask as an image
                    match extract_image_from_xobject(
                        Some(doc),
                        &mask_stream,
                        Some(ref_obj),
                        Some(&color_space_map),
                    ) {
                        Ok(mask_image) => {
                            if let Ok(mask_dyn) = mask_image.to_dynamic_image() {
                                let mask_gray = mask_dyn.to_luma8();
                                let mw = mask_gray.width();
                                let mh = mask_gray.height();
                                let iw = rgba_image.width();
                                let ih = rgba_image.height();
                                for y in 0..ih {
                                    for x in 0..iw {
                                        let mx = (x * mw / iw).min(mw - 1);
                                        let my = (y * mh / ih).min(mh - 1);
                                        let mask_val = mask_gray.get_pixel(mx, my)[0];
                                        let pixel = rgba_image.get_pixel_mut(x, y);
                                        pixel[3] =
                                            ((pixel[3] as u32 * mask_val as u32) / 255) as u8;
                                    }
                                }
                                log::debug!(
                                    "Applied image Mask ({}x{}) to image ({}x{})",
                                    mw,
                                    mh,
                                    iw,
                                    ih
                                );
                            }
                        },
                        Err(_) => {
                            // Fallback: decode stencil mask (ImageMask=true) directly from stream
                            if let Object::Stream { ref dict, .. } = mask_stream {
                                let mask_dict = dict;
                                let is_image_mask = mask_dict
                                    .get("ImageMask")
                                    .map(|o| matches!(o, Object::Boolean(true)))
                                    .unwrap_or(false);
                                if is_image_mask {
                                    let mw = mask_dict
                                        .get("Width")
                                        .and_then(|o| o.as_integer())
                                        .unwrap_or(0)
                                        as u32;
                                    let mh = mask_dict
                                        .get("Height")
                                        .and_then(|o| o.as_integer())
                                        .unwrap_or(0)
                                        as u32;
                                    if mw > 0 && mh > 0 {
                                        if let Ok(raw_mask_data) =
                                            doc.decode_stream_with_encryption(&mask_stream, ref_obj)
                                        {
                                            // CCITT data may be pass-through (not decompressed).
                                            // Check if we need to decompress Group 4 CCITT.
                                            let expected_bytes =
                                                ((mw as usize + 7) / 8) * mh as usize;
                                            let mask_data = if raw_mask_data.len()
                                                < expected_bytes / 2
                                            {
                                                // Data is still compressed — try Group 4 CCITT decompression
                                                let k = mask_dict
                                                    .get("DecodeParms")
                                                    .and_then(|o| o.as_dict())
                                                    .and_then(|d| d.get("K"))
                                                    .and_then(|o| o.as_integer())
                                                    .unwrap_or(0);
                                                if k == -1 {
                                                    #[allow(deprecated)]
                                                    let ccitt_result = crate::extractors::ccitt_bilevel::decompress_ccitt_group4(&raw_mask_data, mw, mh);
                                                    match ccitt_result {
                                                        Ok(decompressed) => {
                                                            log::debug!("CCITT Group4 decompressed mask: {} → {} bytes", raw_mask_data.len(), decompressed.len());
                                                            decompressed
                                                        },
                                                        Err(e) => {
                                                            log::debug!("CCITT decompression failed: {}, using raw data", e);
                                                            raw_mask_data
                                                        },
                                                    }
                                                } else {
                                                    raw_mask_data
                                                }
                                            } else {
                                                raw_mask_data
                                            };
                                            // 1-bit mask: each byte has 8 pixels, MSB first
                                            let iw = rgba_image.width();
                                            let ih = rgba_image.height();
                                            let row_bytes = (mw as usize + 7) / 8;
                                            for y in 0..ih {
                                                for x in 0..iw {
                                                    let mx = (x * mw / iw).min(mw - 1) as usize;
                                                    let my = (y * mh / ih).min(mh - 1) as usize;
                                                    let byte_idx = my * row_bytes + mx / 8;
                                                    let bit_idx = 7 - (mx % 8);
                                                    // PDF spec 8.9.6.2: mask bit 1 = paint (opaque), 0 = don't paint (transparent)
                                                    let mask_val = if byte_idx < mask_data.len() {
                                                        if (mask_data[byte_idx] >> bit_idx) & 1 == 1
                                                        {
                                                            255u8
                                                        } else {
                                                            0u8
                                                        }
                                                    } else {
                                                        255u8
                                                    };
                                                    let pixel = rgba_image.get_pixel_mut(x, y);
                                                    pixel[3] = ((pixel[3] as u32 * mask_val as u32)
                                                        / 255)
                                                        as u8;
                                                }
                                            }
                                            log::debug!("Applied stencil ImageMask ({}x{}) to image ({}x{})", mw, mh, iw, ih);
                                        }
                                    }
                                }
                            }
                        },
                    }
                }
            }
            // If Mask is an array, it's a color-key mask (not yet implemented)
        }

        // Handle SMask if present
        if let Some(smask_ref) = smask_obj {
            if let Ok(resolved_smask) = doc.resolve_object(&smask_ref) {
                let smask_obj_ref = smask_ref.as_reference();
                if let Ok(smask_image) = extract_image_from_xobject(
                    Some(doc),
                    &resolved_smask,
                    smask_obj_ref,
                    Some(&color_space_map),
                ) {
                    if let Ok(smask_dyn) = smask_image.to_dynamic_image() {
                        let smask_gray = smask_dyn.to_luma8();

                        // Apply SMask to alpha channel
                        // Rescale smask if dimensions don't match (simplification)
                        let sw = smask_gray.width();
                        let sh = smask_gray.height();
                        let iw = rgba_image.width();
                        let ih = rgba_image.height();

                        for y in 0..ih {
                            for x in 0..iw {
                                // Map image coordinate to smask coordinate
                                let sx = (x * sw / iw).min(sw - 1);
                                let sy = (y * sh / ih).min(sh - 1);
                                let alpha = smask_gray.get_pixel(sx, sy)[0];

                                let pixel = rgba_image.get_pixel_mut(x, y);
                                // Combine with existing alpha
                                pixel[3] = ((pixel[3] as u32 * alpha as u32) / 255) as u8;
                            }
                        }
                    }
                }
            }
        }

        let src_w = rgba_image.width();
        let src_h = rgba_image.height();

        let image_transform = image_unit_square_transform(transform, src_w, src_h);
        let mut paint = pixmap_paint_for_image_blit(image_transform, gs.fill_alpha, &gs.blend_mode);

        // Fast path: SIMD pre-resize when the transform is a pure scale+translate and
        // the image is being downscaled.  fast_image_resize (AVX2/SSE4.1/NEON) resizes
        // to exact output dimensions; we then blit the already-correct pixels at the
        // right position with a translate-only transform and Nearest quality (no second
        // resampling pass).  For rotated/sheared transforms or upscaling, fall through
        // to the tiny-skia bilinear/bicubic path (already selected by the helper above).
        let use_fast = image_transform.kx.abs() <= 1e-4
            && image_transform.ky.abs() <= 1e-4
            && image_transform.sx > 0.0
            && image_transform.sy > 0.0
            && (image_transform.sx < 0.9 || image_transform.sy < 0.9);

        let (blit_w, blit_h, blit_data, blit_transform) = if use_fast {
            let dst_w = ((image_transform.sx * src_w as f32).round() as u32).max(1);
            let dst_h = ((image_transform.sy * src_h as f32).round() as u32).max(1);
            let resized = resize_rgba(rgba_image.as_raw(), src_w, src_h, dst_w, dst_h);
            if let Some(pixels) = resized {
                // SIMD pre-resize produced the exact output dimensions —
                // the subsequent blit is 1:1, so override to Nearest to
                // skip a second resampling pass.
                paint.quality = tiny_skia::FilterQuality::Nearest;
                let t = Transform::from_translate(image_transform.tx, image_transform.ty);
                (dst_w, dst_h, pixels, t)
            } else {
                // fast_image_resize failed; fall back to tiny_skia
                // resampling with the helper's chosen quality.
                (src_w, src_h, rgba_image.into_raw(), image_transform)
            }
        } else {
            // Rotated / sheared / upscaling path: let tiny_skia resample
            // with the helper's chosen quality.
            (src_w, src_h, rgba_image.into_raw(), image_transform)
        };

        if let Some(img_pixmap) =
            Pixmap::from_vec(blit_data, tiny_skia::IntSize::from_wh(blit_w, blit_h).unwrap())
        {
            pixmap.draw_pixmap(0, 0, img_pixmap.as_ref(), &paint, blit_transform, clip_mask);
        }

        Ok(())
    }

    /// Render an Image XObject with `/ImageMask true` — a 1-bit stencil
    /// painted with the current fill colour.
    ///
    /// Per ISO 32000-1 §8.9.6.4, under the default `/Decode [0 1]` a
    /// sample value of `0` paints the destination with the current
    /// nonstroking colour and `1` leaves it unaffected; `/Decode [1 0]`
    /// reverses the polarity. There is no `/ColorSpace`; the colour
    /// comes from `gs.fill_color_rgb` / `gs.fill_alpha`. The caller (the
    /// `Do` arm in `render_page_with_options`) is responsible for
    /// routing that fill through the resolution pipeline when the
    /// toggle is on, so this helper consumes whatever `gs` it is handed
    /// without re-resolving.
    ///
    /// Only the minimum necessary to make the stencil paintable is
    /// implemented here: 1-bit raw samples (no CCITT decode), default
    /// and inverted `/Decode` polarities, bilinear/bicubic resampling
    /// chosen by the image-space-to-user-space scale (matches
    /// `render_image`). CCITT-compressed inline masks are out of scope
    /// for wave 3 — they share the colour-resolution path and gain the
    /// same pipeline routing as soon as their decode is added.
    fn render_image_mask(
        &mut self,
        pixmap: &mut Pixmap,
        xobject: &Object,
        obj_ref: Option<ObjectRef>,
        transform: Transform,
        doc: &PdfDocument,
        clip_mask: Option<&tiny_skia::Mask>,
        gs: &GraphicsState,
    ) -> Result<()> {
        let dict = xobject
            .as_dict()
            .ok_or_else(|| Error::Image("ImageMask XObject is not a stream".to_string()))?;

        let width = dict
            .get("Width")
            .and_then(|o| o.as_integer())
            .ok_or_else(|| Error::Image("ImageMask missing /Width".to_string()))?
            as u32;
        let height = dict
            .get("Height")
            .and_then(|o| o.as_integer())
            .ok_or_else(|| Error::Image("ImageMask missing /Height".to_string()))?
            as u32;
        if width == 0 || height == 0 {
            return Ok(());
        }

        // PDF §8.9.6.4: ImageMask BitsPerComponent must be 1 when present.
        // Some producers omit it; default to 1.
        let bpc = dict
            .get("BitsPerComponent")
            .and_then(|o| o.as_integer())
            .unwrap_or(1);
        if bpc != 1 {
            return Err(Error::Image(format!("ImageMask requires BitsPerComponent 1, got {bpc}")));
        }

        // /Decode array: [0 1] means bit 1 = opaque (default); [1 0]
        // inverts. Other forms are spec-illegal for ImageMask.
        let invert = match dict.get("Decode") {
            Some(Object::Array(arr)) if arr.len() >= 2 => {
                let first = match &arr[0] {
                    Object::Real(v) => *v as f32,
                    Object::Integer(v) => *v as f32,
                    _ => 0.0,
                };
                first > 0.5
            },
            _ => false,
        };

        let raw = if let Some(r) = obj_ref {
            doc.decode_stream_with_encryption(xobject, r)?
        } else {
            xobject.decode_stream_data()?
        };

        // Stencil pixels → premultiplied RGBA, applying the fill colour
        // to each opaque sample. Rows are packed MSB-first; each row is
        // padded to the next byte boundary.
        let (fr, fg, fb) = gs.fill_color_rgb;
        let fa = gs.fill_alpha.clamp(0.0, 1.0);
        let pa = (fa * 255.0).round().clamp(0.0, 255.0) as u8;
        // Premultiplied opaque sample: tiny-skia's Pixmap is
        // premultiplied; build the channels accordingly so blends and
        // SMask composition stay correct.
        let pr = ((fr.clamp(0.0, 1.0) * fa) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8;
        let pg = ((fg.clamp(0.0, 1.0) * fa) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8;
        let pb = ((fb.clamp(0.0, 1.0) * fa) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8;

        let row_bytes = (width as usize + 7) / 8;
        let expected = row_bytes * height as usize;
        if raw.len() < expected {
            return Err(Error::Image(format!(
                "ImageMask stream too short: {} bytes for {}x{} (expected {})",
                raw.len(),
                width,
                height,
                expected
            )));
        }

        let mut rgba: Vec<u8> = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            let row_off = (y as usize) * row_bytes;
            for x in 0..width {
                let byte_idx = row_off + (x / 8) as usize;
                let bit_idx = 7 - (x % 8);
                let bit = (raw[byte_idx] >> bit_idx) & 1 == 1;
                let opaque = if invert { bit } else { !bit };
                if opaque {
                    let off = ((y * width + x) * 4) as usize;
                    rgba[off] = pr;
                    rgba[off + 1] = pg;
                    rgba[off + 2] = pb;
                    rgba[off + 3] = pa;
                }
            }
        }

        let image_transform = image_unit_square_transform(transform, width, height);
        // Opacity is 1.0 because fill_alpha is already baked into the
        // stencil pixels by the loop above; blend mode + scale-driven
        // quality come from the shared helper.
        let paint = pixmap_paint_for_image_blit(image_transform, 1.0, &gs.blend_mode);

        if let Some(stencil_pixmap) = Pixmap::from_vec(
            rgba,
            tiny_skia::IntSize::from_wh(width, height)
                .ok_or_else(|| Error::Image("ImageMask invalid dimensions".to_string()))?,
        ) {
            pixmap.draw_pixmap(0, 0, stencil_pixmap.as_ref(), &paint, image_transform, clip_mask);
        }

        Ok(())
    }

    /// Render a Form XObject by parsing its content stream recursively.
    ///
    /// Per PDF spec §8.10, a Form XObject contains its own content stream,
    /// optional /Matrix transform, and optional /Resources dictionary.
    fn render_form_xobject(
        &mut self,
        pixmap: &mut Pixmap,
        dict: &std::collections::HashMap<String, Object>,
        data: &[u8],
        parent_transform: Transform,
        doc: &PdfDocument,
        page_num: usize,
        parent_resources: &Object,
    ) -> Result<()> {
        // Parse /Matrix from form dict (default: identity)
        let form_matrix = if let Some(Object::Array(arr)) = dict.get("Matrix") {
            let get_f32 = |i: usize| -> f32 {
                match arr.get(i) {
                    Some(Object::Real(v)) => *v as f32,
                    Some(Object::Integer(v)) => *v as f32,
                    _ => {
                        if i == 0 || i == 3 {
                            1.0
                        } else {
                            0.0
                        }
                    },
                }
            };
            Transform::from_row(
                get_f32(0),
                get_f32(1),
                get_f32(2),
                get_f32(3),
                get_f32(4),
                get_f32(5),
            )
        } else {
            Transform::identity()
        };

        // Combine parent transform with form matrix
        let combined_transform = parent_transform.pre_concat(form_matrix);

        // Check for transparency group (PDF spec section 11.6.6)
        let is_transparency_group = dict
            .get("Group")
            .and_then(|g| g.as_dict())
            .map(|gd| gd.get("S").and_then(|s| s.as_name()) == Some("Transparency"))
            .unwrap_or(false);

        // Get form's /Resources (or fall back to parent resources)
        let form_resources = if let Some(res) = dict.get("Resources") {
            doc.resolve_object(res)?
        } else {
            parent_resources.clone()
        };

        // Parse form content stream
        let operators = match parse_content_stream(data) {
            Ok(ops) => ops,
            Err(e) => {
                return Err(e);
            },
        };

        if is_transparency_group {
            // Per PDF spec 11.6.6: Render transparency group to a separate pixmap,
            // then composite onto the parent. For isolated groups (I=true), the
            // initial backdrop is fully transparent.
            let is_isolated = dict
                .get("Group")
                .and_then(|g| g.as_dict())
                .and_then(|gd| gd.get("I"))
                .map(|i| match i {
                    Object::Boolean(b) => *b,
                    _ => false,
                })
                .unwrap_or(false);

            log::debug!("Rendering transparency group (isolated={})", is_isolated);

            // Create a separate pixmap for the group
            let mut group_pixmap =
                Pixmap::new(pixmap.width(), pixmap.height()).ok_or_else(|| {
                    crate::error::Error::InvalidPdf("Failed to create group pixmap".into())
                })?;

            if !is_isolated {
                // Non-isolated: copy parent content as initial backdrop
                group_pixmap.data_mut().copy_from_slice(pixmap.data());
            }
            // Isolated groups start fully transparent (default Pixmap state)

            // Execute operators into the group pixmap
            self.execute_operators(
                &mut group_pixmap,
                combined_transform,
                &operators,
                doc,
                page_num,
                &form_resources,
            )?;

            if is_isolated {
                // Composite the isolated group onto the parent using over blending
                pixmap.draw_pixmap(
                    0,
                    0,
                    group_pixmap.as_ref(),
                    &tiny_skia::PixmapPaint::default(),
                    Transform::identity(),
                    None,
                );
            } else {
                // Non-isolated: the group pixmap IS the result (it started with parent content)
                pixmap.data_mut().copy_from_slice(group_pixmap.data());
            }
        } else {
            // Non-group form XObject: render directly
            self.execute_operators(
                pixmap,
                combined_transform,
                &operators,
                doc,
                page_num,
                &form_resources,
            )?;
        }

        Ok(())
    }

    /// Apply extended graphics state parameters.
    #[allow(dead_code)]
    fn apply_ext_g_state(
        &self,
        gs: &mut GraphicsState,
        dict_name: &str,
        resources: &Object,
        doc: &PdfDocument,
    ) -> Result<()> {
        // Retained as a thin wrapper for any external caller; the operator
        // loop in `execute_operators` uses the cached fast path via
        // `parse_ext_g_state` instead.
        let parsed = parse_ext_g_state(dict_name, resources, doc).unwrap_or_default();
        parsed.apply(gs);
        Ok(())
    }

    /// Render annotations for a page.
    fn render_annotations(
        &mut self,
        pixmap: &mut Pixmap,
        base_transform: Transform,
        doc: &PdfDocument,
        page_num: usize,
    ) -> Result<()> {
        let annotations = doc.get_annotations(page_num)?;
        // Reuse the per-render snapshot so we don't deep-clone the HashSet here.
        let excluded_snapshot: Option<Arc<HashSet<String>>> = self.excluded_layers_snapshot.clone();
        for annot in annotations {
            // Per ISO 32000-1 §12.5.2, an annotation dict may carry an /OC
            // entry referencing the OCG/OCMD the annotation belongs to. Skip
            // the annotation entirely if its layer is excluded.
            if let Some(ref excluded_layers) = excluded_snapshot {
                if let Some(oc_obj) = annot.raw_dict.as_ref().and_then(|d| d.get("OC")) {
                    if crate::optional_content::annotation_is_excluded(oc_obj, doc, excluded_layers)
                    {
                        continue;
                    }
                }
            }
            // Check if annotation has an appearance stream (/AP)
            if let Some(ap_obj) = annot.raw_dict.as_ref().and_then(|d| d.get("AP")) {
                let ap_stream_obj = doc.resolve_object(ap_obj)?;

                // Normal appearance (N)
                if let Object::Dictionary(ap_dict) = ap_stream_obj {
                    if let Some(n_entry) = ap_dict.get("N").or_else(|| ap_dict.values().next()) {
                        let n_stream_obj = doc.resolve_object(n_entry)?;
                        if let Object::Stream { ref dict, .. } = n_stream_obj {
                            let ap_data = if let Some(r) = n_entry.as_reference() {
                                doc.decode_stream_with_encryption(&n_stream_obj, r)?
                            } else {
                                n_stream_obj.decode_stream_data()?
                            };

                            if let Some(rect) = annot.rect {
                                let x = rect[0] as f32;
                                let y = rect[1] as f32;
                                let annot_transform = base_transform.pre_translate(x, y);

                                let old_fonts = self.fonts.clone();
                                let old_cs = self.color_spaces.clone();
                                if let Some(res) = dict.get("Resources") {
                                    if let Ok(res_obj) = doc.resolve_object(res) {
                                        self.load_resources(doc, &res_obj)?;
                                    }
                                }

                                self.render_form_xobject(
                                    pixmap,
                                    &dict,
                                    &ap_data,
                                    annot_transform,
                                    doc,
                                    page_num,
                                    &Object::Dictionary(std::collections::HashMap::new()),
                                )?;

                                self.fonts = old_fonts;
                                self.color_spaces = old_cs;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Encode Pixmap to JPEG format.
    fn encode_jpeg(&self, pixmap: &Pixmap) -> Result<Vec<u8>> {
        let width = pixmap.width();
        let height = pixmap.height();
        let data = pixmap.data();

        let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
        for i in 0..(width * height) as usize {
            let r = data[i * 4] as f32;
            let g = data[i * 4 + 1] as f32;
            let b = data[i * 4 + 2] as f32;
            let a = data[i * 4 + 3] as f32 / 255.0;

            if a > 0.0 {
                rgb_data.push((r / a).min(255.0) as u8);
                rgb_data.push((g / a).min(255.0) as u8);
                rgb_data.push((b / a).min(255.0) as u8);
            } else {
                rgb_data.push(0);
                rgb_data.push(0);
                rgb_data.push(0);
            }
        }

        let img = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(width, height, rgb_data)
            .ok_or_else(|| Error::InvalidPdf("Failed to create image buffer".to_string()))?;

        let mut output = std::io::Cursor::new(Vec::new());
        img.write_to(&mut output, image::ImageFormat::Jpeg)
            .map_err(|e| Error::InvalidPdf(format!("JPEG encoding failed: {}", e)))?;

        Ok(output.into_inner())
    }

    /// Resolve the colours a path operator needs through the resolution
    /// pipeline and return a `GraphicsState` clone with the resolved RGBA
    /// spliced into the fields the rasteriser reads. Returns `None` when
    /// the toggle is off or when no side produced an RGBA the composite
    /// backend can consume directly — letting the caller borrow the
    /// original `gs` and keep the off-path allocation-free.
    ///
    /// Path-fill (`f`/`F`/`f*`), path-stroke (`S`), and path
    /// fill-stroke combos (`B`/`b`/`B*`/`b*`) all flow through this;
    /// each variant of [`PipelinePaintKind`] decides which side(s) to
    /// resolve. Both sides resolve independently — the pipeline keys
    /// all of its side-specific behaviour off `intent.side`, so a Type 4
    /// Separation on the fill side and a plain DeviceRGB on the stroke
    /// side route correctly without contaminating each other.
    ///
    /// Text operators use the sibling
    /// [`Self::pipeline_resolve_text_colors`] — the text rasteriser
    /// already clones `gs` to advance `text_matrix`, so handing it
    /// colour overrides rather than a pre-cloned `GraphicsState` keeps
    /// the toggle-on text path to one clone per operator instead of
    /// two.
    pub(crate) fn pipeline_resolve_paint_gs(
        &self,
        doc: &PdfDocument,
        gs: &GraphicsState,
        kind: PipelinePaintKind,
    ) -> Option<GraphicsState> {
        if !self.pipeline_enabled_cache {
            return None;
        }
        let (fills, strokes) = match kind {
            // ImageMask paints the stencil with the current fill colour
            // and never reads the stroke side; at this helper layer it
            // is semantically equivalent to PathFill. The variant is
            // kept distinct so the wave-5 separation-backend split can
            // dispatch on it without churning callers.
            PipelinePaintKind::PathFill | PipelinePaintKind::ImageMask => (true, false),
            PipelinePaintKind::PathStroke => (false, true),
            PipelinePaintKind::PathFillStroke => (true, true),
        };
        // Resolve, then short-circuit when the resolved RGBA already
        // equals the GS field that would supply it inline. For
        // Device-family inputs the resolver always returns Some but
        // the answer is the same colour the inline path would read,
        // so a clone here is wasted work. Skipping it keeps the
        // toggle-on Device case allocation-free — the common path
        // most PDFs take.
        let fill_rgba = if fills {
            self.pipeline_resolve_rgba(doc, gs, PaintSide::Fill)
                .filter(|c| !rgba_matches(*c, gs.fill_color_rgb, gs.fill_alpha))
        } else {
            None
        };
        let stroke_rgba = if strokes {
            self.pipeline_resolve_rgba(doc, gs, PaintSide::Stroke)
                .filter(|c| !rgba_matches(*c, gs.stroke_color_rgb, gs.stroke_alpha))
        } else {
            None
        };
        if fill_rgba.is_none() && stroke_rgba.is_none() {
            return None;
        }
        let mut spliced = gs.clone();
        if let Some((r, g, b, a)) = fill_rgba {
            spliced.fill_color_rgb = (r, g, b);
            spliced.fill_alpha = a;
        }
        if let Some((r, g, b, a)) = stroke_rgba {
            spliced.stroke_color_rgb = (r, g, b);
            spliced.stroke_alpha = a;
        }
        Some(spliced)
    }

    /// Resolve the text-painting colours through the resolution
    /// pipeline and return them as side-tagged RGBA tuples for the text
    /// rasteriser to splice into its own `current_gs` clone. Returns
    /// `None` when the toggle is off, when the active `Tr` mode does
    /// not require any resolved side, or when neither side produced an
    /// RGBA the composite backend can consume directly — letting the
    /// caller hand the rasteriser the unmodified `gs` reference.
    ///
    /// Mirrors the side-selection logic of
    /// [`Self::pipeline_resolve_paint_gs`] but returns colours rather
    /// than a `GraphicsState` clone: the text rasteriser already clones
    /// `gs` to walk `text_matrix` per glyph (or per `TJ` element), so
    /// it splices the overrides into that clone — eliminating the
    /// operator-arm-side clone we would otherwise pay on every `Tj` /
    /// `TJ` / `'` / `"`.
    ///
    /// `Tr`-mode handling (ISO 32000-1 §9.3.6 Table 106):
    /// * `0`, `2`, `4`, `6` fill the glyph → resolve fill side.
    /// * `1`, `2`, `5`, `6` stroke the glyph → resolve stroke side.
    /// * `3` is invisible (no painting); skip resolution entirely so
    ///   PDFs that emit text-as-OCR-overlay don't pay any pipeline
    ///   cost.
    pub(crate) fn pipeline_resolve_text_colors(
        &self,
        doc: &PdfDocument,
        gs: &GraphicsState,
    ) -> Option<ResolvedColors> {
        if !self.pipeline_enabled_cache {
            return None;
        }
        if gs.render_mode == 3 {
            return None;
        }
        // Same short-circuit as the path helper: a resolved RGBA that
        // matches the GS field the rasteriser would read inline is a
        // no-op override. Filtering it out lets the operator arm pass
        // `None` straight through and skip the per-element
        // `paint.set_color` write inside `render_text`.
        let fill = if matches!(gs.render_mode, 0 | 2 | 4 | 6) {
            self.pipeline_resolve_rgba(doc, gs, PaintSide::Fill)
                .filter(|c| !rgba_matches(*c, gs.fill_color_rgb, gs.fill_alpha))
        } else {
            None
        };
        let stroke = if matches!(gs.render_mode, 1 | 2 | 5 | 6) {
            self.pipeline_resolve_rgba(doc, gs, PaintSide::Stroke)
                .filter(|c| !rgba_matches(*c, gs.stroke_color_rgb, gs.stroke_alpha))
        } else {
            None
        };
        let colors = ResolvedColors { fill, stroke };
        if colors.is_empty() {
            None
        } else {
            Some(colors)
        }
    }

    /// Resolve the active colour for `side` through the resolution pipeline
    /// when it is enabled. Returns `None` when the toggle is off (caller
    /// keeps its inline behaviour) or when the resolver produces a non-RGBA
    /// variant the composite backend cannot consume directly.
    ///
    /// When `PDF_OXIDE_RESOLUTION_PIPELINE` is unset or equal to "0", the
    /// caller's existing path (read `gs.fill_color_rgb` /
    /// `gs.stroke_color_rgb` directly) is the answer and this returns `None`.
    /// When the toggle is on we route the current colour through
    /// [`ResolutionPipeline`], which handles `Separation`/`DeviceN` colour
    /// spaces backed by PostScript Type 4 tint transforms — the case the
    /// inline match arms a few hundred lines up evaluate as `1.0 - tint`.
    ///
    /// Fill and stroke share one helper because the only differences are
    /// which `gs` fields supply the colour and which `PaintSide` the
    /// pipeline routes against. Splitting these into one helper per side
    /// would mean duplicating the entire context build for no behavioural
    /// reason; the pipeline's colour stage already keys all of its
    /// side-specific behaviour (e.g. alpha fold) off `intent.side`.
    fn pipeline_resolve_rgba(
        &self,
        doc: &PdfDocument,
        gs: &GraphicsState,
        side: PaintSide,
    ) -> Option<(f32, f32, f32, f32)> {
        if !self.pipeline_enabled_cache {
            return None;
        }
        let pipeline = ResolutionPipeline::new();
        let color_spaces = &self.color_spaces;
        let ctx = ResolutionContext::new(doc, color_spaces);
        let (space_name, components) = match side {
            PaintSide::Fill => (gs.fill_color_space.as_str(), &gs.fill_color_components),
            PaintSide::Stroke => (gs.stroke_color_space.as_str(), &gs.stroke_color_components),
        };
        let resolved_space_obj = color_spaces.get(space_name);
        let logical = build_logical_color(space_name, components, resolved_space_obj);

        // No geometry is needed here: the dispatcher only consumes the
        // resolved RGBA to splice into the graphics state, then paints
        // through its own (non-pipeline) rasteriser. `ColorOnly` lets
        // the intent express that without conjuring a placeholder path.
        let intent = PaintIntent {
            kind: PaintKind::ColorOnly,
            side,
            gs,
            color: logical,
            ctm: gs.ctm,
        };
        let cmd = pipeline.resolve(&intent, &ctx, None).ok()?;
        match cmd.color {
            ResolvedColor::Rgba { r, g, b, a } => Some((r, g, b, a)),
            // Non-RGBA variants are produced when the pipeline grows
            // CMYK/PerChannel outputs (for separation backends). The
            // composite backend takes the no-op fallback here, which keeps
            // the door open for migrating without breaking parity.
            _ => None,
        }
    }

    /// `gs`-free overload of the colour-resolution path: route an
    /// explicit colour-space + components tuple through the pipeline and
    /// return the resolved RGBA.
    ///
    /// The path/text/image-mask helpers above read their colour inputs
    /// from `gs.fill_color_space` / `gs.fill_color_components` (or the
    /// stroke equivalents). Shading endpoint colours don't live there —
    /// they sit in the shading dictionary's `/Function /C0` and `/C1`
    /// arrays, alongside the shading dictionary's own `/ColorSpace`. The
    /// dispatcher needs to resolve those two endpoints independently
    /// of `gs` so the gradient backend can hand them to the
    /// interpolator as fixed stops. This helper is that hook: caller
    /// supplies the shading's `/ColorSpace` object directly and the
    /// per-endpoint component list; the helper builds the logical
    /// colour, runs it through the pipeline against a synthesised
    /// graphics state carrying only the requested alpha (every other
    /// `gs` field — blend mode, overprint — is irrelevant for endpoint
    /// resolution because the gradient is composited as a single Source
    /// Over fill by the caller), and returns the RGBA.
    ///
    /// Returns `None` when the toggle is off or when the resolver
    /// produces a non-RGBA variant (per-channel outputs reserved for
    /// future separation backends). The caller is then expected to fall
    /// back to its inline behaviour.
    pub(crate) fn pipeline_resolve_components(
        &self,
        doc: &PdfDocument,
        color_spaces: &HashMap<String, Object>,
        space: &Object,
        components: &[f32],
        alpha: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        if !self.pipeline_enabled_cache {
            return None;
        }
        // Derive a name + resolved-space pair from the supplied space
        // object. Two shapes appear in real PDFs for a shading dict's
        // `/ColorSpace`: a Name (either a Device alias like
        // `/DeviceRGB` or a per-page resource name like `/CS1`), or an
        // inline Array (e.g. `[/Separation /MagentaSpot /DeviceCMYK
        // funcRef]`). `build_logical_color` already handles both via
        // its name + `Option<&Object>` arguments, so this wrapper just
        // dispatches into it.
        let (space_name, resolved_space): (&str, Option<&Object>) = match space {
            Object::Name(n) => {
                // Device aliases short-circuit through the name match
                // inside `build_logical_color`; non-device names look
                // up the resolved space in the per-page colour-space
                // map (the dispatcher pre-resolved the name there).
                let resolved = color_spaces.get(n.as_str());
                (n.as_str(), resolved)
            },
            other => {
                // Inline array (or any other shape — Stream, etc.):
                // hand the resolved space object through directly and
                // use a sentinel name so the Device-family fast-path in
                // `build_logical_color` doesn't fire. The actual
                // colour-space dispatch keys off the resolved space
                // object, not the name, for non-Device families.
                ("", Some(other))
            },
        };
        let logical = build_logical_color(space_name, components, resolved_space);

        // The pipeline reads `gs.fill_alpha` for fill-side alpha fold.
        // A default `GraphicsState` carries `fill_alpha = 1.0`; we patch
        // it to the caller's `alpha` so the resolver folds the correct
        // alpha into the returned RGBA. Every other field is left at
        // its default — overprint and blend stages run on this fake gs
        // too, but their outputs (`OverprintPlan`, `BlendPlan`) are
        // discarded: the caller only consumes the RGBA. The default
        // `GraphicsState` has overprint disabled and blend Normal, so
        // even though the resolved-paint-cmd carries these plans they
        // never surface as observable behaviour.
        let mut synth_gs = GraphicsState::new();
        synth_gs.fill_alpha = alpha;

        let pipeline = ResolutionPipeline::new();
        let ctx = ResolutionContext::new(doc, color_spaces);
        let intent = PaintIntent {
            kind: PaintKind::ColorOnly,
            side: PaintSide::Fill,
            gs: &synth_gs,
            color: logical,
            ctm: synth_gs.ctm,
        };
        let cmd = pipeline.resolve(&intent, &ctx, None).ok()?;
        match cmd.color {
            ResolvedColor::Rgba { r, g, b, a } => Some((r, g, b, a)),
            // Non-RGBA outputs (per-channel for separation backends)
            // can't be fed to the composite gradient backend; the
            // caller's inline behaviour stays in force for those.
            _ => None,
        }
    }
}

/// Read `PDF_OXIDE_RESOLUTION_PIPELINE` and parse it as a boolean toggle.
///
/// Called once per `render_page_with_options` invocation and the result
/// is cached on `PageRenderer::pipeline_enabled_cache`; the dispatcher
/// reads the cached bool. Tests pick up env-var flips on the next
/// `render_page` call.
fn read_pipeline_env() -> bool {
    match std::env::var("PDF_OXIDE_RESOLUTION_PIPELINE") {
        Ok(v) => !v.is_empty() && v != "0" && !v.eq_ignore_ascii_case("false"),
        Err(_) => false,
    }
}

/// Per-channel `f32` comparison tolerance used by [`rgba_matches`]. The
/// resolver folds Device-family inputs through the same RGB encoding the
/// inline path uses, so an exact match is the expected case; the
/// epsilon is sized to absorb single-ulp drift from intermediate
/// computations (alpha fold, CMYK → RGB) without admitting an actual
/// colour change. Anything coarser would risk dropping subtle overrides
/// the renderer needs to honour.
const RGBA_MATCH_EPSILON: f32 = 1.0e-6;

/// Returns `true` when the resolved `(r, g, b, a)` matches the supplied
/// rgb triple and alpha within [`RGBA_MATCH_EPSILON`] on every channel.
///
/// Used by the resolution-pipeline helpers to detect no-op overrides:
/// for Device-family inputs the pipeline always produces an RGBA, but
/// the value is the same one the inline path would have read from
/// `gs.*_color_rgb` directly. Skipping the splice in that case keeps
/// the toggle-on path allocation-free for the common case where no
/// Separation/DeviceN colour space is in play.
fn rgba_matches(resolved: (f32, f32, f32, f32), rgb: (f32, f32, f32), alpha: f32) -> bool {
    let (r, g, b, a) = resolved;
    let (gr, gg, gb) = rgb;
    (r - gr).abs() <= RGBA_MATCH_EPSILON
        && (g - gg).abs() <= RGBA_MATCH_EPSILON
        && (b - gb).abs() <= RGBA_MATCH_EPSILON
        && (a - alpha).abs() <= RGBA_MATCH_EPSILON
}

/// Build a [`LogicalColor`] from the dispatcher's view of the active colour:
/// the fill colour space name, the raw components on the stack, and (when the
/// space is non-Device) the resolved space object from the resources map.
fn build_logical_color<'a>(
    space_name: &str,
    components: &[f32],
    resolved_space: Option<&'a Object>,
) -> LogicalColor<'a> {
    // Device families fold directly into `LogicalColor::Device` — the
    // resolver's spec-conformance for these is verified by colour-stage
    // unit tests; routing through the same Device path keeps the pilot's
    // behaviour identical to the inline path for the non-Separation cases.
    //
    // Component-count mismatch (e.g. `/ColorSpace /DeviceCMYK` with only
    // 1 component on the stack) falls through to the `_ =>` arm below,
    // which routes through the resolver's gray fallback. Output happens
    // to match the inline `parse_color_array` single-element-array
    // expansion `(g, g, g)` — both paths paint the gray value across
    // all three RGB channels.
    match space_name {
        "DeviceGray" | "G" if !components.is_empty() => {
            LogicalColor::Device(DeviceColor::Gray(components[0]))
        },
        "DeviceRGB" | "RGB" if components.len() >= 3 => {
            LogicalColor::Device(DeviceColor::Rgb(components[0], components[1], components[2]))
        },
        "DeviceCMYK" | "CMYK" if components.len() >= 4 => LogicalColor::Device(DeviceColor::Cmyk(
            components[0],
            components[1],
            components[2],
            components[3],
        )),
        _ => {
            // Non-device space: hand the resolver the space object so it
            // can dispatch on Separation / DeviceN / ICCBased / Indexed.
            // Fall back to `DeviceGray` as a logical-colour shape if the
            // resources map didn't carry an entry for this name — the
            // resolver's gray fallback then matches the inline path.
            //
            // Use a thread-local static name object to satisfy the
            // `'a` lifetime on the fallback arm without cloning.
            use std::sync::OnceLock;
            static GRAY_FALLBACK: OnceLock<Object> = OnceLock::new();
            let space = resolved_space.unwrap_or_else(|| {
                GRAY_FALLBACK.get_or_init(|| Object::Name("DeviceGray".to_string()))
            });
            LogicalColor::Spaced {
                space,
                components: components.iter().copied().collect(),
            }
        },
    }
}

/// Resolve the named ExtGState entry from `resources` and parse the fields we
/// need. Kept as a thin wrapper that re-resolves the resource dict per call —
/// the hot path in `execute_operators` uses `parse_ext_g_state_inner` against
/// a pre-resolved resource dict (the per-form ExtGState dict has 10 000+
/// entries on heavy vector figures and deep-cloning it on every `gs` op was
/// the previous bottleneck).
fn parse_ext_g_state(
    dict_name: &str,
    resources: &Object,
    doc: &PdfDocument,
) -> Result<ParsedExtGState> {
    let out = ParsedExtGState::default();
    let res_dict = match resources {
        Object::Dictionary(d) => d,
        _ => return Ok(out),
    };
    let ext_gs_obj = match res_dict.get("ExtGState") {
        Some(o) => o,
        None => return Ok(out),
    };
    let ext_gs_resolved = doc.resolve_object(ext_gs_obj)?;
    let ext_g_states = match ext_gs_resolved.as_dict() {
        Some(d) => d,
        None => return Ok(out),
    };
    let state_obj = match ext_g_states.get(dict_name) {
        Some(o) => o,
        None => return Ok(out),
    };
    parse_ext_g_state_inner(state_obj, doc)
}

/// Resize an RGBA (straight-alpha) byte buffer using SIMD-accelerated bilinear filtering.
///
/// Returns `None` on failure (zero dimensions, SIMD dispatch error) so callers
/// can fall back to tiny_skia's own resampling path.
fn resize_rgba(src: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Option<Vec<u8>> {
    use fast_image_resize::images::Image;
    use fast_image_resize::pixels::PixelType;
    use fast_image_resize::{FilterType, ResizeAlg, ResizeOptions, Resizer};

    // from_slice_u8 needs a mutable slice; copy into a local buffer.
    let mut buf = src.to_vec();
    let src_img = Image::from_slice_u8(src_w, src_h, &mut buf, PixelType::U8x4).ok()?;
    let mut dst_img = Image::new(dst_w, dst_h, PixelType::U8x4);
    Resizer::new()
        .resize(
            &src_img,
            &mut dst_img,
            &ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Bilinear)),
        )
        .ok()?;
    Some(dst_img.into_vec())
}

/// Encode a tiny_skia `Pixmap` to PNG.
///
/// Uses fdeflate (ultra-fast) compression via the `image` crate instead of
/// tiny_skia's built-in `encode_png`, which defaults to flate2 level 6 and is
/// 3–5× slower on typical page images.
fn encode_png(pixmap: &Pixmap) -> Result<Vec<u8>> {
    let w = pixmap.width();
    let h = pixmap.height();

    // Demultiply: tiny_skia stores premultiplied RGBA; PNG expects straight alpha.
    let src = pixmap.data();
    let mut data = src.to_vec();
    for chunk in data.chunks_exact_mut(4) {
        let a = chunk[3];
        if a != 0 && a != 255 {
            let a32 = a as u32;
            chunk[0] = ((chunk[0] as u32 * 255 + a32 / 2) / a32).min(255) as u8;
            chunk[1] = ((chunk[1] as u32 * 255 + a32 / 2) / a32).min(255) as u8;
            chunk[2] = ((chunk[2] as u32 * 255 + a32 / 2) / a32).min(255) as u8;
        }
    }

    use image::codecs::png::{CompressionType, FilterType, PngEncoder};
    use image::ImageEncoder;
    let mut output = Vec::new();
    PngEncoder::new_with_quality(&mut output, CompressionType::Fast, FilterType::Sub)
        .write_image(&data, w, h, image::ExtendedColorType::Rgba8)
        .map_err(|e| Error::InvalidPdf(format!("PNG encoding failed: {}", e)))?;
    Ok(output)
}

/// Combine two transformations.
fn combine_transforms(base: Transform, ctm: &Matrix) -> Transform {
    base.pre_concat(Transform::from_row(ctm.a, ctm.b, ctm.c, ctm.d, ctm.e, ctm.f))
}

/// Build the image-space → user-space transform for a PDF image blit.
///
/// Per ISO 32000-1 §8.9.5, PDF images live in a unit square in the user
/// coordinate system; image rows are top-to-bottom (opposite of PDF's
/// bottom-to-top y axis). The pre-translate-by-1-in-y + pre-scale-by
/// `1/src_w, -1/src_h` flips the rows AND normalises the source-pixel
/// extent to the unit square, so the caller's `parent` CTM places the
/// image where the PDF demands.
///
/// Shared by `render_image` and `render_image_mask`.
fn image_unit_square_transform(parent: Transform, src_w: u32, src_h: u32) -> Transform {
    parent
        .pre_translate(0.0, 1.0)
        .pre_scale(1.0 / src_w as f32, -1.0 / src_h as f32)
}

/// Build the `PixmapPaint` used to blit an already-flipped image into
/// the page pixmap.
///
/// `image_transform` must already be the output of
/// [`image_unit_square_transform`] (or the SIMD fast path's
/// translate-only equivalent); the helper reads its scale to pick
/// Bicubic when the blit is an upscale or 1:1 and Bilinear when it is a
/// downscale — the same heuristic both `render_image` and
/// `render_image_mask` used independently before this consolidation.
/// `opacity` is the source's alpha (the std-image path passes
/// `gs.fill_alpha`; the ImageMask path bakes alpha into the stencil
/// pixels and passes `1.0`). `blend_mode_pdf` is the PDF blend-mode
/// name from `gs.blend_mode`.
///
/// Shared by `render_image` and `render_image_mask`.
fn pixmap_paint_for_image_blit(
    image_transform: Transform,
    opacity: f32,
    blend_mode_pdf: &str,
) -> PixmapPaint {
    let mut paint = PixmapPaint::default();
    paint.opacity = opacity;
    paint.blend_mode = crate::rendering::pdf_blend_mode_to_skia(blend_mode_pdf);
    let (xs, ys) = image_transform.get_scale();
    paint.quality = if xs >= 1.0 || ys >= 1.0 {
        tiny_skia::FilterQuality::Bicubic
    } else {
        tiny_skia::FilterQuality::Bilinear
    };
    paint
}

/// Convert DeviceCMYK (0.0–1.0) to DeviceRGB (0.0–1.0) per ISO 32000-1:2008
/// §10.3.5. The additive-clamp formula `R = 1 − min(1, C+K)` is the
/// spec-mandated fallback when no ICC profile is available.
fn cmyk_to_rgb(c: f32, m: f32, y: f32, k: f32) -> (f32, f32, f32) {
    let r = 1.0 - (c + k).min(1.0);
    let g = 1.0 - (m + k).min(1.0);
    let b = 1.0 - (y + k).min(1.0);
    (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
}

fn apply_pending_clip(
    pending_clip: &mut Option<(tiny_skia::Path, tiny_skia::FillRule)>,
    clip_stack: &mut Vec<Option<tiny_skia::Mask>>,
    pixmap: &Pixmap,
    base_transform: Transform,
    gs_stack: &GraphicsStateStack,
) {
    if let Some((path, fill_rule)) = pending_clip.take() {
        let gs = gs_stack.current();
        let transform = combine_transforms(base_transform, &gs.ctm);

        if let Some(path_transformed) = path.transform(transform) {
            let bounds = path_transformed.bounds();
            log::debug!("Applying clip: fill_rule={:?}, bounds={:?}", fill_rule, bounds);

            let mut new_mask = tiny_skia::Mask::new(pixmap.width(), pixmap.height()).unwrap();
            new_mask.fill_path(
                &path_transformed,
                fill_rule,
                true, // anti-alias
                Transform::identity(),
            );

            if let Some(Some(current_mask)) = clip_stack.last() {
                // Intersect with existing mask
                let mut combined = current_mask.clone();
                let combined_data = combined.data_mut();
                let new_data = new_mask.data();
                for i in 0..combined_data.len() {
                    combined_data[i] = ((combined_data[i] as u32 * new_data[i] as u32) / 255) as u8;
                }
                *clip_stack.last_mut().unwrap() = Some(combined);
            } else {
                *clip_stack.last_mut().unwrap() = Some(new_mask);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::Object;

    #[test]
    fn test_cmyk_to_rgb_white() {
        let (r, g, b) = cmyk_to_rgb(0.0, 0.0, 0.0, 0.0);
        assert!((r - 1.0).abs() < 0.001);
        assert!((g - 1.0).abs() < 0.001);
        assert!((b - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cmyk_to_rgb_black() {
        let (r, g, b) = cmyk_to_rgb(0.0, 0.0, 0.0, 1.0);
        assert!((r - 0.0).abs() < 0.001);
        assert!((g - 0.0).abs() < 0.001);
        assert!((b - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_cmyk_to_rgb_pure_cyan() {
        let (r, g, b) = cmyk_to_rgb(1.0, 0.0, 0.0, 0.0);
        assert!((r - 0.0).abs() < 0.001);
        assert!((g - 1.0).abs() < 0.001);
        assert!((b - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_color_array_rgb() {
        let arr = vec![Object::Real(0.5), Object::Real(0.25), Object::Real(0.75)];
        let (r, g, b) = PageRenderer::parse_color_array(&arr);
        assert!((r - 0.5).abs() < 0.001);
        assert!((g - 0.25).abs() < 0.001);
        assert!((b - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_parse_color_array_grayscale() {
        let arr = vec![Object::Real(0.5)];
        let (r, g, b) = PageRenderer::parse_color_array(&arr);
        assert!((r - 0.5).abs() < 0.001);
        assert_eq!(r, g);
        assert_eq!(g, b);
    }

    #[test]
    fn test_parse_color_array_integers() {
        let arr = vec![Object::Integer(1), Object::Integer(0), Object::Integer(0)];
        let (r, g, b) = PageRenderer::parse_color_array(&arr);
        assert!((r - 1.0).abs() < 0.001);
        assert!((g - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_negative_rect_normalization() {
        // Negative height: re 100 200 50 -30 → should normalize to (100, 170, 50, 30)
        let x: f32 = 100.0;
        let y: f32 = 200.0;
        let w: f32 = 50.0;
        let h: f32 = -30.0;
        let (nx, nw) = if w < 0.0 { (x + w, -w) } else { (x, w) };
        let (ny, nh) = if h < 0.0 { (y + h, -h) } else { (y, h) };
        assert!((nx - 100.0).abs() < 0.001);
        assert!((ny - 170.0).abs() < 0.001);
        assert!((nw - 50.0).abs() < 0.001);
        assert!((nh - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_negative_rect_both_negative() {
        let x: f32 = 100.0;
        let y: f32 = 200.0;
        let w: f32 = -50.0;
        let h: f32 = -30.0;
        let (nx, nw) = if w < 0.0 { (x + w, -w) } else { (x, w) };
        let (ny, nh) = if h < 0.0 { (y + h, -h) } else { (y, h) };
        assert!((nx - 50.0).abs() < 0.001);
        assert!((ny - 170.0).abs() < 0.001);
        assert!((nw - 50.0).abs() < 0.001);
        assert!((nh - 30.0).abs() < 0.001);
    }

    // ---------------------------------------------------------------------
    // Helper-level pins for the text-resolution splice.
    //
    // The text-side integration tests in
    // `tests/test_render_resolution_pipeline_pilot.rs` exercise the full
    // renderer end-to-end, but two properties are not directly observable
    // from there today:
    //
    //   * Stroke-side resolution. The text rasteriser does not currently
    //     paint stroked glyphs, so the spliced stroke colour never reaches
    //     the pixmap. We probe it here by inspecting the
    //     `GraphicsState` the helper returns.
    //
    //   * Helper-returns-`None` on the off-toggle path. The integration
    //     test asserts byte-equal output between off and on for a no-op
    //     splice, which holds whether the helper returns `None` or
    //     `Some(clone)`. We probe the return value directly here.
    //
    // Both probes call `pipeline_resolve_text_colors` directly. The
    // wider integration coverage stays untouched.
    // ---------------------------------------------------------------------

    use crate::content::graphics_state::GraphicsState;
    use crate::rendering::resolution::test_support::fixture_doc;
    use smallvec::smallvec;
    use std::collections::HashMap;

    fn type4_magenta_separation_space() -> Object {
        // `{ 0.0 exch 0.0 0.0 }` — at full tint this yields CMYK(0,1,0,0),
        // which the colour resolver converts to RGB ≈ (1, 0, 1) (magenta).
        // Same shape as the colour-stage and pipeline regression tests.
        let program = b"{ 0.0 exch 0.0 0.0 }";
        let mut func_dict: HashMap<String, Object> = HashMap::new();
        func_dict.insert("FunctionType".into(), Object::Integer(4));
        func_dict
            .insert("Domain".into(), Object::Array(vec![Object::Integer(0), Object::Integer(1)]));
        func_dict.insert(
            "Range".into(),
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(1),
                Object::Integer(0),
                Object::Integer(1),
                Object::Integer(0),
                Object::Integer(1),
                Object::Integer(0),
                Object::Integer(1),
            ]),
        );
        let func_obj = Object::Stream {
            dict: func_dict,
            data: program.to_vec().into(),
        };
        Object::Array(vec![
            Object::Name("Separation".into()),
            Object::Name("MagentaSpot".into()),
            Object::Name("DeviceCMYK".into()),
            func_obj,
        ])
    }

    #[test]
    fn pipeline_resolve_text_colors_strokes_magenta_under_tr1() {
        // T-1 stroke-side resolution probe.
        //
        // Construct a `PageRenderer` with the toggle on and a
        // Separation/DeviceCMYK/Type-4 colour space attached to the
        // stroke side. Under Tr=1 the helper must resolve the stroke
        // side through the pipeline and yield the Type-4-evaluated RGB
        // on the `stroke` channel of the returned `ResolvedColors`. The
        // legacy `1.0 - tint = 0` fallback would put black on the stroke
        // channel; the pipeline must produce magenta (R high, G low, B
        // high).
        let mut renderer = PageRenderer::new(RenderOptions::default());
        renderer.pipeline_enabled_cache = true;
        renderer
            .color_spaces
            .insert("SpotMagenta".to_string(), type4_magenta_separation_space());

        let mut gs = GraphicsState::new();
        gs.render_mode = 1; // Stroke-only text.
        gs.stroke_color_space = "SpotMagenta".to_string();
        gs.stroke_color_components = smallvec![1.0]; // full tint
                                                     // Leave fill side at the GraphicsState default (DeviceGray, no
                                                     // components) so a stray fill-side resolve attempt would fail
                                                     // out — keeping the assertion focused on the stroke channel.

        let doc = fixture_doc();
        let colors = renderer
            .pipeline_resolve_text_colors(&doc, &gs)
            .expect("Tr=1 stroke side must produce ResolvedColors");

        let (r, g, b, _a) = colors.stroke.expect("Tr=1 must populate the stroke side");
        assert!(
            r > 0.78 && g < 0.24 && b > 0.78,
            "stroke side must be magenta (Type-4 evaluated), \
             not the legacy 1-tint=0 black; got ({r}, {g}, {b})"
        );
        // The fill channel must not have been resolved — the helper
        // selects only the side(s) the Tr mode names.
        assert!(colors.fill.is_none(), "Tr=1 must not touch the fill side");
    }

    #[test]
    fn pipeline_resolve_paint_gs_short_circuits_when_resolved_matches_gs() {
        // D-3 short-circuit. With the toggle on and a DeviceRGB fill
        // already set on `gs`, the pipeline resolves to the same
        // (r, g, b, alpha) as `gs.fill_color_rgb` / `gs.fill_alpha`.
        // The helper must skip the GraphicsState clone in that case
        // and return `None` — the caller borrows `gs` directly. This
        // keeps the toggle-on Device-family path (the common case)
        // allocation-free.
        let mut renderer = PageRenderer::new(RenderOptions::default());
        renderer.pipeline_enabled_cache = true;

        let mut gs = GraphicsState::new();
        gs.fill_color_space = "DeviceRGB".to_string();
        gs.fill_color_components = smallvec![0.25, 0.5, 0.75];
        // The dispatcher's inline path keeps `gs.fill_color_rgb` in
        // sync with the components; mirror that here so the
        // short-circuit comparison sees a true no-op.
        gs.fill_color_rgb = (0.25, 0.5, 0.75);
        gs.fill_alpha = 1.0;

        let doc = fixture_doc();
        assert!(
            renderer
                .pipeline_resolve_paint_gs(&doc, &gs, PipelinePaintKind::PathFill)
                .is_none(),
            "Device-family fill that resolves to the same RGBA as gs must short-circuit"
        );
    }

    #[test]
    fn pipeline_resolve_paint_gs_image_mask_short_circuits_same_as_path_fill() {
        // Wave 3 pin. `PipelinePaintKind::ImageMask` must follow the
        // same fill-only resolve-and-short-circuit rules as
        // `PipelinePaintKind::PathFill`: a Device-family fill whose
        // resolved RGBA already matches `gs.fill_color_rgb` returns
        // `None` (no clone), and the stroke side is never touched.
        let mut renderer = PageRenderer::new(RenderOptions::default());
        renderer.pipeline_enabled_cache = true;

        let mut gs = GraphicsState::new();
        gs.fill_color_space = "DeviceRGB".to_string();
        gs.fill_color_components = smallvec![0.25, 0.5, 0.75];
        gs.fill_color_rgb = (0.25, 0.5, 0.75);
        gs.fill_alpha = 1.0;

        let doc = fixture_doc();
        assert!(
            renderer
                .pipeline_resolve_paint_gs(&doc, &gs, PipelinePaintKind::ImageMask)
                .is_none(),
            "ImageMask Device-family fill matching gs must short-circuit"
        );
    }

    #[test]
    fn pipeline_resolve_paint_gs_image_mask_resolves_type4_separation_fill() {
        // Wave 3 capability pin. With the toggle on and a
        // Separation/DeviceCMYK Type 4 colour space on the fill side,
        // the `ImageMask` variant must produce a spliced
        // `GraphicsState` whose `fill_color_rgb` is the Type 4 program
        // output (magenta), NOT the legacy `1 - tint = 0` black. This
        // is the wave-1-class bug regression check for the ImageMask
        // routing — same helper, same colour-stage path, just driven
        // by the new variant.
        let mut renderer = PageRenderer::new(RenderOptions::default());
        renderer.pipeline_enabled_cache = true;
        renderer
            .color_spaces
            .insert("SpotMagenta".to_string(), type4_magenta_separation_space());

        let mut gs = GraphicsState::new();
        gs.fill_color_space = "SpotMagenta".to_string();
        gs.fill_color_components = smallvec![1.0]; // full tint
        gs.fill_color_rgb = (0.0, 0.0, 0.0); // legacy 1-tint=0 black
        gs.fill_alpha = 1.0;

        let doc = fixture_doc();
        let spliced = renderer
            .pipeline_resolve_paint_gs(&doc, &gs, PipelinePaintKind::ImageMask)
            .expect("Type 4 Separation fill must splice through ImageMask variant");

        let (r, g, b) = spliced.fill_color_rgb;
        assert!(
            r > 0.78 && g < 0.24 && b > 0.78,
            "ImageMask fill must be magenta (Type 4 evaluated), not legacy black; got ({r}, {g}, {b})"
        );
        // Stroke side must remain untouched — the variant is fill-only.
        assert_eq!(
            spliced.stroke_color_rgb, gs.stroke_color_rgb,
            "ImageMask variant must not touch the stroke channel"
        );
    }

    #[test]
    fn pipeline_resolve_text_colors_short_circuits_when_resolved_matches_gs() {
        // Same short-circuit on the text-side helper, Tr=0 fill-only:
        // a DeviceRGB whose resolved value equals the current gs fields
        // must produce no override (no per-element paint.set_color in
        // the rasteriser).
        let mut renderer = PageRenderer::new(RenderOptions::default());
        renderer.pipeline_enabled_cache = true;

        let mut gs = GraphicsState::new();
        gs.render_mode = 0;
        gs.fill_color_space = "DeviceRGB".to_string();
        gs.fill_color_components = smallvec![0.1, 0.2, 0.3];
        gs.fill_color_rgb = (0.1, 0.2, 0.3);
        gs.fill_alpha = 1.0;

        let doc = fixture_doc();
        assert!(
            renderer.pipeline_resolve_text_colors(&doc, &gs).is_none(),
            "Device-family text fill that resolves to the same RGBA as gs must short-circuit"
        );
    }

    #[test]
    fn rgba_matches_within_epsilon() {
        // The tolerance must absorb single-ulp drift from intermediate
        // computations but reject any real colour change.
        assert!(rgba_matches((0.25, 0.5, 0.75, 1.0), (0.25, 0.5, 0.75), 1.0));
        // Sub-epsilon drift on every channel still matches.
        let drift = RGBA_MATCH_EPSILON * 0.5;
        assert!(rgba_matches(
            (0.25 + drift, 0.5 + drift, 0.75 + drift, 1.0 + drift),
            (0.25, 0.5, 0.75),
            1.0
        ));
        // Anything beyond the epsilon is a real change and must not
        // short-circuit — single-channel mismatch is enough.
        assert!(!rgba_matches((0.26, 0.5, 0.75, 1.0), (0.25, 0.5, 0.75), 1.0));
        assert!(!rgba_matches((0.25, 0.5, 0.75, 0.5), (0.25, 0.5, 0.75), 1.0));
    }

    #[test]
    fn pipeline_resolve_text_colors_returns_none_when_toggle_off() {
        // T-2 helper-disabled probe. With the toggle off the helper must
        // return `None` regardless of how rich the inputs are — including
        // a Tr=2 fill+stroke configuration that would otherwise drive
        // both sides through the pipeline.
        let renderer = PageRenderer::new(RenderOptions::default());
        // `PageRenderer::new` leaves `pipeline_enabled_cache = false`; we
        // re-assert it here as a guard against future drift in the
        // constructor.
        assert!(
            !renderer.pipeline_enabled_cache,
            "default-constructed renderer must have the pipeline toggle off"
        );

        let mut gs = GraphicsState::new();
        gs.render_mode = 2;
        gs.fill_color_space = "DeviceRGB".to_string();
        gs.fill_color_components = smallvec![1.0, 0.0, 0.0];
        gs.stroke_color_space = "DeviceRGB".to_string();
        gs.stroke_color_components = smallvec![0.0, 0.0, 1.0];

        let doc = fixture_doc();
        let result = renderer.pipeline_resolve_text_colors(&doc, &gs);
        assert!(result.is_none(), "helper must return None when the toggle is off; got Some(_)");
    }

    // ---------------------------------------------------------------------
    // Wave 4 — `pipeline_resolve_components` helper unit pins.
    //
    // The shading integration tests in
    // `tests/test_render_resolution_pipeline_pilot.rs` probe the helper
    // through the renderer. These unit pins probe the helper's own
    // contract directly, so a regression in routing (e.g. Device-family
    // short-circuit vs Spaced dispatch) shows up at the helper level
    // before any pixel-comparison machinery is involved.
    // ---------------------------------------------------------------------

    #[test]
    fn pipeline_resolve_components_resolves_type4_separation_to_correct_rgba() {
        // Capability pin. The Separation/DeviceCMYK/Type-4 space at
        // full tint must come out as magenta after the pipeline runs
        // the PostScript program — the same regression case the
        // colour-stage and full-pipeline unit tests pin at lower
        // levels, here verified via the wave-4 shading-endpoint
        // overload.
        let mut renderer = PageRenderer::new(RenderOptions::default());
        renderer.pipeline_enabled_cache = true;

        let space = type4_magenta_separation_space();
        let doc = fixture_doc();
        let color_spaces: HashMap<String, Object> = HashMap::new();

        let rgba = renderer
            .pipeline_resolve_components(&doc, &color_spaces, &space, &[1.0], 1.0)
            .expect("Type 4 Separation full-tint must resolve to Some(rgba)");
        let (r, g, b, a) = rgba;
        assert!(
            (r - 1.0).abs() < 1.0e-3
                && g.abs() < 1.0e-3
                && (b - 1.0).abs() < 1.0e-3
                && (a - 1.0).abs() < 1.0e-3,
            "Type 4 Separation at tint=1 must produce magenta RGBA (≈1, 0, 1, 1); got ({r}, {g}, {b}, {a})"
        );
    }

    #[test]
    fn pipeline_resolve_components_short_circuits_for_device_families() {
        // Parity pin. For DeviceRGB / DeviceGray / DeviceCMYK the
        // pipeline must produce the same RGBA the inline shading
        // path would compute (modulo the inline path's
        // long-standing DeviceCMYK truncation bug, which is the
        // entire reason wave 4 exists). The pin here is on the
        // resolver's behaviour, not on the inline path: for each
        // device family the resolved RGBA must equal the
        // mathematically-correct device→RGB conversion.
        let mut renderer = PageRenderer::new(RenderOptions::default());
        renderer.pipeline_enabled_cache = true;
        let doc = fixture_doc();
        let color_spaces: HashMap<String, Object> = HashMap::new();

        // DeviceRGB: components pass through verbatim.
        let rgb_space = Object::Name("DeviceRGB".to_string());
        let rgba = renderer
            .pipeline_resolve_components(&doc, &color_spaces, &rgb_space, &[0.5, 0.25, 0.75], 0.8)
            .expect("DeviceRGB must resolve");
        let (r, g, b, a) = rgba;
        assert!(
            (r - 0.5).abs() < 1.0e-6
                && (g - 0.25).abs() < 1.0e-6
                && (b - 0.75).abs() < 1.0e-6
                && (a - 0.8).abs() < 1.0e-6,
            "DeviceRGB must pass components through verbatim with alpha folded; got ({r}, {g}, {b}, {a})"
        );

        // DeviceGray: single component expanded to (g, g, g).
        let gray_space = Object::Name("DeviceGray".to_string());
        let rgba = renderer
            .pipeline_resolve_components(&doc, &color_spaces, &gray_space, &[0.42], 1.0)
            .expect("DeviceGray must resolve");
        let (r, g, b, _a) = rgba;
        assert!(
            (r - 0.42).abs() < 1.0e-6 && (g - 0.42).abs() < 1.0e-6 && (b - 0.42).abs() < 1.0e-6,
            "DeviceGray must expand the single component to (g, g, g); got ({r}, {g}, {b})"
        );

        // DeviceCMYK: additive-clamp conversion `(1-c-k, 1-m-k,
        // 1-y-k)` with clamping to [0, 1]. Pure cyan (1, 0, 0, 0)
        // → RGB(0, 1, 1).
        let cmyk_space = Object::Name("DeviceCMYK".to_string());
        let rgba = renderer
            .pipeline_resolve_components(
                &doc,
                &color_spaces,
                &cmyk_space,
                &[1.0, 0.0, 0.0, 0.0],
                1.0,
            )
            .expect("DeviceCMYK must resolve");
        let (r, g, b, _a) = rgba;
        assert!(
            r.abs() < 1.0e-3 && (g - 1.0).abs() < 1.0e-3 && (b - 1.0).abs() < 1.0e-3,
            "DeviceCMYK pure cyan must map to (0, 1, 1) under additive clamp; got ({r}, {g}, {b})"
        );
    }

    #[test]
    fn pipeline_resolve_components_returns_none_when_toggle_off() {
        // The toggle is the gate the dispatcher relies on. With the
        // cached toggle off, the helper must return None even when
        // the inputs would otherwise resolve cleanly.
        let renderer = PageRenderer::new(RenderOptions::default());
        // Default-constructed renderer has the toggle off; pin that.
        assert!(
            !renderer.pipeline_enabled_cache,
            "default-constructed renderer must have the pipeline toggle off"
        );
        let doc = fixture_doc();
        let color_spaces: HashMap<String, Object> = HashMap::new();
        let space = Object::Name("DeviceRGB".to_string());
        let result = renderer.pipeline_resolve_components(
            &doc,
            &color_spaces,
            &space,
            &[1.0, 0.0, 0.0],
            1.0,
        );
        assert!(
            result.is_none(),
            "helper must return None when toggle is off, even for valid DeviceRGB input"
        );
    }
}
