//! Wave-2 QA probes for the resolution-pipeline migration (text operators).
//!
//! Sibling file to `test_render_resolution_pipeline_qa_wave1.rs`. The pilot
//! tests (`test_render_resolution_pipeline_pilot.rs`) already cover the
//! happy-path parity and the Type 4 fill capability gain for `Tj`. This
//! suite probes:
//!
//! 1. **Scale** — long text-heavy streams, TJ arrays with many segments,
//!    multi-font runs, mixed text + path operators. Any per-call leak or
//!    asymmetric routing surfaces as drift between toggle-off and
//!    toggle-on.
//! 2. **Mode coverage** — all 8 `Tr` modes, including the clip-adding
//!    modes (4-7) that wave-2 doesn't currently exercise in the pilot.
//! 3. **Capability gain on text** — Type 4 Separation / DeviceN / `All` /
//!    `None` colourants on text fill; the wave-1-class bug
//!    ("inline `scn` falls back to `1 - tint`") regresses to text too.
//! 4. **State preservation** — `Tc`, `Tw`, `Tz`, `TL`, `Tm`, `Td`, `TD`
//!    must not be perturbed by the spliced GS clone.
//! 5. **Font system** — CID Type 0, embedded-subset stand-in, built-in
//!    Helvetica fallback, ToUnicode-bearing fonts.
//! 6. **Operator interaction** — `Tj` inside `q/Q`, followed by `f`, under
//!    smask/blend/clip.
//! 7. **Adversarial input** — empty `()`, whitespace-only, extreme TJ
//!    offsets, all-numeric TJ array.
//! 8. **Performance** — 1000-glyph pipeline-on render must not blow up
//!    relative to a 1000-glyph pipeline-off render (one-resolve-per-Tj
//!    invariant).
//!
//! Style mirrors the wave-1 QA suite: build a tiny PDF inline, render
//! twice through `render_with_pipeline`, compare pixmaps byte-for-byte
//! or sample specific pixel regions.

#![cfg(feature = "rendering")]

use pdf_oxide::document::PdfDocument;
use pdf_oxide::rendering::{render_page, ImageFormat, RenderOptions};
use std::sync::Mutex;
use std::time::Instant;

/// Process-wide lock for env-var test orchestration. Cargo runs integration
/// tests in parallel; flipping `PDF_OXIDE_RESOLUTION_PIPELINE` must not race
/// with another test's read.
static PIPELINE_TOGGLE_LOCK: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// PDF construction helpers — self-contained so a fix-pass to the pilot or
// wave-1 QA helpers can't accidentally invalidate the wave-2 invariants.
// ---------------------------------------------------------------------------

/// Build a one-page text-fixture PDF with a Helvetica `/F1` Type 1 font
/// referenced at object 5. `resources_extra` is appended into the page's
/// `/Resources` dictionary (use it for /ColorSpace, /ExtGState, additional
/// /Font entries, etc.).
fn build_pdf_text(content_ops: &str, resources_extra: &str) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    let page_off = buf.len();
    let page = format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
         /Resources << /Font << /F1 5 0 R >> {} >> /Contents 4 0 R >>\nendobj\n",
        resources_extra
    );
    buf.extend_from_slice(page.as_bytes());

    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let font_off = buf.len();
    buf.extend_from_slice(
        b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica \
          /Encoding /WinAnsiEncoding >>\nendobj\n",
    );

    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off, font_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

/// Build a one-page PDF with `/F1` Helvetica AND a second `/F2` standard
/// font (Times-Roman). Used by multi-font probes.
fn build_pdf_two_fonts(content_ops: &str) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    let page_off = buf.len();
    let page = "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
         /Resources << /Font << /F1 5 0 R /F2 6 0 R >> >> /Contents 4 0 R >>\nendobj\n";
    buf.extend_from_slice(page.as_bytes());

    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let font1_off = buf.len();
    buf.extend_from_slice(
        b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica \
          /Encoding /WinAnsiEncoding >>\nendobj\n",
    );

    let font2_off = buf.len();
    buf.extend_from_slice(
        b"6 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Times-Roman \
          /Encoding /WinAnsiEncoding >>\nendobj\n",
    );

    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 7\n0000000000 65535 f \n");
    for off in [
        cat_off, pages_off, page_off, stream_off, font1_off, font2_off,
    ] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 7 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

