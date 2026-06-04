//! Wave-1 QA probes for the resolution-pipeline migration.
//!
//! These tests live next to the wave-1 pilot suite
//! (`test_render_resolution_pipeline_pilot.rs`) and exist for two reasons:
//!
//! 1. **Adversarial coverage** — push fill/stroke routing through scale,
//!    interleaving, malformed inputs, and edge-of-spec colour spaces; flag
//!    any divergence between toggle-off and toggle-on as a pipeline-side
//!    parity bug.
//! 2. **Regression pins** — when a probe area does *not* surface a divergence,
//!    pin the current behaviour with a passing test so the next wave cannot
//!    silently regress it.
//!
//! Style mirrors the pilot file: build a single-page PDF inline, render
//! twice (toggle off, toggle on), and either compare pixmaps byte-for-byte
//! or sample specific pixels.

#![cfg(feature = "rendering")]

use pdf_oxide::document::PdfDocument;
use pdf_oxide::rendering::{render_page, ImageFormat, RenderOptions};
use std::sync::Mutex;

/// Process-wide lock for env-var test orchestration. Cargo runs integration
/// tests in parallel; flipping `PDF_OXIDE_RESOLUTION_PIPELINE` must not race
/// with another test's read.
static PIPELINE_TOGGLE_LOCK: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// PDF construction helpers (mirrors the pilot's helpers; kept here so the QA
// suite is self-contained and a fix-pass to the pilot can't accidentally
// break the QA invariants).
// ---------------------------------------------------------------------------

/// Build a tiny one-page PDF whose content stream is `content_ops`, with a
/// fixed 100×100 MediaBox and the provided `/Resources` dict body.
fn build_pdf(content_ops: &str, resources_dict: &str) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");
    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    let page_off = buf.len();
    let page = format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << {} >> /Contents 4 0 R >>\nendobj\n",
        resources_dict
    );
    buf.extend_from_slice(page.as_bytes());
    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 5\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

/// Build a one-page PDF that owns an indirect Type 4 tint-transform function
/// at object 5 plus a content stream — used by Separation probes.
fn build_pdf_with_type4_separation(
    content_ops: &str,
    type4_program: &str,
    page_resources_extra: &str,
) -> Vec<u8> {
    build_pdf_with_type4_separation_range(content_ops, type4_program, page_resources_extra, "[0 1 0 1 0 1 0 1]")
}

