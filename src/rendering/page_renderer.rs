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
use crate::rendering::ext_gstate::{parse_ext_g_state_inner, ParsedExtGState, SoftMaskSpec};
use std::borrow::Cow;

/// Parse a Form XObject's `/Matrix` entry into a `tiny_skia::Transform`.
/// Defaults to identity when absent or malformed (§8.10.1).
fn parse_form_matrix(dict: &std::collections::HashMap<String, Object>) -> tiny_skia::Transform {
    match dict.get("Matrix") {
        Some(Object::Array(arr)) => {
            let get = |i: usize, default: f32| -> f32 {
                match arr.get(i) {
                    Some(Object::Real(v)) => *v as f32,
                    Some(Object::Integer(v)) => *v as f32,
                    _ => default,
                }
            };
            tiny_skia::Transform::from_row(
                get(0, 1.0),
                get(1, 0.0),
                get(2, 0.0),
                get(3, 1.0),
                get(4, 0.0),
                get(5, 0.0),
            )
        },
        _ => tiny_skia::Transform::identity(),
    }
}

/// Knockout transparency group threshold for the alpha short-circuit
/// (§11.6.6.2). A fully opaque paint with `BM=Normal` is visually
/// identical between the knockout and non-knockout paths, so we
/// short-circuit the buffer dance when `alpha >= 1.0 - eps`. Slightly
/// under 1.0 to absorb f32 rounding.
const KNOCKOUT_ALPHA_OPAQUE: f32 = 0.9999;

/// Compute the value to feed `knockout_aware_paint`'s `effective_alpha`.
/// For `BM=Normal` it's just the GS alpha; for any non-separable / non-
/// Normal blend mode the formula reads the destination, so even a fully
/// opaque paint needs the backdrop-redirect dance in a knockout group.
/// Returning `0.0` forces the helper to take the dance path regardless of
/// the actual GS alpha — `gs_alpha` itself isn't used downstream past the
/// `< KNOCKOUT_ALPHA_OPAQUE` comparison.
fn knockout_paint_alpha(gs_alpha: f32, blend_mode: &str) -> f32 {
    if blend_mode == "Normal" {
        gs_alpha
    } else {
        0.0
    }
}

/// Run a paint operation in a knockout-aware fashion. When the enclosing
/// group has a knockout backdrop and the effective alpha is below
/// [`KNOCKOUT_ALPHA_OPAQUE`], the paint targets a temp pixmap initialised
/// from the backdrop and the result is merged via [`knockout_merge`] —
/// each painted pixel replaces whatever the previous paint left there.
/// Fully opaque paints (and any paint outside a knockout group) target
/// `pixmap` directly, with zero overhead.
fn knockout_aware_paint<F, R>(
    pixmap: &mut Pixmap,
    knockout_backdrop: Option<&Pixmap>,
    effective_alpha: f32,
    paint_fn: F,
) -> R
where
    F: FnOnce(&mut Pixmap) -> R,
{
    if effective_alpha < KNOCKOUT_ALPHA_OPAQUE {
        if let Some(backdrop) = knockout_backdrop {
            // Per-paint clone is O(W·H). Tracked for a follow-up that
            // hoists a single scratch pixmap across the knockout group
            // and writes via `copy_from_slice` instead of `clone`.
            let mut temp = backdrop.clone();
            // If `paint_fn` errors mid-paint, `temp` is left in whatever
            // partial state the rasterizer wrote. We still run
            // `knockout_merge` so the caller observes the same partial-
            // write semantics as the non-knockout path (which would
            // half-paint `pixmap` directly on the same error).
            let result = paint_fn(&mut temp);
            knockout_merge(pixmap, &temp, backdrop);
            return result;
        }
    }
    paint_fn(pixmap)
}

/// Merge a temp pixmap (a paint rendered against the knockout backdrop)
/// back into the group's accumulating buffer. For each pixel where
/// `temp` differs from `backdrop` — i.e. each pixel the paint demonstrably
/// changed from the initial backdrop — `temp`'s value replaces the
/// destination. Pixels that compare equal to the backdrop remain whatever
/// the previous paint left in `dest`.
///
/// Caveat: a paint that legitimately produces a byte-identical result to
/// the backdrop pixel (e.g. drawing white onto white, or fully transparent
/// over fully transparent) is treated as "untouched" and the prior paint's
/// value survives. In practice the affected cases are visually
/// indistinguishable; a coverage-mask-based detector would be needed for
/// strict §11.6.6.2 conformance in those corner cases.
///
/// All three pixmaps must share dimensions; callers always allocate them
/// at the group pixmap's W*H so this holds.
fn knockout_merge(dest: &mut Pixmap, temp: &Pixmap, backdrop: &Pixmap) {
    let temp_data = temp.data();
    let backdrop_data = backdrop.data();
    let dest_data = dest.data_mut();
    let len = dest_data.len();
    // chunks_exact(4) over RGBA pixels. Pixmap allocates W*H*4 bytes by
    // construction so no trailing partial pixel.
    let mut i = 0;
    while i + 4 <= len {
        if temp_data[i..i + 4] != backdrop_data[i..i + 4] {
            dest_data[i..i + 4].copy_from_slice(&temp_data[i..i + 4]);
        }
        i += 4;
    }
}