/// Build a one-page text-fixture PDF with a Helvetica `/F1` Type 1 font
/// AND an indirect Type 4 tint-transform function at object 6. Used by
/// Separation / DeviceN spot-colour probes.
fn build_pdf_text_with_type4_separation(
    content_ops: &str,
    type4_program: &str,
    resources_extra: &str,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    let page_off = buf.len();
    let page = format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
         /Resources << /Font << /F1 5 0 R >> {} >> /Contents 4 0 R >>\nendobj\n",
        resources_extra
    );
    buf.extend_from_slice(page.as_bytes());

    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let font_off = buf.len();
    buf.extend_from_slice(
        b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica \
          /Encoding /WinAnsiEncoding >>\nendobj\n",
    );

    let func_off = buf.len();
    let func_hdr = format!(
        "6 0 obj\n<< /FunctionType 4 /Domain [0 1] /Range [0 1 0 1 0 1 0 1] /Length {} >>\nstream\n",
        type4_program.len()
    );
    buf.extend_from_slice(func_hdr.as_bytes());
    buf.extend_from_slice(type4_program.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 7\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off, font_off, func_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 7 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
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
/// Used by adversarial-input probes — the invariant is "no panic", not
/// "render succeeds".
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

/// Count pixels in `[x0, x1) × [y0, y1)` whose RGB is materially below the
/// white background. Used as a "did any glyph ink land here" probe.
fn count_ink_pixels(rgba: &[u8], x0: u32, y0: u32, x1: u32, y1: u32) -> u32 {
    let w = 100u32;
    let h = 100u32;
    assert_eq!(rgba.len() as u32, w * h * 4);
    let mut n = 0u32;
    for y in y0..y1.min(h) {
        for x in x0..x1.min(w) {
            let off = ((y * w + x) * 4) as usize;
            let r = rgba[off];
            let g = rgba[off + 1];
            let b = rgba[off + 2];
            if r < 240 || g < 240 || b < 240 {
                n += 1;
            }
        }
    }
    n
}

/// Average (r, g, b) over the non-background pixels in the search region.
/// Returns `None` when no ink was found.
fn average_ink_rgb(rgba: &[u8], x0: u32, y0: u32, x1: u32, y1: u32) -> Option<(f32, f32, f32)> {
    let w = 100u32;
    let h = 100u32;
    assert_eq!(rgba.len() as u32, w * h * 4);
    let mut n = 0u64;
    let mut sr = 0u64;
    let mut sg = 0u64;
    let mut sb = 0u64;
    for y in y0..y1.min(h) {
        for x in x0..x1.min(w) {
            let off = ((y * w + x) * 4) as usize;
            let r = rgba[off];
            let g = rgba[off + 1];
            let b = rgba[off + 2];
            if r < 220 || g < 220 || b < 220 {
                sr += r as u64;
                sg += g as u64;
                sb += b as u64;
                n += 1;
            }
        }
    }
    if n == 0 {
        return None;
    }
    Some((sr as f32 / n as f32, sg as f32 / n as f32, sb as f32 / n as f32))
}

// ============================================================================
// Scale probes — long streams, many segments, mixed text/path, multi-font.
// ============================================================================

/// Probe 1 — Long text-heavy page: many `Tj` operators with mid-stream font
/// size changes. The pipeline allocates a fresh resolver per `Tj`; any
/// per-call state leak or asymmetric routing across repeated dispatch would
/// surface as drift between toggle-off and toggle-on.
///
/// Fixture: 12 `Tj` calls, font sizes alternating 8/16/24/32, every call
/// emits a 10-char string. That's >120 glyphs; the rasteriser routes
/// every glyph through the spliced GS the helper produces — so any
/// per-glyph leak through to the resolver also surfaces here.
#[test]
fn qa_text_long_run_many_tj_calls_byte_identical() {
    let mut content = String::new();
    content.push_str("BT 1 0 0 rg /F1 8 Tf 5 90 Td ");
    let sizes = [8u32, 16, 24, 32];
    let strings = ["AAAAAAAAAA", "BBBBBBBBBB", "CCCCCCCCCC", "DDDDDDDDDD"];
    for i in 0..12 {
        let size = sizes[i % 4];
        let s = strings[i % 4];
        content.push_str(&format!("/F1 {} Tf 0 -7 Td ({}) Tj ", size, s));
    }
    content.push_str("ET\n");
    let bytes = build_pdf_text(&content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "long text-heavy run (>120 glyphs across 12 Tj calls + font-size changes) \
         must be byte-identical off vs on"
    );
    // Sanity: actual ink painted somewhere — the test would pass trivially
    // if both renders produced empty pages.
    assert!(count_ink_pixels(&on, 0, 0, 100, 100) > 50, "expected substantial ink");
}

/// Probe 2 — TJ array with 20+ alternating strings and numeric kerning
/// offsets. Each numeric entry adjusts the text matrix between glyph
/// emissions; the spliced GS is borrowed for the whole array. If the
/// pipeline were to re-resolve per array element or per glyph it would
/// drift on this fixture.
#[test]
fn qa_text_tj_array_many_segments_byte_identical() {
    // 20 segments: alternating 1-char strings and small numeric kern
    // offsets. Build it in a loop so the count is unambiguous.
    let mut array = String::new();
    for i in 0..20 {
        let ch = match i % 5 {
            0 => 'H',
            1 => 'i',
            2 => 'l',
            3 => 'o',
            _ => 'W',
        };
        array.push_str(&format!("({}) -50 ", ch));
    }
    let content = format!("BT 0 0 1 rg /F1 12 Tf 5 50 Td [{}] TJ ET\n", array);
    let bytes = build_pdf_text(&content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "TJ array with 20 string segments + 20 numeric kerning offsets must be \
         byte-identical off vs on"
    );
    // Sanity: blue ink lands somewhere.
    let avg = average_ink_rgb(&on, 0, 30, 100, 70);
    assert!(avg.is_some(), "expected blue glyph ink from long TJ array");
    let (r, g, b) = avg.unwrap();
    assert!(
        r < 100.0 && g < 100.0 && b > 150.0,
        "TJ array glyph ink must be blue, got ({r:.1}, {g:.1}, {b:.1})"
    );
}

/// Probe 3 — Real-world style content: interleaved `BT/ET` text blocks
/// with `re/f` and `re/S` path operators. Text blocks change colour and
/// font size between iterations to ensure the pipeline state correctly
/// tears down between operator arms.
#[test]
fn qa_text_interleaved_with_path_operators_byte_identical() {
    let content = "\
        1 0 0 rg 10 10 30 30 re f\n\
        BT 0 0 1 rg /F1 14 Tf 10 60 Td (Hello) Tj ET\n\
        0 1 0 RG 5 w 50 50 30 30 re S\n\
        BT 1 0 0 rg /F1 20 Tf 10 40 Td (World) Tj ET\n\
        0.3 g 50 5 40 20 re f\n\
        BT 0.5 g /F1 10 Tf 10 25 Td (Mixed) ' ET\n\
        0 0 0 RG 1 w 5 5 90 90 re S\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "interleaved text/path stream must be byte-identical off vs on");
    // Sanity: page is well-inked.
    assert!(count_ink_pixels(&on, 0, 0, 100, 100) > 200);
}

/// Probe 4 — Multi-font text run: `Tj` across `Tf` switches mid-stream.
/// The pipeline routes colour, not font; switching fonts mid-`BT/ET`
/// must not perturb the resolved colour for either side.
#[test]
fn qa_text_multi_font_run_byte_identical() {
    // Use the two-font fixture: alternate /F1 (Helvetica) and /F2
    // (Times-Roman). Same text content, same fill colour through the
    // whole run.
    let content = "BT 1 0 0 rg /F1 20 Tf 5 50 Td (A) Tj \
                   /F2 20 Tf (B) Tj \
                   /F1 20 Tf (C) Tj \
                   /F2 20 Tf (D) Tj ET\n";
    let bytes = build_pdf_two_fonts(content);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "multi-font Tj run must be byte-identical off vs on");
    let avg = average_ink_rgb(&on, 0, 20, 100, 90);
    assert!(avg.is_some(), "expected red glyph ink from multi-font run");
    let (r, g, b) = avg.unwrap();
    assert!(
        r > 180.0 && g < 80.0 && b < 80.0,
        "multi-font Tj run must paint red, got ({r:.1}, {g:.1}, {b:.1})"
    );
}

