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
