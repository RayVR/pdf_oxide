//! Per-page compositing sidecar for transparency + spot-ink rendering.
//!
//! ISO 32000-1:2008 §11.4 (and §11.4 in ISO 32000-2:2020) defines
//! transparency compositing as a *source-space* operation: each paint
//! is blended against the backdrop in the page-group blend space, and
//! only after every transparency / soft-mask / knockout operation has
//! been resolved does the output get handed off to the device. For a
//! press-target output the blend space is `DeviceCMYK` (or calibrated
//! CMYK via an `ICCBased` profile) and the final hand-off goes to
//! per-plate separations — that is the "composite-then-separate"
//! workflow §11.7.3 / §11.7.4 describe.
//!
//! The page renderer keeps a 4-channel `DeviceCMYK` plane alongside
//! the visible RGBA pixmap so the compose-first and overprint helpers
//! can read the backdrop CMYK quadruple directly instead of inverting
//! the post-ICC RGB (which is lossy under non-linear OutputIntent
//! profiles). This sidecar IS the §11.4 compositing buffer for the
//! process channels.
//!
//! # Spot inks
//!
//! ISO 32000-1 §11.3.4 enumerates the legal blend colour spaces
//! (`DeviceGray`, `DeviceRGB`, `DeviceCMYK`, CIE-based equivalents,
//! and bidirectional `ICCBased` of those) and explicitly excludes
//! `Separation` and `DeviceN`:
//!
//! > "The blending colour space shall be consulted only for process
//! > colours. … such colours shall not be converted to a blending
//! > colour space … the specified colour components shall be blended
//! > individually with the corresponding components of the backdrop."
//!
//! §11.6.6 (Table 147 `/CS` entry) carries the same restriction
//! forward for transparency-group colour spaces. §11.7.3 prescribes
//! the sidecar model:
//!
//! > "When an object is painted transparently with a spot colour
//! > component that is available in the output device, that colour
//! > shall be composited with the corresponding spot colour
//! > component of the backdrop, independently of the compositing that
//! > is performed for process colours. A spot colour retains its own
//! > identity; it shall not be subject to conversion to or from the
//! > colour space of the enclosing transparency group or page."
//!
//! Concretely: the spot lanes ride *alongside* the process blend
//! space, not inside it. They are per-component buffers that the
//! compositing math touches separately from the process lanes.
//!
//! # §11.7.4.2 blend-mode split
//!
//! §11.7.4.2 is the dispositive rule for non-separable and
//! non-white-preserving blend modes on spot channels:
//!
//! > "The PDF graphics state specifies only one current blend mode
//! > parameter, which shall always apply to process colorants and
//! > sometimes to spot colorants as well. Specifically, only
//! > separable, white-preserving blend modes shall be used for spot
//! > colours. If the specified blend mode is not separable and
//! > white-preserving, it shall apply only to process colour
//! > components, and the **Normal** blend mode shall be substituted
//! > for spot colours."
//!
//! The four non-separable modes (`/Hue`, `/Saturation`, `/Color`,
//! `/Luminosity`, §11.3.5.3) AND the two separable-but-non-white-
//! preserving modes (`/Difference`, `/Exclusion`, §11.3.5.2 Note 2)
//! all trigger `/Normal` substitution on spot lanes. This is encoded
//! by [`BlendModeClass`](crate::rendering::sidecar::BlendModeClass)
//! below.
//!
//! Process lanes always honour the requested blend mode; for non-sep
//! modes the §11.3.5.3 CMYK projection (complement `CMY → RGB`,
//! blend, complement back; `K = K_b` for Hue / Saturation / Color and
//! `K = K_s` for Luminosity) applies. That math lives in the renderer
//! (round 2 will wire it for the spot-aware paths); this module
//! supplies only the classification helper.
//!
//! # Storage layout
//!
//! The `CmykSidecar` storage type (crate-private; see the type
//! definition below) owns two separate buffers:
//!
//! - `cmyk`: a packed `4·w·h` byte plane with the four `DeviceCMYK`
//!   channels in `(C, M, Y, K)` order, row-major, top-left origin.
//!   This matches the round-4 layout exactly so every existing
//!   process-plane helper (mirror, compose-first, overprint) consumes
//!   it unchanged.
//! - `spots`: a plane-per-ink stack. For `N` discovered spot inks the
//!   buffer is `N·w·h` bytes long; spot `i`'s plane is the slice
//!   `spots[i·w·h .. (i+1)·w·h]`. Each byte is a tint value (0 = no
//!   ink, 255 = full tint) per the §8.6.6 model and §11.7.3
//!   "additive value of 1.0 (or subtractive tint value of 0.0)"
//!   resting-state rule.
//!
//! Spot names live in `spot_names`, ordered as `get_page_inks_deep`
//! returns them (sorted ASCII, deduped, with `/All` and `/None`
//! filtered out per §8.6.6.4).