/// Returns the current effective clip for paint operators — the intersection
/// of the active clipping-path mask and the active ExtGState soft-mask
/// (§11.6.5.2). The intersection composes the two alphas multiplicatively
/// per pixel (`out = a * b / 255`), which is the §11.3.4 "shape × opacity"
/// rule for 8-bpc alpha. Allocation only happens when *both* stacks
/// contribute a mask at the current level.
fn effective_clip<'a>(
    clip_stack: &'a [Option<tiny_skia::Mask>],
    soft_mask_stack: &'a [Option<tiny_skia::Mask>],
) -> Option<Cow<'a, tiny_skia::Mask>> {
    let clip = clip_stack.last().and_then(|c| c.as_ref());
    let smask = soft_mask_stack.last().and_then(|s| s.as_ref());
    match (clip, smask) {
        (None, None) => None,
        (Some(c), None) => Some(Cow::Borrowed(c)),
        (None, Some(s)) => Some(Cow::Borrowed(s)),
        (Some(c), Some(s)) => {
            if c.width() != s.width() || c.height() != s.height() {
                // Mismatched mask dimensions — fall back to the clip alone
                // rather than mixing buffers of different sizes. The MVP
                // soft-mask code renders the group at page pixmap dimensions,
                // so this should not fire in practice.
                return Some(Cow::Borrowed(c));
            }
            let mut out = c.clone();
            let dst = out.data_mut();
            let src = s.data();
            for (d, &v) in dst.iter_mut().zip(src.iter()) {
                *d = ((*d as u32 * v as u32) / 255) as u8;
            }
            Some(Cow::Owned(out))
        },
    }
}
use crate::rendering::path_rasterizer::PathRasterizer;
use crate::rendering::text_rasterizer::TextRasterizer;

use crate::fonts::FontInfo;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tiny_skia::{Color, PathBuilder, Pixmap, PixmapPaint, Transform};

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
    /// Re-entrancy depth of `materialise_soft_mask_alpha`. The chain
    /// SMask → /G content stream → `Do` → /GS → SMask is legal but
    /// adversarial PDFs can construct self-referential cycles that would
    /// stack-overflow the process. Capped at [`MAX_SMASK_DEPTH`].
    smask_depth: u32,
}