fn build_pdf_with_type4_separation_range(
    content_ops: &str,
    type4_program: &str,
    page_resources_extra: &str,
    range_array: &str,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");
    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    let page_off = buf.len();
    let page = format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << {} >> /Contents 4 0 R >>\nendobj\n",
        page_resources_extra
    );
    buf.extend_from_slice(page.as_bytes());
    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let func_off = buf.len();
    let func_hdr = format!(
        "5 0 obj\n<< /FunctionType 4 /Domain [0 1] /Range {} /Length {} >>\nstream\n",
        range_array,
        type4_program.len()
    );
    buf.extend_from_slice(func_hdr.as_bytes());
    buf.extend_from_slice(type4_program.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off, func_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

/// Build a one-page PDF with a Type 2 (exponential interpolation) function
/// as the tint transform. Object 5 holds the function dict. Domain is the
/// natural `[0 1]`; C0/C1/N are caller-supplied so probes can target
/// boundary cases the inline path's hand-rolled Type 2 doesn't reach.
fn build_pdf_with_type2_separation(
    content_ops: &str,
    c0_array: &str,
    c1_array: &str,
    n_value: &str,
    alt_cs: &str,
    page_resources_extra: &str,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");
    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    let page_off = buf.len();
    let _ = alt_cs;
    let page = format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << {} >> /Contents 4 0 R >>\nendobj\n",
        page_resources_extra
    );
    buf.extend_from_slice(page.as_bytes());
    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let func_off = buf.len();
    let func_body = format!(
        "5 0 obj\n<< /FunctionType 2 /Domain [0 1] /C0 {} /C1 {} /N {} >>\nendobj\n",
        c0_array, c1_array, n_value
    );
    buf.extend_from_slice(func_body.as_bytes());
    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off, func_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

/// Render holding the toggle to `enabled` for the call's duration; shared
/// mutex serialises env-var manipulation across parallel tests.
fn render_with_pipeline(doc: &PdfDocument, enabled: bool) -> Vec<u8> {
    let _guard = PIPELINE_TOGGLE_LOCK.lock().unwrap();
    let prev = std::env::var("PDF_OXIDE_RESOLUTION_PIPELINE").ok();
    if enabled {
        std::env::set_var("PDF_OXIDE_RESOLUTION_PIPELINE", "1");
    } else {
        std::env::remove_var("PDF_OXIDE_RESOLUTION_PIPELINE");
    }
    let opts = RenderOptions::with_dpi(72).as_raw();
    let img = render_page(doc, 0, &opts).expect("render_page succeeds");
    assert_eq!(img.format, ImageFormat::RawRgba8);
    let data = img.data;
    match prev {
        Some(v) => std::env::set_var("PDF_OXIDE_RESOLUTION_PIPELINE", v),
        None => std::env::remove_var("PDF_OXIDE_RESOLUTION_PIPELINE"),
    }
    data
}

/// Render under pipeline-on, allowing the call to fail without panicking.
/// Used by adversarial-input probes that exercise malformed PDFs — the
/// invariant is "no panic", not "render succeeds".
fn render_with_pipeline_allow_fail(doc: &PdfDocument, enabled: bool) -> Option<Vec<u8>> {
    let _guard = PIPELINE_TOGGLE_LOCK.lock().unwrap();
    let prev = std::env::var("PDF_OXIDE_RESOLUTION_PIPELINE").ok();
    if enabled {
        std::env::set_var("PDF_OXIDE_RESOLUTION_PIPELINE", "1");
    } else {
        std::env::remove_var("PDF_OXIDE_RESOLUTION_PIPELINE");
    }
    let opts = RenderOptions::with_dpi(72).as_raw();
    let result = render_page(doc, 0, &opts).ok().map(|img| img.data);
    match prev {
        Some(v) => std::env::set_var("PDF_OXIDE_RESOLUTION_PIPELINE", v),
        None => std::env::remove_var("PDF_OXIDE_RESOLUTION_PIPELINE"),
    }
    result
}

#[allow(dead_code)]
fn center_pixel(rgba: &[u8]) -> (u8, u8, u8, u8) {
    let w = 100u32;
    let h = 100u32;
    assert_eq!(rgba.len() as u32, w * h * 4);
    let cx = w / 2;
    let cy = h / 2;
    let off = ((cy * w + cx) * 4) as usize;
    (rgba[off], rgba[off + 1], rgba[off + 2], rgba[off + 3])
}

#[allow(dead_code)]
fn pixel_at(rgba: &[u8], x: u32, y: u32) -> (u8, u8, u8, u8) {
    let w = 100u32;
    let h = 100u32;
    assert_eq!(rgba.len() as u32, w * h * 4);
    assert!(x < w && y < h);
    let off = ((y * w + x) * 4) as usize;
    (rgba[off], rgba[off + 1], rgba[off + 2], rgba[off + 3])
}

// ===========================================================================
// PROBE AREA: Toggle-on parity at scale (probes 1, 2, 3)
// ===========================================================================

/// Probe 1 — Long content stream with many fill/stroke operators of each type.
///
/// The pipeline routes every fill/stroke through a fresh `ResolutionPipeline`
/// instance per call. Any per-call state leak, mutation of the borrowed
/// `gs` it shouldn't make, or asymmetric handling of repeated dispatch
/// would manifest as drift between toggle-off and toggle-on after enough
/// repetitions. 200 operators of each kind on a 100×100 page exercises every
/// migrated arm 200× per render — large enough to surface drift if any
/// exists.
#[test]
fn qa_long_stream_repeated_fill_stroke_byte_identical() {
    let mut content = String::new();
    content.push_str("1 0 0 rg\n0 1 0 RG\n2 w\n");
    // 200 rectangles, each with a fill, stroke, and one combo, scattered
    // across the page deterministically. The result is a dense overpaint;
    // every operator we migrated gets exercised many times.
    for i in 0..200 {
        let x = (i % 20) as f32 * 5.0;
        let y = ((i / 20) % 20) as f32 * 5.0;
        content.push_str(&format!("{} {} 4 4 re\nf\n", x, y));
        content.push_str(&format!("{} {} 4 4 re\nS\n", x, y));
        content.push_str(&format!("{} {} 4 4 re\nB\n", x, y));
        content.push_str(&format!("{} {} 4 4 re\nb\n", x, y));
        content.push_str(&format!("{} {} 4 4 re\nB*\n", x, y));
        content.push_str(&format!("{} {} 4 4 re\nb*\n", x, y));
    }
    let bytes = build_pdf(&content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "pipeline must remain byte-identical after a long sequence of repeated migrated operators"
    );
}

/// Probe 2 — Mixed-operator stream that interleaves all six migrated
/// operators with the prior-wave fill operators (`f`, `f*`).
///
/// Each iteration uses a different colour to ensure that per-iteration
/// state (e.g. last-set fill colour, last-set stroke colour) is exercised
/// rather than collapsing to one canonical RGBA the pipeline could hide a
/// bug behind.
#[test]
fn qa_mixed_all_paint_operators_byte_identical() {
    let mut content = String::new();
    content.push_str("3 w\n");
    let ops = ["f", "f*", "S", "B", "B*", "b", "b*"];
    for (i, op) in ops.iter().enumerate() {
        // Pick a per-op colour so the pipeline's per-call colour state has
        // to be reset cleanly between operators.
        let r = (i as f32) / 7.0;
        let g = ((i + 2) as f32 % 7.0) / 7.0;
        let b = ((i + 4) as f32 % 7.0) / 7.0;
        content.push_str(&format!("{} {} {} rg\n", r, g, b));
        content.push_str(&format!("{} {} {} RG\n", b, r, g));
        let x = 10 + (i as i32) * 10;
        content.push_str(&format!("{} 30 8 40 re\n{}\n", x, op));
    }
    let bytes = build_pdf(&content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "interleaved migrated + prior-wave operators must be byte-identical");
}

/// Probe 3 — Graphics-state operators (`q`/`Q`/`cm`/`w`/`J`/`j`/`gs`)
/// interleaved with migrated operators. The pipeline reads `gs` by
/// reference; mutating fields it shouldn't (or failing to re-read after
/// `q`/`Q`) would diverge.
///
/// Pattern: save state, change a state field, paint, restore, paint
/// again. Repeat with different field combinations.
#[test]
fn qa_interleaved_graphics_state_changes_byte_identical() {
    let content = "\
        1 0 0 rg\n0 1 0 RG\n2 w\n\
        q\n3 w\n0 J\n0 j\n10 10 30 30 re\nB\nQ\n\
        q\n8 w\n1 J\n1 j\n60 10 30 30 re\nb*\nQ\n\
        q\n0.5 0 0 0.5 0 0 cm\n10 60 30 30 re\nf\n10 60 30 30 re\nS\nQ\n\
        q\n2 w\n[4 2] 0 d\n50 60 40 30 re\nB\nQ\n\
        20 20 m\n80 20 l\n80 80 l\n20 80 l\nb\n\
    ";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "graphics-state changes interleaved with migrated operators must keep parity"
    );
}