use std::collections::HashMap;

use crate::document::PdfDocument;
use crate::object::Object;

/// Classification of a PDF blend-mode name into the three categories
/// §11.7.4.2 cares about.
///
/// Used by the compositor to decide whether the spot lanes should
/// honour the requested blend mode or substitute `/Normal`. Process
/// lanes always honour the requested mode regardless of class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendModeClass {
    /// Separable AND white-preserving. ISO 32000-1 §11.3.5.2: the
    /// ten standard modes whose formula reduces to the source colour
    /// when the backdrop is white. Spot lanes apply the requested
    /// mode component-wise.
    ///
    /// Members: `/Normal`, `/Multiply`, `/Screen`, `/Overlay`,
    /// `/Darken`, `/Lighten`, `/ColorDodge`, `/ColorBurn`,
    /// `/HardLight`, `/SoftLight`.
    SeparableWhitePreserving,
    /// Separable but NOT white-preserving. ISO 32000-1 §11.3.5.2
    /// Note 2 names exactly two: `/Difference` and `/Exclusion`.
    /// Spot lanes substitute `/Normal` per §11.7.4.2.
    SeparableNonWhitePreserving,
    /// Non-separable. ISO 32000-1 §11.3.5.3 lists exactly four:
    /// `/Hue`, `/Saturation`, `/Color`, `/Luminosity`. Their formulas
    /// project to 3-component RGB; on a CMYK blend space the CMY
    /// channels run through the projection and the K channel follows
    /// the §11.3.5.3 rule (backdrop K for Hue/Saturation/Color,
    /// source K for Luminosity). Spot lanes substitute `/Normal` per
    /// §11.7.4.2.
    NonSeparable,
}

/// Process-lane dispatch under §11.7.4.2. The rule is one-line: the
/// process lanes always honour the requested blend mode. The enum
/// exists so the call site reads as "process_dispatch == UseRequested"
/// (single variant today) and round 2's wiring can match on it without
/// magic booleans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessBlendDispatch {
    /// Run the requested PDF blend mode on the process lanes. For
    /// separable modes this is component-wise per §11.3.5.2; for
    /// non-separable modes this is the §11.3.5.3 RGB-projection with
    /// the K-channel rule for CMYK blend spaces.
    UseRequested,
}

/// Spot-lane dispatch under §11.7.4.2. Either "apply the requested
/// blend mode component-wise" (only when the BM is separable AND
/// white-preserving) or "substitute `/Normal`" (every other class).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpotBlendDispatch {
    /// Apply the requested blend mode to spot lanes component-wise.
    /// Reachable only when the BM is separable AND white-preserving.
    UseRequested,
    /// Substitute `/Normal` (source-over) on spot lanes regardless of
    /// the requested blend mode. The §11.7.4.2 rule: non-separable
    /// AND non-white-preserving modes have no defensible spot-lane
    /// behaviour, so the conforming reader paints spots as if the
    /// graphics state declared `/BM /Normal`.
    SubstituteNormal,
}