/// Hard cap on nested ExtGState soft-mask materialisations within a single
/// page render. Cyclic `/G` references would otherwise recurse without
/// bound. 32 levels is well above any legitimate artwork; deeper than this
/// strongly indicates a malformed or adversarial fixture.
const MAX_SMASK_DEPTH: u32 = 32;

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
            smask_depth: 0,
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
        self.execute_operators(
            &mut pixmap,
            transform,
            &operators,
            doc,
            page_num,
            &resources,
            None,
        )?;

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
    #[allow(clippy::too_many_arguments)]
    fn execute_operators(
        &mut self,
        pixmap: &mut Pixmap,
        base_transform: Transform,
        operators: &[Operator],
        doc: &PdfDocument,
        page_num: usize,
        resources: &Object,
        knockout_backdrop: Option<&Pixmap>,
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
                                                                       // §11.6.5.2 soft-mask stack — mirrors `clip_stack` so q/Q save/restore
                                                                       // the active mask along with the rest of the graphics state. The
                                                                       // `Option<Mask>` is a pre-rendered alpha buffer (subtype `/Alpha`);
                                                                       // see `materialise_soft_mask_alpha` for how it is built.
        let mut soft_mask_stack: Vec<Option<tiny_skia::Mask>> = vec![None];
        // §11.6.6.2 knockout backdrop. When the enclosing transparency group
        // has /K true the parent calls us with a snapshot of the group's
        // initial pixmap; every translucent paint inside this content stream
        // composites against that backdrop rather than the accumulating
        // group buffer. `Option` keeps the non-knockout path zero-cost.
        let knockout_backdrop: Option<Pixmap> = knockout_backdrop.cloned();

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
                    let current_smask = soft_mask_stack.last().cloned().flatten();
                    soft_mask_stack.push(current_smask);
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
                    if soft_mask_stack.len() > 1 {
                        soft_mask_stack.pop();
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
                    gs_stack.current_mut().fill_color_rgb = (*r, *g, *b);
                    gs_stack.current_mut().fill_color_space = "DeviceRGB".to_string();
                    log::debug!("SetFillRgb: [{}, {}, {}]", r, g, b);
                },
                Operator::SetStrokeRgb { r, g, b } => {
                    gs_stack.current_mut().stroke_color_rgb = (*r, *g, *b);
                    gs_stack.current_mut().stroke_color_space = "DeviceRGB".to_string();
                    log::debug!("SetStrokeRgb: [{}, {}, {}]", r, g, b);
                },
                Operator::SetFillGray { gray } => {
                    let g = *gray;
                    gs_stack.current_mut().fill_color_rgb = (g, g, g);
                    gs_stack.current_mut().fill_color_space = "DeviceGray".to_string();
                    log::debug!("SetFillGray: {}", g);
                },
                Operator::SetStrokeGray { gray } => {
                    let g = *gray;
                    gs_stack.current_mut().stroke_color_rgb = (g, g, g);
                    gs_stack.current_mut().stroke_color_space = "DeviceGray".to_string();
                    log::debug!("SetStrokeGray: {}", g);
                },
                Operator::SetFillCmyk { c, m, y, k } => {
                    // Convert CMYK to RGB
                    let (r, g, b) = cmyk_to_rgb(*c, *m, *y, *k);
                    gs_stack.current_mut().fill_color_rgb = (r, g, b);
                    gs_stack.current_mut().fill_color_cmyk = Some((*c, *m, *y, *k));
                    gs_stack.current_mut().fill_color_space = "DeviceCMYK".to_string();
                    log::debug!("SetFillCmyk: [{}, {}, {}, {}] -> {:?}", c, m, y, k, (r, g, b));
                },
                Operator::SetStrokeCmyk { c, m, y, k } => {
                    let (r, g, b) = cmyk_to_rgb(*c, *m, *y, *k);
                    gs_stack.current_mut().stroke_color_rgb = (r, g, b);
                    gs_stack.current_mut().stroke_color_cmyk = Some((*c, *m, *y, *k));
                    gs_stack.current_mut().stroke_color_space = "DeviceCMYK".to_string();
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
                        let clip_owned = effective_clip(&clip_stack, &soft_mask_stack);
                        let clip = clip_owned.as_deref();
                        if let Some(path) = current_path.finish() {
                            let gs = gs_stack.current();
                            let transform = combine_transforms(base_transform, &gs.ctm);
                            let path_rasterizer = &mut self.path_rasterizer;
                            knockout_aware_paint(
                                pixmap,
                                knockout_backdrop.as_ref(),
                                knockout_paint_alpha(gs.stroke_alpha, &gs.blend_mode),
                                |target| {
                                    path_rasterizer
                                        .stroke_path_clipped(target, &path, transform, gs, clip);
                                },
                            );
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
                        let clip_owned = effective_clip(&clip_stack, &soft_mask_stack);
                        let clip = clip_owned.as_deref();
                        if let Some(path) = current_path.finish() {
                            let gs = gs_stack.current();
                            let transform = combine_transforms(base_transform, &gs.ctm);
                            let path_rasterizer = &mut self.path_rasterizer;
                            knockout_aware_paint(
                                pixmap,
                                knockout_backdrop.as_ref(),
                                knockout_paint_alpha(gs.fill_alpha, &gs.blend_mode),
                                |target| {
                                    path_rasterizer.fill_path_clipped(
                                        target,
                                        &path,
                                        transform,
                                        gs,
                                        tiny_skia::FillRule::Winding,
                                        clip,
                                    );
                                },
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
                        let clip_owned = effective_clip(&clip_stack, &soft_mask_stack);
                        let clip = clip_owned.as_deref();
                        if let Some(path) = current_path.finish() {
                            let gs = gs_stack.current();
                            let transform = combine_transforms(base_transform, &gs.ctm);
                            let fill_rule = if matches!(op, Operator::CloseFillStrokeEvenOdd) {
                                tiny_skia::FillRule::EvenOdd
                            } else {
                                tiny_skia::FillRule::Winding
                            };
                            let path_rasterizer = &mut self.path_rasterizer;
                            knockout_aware_paint(
                                pixmap,
                                knockout_backdrop.as_ref(),
                                knockout_paint_alpha(gs.fill_alpha, &gs.blend_mode),
                                |target| {
                                    path_rasterizer.fill_path_clipped(
                                        target, &path, transform, gs, fill_rule, clip,
                                    );
                                },
                            );
                            knockout_aware_paint(
                                pixmap,
                                knockout_backdrop.as_ref(),
                                knockout_paint_alpha(gs.stroke_alpha, &gs.blend_mode),
                                |target| {
                                    path_rasterizer
                                        .stroke_path_clipped(target, &path, transform, gs, clip);
                                },
                            );
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
                        let clip_owned = effective_clip(&clip_stack, &soft_mask_stack);
                        let clip = clip_owned.as_deref();
                        if let Some(path) = current_path.finish() {
                            let gs = gs_stack.current();
                            let transform = combine_transforms(base_transform, &gs.ctm);
                            let path_rasterizer = &mut self.path_rasterizer;
                            knockout_aware_paint(
                                pixmap,
                                knockout_backdrop.as_ref(),
                                knockout_paint_alpha(gs.fill_alpha, &gs.blend_mode),
                                |target| {
                                    path_rasterizer.fill_path_clipped(
                                        target,
                                        &path,
                                        transform,
                                        gs,
                                        tiny_skia::FillRule::EvenOdd,
                                        clip,
                                    );
                                },
                            );
                            if matches!(op, Operator::FillStrokeEvenOdd) {
                                knockout_aware_paint(
                                    pixmap,
                                    knockout_backdrop.as_ref(),
                                    knockout_paint_alpha(gs.stroke_alpha, &gs.blend_mode),
                                    |target| {
                                        path_rasterizer.stroke_path_clipped(
                                            target, &path, transform, gs, clip,
                                        );
                                    },
                                );
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
                            let clip_owned = effective_clip(&clip_stack, &soft_mask_stack);
                            let clip = clip_owned.as_deref();
                            let transform = combine_transforms(base_transform, &gs.ctm);
                            let text_rasterizer = &mut self.text_rasterizer;
                            let fonts = &self.fonts;
                            knockout_aware_paint(
                                pixmap,
                                knockout_backdrop.as_ref(),
                                knockout_paint_alpha(gs.fill_alpha, &gs.blend_mode),
                                |target| {
                                    text_rasterizer.render_text(
                                        target, text, transform, gs, resources, doc, clip, fonts,
                                    )
                                },
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
                            let clip_owned = effective_clip(&clip_stack, &soft_mask_stack);
                            let clip = clip_owned.as_deref();
                            let transform = combine_transforms(base_transform, &gs.ctm);
                            let text_rasterizer = &mut self.text_rasterizer;
                            let fonts = &self.fonts;
                            knockout_aware_paint(
                                pixmap,
                                knockout_backdrop.as_ref(),
                                knockout_paint_alpha(gs.fill_alpha, &gs.blend_mode),
                                |target| {
                                    text_rasterizer.render_text(
                                        target, text, transform, gs, resources, doc, clip, fonts,
                                    )
                                },
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
                            let clip_owned = effective_clip(&clip_stack, &soft_mask_stack);
                            let clip = clip_owned.as_deref();
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
                            let text_rasterizer = &mut self.text_rasterizer;
                            let fonts = &self.fonts;
                            knockout_aware_paint(
                                pixmap,
                                knockout_backdrop.as_ref(),
                                knockout_paint_alpha(gs.fill_alpha, &gs.blend_mode),
                                |target| {
                                    text_rasterizer.render_tj_array(
                                        target, array, transform, gs, resources, doc, clip, fonts,
                                    )
                                },
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
                            let clip_owned = effective_clip(&clip_stack, &soft_mask_stack);
                            let clip = clip_owned.as_deref();
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
                            let text_rasterizer = &mut self.text_rasterizer;
                            let fonts = &self.fonts;
                            knockout_aware_paint(
                                pixmap,
                                knockout_backdrop.as_ref(),
                                knockout_paint_alpha(gs.fill_alpha, &gs.blend_mode),
                                |target| {
                                    text_rasterizer.render_text(
                                        target, text, transform, gs, resources, doc, clip, fonts,
                                    )
                                },
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
                        let clip_owned = effective_clip(&clip_stack, &soft_mask_stack);
                        let clip = clip_owned.as_deref();
                        log::debug!("Do: rendering XObject '{}'", name);
                        // §11.6.6.2: treat the Do as one element for
                        // knockout purposes — render the image or form into
                        // a backdrop-relative temp and merge so it replaces
                        // (rather than blends with) prior paints in the group.
                        let alpha = knockout_paint_alpha(gs.fill_alpha, &gs.blend_mode);
                        knockout_aware_paint(
                            pixmap,
                            knockout_backdrop.as_ref(),
                            alpha,
                            |target| {
                                self.render_xobject(
                                    target, name, transform, gs, resources, doc, page_num, clip,
                                )
                            },
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
                    // §11.6.5.2 soft mask handling. `/SMask /None` clears
                    // the active mask; `/SMask <dict>` rasterises the
                    // group into a pixmap and stashes its alpha channel
                    // as the new mask. Rasterisation is deferred to here
                    // (rather than the parser) because it needs the page
                    // pixmap context and the renderer's own `render_form_xobject`.
                    if let Some(spec) = entry.soft_mask.clone() {
                        match spec {
                            SoftMaskSpec::None => {
                                if let Some(slot) = soft_mask_stack.last_mut() {
                                    *slot = None;
                                }
                            },
                            SoftMaskSpec::Dict(dict_obj) => {
                                // §11.6.5.2: the SMask group is rendered at the
                                // CTM that was current at install time, not the
                                // page-level base transform alone.
                                let install_transform =
                                    combine_transforms(base_transform, &gs_stack.current().ctm);
                                // Cache reuse: only when the install CTM
                                // matches bitwise. A different CTM produces a
                                // different mask, so falling through to the
                                // materialise path is required for correctness.
                                let cache_hit = entry
                                    .cached_install_transform
                                    .map(|t| t == install_transform)
                                    .unwrap_or(false)
                                    && entry.cached_soft_mask_alpha.is_some();
                                if cache_hit {
                                    if let Some(slot) = soft_mask_stack.last_mut() {
                                        *slot = entry.cached_soft_mask_alpha.clone();
                                    }
                                } else {
                                    match self.materialise_soft_mask_alpha(
                                        &dict_obj,
                                        pixmap.width(),
                                        pixmap.height(),
                                        install_transform,
                                        doc,
                                        page_num,
                                        resources,
                                    ) {
                                        Ok(mask) => {
                                            entry.cached_soft_mask_alpha = Some(mask.clone());
                                            entry.cached_install_transform =
                                                Some(install_transform);
                                            if let Some(slot) = soft_mask_stack.last_mut() {
                                                *slot = Some(mask);
                                            }
                                        },
                                        Err(e) => {
                                            log::warn!("Skipping SMask on /{}: {}", dict_name, e);
                                        },
                                    }
                                }
                            },
                        }
                    }
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
                        let clip_owned = effective_clip(&clip_stack, &soft_mask_stack);
                        let clip = clip_owned.as_deref();
                        let alpha = knockout_paint_alpha(gs.fill_alpha, &gs.blend_mode);
                        knockout_aware_paint(
                            pixmap,
                            knockout_backdrop.as_ref(),
                            alpha,
                            |target| {
                                self.render_shading(
                                    target, name, transform, gs, resources, doc, clip,
                                )
                            },
                        )?;
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

        match shading_type {
            2 => self.render_axial_shading(pixmap, &shading, transform, gs, doc, clip_mask),
            3 => self.render_radial_shading(pixmap, &shading, transform, gs, doc, clip_mask),
            _ => {
                log::debug!("Unsupported shading type {} for '{}'", shading_type, name);
                Ok(())
            },
        }
    }

    /// Render axial (linear) gradient shading (Type 2).
    fn render_axial_shading(
        &self,
        pixmap: &mut Pixmap,
        shading: &std::collections::HashMap<String, Object>,
        transform: Transform,
        gs: &GraphicsState,
        doc: &PdfDocument,
        clip_mask: Option<&tiny_skia::Mask>,
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

        // Parse Function to get start and end colors
        // For simplicity, evaluate at t=0 and t=1 to get endpoint colors
        let (c0, c1) = self.evaluate_shading_function(shading, doc)?;

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
                    tiny_skia::Color::from_rgba(c0.0, c0.1, c0.2, gs.fill_alpha)
                        .unwrap_or(tiny_skia::Color::BLACK),
                ),
                tiny_skia::GradientStop::new(
                    1.0,
                    tiny_skia::Color::from_rgba(c1.0, c1.1, c1.2, gs.fill_alpha)
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
    fn render_radial_shading(
        &self,
        pixmap: &mut Pixmap,
        shading: &std::collections::HashMap<String, Object>,
        transform: Transform,
        gs: &GraphicsState,
        doc: &PdfDocument,
        clip_mask: Option<&tiny_skia::Mask>,
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
        let (_x0, _y0, _r0, x1, y1, r1) =
            (get_f(0), get_f(1), get_f(2), get_f(3), get_f(4), get_f(5));

        let (c0, c1) = self.evaluate_shading_function(shading, doc)?;

        let mut center = tiny_skia::Point { x: x1, y: y1 };
        let mut edge = tiny_skia::Point { x: x1 + r1, y: y1 };
        transform.map_point(&mut center);
        transform.map_point(&mut edge);
        let radius = ((edge.x - center.x).powi(2) + (edge.y - center.y).powi(2)).sqrt();

        let gradient = tiny_skia::RadialGradient::new(
            tiny_skia::Point {
                x: center.x,
                y: center.y,
            },
            0.0, // start_radius (inner circle)
            tiny_skia::Point {
                x: center.x,
                y: center.y,
            },
            radius, // end_radius
            vec![
                tiny_skia::GradientStop::new(
                    0.0,
                    tiny_skia::Color::from_rgba(c0.0, c0.1, c0.2, gs.fill_alpha)
                        .unwrap_or(tiny_skia::Color::BLACK),
                ),
                tiny_skia::GradientStop::new(
                    1.0,
                    tiny_skia::Color::from_rgba(c1.0, c1.1, c1.2, gs.fill_alpha)
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
                "Rendered radial gradient at ({:.1},{:.1}) r={:.1}",
                center.x,
                center.y,
                radius
            );
        }

        Ok(())
    }

    /// Evaluate a shading function at t=0 and t=1 to get start/end colors.
    fn evaluate_shading_function(
        &self,
        shading: &std::collections::HashMap<String, Object>,
        doc: &PdfDocument,
    ) -> Result<((f32, f32, f32), (f32, f32, f32))> {
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
                    // Type 2: Exponential interpolation f(x) = C0 + x^N * (C1 - C0)
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
                    return Ok((c0, c1));
                } else if func_type == 3 {
                    // Type 3: Stitching function — wraps multiple sub-functions
                    // For gradient endpoints, evaluate first sub-function at domain bounds
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

        // PDF images occupy a unit square in user space; image rows are top-to-bottom
        // (opposite of PDF's bottom-to-top y axis), so the pre_scale flips them.
        let image_transform = transform
            .pre_translate(0.0, 1.0)
            .pre_scale(1.0 / src_w as f32, -1.0 / src_h as f32);

        let mut paint = PixmapPaint::default();
        paint.opacity = gs.fill_alpha;
        paint.blend_mode = crate::rendering::pdf_blend_mode_to_skia(&gs.blend_mode);

        // Fast path: SIMD pre-resize when the transform is a pure scale+translate and
        // the image is being downscaled.  fast_image_resize (AVX2/SSE4.1/NEON) resizes
        // to exact output dimensions; we then blit the already-correct pixels at the
        // right position with a translate-only transform and Nearest quality (no second
        // resampling pass).  For rotated/sheared transforms or upscaling, fall through
        // to the tiny-skia bilinear/bicubic path.
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
                paint.quality = tiny_skia::FilterQuality::Nearest;
                let t = Transform::from_translate(image_transform.tx, image_transform.ty);
                (dst_w, dst_h, pixels, t)
            } else {
                // fast_image_resize failed; fall back to bilinear via tiny_skia
                let (xs, ys) = image_transform.get_scale();
                paint.quality = if xs >= 1.0 || ys >= 1.0 {
                    tiny_skia::FilterQuality::Bicubic
                } else {
                    tiny_skia::FilterQuality::Bilinear
                };
                (src_w, src_h, rgba_image.into_raw(), image_transform)
            }
        } else {
            // Rotated / sheared / upscaling path: let tiny_skia resample.
            let (xs, ys) = image_transform.get_scale();
            paint.quality = if xs >= 1.0 || ys >= 1.0 {
                tiny_skia::FilterQuality::Bicubic
            } else {
                tiny_skia::FilterQuality::Bilinear
            };
            (src_w, src_h, rgba_image.into_raw(), image_transform)
        };

        if let Some(img_pixmap) =
            Pixmap::from_vec(blit_data, tiny_skia::IntSize::from_wh(blit_w, blit_h).unwrap())
        {
            pixmap.draw_pixmap(0, 0, img_pixmap.as_ref(), &paint, blit_transform, clip_mask);
        }

        Ok(())
    }
}

/// Which channel of the rendered SMask group becomes the alpha mask buffer
/// (ISO 32000-1 §11.6.5).
#[derive(Clone, Copy)]
enum SoftMaskKind {
    /// Subtype `/Alpha` (§11.6.5.2): use the group's alpha channel.
    Alpha,
    /// Subtype `/Luminosity` (§11.6.5.3): use the per-pixel BT.601 luma of
    /// the group's premultiplied RGB.
    Luminosity,
}

/// SMask `/TR` transfer function. PDF functions can be of several types;
/// only the ones plausibly used for soft masks are implemented here.
/// Identity is represented by `None` at the call site.
#[derive(Clone, Debug)]
enum SoftMaskTransfer {
    /// Type 2 exponential: `y = C0 + x^N * (C1 - C0)`.
    /// For SMask mask buffers `C0` and `C1` are always 1-vector entries.
    Type2 { c0: f64, c1: f64, n: f64 },
}

impl SoftMaskTransfer {
    /// Apply the transfer function to a mask value in `[0, 1]`.
    fn apply(&self, x: f64) -> f64 {
        match *self {
            SoftMaskTransfer::Type2 { c0, c1, n } => c0 + x.powf(n) * (c1 - c0),
        }
    }
}

/// Parse the SMask `/TR` entry. Returns `None` when the entry is absent,
/// is the name `/Identity`, or is a function of an unsupported type — the
/// caller treats those identically to "skip transfer". Unknown shapes log
/// a `debug` so production logs don't flood.
fn parse_soft_mask_transfer(
    tr_obj: Option<&Object>,
    doc: &PdfDocument,
) -> Option<SoftMaskTransfer> {
    let tr = tr_obj?;
    let resolved = doc.resolve_object(tr).ok()?;
    if matches!(&resolved, Object::Name(n) if n == "Identity") {
        return None;
    }
    let dict = resolved.as_dict()?;
    let func_type = dict.get("FunctionType").and_then(|o| o.as_integer())?;
    match func_type {
        2 => {
            // §7.10.3: Type 2 produces `n`-component output where
            // `n = len(C0) = len(C1)`. SMask /TR is single-component;
            // reject multi-component functions outright rather than
            // silently picking C0[0] / C1[0] from a wider vector.
            let c0_arr = dict.get("C0").and_then(|o| o.as_array());
            let c1_arr = dict.get("C1").and_then(|o| o.as_array());
            let c0 = if let Some(arr) = c0_arr {
                if arr.len() != 1 {
                    log::debug!("SMask /TR Type 2 has {}-component /C0; expected 1", arr.len());
                    return None;
                }
                as_f64(arr.first()?).unwrap_or(0.0)
            } else {
                0.0
            };
            let c1 = if let Some(arr) = c1_arr {
                if arr.len() != 1 {
                    log::debug!("SMask /TR Type 2 has {}-component /C1; expected 1", arr.len());
                    return None;
                }
                as_f64(arr.first()?).unwrap_or(1.0)
            } else {
                1.0
            };
            let n = dict.get("N").and_then(as_f64).unwrap_or(1.0);
            // §7.10.3 requires N > 0 (and for non-integer N, Domain must
            // exclude 0). Reject N <= 0 — `0_f64.powf(0.0)` returns 1.0
            // (IEEE 754) which would flip a "blocked" mask pixel into
            // "fully passes", exactly the wrong direction for a malformed
            // function.
            if !(n > 0.0 && n.is_finite()) {
                log::debug!("SMask /TR Type 2 has invalid /N = {n}; skipping");
                return None;
            }
            Some(SoftMaskTransfer::Type2 { c0, c1, n })
        },
        other => {
            log::debug!("SMask /TR FunctionType {other} not supported; skipping");
            None
        },
    }
}

/// Coerce a PDF numeric object to `f64`. Returns `None` for anything else.
fn as_f64(obj: &Object) -> Option<f64> {
    match obj {
        Object::Real(v) => Some(*v),
        Object::Integer(v) => Some(*v as f64),
        _ => None,
    }
}

/// Parse the SMask `/BC` (backdrop colour) entry into an opaque RGBA pixel.
/// `group_cs` is the group's blend colour space name (e.g. `DeviceRGB`,
/// `DeviceGray`, `DeviceCMYK`); other / array spaces fall through to
/// "treat the components as RGB if 3, gray if 1, CMYK if 4".
///
/// Returns `None` for the default backdrop (black in the group CS), which
/// already matches the all-zero initial state of `Pixmap::new`. The caller
/// only needs to pre-fill when a non-default backdrop is present.
fn parse_soft_mask_backdrop(bc_obj: Option<&Object>, group_cs: &str) -> Option<[u8; 4]> {
    let arr = bc_obj?.as_array()?;
    let get = |i: usize| -> Option<f32> { arr.get(i).and_then(as_f64).map(|v| v as f32) };
    // Determine component count from /CS, falling back to array length when
    // the CS is unknown.
    let (r, g, b) = match (group_cs, arr.len()) {
        ("DeviceGray" | "CalGray", _) | (_, 1) => {
            let v = get(0)?;
            let q = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            (q, q, q)
        },
        ("DeviceRGB" | "CalRGB" | "ICCBased", _) | (_, 3) => {
            let r = (get(0)?.clamp(0.0, 1.0) * 255.0).round() as u8;
            let g = (get(1)?.clamp(0.0, 1.0) * 255.0).round() as u8;
            let b = (get(2)?.clamp(0.0, 1.0) * 255.0).round() as u8;
            (r, g, b)
        },
        ("DeviceCMYK" | "CalCMYK", _) | (_, 4) => {
            // Approximate CMYK→RGB without ICC: R = (1 - C)(1 - K) etc.
            // Correct for the common "K-only" and "process-CMYK" backdrops
            // we expect from real artwork; full ICC fidelity is out of scope.
            let c = get(0)?.clamp(0.0, 1.0);
            let m = get(1)?.clamp(0.0, 1.0);
            let y = get(2)?.clamp(0.0, 1.0);
            let k = get(3)?.clamp(0.0, 1.0);
            let r = ((1.0 - c) * (1.0 - k) * 255.0).round() as u8;
            let g = ((1.0 - m) * (1.0 - k) * 255.0).round() as u8;
            let b = ((1.0 - y) * (1.0 - k) * 255.0).round() as u8;
            (r, g, b)
        },
        _ => return None,
    };
    // Default backdrop (black) requires no pre-fill; the pixmap is already
    // (0, 0, 0, 0).
    if r == 0 && g == 0 && b == 0 {
        return None;
    }
    Some([r, g, b, 255])
}

impl PageRenderer {
    /// Render an ExtGState `/SMask` group into an offscreen pixmap and
    /// return its mask buffer as a `tiny_skia::Mask` for use as a clip on
    /// subsequent paint operations (ISO 32000-1 §11.6.5.2 / §11.6.5.3).
    ///
    /// Subtypes:
    ///   - `/Alpha`: the rendered group's alpha channel is the mask.
    ///     `/BC` is ignored per spec.
    ///   - `/Luminosity`: per-pixel BT.601 luma of the rendered group's
    ///     premultiplied RGB.  Always BT.601 on the rasterised RGB —
    ///     there is no `/CS`-aware luma dispatch.  Implications:
    ///       * Valid DeviceGray groups (`R = G = B`) collapse to
    ///         `Y = R`, matching the spec result.
    ///       * Valid DeviceRGB groups get the spec result up to the
    ///         BT.601 vs Rec.709 vs ICC-defined luma weighting choice.
    ///       * Valid DeviceCMYK groups go through the renderer's
    ///         CMYK→RGB pre-conversion before luma is read; this is an
    ///         approximation and will drift from a spec-correct
    ///         CMYK-blend-space luma calculation.
    ///       * Malformed groups (e.g. `/CS /DeviceGray` with RGB paint
    ///         operators) get BT.601 on the actual RGB; see
    ///         `tests/test_smask_alpha.rs::ext_gstate_luminosity_smask_malformed_devicegray_with_rgb_paint_uses_bt601`.
    ///
    ///     A proper `/CS` dispatch would need a non-RGB blend buffer
    ///     (separate gray / CMYK pixmaps) which the renderer does not
    ///     currently provide.
    ///
    /// `/BC` (backdrop colour) for Luminosity: parsed against the group's
    /// declared `/CS` (DeviceGray / DeviceRGB / DeviceCMYK; other CS fall
    /// back to component-count inference) and pre-filled into the offscreen
    /// pixmap so unpainted areas contribute the right luminance. ICC and
    /// Lab conversions for `/BC` are not implemented.
    ///
    /// `/TR` (transfer function): applied pointwise after the subtype
    /// buffer is computed. Type 2 (exponential) is supported directly;
    /// `/Identity`, missing `/TR`, and other types (0 sampled, 3 stitching,
    /// 4 PostScript) are no-ops.
    #[allow(clippy::too_many_arguments)]
    fn materialise_soft_mask_alpha(
        &mut self,
        smask_dict_obj: &Object,
        width: u32,
        height: u32,
        base_transform: Transform,
        doc: &PdfDocument,
        page_num: usize,
        resources: &Object,
    ) -> Result<tiny_skia::Mask> {
        if self.smask_depth >= MAX_SMASK_DEPTH {
            return Err(crate::error::Error::InvalidPdf(format!(
                "SMask nesting exceeded {MAX_SMASK_DEPTH} levels — possible cyclic /G reference",
            )));
        }

        let smask_dict = smask_dict_obj.as_dict().ok_or_else(|| {
            crate::error::Error::InvalidPdf("SMask is not a dictionary".to_string())
        })?;

        // §11.6.5.2 Table 144 marks /S as required. Real-world PDFs
        // occasionally omit it; rather than picking a wrong default and
        // mis-rasterising the group, skip-with-debug. The outer
        // SetExtGState handler logs a `warn` for the skip itself.
        let subtype = smask_dict
            .get("S")
            .and_then(|o| o.as_name())
            .ok_or_else(|| {
                crate::error::Error::InvalidPdf(
                    "SMask dict missing required /S (subtype) — skipping".to_string(),
                )
            })?;
        let smask_kind = match subtype {
            "Alpha" => SoftMaskKind::Alpha,
            "Luminosity" => SoftMaskKind::Luminosity,
            other => {
                return Err(crate::error::Error::InvalidPdf(format!(
                    "SMask subtype /{other} not recognised"
                )));
            },
        };

        // §11.6.5.3 /TR — a function applied to each mask value after the
        // subtype-specific buffer is computed. Most SMasks have no /TR or
        // use /Identity, in which case `transfer` stays `None`.
        let transfer = parse_soft_mask_transfer(smask_dict.get("TR"), doc);

        let group_obj = smask_dict.get("G").ok_or_else(|| {
            crate::error::Error::InvalidPdf("SMask missing /G transparency group".to_string())
        })?;
        let group_resolved = doc.resolve_object(group_obj)?;
        let group_dict = match &group_resolved {
            Object::Stream { dict, .. } => dict.clone(),
            _ => {
                return Err(crate::error::Error::InvalidPdf("SMask /G is not a stream".to_string()))
            },
        };
        let group_data = if let Some(stream_ref) = group_obj.as_reference() {
            doc.decode_stream_with_encryption(&group_resolved, stream_ref)?
        } else {
            group_resolved.decode_stream_data()?
        };

        // Render the group into a fresh pixmap matching the page's dimensions.
        // Form /Matrix + /BBox position the painted content inside that buffer.
        // Areas outside the group's painted region keep their initial alpha = 0,
        // which is the correct subtractive default for `/S /Alpha`.
        let mut group_pixmap = Pixmap::new(width, height).ok_or_else(|| {
            crate::error::Error::InvalidPdf("Failed to allocate SMask group pixmap".to_string())
        })?;

        // §7.8.3: /Resources may be an indirect reference. The previous code
        // grabbed it raw and `load_resources` would short-circuit because it
        // only handles Dictionaries — fonts/colorspaces declared by the SMask
        // group itself silently failed to load.
        let form_resources = match group_dict.get("Resources") {
            Some(o) => doc.resolve_object(o)?,
            None => resources.clone(),
        };
        let old_fonts = self.fonts.clone();
        let old_cs = self.color_spaces.clone();
        self.load_resources(doc, &form_resources)?;

        // §11.6.5.3 /BC + §11.6.6 Group /CS — pre-fill the group pixmap
        // with the backdrop colour for Luminosity masks. Alpha masks ignore
        // /BC by spec (the alpha channel of "no paint" is 0 regardless).
        // /BC is only honoured for Luminosity; the default black backdrop
        // requires no pre-fill since `Pixmap::new` already gives (0,0,0,0).
        // §11.6.6 group /CS may be a name or an array (`[/ICCBased <ref>]`,
        // `[/CalRGB <dict>]`, etc.). Resolve the array form to its first-
        // element name so /BC interprets components against the right CS
        // family; defaults to DeviceRGB when neither shape is present.
        let group_cs = group_dict
            .get("Group")
            .and_then(|g| g.as_dict())
            .and_then(|gd| gd.get("CS"))
            .and_then(|cs| match cs {
                Object::Name(n) => Some(n.as_str()),
                Object::Array(a) => a.first().and_then(|o| o.as_name()),
                _ => None,
            })
            .unwrap_or("DeviceRGB");
        if matches!(smask_kind, SoftMaskKind::Luminosity) {
            if let Some(bg) = parse_soft_mask_backdrop(smask_dict.get("BC"), group_cs) {
                for chunk in group_pixmap.data_mut().chunks_exact_mut(4) {
                    chunk.copy_from_slice(&bg);
                }
            }
        }

        // Render the form's contents directly into `group_pixmap`. We
        // deliberately bypass `render_form_xobject`: that path would detect
        // the form's `/Group /S /Transparency` and allocate a *second*
        // page-sized pixmap to act as the transparency-group buffer. But
        // `group_pixmap` *is* the transparency-group buffer — it starts
        // fully transparent (Pixmap::new) which is the correct isolated-
        // group initial backdrop, so the double allocation is pure waste.
        // Nested Form XObjects inside the SMask group still go through
        // `render_form_xobject` via `Operator::Do`, so their own
        // transparency groups are honoured.
        let form_matrix = parse_form_matrix(&group_dict);
        let install_transform = base_transform.pre_concat(form_matrix);
        let operators = parse_content_stream(&group_data)?;
        self.smask_depth += 1;
        let render_res = self.execute_operators(
            &mut group_pixmap,
            install_transform,
            &operators,
            doc,
            page_num,
            &form_resources,
            None,
        );
        self.smask_depth -= 1;

        self.fonts = old_fonts;
        self.color_spaces = old_cs;
        render_res?;

        // Build the Mask buffer from the group pixmap. Source pixels are
        // tiny-skia's premultiplied RGBA; for /Luminosity we read straight
        // from the premultiplied R/G/B which is correct for the default
        // black /BC (unpainted pixels contribute zero, painted pixels'
        // luminance scales with their own alpha — both spec-aligned for the
        // common case).
        let mut mask = tiny_skia::Mask::new(width, height).ok_or_else(|| {
            crate::error::Error::InvalidPdf("Failed to allocate SMask buffer".to_string())
        })?;
        let mask_data = mask.data_mut();
        match smask_kind {
            SoftMaskKind::Alpha => {
                for (i, chunk) in group_pixmap.data().chunks_exact(4).enumerate() {
                    mask_data[i] = chunk[3];
                }
            },
            SoftMaskKind::Luminosity => {
                for (i, chunk) in group_pixmap.data().chunks_exact(4).enumerate() {
                    // BT.601 luma: Y = 0.299·R + 0.587·G + 0.114·B.
                    // Integer form with weights × 256 (77 + 150 + 29 = 256)
                    // and `>> 8` so the result stays inside u8.
                    let r = chunk[0] as u32;
                    let g = chunk[1] as u32;
                    let b = chunk[2] as u32;
                    mask_data[i] = ((r * 77 + g * 150 + b * 29) >> 8) as u8;
                }
            },
        }

        // §11.6.5 /TR — apply the transfer function pointwise to the
        // computed mask buffer. Mask byte 0..=255 maps to function input
        // 0.0..=1.0; the function's output is clamped back to 0..=255.
        if let Some(tr) = transfer.as_ref() {
            for byte in mask_data.iter_mut() {
                let input = *byte as f64 / 255.0;
                let output = tr.apply(input).clamp(0.0, 1.0);
                *byte = (output * 255.0).round() as u8;
            }
        }

        Ok(mask)
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
        let form_matrix = parse_form_matrix(dict);

        // Combine parent transform with form matrix
        let combined_transform = parent_transform.pre_concat(form_matrix);

        // Check for transparency group (PDF spec section 11.6.6). /Group may be
        // an indirect reference (`/Group 12 0 R`) in real-world output; resolve
        // it before reading its fields.
        let group_obj = dict.get("Group").and_then(|g| doc.resolve_object(g).ok());
        let group_dict = group_obj.as_ref().and_then(|g| g.as_dict());
        let is_transparency_group = group_dict
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
            //
            // Accept boolean true *or* a non-zero integer for /I and /K —
            // some legacy tools emit `/K 1` instead of `/K true`.
            let parse_flag = |v: &Object| -> bool {
                match v {
                    Object::Boolean(b) => *b,
                    Object::Integer(n) => *n != 0,
                    _ => false,
                }
            };
            let is_isolated = group_dict
                .and_then(|gd| gd.get("I"))
                .map(&parse_flag)
                .unwrap_or(false);
            // §11.6.6.2 knockout flag — when true, each painted element
            // composites against the group's *initial backdrop* rather than
            // the accumulating result.
            let is_knockout = group_dict
                .and_then(|gd| gd.get("K"))
                .map(&parse_flag)
                .unwrap_or(false);

            log::debug!(
                "Rendering transparency group (isolated={is_isolated}, knockout={is_knockout})"
            );

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

            // For knockout groups, snapshot the initial pixmap state — every
            // paint inside the group's content stream needs to composite
            // against this backdrop instead of the accumulating buffer.
            let knockout_backdrop = if is_knockout {
                Some(group_pixmap.clone())
            } else {
                None
            };

            // Execute operators into the group pixmap
            self.execute_operators(
                &mut group_pixmap,
                combined_transform,
                &operators,
                doc,
                page_num,
                &form_resources,
                knockout_backdrop.as_ref(),
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
                None,
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
}