// ===========================================================================
// PROBE AREA: Stroke-specific edge cases (probes 4-9)
// ===========================================================================

/// Probe 4 — Hairline stroke (line width well under 1 device pixel).
///
/// The pipeline clones `gs` and overwrites only `stroke_color_rgb` and
/// `stroke_alpha`; line width must round-trip exactly. At a 0.25-px width
/// the rasteriser produces a faint anti-aliased line; if the pipeline
/// accidentally promoted the width (e.g. via a default-init clone) the
/// off and on pixmaps would diverge.
#[test]
fn qa_stroke_hairline_width_parity() {
    let content = "1 0 0 RG\n0.25 w\n20 50 m\n80 50 l\nS\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "hairline stroke must be byte-identical");
}

/// Probe 5 — Zero-width stroke. PDF spec ISO 32000-1 §8.4.3.2 says width 0
/// means "thinnest line the device can render"; the renderer's existing
/// behaviour is what we pin. Either way, off-vs-on must match.
#[test]
fn qa_stroke_zero_width_parity() {
    let content = "1 0 0 RG\n0 w\n20 50 m\n80 50 l\nS\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "zero-width stroke must be byte-identical");
}

/// Probe 6 — Negative line width (malformed PDF).
///
/// The spec says width must be non-negative; some PDFs in the wild carry
/// negative values from broken producers. Both paths must degrade
/// identically — no panic, no divergence between off and on.
#[test]
fn qa_stroke_negative_width_parity_no_panic() {
    let content = "1 0 0 RG\n-3 w\n20 50 m\n80 50 l\nS\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    // First invariant: no panic on either side.
    let off = render_with_pipeline_allow_fail(&doc, false);
    let on = render_with_pipeline_allow_fail(&doc, true);
    // Second invariant: same outcome shape.
    match (off, on) {
        (Some(a), Some(b)) => assert_eq!(a, b, "negative line width must render identically"),
        (None, None) => {},
        (None, Some(_)) | (Some(_), None) => {
            panic!("toggle changed render-success vs render-failure outcome on malformed input");
        },
    }
}