impl BlendModeClass {
    /// Classify a PDF blend-mode name into one of the three §11.7.4.2
    /// categories.
    ///
    /// Per ISO 32000-1 §11.6.3, an unknown blend mode name shall fall
    /// back to `/Normal`. We honour that by classifying unknown names
    /// as [`BlendModeClass::SeparableWhitePreserving`] — the same
    /// class `/Normal` itself belongs to. This matches the existing
    /// `pdf_blend_mode_to_skia` fallback in `src/rendering/mod.rs`.
    pub fn from_name(name: &str) -> Self {
        match name {
            // ISO 32000-1 §11.3.5.2: ten separable modes; all
            // white-preserving except Difference and Exclusion (Note 2).
            "Normal" | "Multiply" | "Screen" | "Overlay" | "Darken" | "Lighten" | "ColorDodge"
            | "ColorBurn" | "HardLight" | "SoftLight" => Self::SeparableWhitePreserving,
            "Difference" | "Exclusion" => Self::SeparableNonWhitePreserving,
            // ISO 32000-1 §11.3.5.3: four non-separable modes.
            "Hue" | "Saturation" | "Color" | "Luminosity" => Self::NonSeparable,
            // §11.6.3 fallback: unknown names render as /Normal.
            _ => Self::SeparableWhitePreserving,
        }
    }

    /// Process-lane dispatch decision. Always
    /// [`ProcessBlendDispatch::UseRequested`] per §11.7.4.2: "the
    /// current blend mode parameter … shall always apply to process
    /// colorants".
    pub fn process_dispatch(&self) -> ProcessBlendDispatch {
        ProcessBlendDispatch::UseRequested
    }

    /// Spot-lane dispatch decision per §11.7.4.2.
    pub fn spot_dispatch(&self) -> SpotBlendDispatch {
        match self {
            Self::SeparableWhitePreserving => SpotBlendDispatch::UseRequested,
            Self::SeparableNonWhitePreserving | Self::NonSeparable => {
                SpotBlendDispatch::SubstituteNormal
            },
        }
    }
}

// `spot_names` and the spot tint planes are populated by the
// discovery pre-pass at page setup; the per-paint operator writes
// land in round 2. Round 1 only exposes them through the
// `test-support` feature accessors on `PageRenderer`, so without
// `test-support` the fields and the readers are dead.
//
// We allow `dead_code` on the impl rather than `#[cfg(feature = ...)]`
// on each method because round 2 will wire these into the renderer's
// hot path unconditionally; gating them on `test-support` now would
// just be churn to undo.
#[allow(dead_code)]
/// Per-page CMYK + spot-ink compositing sidecar.
///
/// Allocated once at the top of [`super::PageRenderer::render_page_with_options`]
/// when the page declares a CMYK `OutputIntent` and any
/// transparency / overprint trigger. The sidecar lives until the page
/// finishes rendering, then is dropped.
///
/// The CMYK plane is the §11.4 compositing buffer for the four
/// process channels (`DeviceCMYK` blend space). The spot planes are
/// the §11.7.3 sidecar — one byte per pixel per ink, blended
/// independently of the process channels.
///
/// Round 1 introduces the spot-plane storage and the page-level
/// discovery pre-pass; round 2 will wire per-paint-op writes from
/// `Separation` / `DeviceN` paint operators into the spot lanes.
#[derive(Debug)]
pub(crate) struct CmykSidecar {
    /// Pixmap dimensions `(width, height)`. Captured at allocation
    /// time and used for spot-plane indexing.
    dims: (u32, u32),
    /// Packed 4-byte-per-pixel `DeviceCMYK` plane in `(C, M, Y, K)`
    /// order, row-major, top-left origin. Length is `4 · w · h`.
    /// This is the round-4 layout preserved byte-for-byte so every
    /// existing process-lane helper continues to work unchanged.
    cmyk: Vec<u8>,
    /// Ordered names of every discovered spot ink. Order matches the
    /// `spots` plane stack: `spot_names[i]` is the colorant name of
    /// the plane at `spots[i·w·h .. (i+1)·w·h]`. Populated by the
    /// pre-pass via [`PdfDocument::get_page_inks_deep`] which sorts
    /// ASCII and dedups; `/All` and `/None` are filtered out by that
    /// helper per §8.6.6.4.
    spot_names: Vec<String>,
    /// Stack of per-ink tint planes. Length is `spot_names.len() · w
    /// · h`. Plane `i` lives at `spots[i·w·h .. (i+1)·w·h]`, one byte
    /// per pixel (0 = no ink, 255 = full tint). Initialised to zero
    /// per §11.7.3 ("an additive value of 1.0 or a subtractive tint
    /// value of 0.0 shall be assumed" for an unset component).
    spots: Vec<u8>,
}