// ============================================================================
// Text rendering mode probes — Tr=0..7.
// ============================================================================
//
// `pipeline_resolve_text_gs` short-circuits Tr=3 to None, resolves fill for
// 0/2/4/6 and stroke for 1/2/5/6. Tr=4-7 add to the current clipping path
// in the spec; the current text rasteriser does NOT implement clip-add for
// text, so today both paths paint just like 0-2 and don't accumulate clip
// state. These tests assert PARITY across the toggle — any divergence
// flags either a pipeline bug or a clip-handling regression.
//
// If the implementation later adds clip-from-text support, these tests'
// assertions will still hold (parity invariant) but the *inline* path
// would have to do the clip add as well; otherwise toggle-on would
// diverge.

/// Probe 5a — Tr=0 (fill-only): pipeline parity on a plain DeviceRGB fill.
/// Already covered by the pilot's `pilot_tj_device_rgb_parity` — this is
/// just the QA-suite anchor making sure the suite's wider Tr coverage has
/// the trivial mode pinned too.
#[test]
fn qa_text_tr0_fill_only_parity() {
    let content = "BT 1 0 0 rg /F1 40 Tf 0 Tr 10 30 Td (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Tr=0 fill-only must be byte-identical off vs on");
}

/// Probe 5b — Tr=1 (stroke-only). Pipeline resolves the stroke side only.
/// The current text rasteriser doesn't emit per-glyph strokes, so the
/// painted page is blank either way; the invariant is byte-identical
/// parity (no spurious paint introduced by the pipeline path).
#[test]
fn qa_text_tr1_stroke_only_parity() {
    let content = "BT 1 0 0 RG /F1 40 Tf 1 Tr 10 30 Td (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Tr=1 stroke-only must be byte-identical off vs on");
}