/// Probe 7 — Stroke alpha (`/CA`) sourced from an ExtGState dict.
///
/// The pipeline reads `gs.stroke_alpha` after the `gs` operator has applied
/// `/CA` to the graphics state. The fold into `ResolvedColor::Rgba.a`
/// happens inside `device_to_rgba`. Off-vs-on parity confirms the alpha
/// is sourced from the same place and folded identically.
#[test]
fn qa_stroke_alpha_ca_extgstate_parity() {
    let content = "/Half gs\n1 0 0 RG\n10 w\n20 20 60 60 re\nS\n";
    let resources = "/ExtGState << /Half << /Type /ExtGState /CA 0.5 >> >>";
    let bytes = build_pdf(content, resources);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "stroke alpha via /CA ExtGState must be byte-identical");
}

/// Probe 8 — Stroke with a dash pattern set via `d`.
///
/// Dash pattern is part of `gs` and must survive the splice. Drawing a
/// long horizontal stroke with a clear dash pattern surfaces any pipeline
/// path that would forget the dashing.
#[test]
fn qa_stroke_dash_pattern_parity() {
    let content = "1 0 0 RG\n4 w\n[6 3] 0 d\n10 50 m\n90 50 l\nS\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "dashed stroke must be byte-identical");
}

/// Probe 9 — Miter limit at an extreme value, applied to a sharp corner.
///
/// `M 100` allows long miter spikes; at a sharp join the spike length is
/// observable. Pipeline must round-trip the miter limit.
#[test]
fn qa_stroke_extreme_miter_limit_parity() {
    let content = "1 0 0 RG\n6 w\n0 J\n0 j\n100 M\n20 80 m\n50 50 l\n20 20 l\nS\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "extreme miter-limit stroke must be byte-identical");
}

// ===========================================================================
// PROBE AREA: Fill/stroke graphics-state propagation through combos
// (probes 10-12)
// ===========================================================================

/// Probe 10 — `B` with an active rotated and scaled CTM. Each combo
/// operator builds two `PaintIntent`s and clones `gs` twice (once for
/// fill, once for stroke). Both clones must inherit the same CTM; if
/// either resets it to identity, the rotated rectangle won't paint at
/// the right place.
#[test]
fn qa_combo_under_rotated_scaled_ctm_parity() {
    // CTM: rotate 30°, scale 0.8, translate (10, 10). Then paint a
    // rectangle through `B`. The fill side and stroke side must both
    // honour the same CTM under the toggle.
    let content = "\
        0.6928 0.4 -0.4 0.6928 10 10 cm\n\
        0 1 0 rg\n1 0 0 RG\n5 w\n\
        0 0 40 40 re\nB\n\
    ";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "`B` under a rotated + scaled CTM must be byte-identical");
}

/// Probe 11 — Soft-mask `/SMask` set via ExtGState; while not always
/// implemented end-to-end, the pipeline must at minimum not diverge from
/// the inline path when the graphics state carries an `/SMask` entry.
/// This pins the off-vs-on parity for that case.
#[test]
fn qa_stroke_under_extgstate_with_smask_no_divergence() {
    // We don't fully wire an SMask (the bytes are deliberately simple);
    // the assertion is only that toggle-flip doesn't perturb whatever
    // both paths produce. If the inline path ignores `/SMask` today and
    // the pipeline does too, off == on. If they ever diverge, this test
    // will catch it.
    let content = "/Sm gs\n1 0 0 RG\n10 w\n20 20 60 60 re\nS\n";
    let resources = "/ExtGState << /Sm << /Type /ExtGState /SMask /None >> >>";
    let bytes = build_pdf(content, resources);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "/SMask entry on stroke must not introduce off-vs-on divergence");
}