#[allow(dead_code)]
impl CmykSidecar {
    /// Allocate the sidecar for a page of `(width, height)` pixels
    /// and the given set of spot ink names.
    ///
    /// The CMYK plane and every spot plane initialise to zero — the
    /// §11.7.3 subtractive resting state. The caller is responsible
    /// for driving the per-paint mirrors that update both the CMYK
    /// and spot lanes as the content stream renders.
    pub(crate) fn new(width: u32, height: u32, spot_names: Vec<String>) -> Self {
        let pixels = (width as usize) * (height as usize);
        let cmyk = vec![0u8; 4 * pixels];
        let spots = vec![0u8; spot_names.len() * pixels];
        Self {
            dims: (width, height),
            cmyk,
            spot_names,
            spots,
        }
    }

    /// Pixmap dimensions in `(width, height)` order.
    pub(crate) fn dims(&self) -> (u32, u32) {
        self.dims
    }

    /// Read-only slice over the packed `(C, M, Y, K)` plane.
    pub(crate) fn cmyk(&self) -> &[u8] {
        &self.cmyk
    }

    /// Mutable slice over the packed `(C, M, Y, K)` plane.
    pub(crate) fn cmyk_mut(&mut self) -> &mut [u8] {
        &mut self.cmyk
    }

    /// Ordered list of spot ink names. Empty when the page declares
    /// no `Separation` / non-process `DeviceN` colorants.
    pub(crate) fn spot_names(&self) -> &[String] {
        &self.spot_names
    }

    /// Read-only slice over the tint plane for spot ink `index`.
    /// Returns `None` when `index >= spot_count()`.
    pub(crate) fn spot_plane(&self, index: usize) -> Option<&[u8]> {
        let (w, h) = self.dims;
        let plane_size = (w as usize) * (h as usize);
        let start = index.checked_mul(plane_size)?;
        let end = start.checked_add(plane_size)?;
        if end > self.spots.len() {
            return None;
        }
        Some(&self.spots[start..end])
    }
}

/// Discover the set of `/Separation` and `/DeviceN` spot colorants
/// declared on `page_index` and within any nested Form XObject
/// `/Resources/ColorSpace` reached through `Do` operators in the
/// page's content stream.
///
/// Round 1 wraps [`PdfDocument::get_page_inks_deep`] so the sidecar's
/// spot set matches the spot set the separation renderer's per-plate
/// path already allocates. The walker filters `/All` and `/None` per
/// §8.6.6.4, sorts ASCII, and dedups. The result is stable across
/// renders of the same page.
///
/// Returns an empty vector when the page declares no spot colorants
/// (including the common case of a CMYK-only press job whose only
/// inks are the four process colorants Cyan / Magenta / Yellow /
/// Black). The four process inks are NOT surfaced here — they live
/// on the CMYK plane, not in the spot list.
///
/// # Error handling
///
/// On a parse error, malformed colorant array, or recursion-bound
/// trip from [`PdfDocument::get_page_inks_deep`], this function emits
/// a `log::warn!` naming the page and the underlying error, then
/// returns an empty vector. The render continues with degraded spot
/// fidelity (the sidecar allocates a zero-length spot stack and any
/// downstream paint-op writes that target spot lanes will find no
/// lane to write to — i.e. the spot ink quietly drops out of the
/// composite). This matches how the separation renderer handles the
/// same error (its per-plate path also degrades on a malformed
/// resource tree). The warning is the diagnostic signal that lets the
/// caller see the silent fidelity loss in a log scrape.
pub(crate) fn discover_page_spot_inks(doc: &PdfDocument, page_index: usize) -> Vec<String> {
    // get_page_inks_deep already enforces the §8.6.6.4 rules: filters
    // /All and /None, dedups, sorts. On error, surface via log::warn
    // so the silent-degradation is visible to the host application's
    // log pipeline — a silent unwrap_or_default would let the spot
    // lanes drop out of the composite without any signal.
    match doc.get_page_inks_deep(page_index) {
        Ok(inks) => inks,
        Err(e) => {
            log::warn!(
                "sidecar: failed to discover spot inks for page {}: {}; the \
                 transparency composite will proceed with no spot lanes",
                page_index,
                e
            );
            Vec::new()
        },
    }
}