/// Probe 5c — Tr=2 (fill+stroke). Pipeline resolves BOTH sides; the
/// rasteriser today only paints the fill side. Parity invariant holds.
#[test]
fn qa_text_tr2_fill_and_stroke_parity() {
    let content = "BT 1 0 0 rg 0 0 1 RG /F1 40 Tf 2 Tr 10 30 Td (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Tr=2 fill+stroke must be byte-identical off vs on");
}

/// Probe 5d — Tr=3 (invisible). Pipeline short-circuits to None — no
/// clone of `gs` happens. Parity invariant must hold AND the page must
/// stay at the white background (the rasteriser zeroes alpha for Tr=3).
#[test]
fn qa_text_tr3_invisible_parity_and_no_ink() {
    let content = "BT 1 0 0 rg /F1 40 Tf 3 Tr 10 30 Td (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Tr=3 invisible must be byte-identical off vs on");
    assert_eq!(
        count_ink_pixels(&on, 0, 0, 100, 100),
        0,
        "Tr=3 invisible text must paint zero pixels"
    );
}

/// Probe 5e — Tr=4 (fill + add to clip path). Pipeline resolves the fill
/// side. The rasteriser today doesn't implement clip-from-text, so the
/// painted output is the same as Tr=0; the parity invariant must hold.
///
/// This pins the CURRENT behaviour. When clip-from-text lands, this test's
/// parity assertion still holds — what would change is the assertion on
/// where ink appears (clip would suppress subsequent paints outside the
/// glyph silhouette).
#[test]
fn qa_text_tr4_fill_plus_clip_parity_pin() {
    let content = "BT 1 0 0 rg /F1 40 Tf 4 Tr 10 30 Td (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Tr=4 fill+clip parity invariant must hold");
    // Sanity: fill side painted (red ink lands somewhere).
    let avg = average_ink_rgb(&on, 0, 0, 100, 100);
    assert!(avg.is_some(), "Tr=4 must paint the fill side (red glyph)");
}

/// Probe 5f — Tr=5 (stroke + add to clip path). Pipeline resolves the
/// stroke side. Rasteriser doesn't paint strokes for text; parity
/// invariant must hold; no spurious paint.
#[test]
fn qa_text_tr5_stroke_plus_clip_parity_pin() {
    let content = "BT 1 0 0 RG /F1 40 Tf 5 Tr 10 30 Td (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Tr=5 stroke+clip parity invariant must hold");
}

/// Probe 5g — Tr=6 (fill + stroke + add to clip path). Pipeline resolves
/// BOTH sides; rasteriser paints fill only. Parity invariant must hold;
/// painted ink is the fill colour.
#[test]
fn qa_text_tr6_fill_stroke_plus_clip_parity_pin() {
    let content = "BT 1 0 0 rg 0 0 1 RG /F1 40 Tf 6 Tr 10 30 Td (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Tr=6 fill+stroke+clip parity invariant must hold");
    // Painted ink must be FILL colour (red), not stroke (blue).
    let avg = average_ink_rgb(&on, 0, 0, 100, 100);
    assert!(avg.is_some(), "Tr=6 must paint the fill side");
    let (r, g, b) = avg.unwrap();
    assert!(
        r > 180.0 && g < 80.0 && b < 80.0,
        "Tr=6 painted ink must be FILL red, not stroke blue, got ({r:.1}, {g:.1}, {b:.1})"
    );
}

/// Probe 5h — Tr=7 (add to clip path only). Per the spec Tr=7 is a
/// clip-only mode that paints nothing. The pipeline helper's `matches!`
/// rules don't include 7 for either fills or strokes — so the helper
/// returns None and no GS clone happens. Parity invariant must hold;
/// no ink should appear.
#[test]
fn qa_text_tr7_clip_only_parity_pin() {
    let content = "BT 1 0 0 rg /F1 40 Tf 7 Tr 10 30 Td (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Tr=7 clip-only parity invariant must hold");
}

/// Probe 6 — Tr changes mid-stream. Sequence: Tr=0 Tj, `Tr 2`, Tr=2 Tj.
/// Each call gets its own pipeline-resolve; the previous call's spliced
/// GS clone must not leak into the next call's borrowed `gs`.
#[test]
fn qa_text_tr_change_mid_stream_parity() {
    let content = "BT 1 0 0 rg 0 0 1 RG /F1 20 Tf 5 60 Td \
                   0 Tr (A) Tj \
                   2 Tr (B) Tj \
                   0 Tr (C) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Tr change mid-stream must be byte-identical off vs on");
}