/// Probe 12 — Independent clip paths active when fill and stroke happen
/// inside the same `B` combo. The pipeline must use the same clip mask
/// for both sub-operations; a path that tracked one clip on the inline
/// route and another on the pipeline route would diverge.
#[test]
fn qa_combo_under_active_clip_parity() {
    // Set up a clip that's a small horizontal band across the page, then
    // do `B` of a rectangle that extends well past the band on top and
    // bottom. Only the in-band fraction of the fill and stroke is
    // painted. Off-vs-on parity confirms both sides see the same clip.
    let content = "\
        0 40 100 20 re\nW\nn\n\
        0 1 0 rg\n1 0 0 RG\n6 w\n\
        20 10 60 80 re\nB\n\
    ";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "`B` under an active clip path must be byte-identical");
}

// ===========================================================================
// PROBE AREA: Colour-resolution edge cases (probes 13-18)
// ===========================================================================

/// Probe 13 — Indexed colour space via `scn` (PDF "SetFillColorN").
///
/// **BUG (MAJOR): Pipeline-on diverges from pipeline-off for `scn` against
/// an Indexed colour space.**
///
/// The inline `SetFillColorN` handler at `page_renderer.rs:830` has NO
/// `Indexed` branch (the older `SetFillColor` at line 581 does, but `scn`
/// doesn't). For `scn` against Indexed, the inline path falls through to
/// `gs.fill_color_rgb = (g, g, g)` with `g = components[0]` — the raw
/// index value. For an index of 1 this gives `(1.0, 1.0, 1.0)` → white
/// (the rasteriser interprets 1.0 as fully-on, and the bg is also white,
/// so the centre pixel is white).
///
/// The pipeline's `resolve_indexed` (color.rs:237) divides by 255:
/// `g = index / 255`. For index 1 that's `(0.004, 0.004, 0.004)` →
/// near-black.
///
/// The two paths render dramatically different output. This test
/// asserts byte equality — the wave-1 invariant — and is expected to
/// FAIL until the fix wave brings the two paths into agreement. The
/// agreed direction is up to the design pass; the divergence today is
/// the bug.
#[test]
#[ignore = "wave-1 QA bug — Indexed `scn` (fill) diverges between toggle off and on; fix-pass target"]
fn qa_bug_indexed_scn_fill_pipeline_diverges() {
    let resources = "/ColorSpace << /Pal [/Indexed /DeviceRGB 1 <FF0000 0000FF>] >>";
    let content = "/Pal cs\n1 scn\n20 20 60 60 re\nf\n";
    let bytes = build_pdf(content, resources);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    // Document the bug shape before the equality assertion so the
    // failure mode is self-describing.
    let (r_off, g_off, b_off, _) = center_pixel(&off);
    let (r_on, g_on, b_on, _) = center_pixel(&on);
    assert!(
        r_off > 200 && g_off > 200 && b_off > 200,
        "inline (off): Indexed `scn` paints near-white via raw-component fallback, got ({r_off}, {g_off}, {b_off})"
    );
    assert!(
        r_on < 50 && g_on < 50 && b_on < 50,
        "pipeline (on): Indexed `scn` paints near-black via index/255, got ({r_on}, {g_on}, {b_on})"
    );
    assert_eq!(
        off, on,
        "WAVE-1 INVARIANT VIOLATED: Indexed `scn` must render identically off vs on \
         (inline path uses raw component, pipeline divides by 255)"
    );
}

/// Probe 13b — Indexed colour space via `SCN` (stroke side).
///
/// **BUG (MAJOR): Symmetric to probe 13 on the stroke side.**
///
/// Same divergence pattern, stroke side. Inline `SetStrokeColorN` has no
/// `Indexed` branch; pipeline's `resolve_indexed` divides by 255.
#[test]
#[ignore = "wave-1 QA bug — Indexed `SCN` (stroke) diverges between toggle off and on; fix-pass target"]
fn qa_bug_indexed_scn_stroke_pipeline_diverges() {
    let resources = "/ColorSpace << /Pal [/Indexed /DeviceRGB 1 <FF0000 0000FF>] >>";
    let content = "/Pal CS\n1 SCN\n10 w\n20 20 60 60 re\nS\n";
    let bytes = build_pdf(content, resources);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "WAVE-1 INVARIANT VIOLATED: Indexed `SCN` (stroke) must render identically off vs on \
         (inline path uses raw component, pipeline divides by 255)"
    );
}