/// Conservative detection: does this page declare any resource that
/// could drive transparency or overprint? Returns `true` when the
/// sidecar should be allocated for the page.
///
/// Detection criteria (matches the round-4 pre-pass):
///
///   * Any `ExtGState` in `/Resources/ExtGState` declares one of:
///     - `/OP true` or `/op true` (overprint)
///     - `/CA < 1.0` or `/ca < 1.0` (transparent paint)
///     - `/SMask` non-null (soft mask)
///     - `/BM` non-Normal (non-trivial blend mode)
///   * Any Form XObject in `/Resources/XObject` declares a `/Group`
///     dict (transparency group) or carries an `/SMask` entry.
///
/// The detection-OFF path is byte-identical to a sidecar-less render
/// because the sidecar-consuming helpers fall back to additive-clamp
/// inversion when the sidecar is `None`.
pub(crate) fn page_declares_transparency_or_overprint(
    doc: &PdfDocument,
    resources: &Object,
) -> bool {
    let res_dict = match resources {
        Object::Dictionary(d) => d,
        _ => return false,
    };

    if let Some(ext_gs_obj) = res_dict.get("ExtGState") {
        if let Ok(ext_gs_resolved) = doc.resolve_object(ext_gs_obj) {
            if let Some(ext_g_states) = ext_gs_resolved.as_dict() {
                if ext_g_states_signal_transparency(doc, ext_g_states) {
                    return true;
                }
            }
        }
    }

    if let Some(xobj_obj) = res_dict.get("XObject") {
        if let Ok(xobj_resolved) = doc.resolve_object(xobj_obj) {
            if let Some(xobj_dict) = xobj_resolved.as_dict() {
                for obj in xobj_dict.values() {
                    if let Ok(resolved) = doc.resolve_object(obj) {
                        let dict = match &resolved {
                            Object::Stream { dict, .. } => Some(dict),
                            _ => None,
                        };
                        if let Some(dict) = dict {
                            if dict.contains_key("Group") || dict.contains_key("SMask") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

fn ext_g_states_signal_transparency(
    doc: &PdfDocument,
    ext_g_states: &HashMap<String, Object>,
) -> bool {
    for state in ext_g_states.values() {
        let state_resolved = match doc.resolve_object(state) {
            Ok(o) => o,
            Err(_) => continue,
        };
        let Some(state_dict) = state_resolved.as_dict() else {
            continue;
        };
        if state_dict
            .get("OP")
            .map(|o| matches!(o, Object::Boolean(true)))
            .unwrap_or(false)
            || state_dict
                .get("op")
                .map(|o| matches!(o, Object::Boolean(true)))
                .unwrap_or(false)
        {
            return true;
        }
        for key in ["CA", "ca"] {
            if let Some(v) = state_dict.get(key) {
                let alpha = match v {
                    Object::Real(r) => *r as f32,
                    Object::Integer(i) => *i as f32,
                    _ => 1.0,
                };
                if alpha < 1.0 {
                    return true;
                }
            }
        }
        if let Some(smask) = state_dict.get("SMask") {
            if !matches!(smask, Object::Name(n) if n == "None") {
                return true;
            }
        }
        // ISO 32000-1 §11.3.5 + §11.6.3: `/BM` may be a name OR an
        // array of names. For an array, "the first name that names a
        // blend mode supported by the conforming reader shall be used".
        // An unrecognised name maps to /Normal per §11.6.3. Walk both
        // shapes; fire the detection trigger only when the resolved
        // mode is non-/Normal.
        if let Some(bm) = state_dict.get("BM") {
            if bm_is_non_normal(bm) {
                return true;
            }
        }
    }
    false
}

/// Resolve a `/BM` entry to "is this a recognised non-Normal blend
/// mode?". Handles both the name and array forms per §11.3.5 +
/// §11.6.3: the array form picks the FIRST recognised name; the name
/// form is classified directly. Unrecognised names fall through to
/// /Normal per the §11.6.3 fallback.
fn bm_is_non_normal(bm: &Object) -> bool {
    match bm {
        Object::Name(name) => is_non_normal_mode(name),
        Object::Array(arr) => arr
            .iter()
            .filter_map(Object::as_name)
            .find(|name| is_recognised_mode(name))
            .map(is_non_normal_mode)
            .unwrap_or(false),
        _ => false,
    }
}

/// True when `name` is one of the standard blend-mode names ISO 32000-1
/// §11.3.5 enumerates (separable §11.3.5.2 or non-separable §11.3.5.3).
/// `/Normal` counts as recognised. Unknown names are NOT recognised and
/// trigger the §11.6.3 fallback at the call site.
fn is_recognised_mode(name: &str) -> bool {
    matches!(
        name,
        "Normal"
            | "Multiply"
            | "Screen"
            | "Overlay"
            | "Darken"
            | "Lighten"
            | "ColorDodge"
            | "ColorBurn"
            | "HardLight"
            | "SoftLight"
            | "Difference"
            | "Exclusion"
            | "Hue"
            | "Saturation"
            | "Color"
            | "Luminosity"
    )
}

/// True when `name` is a recognised non-/Normal blend mode. The
/// transparency trigger fires only on this set.
fn is_non_normal_mode(name: &str) -> bool {
    is_recognised_mode(name) && name != "Normal"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_normal_is_separable_white_preserving() {
        assert_eq!(BlendModeClass::from_name("Normal"), BlendModeClass::SeparableWhitePreserving);
    }

    #[test]
    fn classify_luminosity_is_non_separable() {
        assert_eq!(BlendModeClass::from_name("Luminosity"), BlendModeClass::NonSeparable);
    }

    #[test]
    fn classify_difference_is_separable_non_white_preserving() {
        assert_eq!(
            BlendModeClass::from_name("Difference"),
            BlendModeClass::SeparableNonWhitePreserving
        );
    }

    #[test]
    fn classify_unknown_falls_back_to_normal_class() {
        // ISO 32000-1 §11.6.3: unknown blend mode names render as
        // /Normal. The classifier reflects that by returning the same
        // class /Normal itself belongs to.
        assert_eq!(
            BlendModeClass::from_name("MarketingInventedMode"),
            BlendModeClass::SeparableWhitePreserving
        );
    }

    #[test]
    fn spot_dispatch_substitutes_normal_for_non_sep_and_non_wp() {
        // §11.7.4.2: only separable AND white-preserving modes apply
        // to spot lanes; every other class substitutes /Normal.
        assert_eq!(
            BlendModeClass::SeparableWhitePreserving.spot_dispatch(),
            SpotBlendDispatch::UseRequested
        );
        assert_eq!(
            BlendModeClass::SeparableNonWhitePreserving.spot_dispatch(),
            SpotBlendDispatch::SubstituteNormal
        );
        assert_eq!(
            BlendModeClass::NonSeparable.spot_dispatch(),
            SpotBlendDispatch::SubstituteNormal
        );
    }

    #[test]
    fn process_dispatch_is_identity_for_every_class() {
        // §11.7.4.2: process lanes always honour the requested BM.
        for class in &[
            BlendModeClass::SeparableWhitePreserving,
            BlendModeClass::SeparableNonWhitePreserving,
            BlendModeClass::NonSeparable,
        ] {
            assert_eq!(class.process_dispatch(), ProcessBlendDispatch::UseRequested);
        }
    }

    #[test]
    fn sidecar_allocates_cmyk_and_spot_planes() {
        let s = CmykSidecar::new(10, 5, vec!["PMS 185 C".into(), "Dieline".into()]);
        assert_eq!(s.dims(), (10, 5));
        assert_eq!(s.cmyk().len(), 4 * 10 * 5);
        assert!(s.cmyk().iter().all(|&b| b == 0));
        assert_eq!(s.spot_names(), &["PMS 185 C".to_string(), "Dieline".to_string()]);
        let p0 = s.spot_plane(0).unwrap();
        let p1 = s.spot_plane(1).unwrap();
        assert_eq!(p0.len(), 10 * 5);
        assert_eq!(p1.len(), 10 * 5);
        assert!(p0.iter().all(|&b| b == 0) && p1.iter().all(|&b| b == 0));
        assert!(s.spot_plane(2).is_none());
    }

    #[test]
    fn sidecar_no_spots_has_zero_length_spot_stack() {
        let s = CmykSidecar::new(7, 3, vec![]);
        assert_eq!(s.dims(), (7, 3));
        assert_eq!(s.cmyk().len(), 4 * 7 * 3);
        assert!(s.spot_names().is_empty());
        assert!(s.spot_plane(0).is_none());
    }

    /// A test-only `log::Log` that captures every record into a
    /// shared buffer. Lets the discover-error probe assert "warn!
    /// emitted the expected diagnostic" without pulling in a test
    /// crate. `log::set_boxed_logger` is idempotent once-only, so the
    /// installation is gated on `OnceLock`.
    struct CapturingLogger {
        buf: std::sync::Mutex<Vec<String>>,
    }
    impl log::Log for CapturingLogger {
        fn enabled(&self, m: &log::Metadata) -> bool {
            m.level() <= log::Level::Warn
        }
        fn log(&self, record: &log::Record) {
            if self.enabled(record.metadata()) {
                let mut g = self.buf.lock().unwrap();
                g.push(format!("{}", record.args()));
            }
        }
        fn flush(&self) {}
    }
    static CAPTURING_LOGGER: std::sync::OnceLock<&'static CapturingLogger> =
        std::sync::OnceLock::new();
    fn install_capturing_logger() -> &'static CapturingLogger {
        CAPTURING_LOGGER.get_or_init(|| {
            let leaked: &'static CapturingLogger = Box::leak(Box::new(CapturingLogger {
                buf: std::sync::Mutex::new(Vec::new()),
            }));
            // Tolerate prior installation (other tests may install their own
            // logger first). If installation fails, the buffer stays empty
            // and the probe will fail loudly with a clear message.
            let _ = log::set_logger(leaked);
            log::set_max_level(log::LevelFilter::Warn);
            leaked
        })
    }

    /// Round-1 QA — surface, don't swallow, the deep-walk error.
    ///
    /// `discover_page_spot_inks` previously called
    /// `get_page_inks_deep(...).unwrap_or_default()`, silently mapping
    /// every error to an empty vec. A page that genuinely has spots
    /// but whose deep walk trips (parse error, recursion bound, page
    /// lookup miss) would then allocate a zero-length spot stack — and
    /// any downstream paint-op writes to those lanes would quietly
    /// drop on the floor.
    ///
    /// The fix emits `log::warn!` on the error path AND returns the
    /// empty vec (matching how the separation renderer handles the
    /// same `get_page_inks_deep` failure). This probe pins both halves
    /// of the contract: empty-vec return, AND a warn record surfaces.
    #[test]
    fn discover_page_spot_inks_warns_on_deep_walk_error() {
        let logger = install_capturing_logger();
        // Snapshot any prior records so we only inspect ours.
        let start_len = logger.buf.lock().unwrap().len();

        // Single-page synthetic PDF. We will then ask for page 42 — out
        // of range — so `get_page_inks_deep` returns Err on the page
        // tree walk.
        let pdf = b"%PDF-1.4\n\
                    1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
                    2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\
                    3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 10 10] >>\nendobj\n\
                    xref\n0 4\n\
                    0000000000 65535 f \n\
                    0000000010 00000 n \n\
                    0000000059 00000 n \n\
                    0000000110 00000 n \n\
                    trailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n175\n%%EOF\n"
            .to_vec();
        let doc = PdfDocument::from_bytes(pdf).expect("synthetic PDF parses");

        let spots = discover_page_spot_inks(&doc, 42);
        assert!(
            spots.is_empty(),
            "discover_page_spot_inks must return an empty vec on \
             deep-walk error (not panic, not propagate); got {:?}",
            spots
        );

        // The warning message names the page index and includes the
        // word "spot inks" so a log scrape can find it.
        let new_records: Vec<String> = {
            let guard = logger.buf.lock().unwrap();
            guard[start_len..].to_vec()
        };
        let saw_warning = new_records
            .iter()
            .any(|m| m.contains("page 42") && m.contains("spot inks"));
        assert!(
            saw_warning,
            "expected log::warn! naming page 42 and 'spot inks' on the \
             deep-walk error path; captured records since start: {:?}",
            new_records
        );
    }
}