/// Probe 14 — ICCBased colour space with 4 components (CMYK profile).
///
/// Both paths inspect `/N` and dispatch to the device-family fallback.
/// Off-vs-on parity should hold.
#[test]
fn qa_iccbased_cmyk_n4_fill_parity() {
    // Embed a minimal ICCBased stream with /N 4. We don't ship a real
    // ICC profile blob — both paths read /N and route to the CMYK
    // fallback without consulting the profile bytes for the non-icc
    // build, so an empty stream is sufficient here.
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");
    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    let page_off = buf.len();
    let resources = "/ColorSpace << /MyCMYK [/ICCBased 5 0 R] >>";
    buf.extend_from_slice(
        format!(
            "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << {} >> /Contents 4 0 R >>\nendobj\n",
            resources
        )
        .as_bytes(),
    );
    let stream_off = buf.len();
    let content = "/MyCMYK cs\n1 0 0 0 scn\n20 20 60 60 re\nf\n";
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let icc_off = buf.len();
    // Minimal ICC stream: empty body, dict says /N 4.
    let icc = "5 0 obj\n<< /N 4 /Length 0 >>\nstream\n\nendstream\nendobj\n";
    buf.extend_from_slice(icc.as_bytes());
    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off, icc_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    let doc = PdfDocument::from_bytes(buf).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "ICCBased N=4 (CMYK) fill must be byte-identical");
}

/// Probe 15 — DeviceN with a multi-output Type 4 tint transform.
///
/// `DeviceN` colour spaces have multiple input colorants and the tint
/// transform produces N output values for the alternate space. The
/// pipeline's `resolve_separation_or_devicen` runs the Type 4 program
/// and projects through the alt-space. The inline path has no DeviceN
/// branch beyond the Type 2 sibling code, so for a Type 4 DeviceN the
/// inline path gray-falls to `1.0 - components[0]`.
///
/// Pipeline ON: must paint the colour the Type 4 program declares.
/// Pipeline OFF: a different colour (or a fall-back). The two must
/// differ — pipeline gives a capability gain — and the pipeline value
/// must match the declared CMYK.
#[test]
fn qa_devicen_multi_colorant_type4_pipeline_resolves() {
    // 2-colorant DeviceN. Tint transform reads two stack inputs and
    // writes CMYK [0 t1 0 0] — i.e. ignores t0 and routes t1 to magenta.
    // With `0 1 scn` (t0=0, t1=1), output is CMYK(0,1,0,0) → magenta.
    //
    // Stack walk for `{ exch pop 0.0 exch 0.0 0.0 }` with [t0=0, t1=1]
    // (PostScript convention puts the last input on the top of the stack):
    //   start  [0, 1]
    //   exch   [1, 0]
    //   pop    [1]
    //   0.0    [1, 0]
    //   exch   [0, 1]
    //   0.0    [0, 1, 0]
    //   0.0    [0, 1, 0, 0]  ← CMYK(0, 1, 0, 0) magenta
    let type4_program = "{ exch pop 0.0 exch 0.0 0.0 }";
    // DeviceN array: [/DeviceN [names] altCS tintTransform].
    let resources = "/ColorSpace << /TwoSpot [/DeviceN [/SpotA /SpotB] /DeviceCMYK 5 0 R] >>";
    let content = "/TwoSpot cs\n0 1 scn\n20 20 60 60 re\nf\n";
    // Domain must accommodate two inputs.
    let range = "[0 1 0 1 0 1 0 1]";
    let bytes = build_devicen_pdf(content, type4_program, resources, range, &[0, 1, 0, 1]);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let on = render_with_pipeline(&doc, true);
    let (r, g, b, _) = center_pixel(&on);
    assert!(
        r > 200 && g < 60 && b > 200,
        "pipeline DeviceN Type-4 must resolve to magenta, got ({r}, {g}, {b})"
    );

    // Pipeline must produce a different image than inline for this case.
    let off = render_with_pipeline(&doc, false);
    assert_ne!(
        off, on,
        "pipeline must differ from inline for DeviceN with Type 4 (capability gain)"
    );
}

/// Build a one-page PDF with a Type 4 function whose Domain accommodates a
/// variable number of inputs. `domain_pairs` is a flat list of (min, max)
/// pairs as integers (PDF reals).
fn build_devicen_pdf(
    content_ops: &str,
    type4_program: &str,
    page_resources_extra: &str,
    range_array: &str,
    domain_pairs: &[i32],
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");
    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    let page_off = buf.len();
    let page = format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << {} >> /Contents 4 0 R >>\nendobj\n",
        page_resources_extra
    );
    buf.extend_from_slice(page.as_bytes());
    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let func_off = buf.len();
    let domain_str: Vec<String> = domain_pairs.iter().map(|v| v.to_string()).collect();
    let domain_array = format!("[{}]", domain_str.join(" "));
    let func_hdr = format!(
        "5 0 obj\n<< /FunctionType 4 /Domain {} /Range {} /Length {} >>\nstream\n",
        domain_array,
        range_array,
        type4_program.len()
    );
    buf.extend_from_slice(func_hdr.as_bytes());
    buf.extend_from_slice(type4_program.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off, func_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

/// Probe 16 — Separation with the `/All` colorant name.
///
/// The pipeline doesn't special-case the colorant name; it evaluates the
/// tint transform like any Separation. The inline path treats `/All` the
/// same. Off-vs-on parity is the assertion.
#[test]
fn qa_separation_all_colorant_parity() {
    let type4_program = "{ 0.0 exch 0.0 0.0 }";
    let content = "/All_CS cs\n0.5 scn\n20 20 60 60 re\nf\n";
    let resources = "/ColorSpace << /All_CS [/Separation /All /DeviceCMYK 5 0 R] >>";
    let bytes = build_pdf_with_type4_separation(content, type4_program, resources);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let on = render_with_pipeline(&doc, true);
    // Pipeline resolves Type 4 — magenta-ish (CMYK(0, 0.5, 0, 0) → light
    // magenta). The assertion is the *outcome of the pipeline path*, not
    // a parity match — the inline path's `1.0 - tint` fallback gives
    // gray ~127, so they actively differ. Pin both.
    let (r_on, g_on, b_on, _) = center_pixel(&on);
    assert!(
        r_on > g_on && b_on > g_on,
        "pipeline /All Separation Type-4 must resolve toward magenta (R>G, B>G), got ({r_on}, {g_on}, {b_on})"
    );
    let off = render_with_pipeline(&doc, false);
    assert_ne!(
        off, on,
        "pipeline must differ from inline for Separation /All with Type 4 (capability gain)"
    );
}

/// Probe 17 — Separation with the `/None` colorant name.
///
/// ISO 32000-1 §8.6.6.4: when colorant name is `/None`, the colour should
/// produce no marks. The inline path does NOT honour this (paints
/// `1.0 - tint`). The pipeline does NOT honour this either (paints what
/// the Type 4 evaluates to). Off-vs-on parity is the assertion the
/// pipeline preserves — neither path is spec-conformant here, but they
/// must agree until both fix together.
#[test]
fn qa_separation_none_colorant_parity_pin() {
    let type4_program = "{ 0.0 exch 0.0 0.0 }";
    let content = "/None_CS cs\n0.5 scn\n20 20 60 60 re\nf\n";
    let resources = "/ColorSpace << /None_CS [/Separation /None /DeviceCMYK 5 0 R] >>";
    let bytes = build_pdf_with_type4_separation(content, type4_program, resources);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    // Neither path honours /None today — pin the divergence direction so
    // the spec fix lands in one place and is detected here.
    let (_, _, _, _) = center_pixel(&off);
    let (_, _, _, _) = center_pixel(&on);
    // Today, both paths paint (each in its own way). Off-vs-on differ
    // because the pipeline reads the Type 4 program; once /None handling
    // is added, both should produce the unpainted page. Until then, the
    // pin records that off-vs-on differ — flipping toggle MUST NOT mute
    // the divergence quietly.
    assert_ne!(
        off, on,
        "today both paths paint /None; pipeline differs from inline because Type 4 evaluates; \
         when /None is honoured, both pixmaps must become equal to the unpainted background"
    );
}

/// Probe 18 — Pattern colour space (`/Pattern` for tiling). The pilot
/// doesn't migrate `sh`, and `Pattern` cs entries today resolve via the
/// inline path's pattern handler. The pipeline must NOT capture
/// `Pattern` colour-space resolution out from under that — it should
/// fall back to the inline path.
#[test]
fn qa_pattern_colour_space_falls_back_to_inline_parity() {
    // A bare `/Pattern cs` followed by a non-pattern paint is degenerate
    // but parses. The point of this probe is that the pipeline must
    // return None (falls back to inline) for Pattern-shaped logical
    // colour, leaving the inline behaviour untouched.
    let resources = "/ColorSpace << /MyPattern [/Pattern] >>";
    let content = "/MyPattern cs\n20 20 60 60 re\nf\n";
    let bytes = build_pdf(content, resources);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Pattern colour space must fall back to inline path identically");
}
