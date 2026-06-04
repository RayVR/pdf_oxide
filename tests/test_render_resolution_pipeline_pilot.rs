//! Pilot-operator regression tests for the resolution pipeline.
//!
//! These tests exercise the path-fill (`f`) operator through the renderer
//! end-to-end and assert on the rendered pixels rather than on intermediate
//! values. They are the integration-side proof of what the per-stage unit
//! tests assert at the API level:
//!
//! 1. **Parity** — for fills the inline match arm handles correctly
//!    (DeviceRGB / DeviceGray / DeviceCMYK), the pipeline produces the same
//!    output byte-for-byte.
//! 2. **Capability gain** — for a `Separation` colour space with a
//!    PostScript Type 4 tint transform, the inline match arm falls back to
//!    `1.0 - tint` (renders the full-tint area as solid black); the
//!    pipeline correctly evaluates the Type 4 program and renders the
//!    expected colour.
//!
//! The pipeline is gated behind `PDF_OXIDE_RESOLUTION_PIPELINE`. Tests flip
//! the env var around `render_page` calls; a process-wide mutex serialises
//! the flips so parallel test execution doesn't interleave them.

#![cfg(feature = "rendering")]

use pdf_oxide::document::PdfDocument;
use pdf_oxide::rendering::{render_page, ImageFormat, RenderOptions};
use std::sync::Mutex;

/// Process-wide lock around env-var-based test orchestration. Cargo runs
/// integration tests in parallel by default; flipping the
/// `PDF_OXIDE_RESOLUTION_PIPELINE` toggle inside a test thread must not race
/// with another test reading it.
static PIPELINE_TOGGLE_LOCK: Mutex<()> = Mutex::new(());

/// Build a tiny one-page PDF whose content stream is `content_ops`, with a
/// fixed 100×100 MediaBox and the provided `resources_dict` body (the bytes
/// between `<<` and `>>` of the page's `/Resources` entry).
fn build_pdf(content_ops: &str, resources_dict: &str) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    // 1 0 obj: Catalog
    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // 2 0 obj: Pages
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    // 3 0 obj: Page
    let page_off = buf.len();
    let page = format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << {} >> /Contents 4 0 R >>\nendobj\n",
        resources_dict
    );
    buf.extend_from_slice(page.as_bytes());

    // 4 0 obj: Content stream
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

/// Build a one-page PDF with an indirect Type 4 function as object 5 and a
/// Separation colour space defined in the page resources that references it.
fn build_pdf_with_type4_separation(
    content_ops: &str,
    type4_program: &str,
    page_resources_extra: &str,
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

    // 5 0 obj: Type 4 function (a stream).
    let func_off = buf.len();
    let func_hdr = format!(
        "5 0 obj\n<< /FunctionType 4 /Domain [0 1] /Range [0 1 0 1 0 1 0 1] /Length {} >>\nstream\n",
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

/// Render with the toggle held to `enabled` for the duration of the call.
/// The shared mutex ensures one test's env-var manipulation doesn't bleed
/// into another's read.
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
    // Restore the previous state — pristine env-var preserved.
    match prev {
        Some(v) => std::env::set_var("PDF_OXIDE_RESOLUTION_PIPELINE", v),
        None => std::env::remove_var("PDF_OXIDE_RESOLUTION_PIPELINE"),
    }
    data
}

/// Sample a central pixel of the 100×100 page (72 dpi → 100×100 px).
/// Returns `(r, g, b, a)` as bytes.
fn center_pixel(rgba: &[u8]) -> (u8, u8, u8, u8) {
    let w = 100u32;
    let h = 100u32;
    assert_eq!(rgba.len() as u32, w * h * 4);
    let cx = w / 2;
    let cy = h / 2;
    let off = ((cy * w + cx) * 4) as usize;
    (rgba[off], rgba[off + 1], rgba[off + 2], rgba[off + 3])
}

#[test]
fn pilot_path_fill_device_rgb_parity_pipeline_off_vs_on() {
    // Both code paths handle DeviceRGB correctly. The pipeline must produce
    // the same output as the inline path, byte-for-byte, so flipping the
    // toggle is safe on every PDF that already renders correctly today.
    //
    // Content stream: paint a 40×40 red rectangle centred on the 100×100
    // page using DeviceRGB(1, 0, 0).
    //
    // PDF user space has its origin at the bottom-left; tiny-skia output
    // has its origin at the top-left and the renderer flips Y as part of
    // the base transform. Either way, the centre pixel of a centred
    // rectangle stays the centre pixel under the flip, so this test does
    // not depend on the Y orientation.
    let content = "1 0 0 rg\n30 30 40 40 re\nf\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);

    assert_eq!(off.len(), on.len(), "pipeline output size must match inline for parity case");
    assert_eq!(
        off, on,
        "pipeline must produce byte-identical output for DeviceRGB path-fill (parity invariant)"
    );

    // Sanity: the centre pixel really did get painted red. Without this
    // the test could pass trivially if both paths produced the same
    // background.
    let (r, g, b, _a) = center_pixel(&on);
    assert!(r > 200 && g < 60 && b < 60, "centre pixel must be red, got ({r}, {g}, {b})");
}

#[test]
fn pilot_path_fill_device_gray_parity_pipeline_off_vs_on() {
    // DeviceGray parity case. The pipeline's Gray path goes through
    // `LogicalColor::Device(DeviceColor::Gray(_))` while the inline path
    // populates `gs.fill_color_rgb` directly; both must yield the same
    // pixmap.
    let content = "0.5 g\n10 10 80 80 re\nf\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "DeviceGray path-fill must be byte-identical");
}

#[test]
fn pilot_path_fill_device_cmyk_parity_pipeline_off_vs_on() {
    // DeviceCMYK parity case. Both the inline path (`page_renderer.rs`
    // `cmyk_to_rgb`) and the pipeline's `ColorResolver` evaluate the same
    // ISO 32000-1 §10.3.5 additive-clamp formula, so the output must be
    // byte-identical.
    let content = "1 0 0 0 k\n10 10 80 80 re\nf\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "DeviceCMYK path-fill must be byte-identical");
}

#[test]
fn pilot_path_fill_type4_separation_pipeline_resolves_correctly() {
    // The Type-4 capability gain. The Separation colour space carries a
    // PostScript Type 4 tint transform; the inline match arm at
    // `page_renderer.rs:629-693` only recognises Type 2 and falls back to
    // `1.0 - tint`. With `tint = 1.0` the fall-back produces solid black;
    // the pipeline runs the Type 4 program and produces the colour the
    // function declares.
    //
    // Program: `{ 0.0 exch 0.0 0.0 }` — leaves CMYK(0, tint, 0, 0) on the
    // stack. At tint=1.0 that's pure magenta (CMYK(0,1,0,0) → RGB(1,0,1)).
    //
    // The Separation colour space is referenced as `/SpotMagenta` in the
    // page resources; the content stream sets the fill space and paints
    // a 60×60 rectangle.
    let type4_program = "{ 0.0 exch 0.0 0.0 }";
    let content = "/SpotMagenta cs\n1 scn\n20 20 60 60 re\nf\n";
    let resources = "/ColorSpace << /SpotMagenta [/Separation /MagentaSpot /DeviceCMYK 5 0 R] >>";

    let bytes = build_pdf_with_type4_separation(content, type4_program, resources);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);

    // Inline path: full-tint Type 4 spot → solid black centre.
    let (r_off, g_off, b_off, _a) = center_pixel(&off);
    assert!(
        r_off < 50 && g_off < 50 && b_off < 50,
        "inline path must produce ~solid black for full-tint Type 4 Separation, got ({r_off}, {g_off}, {b_off})"
    );

    // Pipeline: magenta — high R, low G, high B.
    let (r_on, g_on, b_on, _a) = center_pixel(&on);
    assert!(
        r_on > 200 && g_on < 60 && b_on > 200,
        "pipeline must resolve Type 4 Separation to magenta, got ({r_on}, {g_on}, {b_on})"
    );

    // And the pixmaps must differ overall — the toggle has a visible
    // effect on the output for this PDF.
    assert_ne!(off, on, "pipeline output must differ from inline output for Type 4 Separation");
}

// =====================================================================
// Stroke (`S`) and combo (`B`, `B*`, `b`, `b*`) operators — same env-var
// gating, mirror-image of the path-fill pilot.
// =====================================================================

/// Sample the pixel at `(x, y)` on a 100×100 page. Returns `(r, g, b, a)`.
fn pixel_at(rgba: &[u8], x: u32, y: u32) -> (u8, u8, u8, u8) {
    let w = 100u32;
    let h = 100u32;
    assert_eq!(rgba.len() as u32, w * h * 4);
    assert!(x < w && y < h);
    let off = ((y * w + x) * 4) as usize;
    (rgba[off], rgba[off + 1], rgba[off + 2], rgba[off + 3])
}

/// Build a one-page PDF that owns TWO indirect Type 4 functions plus a
/// content stream — used by combo tests that need an independent tint
/// transform for fill and stroke.
fn build_pdf_with_two_type4_separations(
    content_ops: &str,
    type4_program_a: &str,
    type4_program_b: &str,
    page_resources_extra: &str,
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

    let func_a_off = buf.len();
    let func_a_hdr = format!(
        "5 0 obj\n<< /FunctionType 4 /Domain [0 1] /Range [0 1 0 1 0 1 0 1] /Length {} >>\nstream\n",
        type4_program_a.len()
    );
    buf.extend_from_slice(func_a_hdr.as_bytes());
    buf.extend_from_slice(type4_program_a.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let func_b_off = buf.len();
    let func_b_hdr = format!(
        "6 0 obj\n<< /FunctionType 4 /Domain [0 1] /Range [0 1 0 1 0 1 0 1] /Length {} >>\nstream\n",
        type4_program_b.len()
    );
    buf.extend_from_slice(func_b_hdr.as_bytes());
    buf.extend_from_slice(type4_program_b.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 7\n0000000000 65535 f \n");
    for off in [
        cat_off, pages_off, page_off, stream_off, func_a_off, func_b_off,
    ] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 7 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

// ---------- Stroke `S` parity tests ----------

#[test]
fn pilot_stroke_device_rgb_parity_pipeline_off_vs_on() {
    // DeviceRGB stroke — paint a 10-px-wide red rectangle outline so the
    // centre pixel of each edge sits well inside the stroked band.
    // Both code paths must produce byte-identical output.
    let content = "1 0 0 RG\n10 w\n20 20 60 60 re\nS\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "DeviceRGB stroke must be byte-identical off vs on");

    // Sanity: top edge of the stroked rectangle (at y=20, x=50) is red.
    let (r, g, b, _a) = pixel_at(&on, 50, 20);
    assert!(r > 200 && g < 60 && b < 60, "stroke edge must be red, got ({r}, {g}, {b})");
}

#[test]
fn pilot_stroke_device_gray_parity_pipeline_off_vs_on() {
    let content = "0.5 G\n10 w\n20 20 60 60 re\nS\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "DeviceGray stroke must be byte-identical off vs on");
}

#[test]
fn pilot_stroke_device_cmyk_parity_pipeline_off_vs_on() {
    // Pure cyan (CMYK 1,0,0,0) → RGB (0, 1, 1) under the additive-clamp
    // fallback. Same as the fill DeviceCMYK case.
    let content = "1 0 0 0 K\n10 w\n20 20 60 60 re\nS\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "DeviceCMYK stroke must be byte-identical off vs on");
}

#[test]
fn pilot_close_stroke_device_rgb_parity_pipeline_off_vs_on() {
    // `s` (close path + stroke) decomposes at the parser into ClosePath +
    // Stroke, so this exercises exactly the same arm as `S` but verifies
    // ClosePath in front doesn't perturb routing. Parity invariant still
    // holds.
    let content = "0 0 1 RG\n6 w\n30 30 m\n70 30 l\n70 70 l\n30 70 l\ns\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Close-Stroke (`s`) must be byte-identical off vs on");
}

// ---------- FillStroke (`B`) parity tests ----------

#[test]
fn pilot_fill_stroke_b_device_rgb_parity_pipeline_off_vs_on() {
    // `B` paints fill then stroke. Fill DeviceRGB green, stroke DeviceRGB
    // red, thick stroke so the edges read clearly.
    let content = "0 1 0 rg\n1 0 0 RG\n8 w\n25 25 50 50 re\nB\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "FillStroke (`B`) DeviceRGB must be byte-identical off vs on");
}

#[test]
fn pilot_fill_stroke_b_device_gray_parity_pipeline_off_vs_on() {
    let content = "0.8 g\n0.2 G\n8 w\n25 25 50 50 re\nB\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "FillStroke (`B`) DeviceGray must be byte-identical off vs on");
}

#[test]
fn pilot_fill_stroke_b_device_cmyk_parity_pipeline_off_vs_on() {
    let content = "0 1 0 0 k\n1 0 0 0 K\n8 w\n25 25 50 50 re\nB\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "FillStroke (`B`) DeviceCMYK must be byte-identical off vs on");
}

// ---------- FillStrokeEvenOdd (`B*`) parity tests ----------

#[test]
fn pilot_fill_stroke_b_star_device_rgb_parity_pipeline_off_vs_on() {
    let content = "0 1 0 rg\n1 0 0 RG\n8 w\n25 25 50 50 re\nB*\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "FillStrokeEvenOdd (`B*`) DeviceRGB must be byte-identical off vs on");
}

#[test]
fn pilot_fill_stroke_b_star_device_gray_parity_pipeline_off_vs_on() {
    let content = "0.8 g\n0.2 G\n8 w\n25 25 50 50 re\nB*\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "FillStrokeEvenOdd (`B*`) DeviceGray must be byte-identical off vs on");
}

#[test]
fn pilot_fill_stroke_b_star_device_cmyk_parity_pipeline_off_vs_on() {
    let content = "0 1 0 0 k\n1 0 0 0 K\n8 w\n25 25 50 50 re\nB*\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "FillStrokeEvenOdd (`B*`) DeviceCMYK must be byte-identical off vs on");
}

// ---------- CloseFillStroke (`b`) parity tests ----------

#[test]
fn pilot_close_fill_stroke_b_device_rgb_parity_pipeline_off_vs_on() {
    let content = "0 1 0 rg\n1 0 0 RG\n8 w\n30 30 m\n70 30 l\n70 70 l\n30 70 l\nb\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "CloseFillStroke (`b`) DeviceRGB must be byte-identical off vs on");
}

#[test]
fn pilot_close_fill_stroke_b_device_gray_parity_pipeline_off_vs_on() {
    let content = "0.8 g\n0.2 G\n8 w\n30 30 m\n70 30 l\n70 70 l\n30 70 l\nb\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "CloseFillStroke (`b`) DeviceGray must be byte-identical off vs on");
}

#[test]
fn pilot_close_fill_stroke_b_device_cmyk_parity_pipeline_off_vs_on() {
    let content = "0 1 0 0 k\n1 0 0 0 K\n8 w\n30 30 m\n70 30 l\n70 70 l\n30 70 l\nb\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "CloseFillStroke (`b`) DeviceCMYK must be byte-identical off vs on");
}

// ---------- CloseFillStrokeEvenOdd (`b*`) parity tests ----------

#[test]
fn pilot_close_fill_stroke_b_star_device_rgb_parity_pipeline_off_vs_on() {
    let content = "0 1 0 rg\n1 0 0 RG\n8 w\n30 30 m\n70 30 l\n70 70 l\n30 70 l\nb*\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "CloseFillStrokeEvenOdd (`b*`) DeviceRGB must be byte-identical off vs on"
    );
}

#[test]
fn pilot_close_fill_stroke_b_star_device_gray_parity_pipeline_off_vs_on() {
    let content = "0.8 g\n0.2 G\n8 w\n30 30 m\n70 30 l\n70 70 l\n30 70 l\nb*\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "CloseFillStrokeEvenOdd (`b*`) DeviceGray must be byte-identical off vs on"
    );
}

#[test]
fn pilot_close_fill_stroke_b_star_device_cmyk_parity_pipeline_off_vs_on() {
    let content = "0 1 0 0 k\n1 0 0 0 K\n8 w\n30 30 m\n70 30 l\n70 70 l\n30 70 l\nb*\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "CloseFillStrokeEvenOdd (`b*`) DeviceCMYK must be byte-identical off vs on"
    );
}

// ---------- Stroke capability test: Type 4 Separation ----------

#[test]
fn pilot_stroke_type4_separation_pipeline_resolves_correctly() {
    // Mirror of `pilot_path_fill_type4_separation_*` for the stroke side.
    // The inline `SCN` arm has no Separation/DeviceN branch at all and
    // ends up gray-clamping the first component (so full-tint resolves
    // to ~white, not the declared colour). The pipeline must resolve it
    // to the actual colour the program declares.
    //
    // Program: `{ 0.0 exch 0.0 0.0 }` leaves CMYK(0, tint, 0, 0) on the
    // stack — magenta at tint=1.
    //
    // The PDF strokes a 12-pixel-wide outline around a rectangle so the
    // edge pixels are deep inside the stroked band. PDF user-space has
    // its origin at the bottom-left and the renderer flips Y; a stroke
    // along the rectangle edges paints two horizontal bands at output
    // y∈[14, 26] and y∈[74, 86]. Sample (50, 80) — the centre of the
    // band corresponding to the PDF-coord y=20 edge.
    let type4_program = "{ 0.0 exch 0.0 0.0 }";
    let content = "/SpotMagenta CS\n1 SCN\n12 w\n20 20 60 60 re\nS\n";
    let resources = "/ColorSpace << /SpotMagenta [/Separation /MagentaSpot /DeviceCMYK 5 0 R] >>";
    let bytes = build_pdf_with_type4_separation(content, type4_program, resources);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);

    // Inline stroke path: the SCN dispatcher routes Separation/DeviceN
    // through the `g = 1.0 - tint` fallback (mirroring the long-standing
    // `scn` fill behaviour). At full tint that is g = 0 → solid black
    // stroke band. The Type 4 program never runs on this path.
    //
    // Pin the actual fallback colour, not just "not magenta" — a
    // regression that painted any non-magenta non-black colour (e.g. the
    // old SCN white-clamp) would slip past a bare negation.
    let (r_off, g_off, b_off, _a) = pixel_at(&off, 50, 80);
    assert!(
        r_off < 30 && g_off < 30 && b_off < 30,
        "inline stroke path: full-tint Type 4 Separation must hit the SCN `1.0 - tint` \
         fallback (g=0 → near-black stroke band), got ({r_off}, {g_off}, {b_off})"
    );

    // Pipeline: the Type 4 program is evaluated → magenta lands on the
    // stroked band.
    let (r_on, g_on, b_on, _a) = pixel_at(&on, 50, 80);
    assert!(
        r_on > 200 && g_on < 60 && b_on > 200,
        "pipeline stroke must resolve Type 4 Separation to magenta, got ({r_on}, {g_on}, {b_on})"
    );
    assert_ne!(
        off, on,
        "pipeline output must differ from inline output for Type 4 Separation stroke"
    );
}

// ---------- FillStroke distinct-colour combo capability test ----------

#[test]
fn pilot_fill_stroke_resolves_fill_and_stroke_independently() {
    // Two-intent verification. Fill uses a Type 4 Separation that resolves
    // to magenta (only the pipeline gets this right). Stroke uses DeviceRGB
    // cyan, which both paths handle identically.
    //
    // With the toggle off, the fill is solid black (the `1.0 - tint`
    // fallback) but the stroke is cyan — so the centre pixel is black and
    // the top-edge pixel is cyan. With the toggle on, the centre pixel
    // becomes magenta while the top-edge stroke stays cyan. That isolates
    // the fill-side capability gain from the stroke-side correctness.
    let type4_program = "{ 0.0 exch 0.0 0.0 }";
    let content = "/SpotMagenta cs\n1 scn\n0 1 1 RG\n10 w\n20 20 60 60 re\nB\n";
    let resources = "/ColorSpace << /SpotMagenta [/Separation /MagentaSpot /DeviceCMYK 5 0 R] >>";
    let bytes = build_pdf_with_type4_separation(content, type4_program, resources);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);

    // Centre pixel — interior of the filled rectangle.
    let (r_off_c, g_off_c, b_off_c, _) = center_pixel(&off);
    assert!(
        r_off_c < 60 && g_off_c < 60 && b_off_c < 60,
        "inline path: fill centre must be ~solid black (Type 4 fallback), got ({r_off_c}, {g_off_c}, {b_off_c})"
    );
    let (r_on_c, g_on_c, b_on_c, _) = center_pixel(&on);
    assert!(
        r_on_c > 200 && g_on_c < 60 && b_on_c > 200,
        "pipeline: fill centre must be magenta (Type 4 resolved), got ({r_on_c}, {g_on_c}, {b_on_c})"
    );

    // Top-edge stroke pixel — both paths must yield cyan (low R, high G,
    // high B). This is the "stroke side independence" assertion: the
    // capability-gaining fill side did not perturb the stroke side.
    let (r_off_s, g_off_s, b_off_s, _) = pixel_at(&off, 50, 20);
    assert!(
        r_off_s < 60 && g_off_s > 200 && b_off_s > 200,
        "inline path: stroke edge must be cyan, got ({r_off_s}, {g_off_s}, {b_off_s})"
    );
    let (r_on_s, g_on_s, b_on_s, _) = pixel_at(&on, 50, 20);
    assert!(
        r_on_s < 60 && g_on_s > 200 && b_on_s > 200,
        "pipeline: stroke edge must remain cyan, got ({r_on_s}, {g_on_s}, {b_on_s})"
    );
}

// ---------- Stroke graphics state preservation tests ----------

#[test]
fn pilot_stroke_preserves_line_width_under_pipeline() {
    // The stroke side routes through the pipeline by cloning `gs` and
    // overwriting only the colour fields; line width must survive
    // untouched. Render a stroke with two different line widths through
    // the pipeline path; pixel coverage at the band centre vs. just
    // outside the expected band confirms width was honoured.
    //
    // Thick band (width 16, centred at y=20) covers y=12..28.
    // Thin band (width 2, centred at y=20) covers y=19..21.
    let thick_content = "1 0 0 RG\n16 w\n20 20 60 60 re\nS\n";
    let thin_content = "1 0 0 RG\n2 w\n20 20 60 60 re\nS\n";
    let thick = PdfDocument::from_bytes(build_pdf(thick_content, "")).unwrap();
    let thin = PdfDocument::from_bytes(build_pdf(thin_content, "")).unwrap();

    let thick_on = render_with_pipeline(&thick, true);
    let thin_on = render_with_pipeline(&thin, true);

    // Inside the thick band but outside the thin band: y=16, x=50.
    // Background pixels are opaque white (R=G=B=255); red stroke pixels
    // have low G — checking G is the cleanest "painted vs. background"
    // discriminator for a pure-red stroke.
    let (_r_thick, g_thick, _b, _a) = pixel_at(&thick_on, 50, 16);
    let (_r_thin, g_thin, _b, _a) = pixel_at(&thin_on, 50, 16);
    assert!(g_thick < 60, "thick stroke must paint y=16 in red, got G={g_thick}");
    assert!(
        g_thin > 200,
        "thin stroke must NOT paint y=16 (background remains white), got G={g_thin}"
    );

    // And both must equal their off-mode counterparts (parity).
    let thick_off = render_with_pipeline(&thick, false);
    let thin_off = render_with_pipeline(&thin, false);
    assert_eq!(thick_off, thick_on, "line width must round-trip through pipeline path");
    assert_eq!(thin_off, thin_on, "line width must round-trip through pipeline path");
}

#[test]
fn pilot_stroke_preserves_line_cap_join_under_pipeline() {
    // Set line cap = 1 (round) and line join = 1 (round) via `J` and `j`
    // operators. Render an L-shape so the join corner is visible; a
    // round join produces a smooth outer arc, a miter join produces a
    // pointed corner. The off-vs-on parity check confirms the pipeline
    // didn't replace any of these graphics-state fields.
    //
    // Two PDFs: round vs miter. Render each on pipeline-on, compare to
    // the same content rendered off — they must match byte-for-byte.
    // Additionally, the round and miter renders must DIFFER from each
    // other under pipeline-on (so we know the GS dial actually has an
    // observable effect through the routed code path; a no-op pipeline
    // that ignored gs.line_join would render them identically and the
    // assertion below would catch it).
    let round_content = "1 0 0 RG\n10 w\n1 J\n1 j\n20 80 m\n20 20 l\n80 20 l\nS\n";
    let miter_content = "1 0 0 RG\n10 w\n0 J\n0 j\n20 80 m\n20 20 l\n80 20 l\nS\n";

    let round = PdfDocument::from_bytes(build_pdf(round_content, "")).unwrap();
    let miter = PdfDocument::from_bytes(build_pdf(miter_content, "")).unwrap();

    let round_on = render_with_pipeline(&round, true);
    let miter_on = render_with_pipeline(&miter, true);
    let round_off = render_with_pipeline(&round, false);
    let miter_off = render_with_pipeline(&miter, false);

    assert_eq!(round_off, round_on, "round cap/join must round-trip through pipeline path");
    assert_eq!(miter_off, miter_on, "miter cap/join must round-trip through pipeline path");
    assert_ne!(
        round_on, miter_on,
        "different cap/join settings must produce different pixels through the pipeline"
    );
}

#[test]
fn pilot_fill_stroke_two_type4_separations_both_resolved_independently() {
    // The strongest combo capability check: both fill and stroke use
    // distinct Type 4 Separations. Today's inline path mishandles both —
    // fill goes `1.0 - tint` (black), stroke has no Separation branch at
    // all (gray-clamps the tint, ~white). The pipeline routes each side
    // through its own `PaintIntent` and lands the correct colour on
    // each.
    //
    // Fill program: `{ 0.0 exch 0.0 0.0 }` → CMYK(0, t, 0, 0) → magenta
    //   at t=1.
    // Stroke program: `{ 0.0 0.0 0.0 }` (tint passes through to the
    //   first/cyan channel) → CMYK(t, 0, 0, 0) → cyan at t=1.
    let fill_program = "{ 0.0 exch 0.0 0.0 }";
    let stroke_program = "{ 0.0 0.0 0.0 }";
    let content = "/Magenta cs\n1 scn\n/Cyan CS\n1 SCN\n10 w\n20 20 60 60 re\nB\n";
    let resources = "/ColorSpace << \
        /Magenta [/Separation /MagentaSpot /DeviceCMYK 5 0 R] \
        /Cyan [/Separation /CyanSpot /DeviceCMYK 6 0 R] \
    >>";
    let bytes =
        build_pdf_with_two_type4_separations(content, fill_program, stroke_program, resources);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);

    // Pipeline ON — fill centre is magenta.
    let (r_c, g_c, b_c, _) = center_pixel(&on);
    assert!(
        r_c > 200 && g_c < 60 && b_c > 200,
        "pipeline: fill centre must resolve to magenta, got ({r_c}, {g_c}, {b_c})"
    );

    // Pipeline ON — top stroke band (output y=80, corresponding to PDF
    // y=20 edge after the renderer's Y-flip) is cyan.
    let (r_s, g_s, b_s, _) = pixel_at(&on, 50, 80);
    assert!(
        r_s < 60 && g_s > 200 && b_s > 200,
        "pipeline: stroke band must resolve to cyan, got ({r_s}, {g_s}, {b_s})"
    );

    // And both sides must actually have produced different output from
    // the inline path — neither side's mishandling is silently masked by
    // the other.
    assert_ne!(off, on, "pipeline output must differ from inline for two-Type-4 combo");
}

// ---------- `b` / `b*` close-edge positive tests ----------
//
// ISO 32000-1 §8.5.3.1 Table 60: the `b` and `b*` operators must close the
// active subpath before fill+stroke. The parser does NOT decompose these
// (only `s` is emitted as ClosePath + Stroke), so the dispatcher arm must
// perform the close itself. Before the fix, the closing edge of an open
// subpath was omitted from the stroke — a visible gap.
//
// These tests draw an open four-segment path (a square missing the left
// edge) using `b` / `b*`, then sample a pixel on the closing edge. Stroke
// is wide enough that the sampled pixel lies firmly inside the painted
// band when the close happens, and stays at the background colour when it
// doesn't. The fixture deliberately uses a thick (width 8) pure-blue
// stroke against a clear background so the discriminator is unambiguous.
//
// Coordinate note: the open path goes (30,30) → (70,30) → (70,70) →
// (30,70). The missing segment is the left edge at x=30, between y=30
// and y=70 in PDF user space. The renderer flips Y, so the output band
// remains centred at output x=30, output y∈[30, 70]. Sample (30, 50).

#[test]
fn pilot_b_operator_paints_close_edge_under_pipeline() {
    // Open four-segment subpath finished with `b`. With the spec-required
    // close in place the missing left edge gets painted blue; without it
    // the pixel at (30, 50) stays background-white.
    let content = "0 0 1 RG\n0 0 1 rg\n8 w\n30 30 m\n70 30 l\n70 70 l\n30 70 l\nb\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    // Pipeline ON — close-edge must be blue (B high, R low, G low).
    let on = render_with_pipeline(&doc, true);
    let (r_on, g_on, b_on, _a) = pixel_at(&on, 30, 50);
    assert!(
        r_on < 60 && g_on < 60 && b_on > 200,
        "pipeline: `b` close-edge pixel (30, 50) must be blue (the spec-required close \
         segment of the stroke was painted), got ({r_on}, {g_on}, {b_on})"
    );

    // Pipeline OFF — same close arm in the dispatcher, same assertion.
    let off = render_with_pipeline(&doc, false);
    let (r_off, g_off, b_off, _a) = pixel_at(&off, 30, 50);
    assert!(
        r_off < 60 && g_off < 60 && b_off > 200,
        "inline path: `b` close-edge pixel (30, 50) must be blue, got ({r_off}, {g_off}, {b_off})"
    );
}

#[test]
fn pilot_b_star_operator_paints_close_edge_under_pipeline() {
    // Same fixture as the `b` test but with `b*` (even-odd fill rule). The
    // close-edge geometry is path-rule-independent — the stroke side
    // paints it either way once the dispatcher calls `.close()`.
    let content = "0 0 1 RG\n0 0 1 rg\n8 w\n30 30 m\n70 30 l\n70 70 l\n30 70 l\nb*\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let on = render_with_pipeline(&doc, true);
    let (r_on, g_on, b_on, _a) = pixel_at(&on, 30, 50);
    assert!(
        r_on < 60 && g_on < 60 && b_on > 200,
        "pipeline: `b*` close-edge pixel (30, 50) must be blue (the spec-required close \
         segment of the stroke was painted), got ({r_on}, {g_on}, {b_on})"
    );

    let off = render_with_pipeline(&doc, false);
    let (r_off, g_off, b_off, _a) = pixel_at(&off, 30, 50);
    assert!(
        r_off < 60 && g_off < 60 && b_off > 200,
        "inline path: `b*` close-edge pixel (30, 50) must be blue, got ({r_off}, {g_off}, {b_off})"
    );
}

// ---------- `B*` / `b*` even-odd fill-rule fixture ----------
//
// Review MAJOR-2: the existing `B*`/`b*` parity tests use a convex
// rectangle, where even-odd and nonzero produce identical fills. A
// regression that silently routed `B*` through `FillRule::Winding` would
// not be caught.
//
// This test renders a self-intersecting bowtie (two triangles crossing
// at the centre), where the two rules disagree:
//   - Winding: the centre interior is filled.
//   - EvenOdd: the centre interior is NOT filled — the path winds the
//     same region twice with opposite orientations, so the parity is
//     even and the pixel is hollow.
//
// Constructing the bowtie: triangle ABC where A=(20,20), B=(80,80), and
// the path goes A → B → (80,20) → A (one closed triangle), continued
// by A → (20,80) → B → A (a second closed triangle that shares the
// AB diagonal). The two triangles overlap in the central diamond
// region; even-odd rule cancels the overlap, nonzero rule does not.
//
// We don't need that level of complexity; a simpler self-intersecting
// quad does the job. The classic figure-of-eight: trace a quad whose
// edges cross. (20,20) → (80,80) → (20,80) → (80,20) → back to (20,20).
// The two triangles formed share the central crossing; even-odd leaves
// the centre filled actually because the crossing parity is odd. Use a
// different shape — the standard test fixture is the star-of-david /
// two overlapping triangles with the SAME winding direction. Even
// simpler: two overlapping rectangles in the same subpath.
//
// Path: a 60×60 outer square plus a smaller 30×30 inner square, both
// closed. Under Winding (same orientation) both are filled solid.
// Under EvenOdd the inner square cancels — the centre pixel becomes a
// hole.

#[test]
fn pilot_b_star_even_odd_fill_rule_actually_evenodd() {
    // Outer square (50×50 centred) + inner square (20×20 centred),
    // both wound the same direction. EvenOdd → the inner square is a
    // hole; Winding → it's filled solid. Sample the centre pixel and
    // a pixel inside the outer-but-outside-inner ring to confirm.
    //
    // Fill = red, stroke = blue, thin stroke so the centre pixel reads
    // the FILL not the stroke colour.
    let content = "1 0 0 rg\n0 0 1 RG\n1 w\n\
                   25 25 m\n75 25 l\n75 75 l\n25 75 l\n25 25 l\n\
                   40 40 m\n60 40 l\n60 60 l\n40 60 l\n40 40 l\n\
                   b*\n";
    let bytes = build_pdf(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let on = render_with_pipeline(&doc, true);

    // Centre pixel sits inside the INNER square. Under EvenOdd it must
    // be the page background (white), NOT red. If `B*` regressed to
    // `FillRule::Winding`, this pixel would be red.
    let (r_c, g_c, b_c, _) = center_pixel(&on);
    assert!(
        r_c > 240 && g_c > 240 && b_c > 240,
        "pipeline: `b*` centre pixel must be the background (even-odd cancels the inner \
         square), got ({r_c}, {g_c}, {b_c})"
    );

    // Ring pixel — inside outer, outside inner. Must be red (the fill
    // colour). Sample (32, 50): x=32 is between the outer-left at x=25
    // and the inner-left at x=40.
    let (r_r, g_r, b_r, _) = pixel_at(&on, 32, 50);
    assert!(
        r_r > 200 && g_r < 60 && b_r < 60,
        "pipeline: `b*` ring pixel must be the fill colour, got ({r_r}, {g_r}, {b_r})"
    );

    // Off-toggle must agree (both paths share the dispatcher arm for the
    // fill rule, this is a regression net for both).
    let off = render_with_pipeline(&doc, false);
    let (r_co, g_co, b_co, _) = center_pixel(&off);
    assert!(
        r_co > 240 && g_co > 240 && b_co > 240,
        "inline: `b*` centre pixel must be the background, got ({r_co}, {g_co}, {b_co})"
    );
    let (r_ro, g_ro, b_ro, _) = pixel_at(&off, 32, 50);
    assert!(
        r_ro > 200 && g_ro < 60 && b_ro < 60,
        "inline: `b*` ring pixel must be the fill colour, got ({r_ro}, {g_ro}, {b_ro})"
    );
}

// =====================================================================
// Wave 2 — text operators (`Tj`, `TJ`, `'`, `"`) through the pipeline.
// =====================================================================
//
// Same env-var gating as wave 1. Text uses the embedded Helvetica Type 1
// standard font (no font file shipped — the rasteriser falls back to a
// system font for outline data; pixel coverage is enough for the tests
// to discriminate background from glyph ink and to compare RGB tint
// between two renders).
//
// Page coordinate sanity check: a 100×100 MediaBox renders to 100×100
// pixels at 72 dpi. The renderer flips Y so PDF y=0 is the BOTTOM of
// the image. Text positioned at PDF (x=10, y=30) with a big font size
// paints a glyph whose ink covers a wide horizontal band centred near
// output y=70..40 (PDF y=30 → image y=70, font extends upward in PDF
// space → image rows decrease). Tests probe a region rather than a
// single pixel so the exact ascent/descent of the system fallback font
// can't flake the assertion.

/// Build a one-page text-fixture PDF with a Helvetica Type 1 Font at
/// object 5 referenced as `/F1` in the page resources. Extra resources
/// (colour spaces, etc.) are appended inside the page's `/Resources`
/// dictionary via `resources_extra`.
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

/// Build a one-page text-fixture PDF with a Helvetica Type 1 Font at
/// object 5 AND an indirect Type 4 tint-transform function at object 6.
/// Use this for the spot-colour pipeline-gain tests.
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

/// Count pixels in a region whose RGB is materially below the white
/// background. Used as a "did any glyph ink land here" probe that's
/// font-fallback-resilient — a Helvetica fallback that paints a slightly
/// different shape than the real font is still a single connected band of
/// non-white pixels, so the count is non-zero whenever the rasteriser ran.
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
/// Returns `None` when no ink was found. Used to pin the *colour* of the
/// painted text without requiring an exact-pixel match — different system
/// fallback fonts hit slightly different anti-aliased subpixels but the
/// average colour of the rendered ink is invariant.
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
            // Skip background-ish pixels so the average reflects the painted
            // colour only — the AA halo around glyphs would otherwise drag
            // every channel toward white.
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

// ---------- Tj parity tests ----------

#[test]
fn pilot_tj_device_rgb_parity_pipeline_off_vs_on() {
    // DeviceRGB fill via `rg`. Big font, single 'M' glyph painted left-ish
    // of centre so its bounding box lands clearly inside the page. Parity:
    // pipeline output is byte-identical to inline output.
    let content = "BT 1 0 0 rg /F1 60 Tf 10 30 Td (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Tj DeviceRGB must be byte-identical off vs on");

    // Sanity: somewhere in the glyph search band, red ink actually painted.
    let avg = average_ink_rgb(&on, 5, 30, 55, 95);
    assert!(avg.is_some(), "expected red glyph ink to be painted somewhere");
    let (r, g, b) = avg.unwrap();
    assert!(
        r > 180.0 && g < 80.0 && b < 80.0,
        "Tj glyph ink must be red, got avg=({r:.1}, {g:.1}, {b:.1})"
    );
}

#[test]
fn pilot_tj_device_gray_parity_pipeline_off_vs_on() {
    // 0.5 g → mid-grey fill. Parity invariant.
    let content = "BT 0.5 g /F1 60 Tf 10 30 Td (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Tj DeviceGray must be byte-identical off vs on");
}

#[test]
fn pilot_tj_device_cmyk_parity_pipeline_off_vs_on() {
    // CMYK pure magenta (0, 1, 0, 0) → both paths use the same
    // additive-clamp fallback → byte-identical output.
    let content = "BT 0 1 0 0 k /F1 60 Tf 10 30 Td (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Tj DeviceCMYK must be byte-identical off vs on");
}

#[test]
fn pilot_tj_tr1_stroke_only_parity_pipeline_off_vs_on() {
    // Tr=1 strokes glyph outlines. The current text rasteriser doesn't
    // emit per-glyph strokes yet — render_mode is consulted only to skip
    // invisible (Tr=3) text. The pipeline migration must still be a
    // no-op on Tr=1 (no fill happens, the page stays blank), and the
    // off-vs-on output must be byte-identical so the toggle doesn't
    // introduce any spurious paint.
    let content = "BT 1 0 0 RG /F1 60 Tf 1 Tr 10 30 Td (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Tj Tr=1 stroke-only must be byte-identical off vs on");
}

#[test]
fn pilot_tj_tr2_fill_and_stroke_parity_pipeline_off_vs_on() {
    // Tr=2 fills AND strokes glyphs. Today's text rasteriser only paints
    // the fill side (the stroke side is a future-wave migration). The
    // pipeline migration must remain byte-identical to the inline path —
    // both fill and stroke colours route through the pipeline but only
    // the fill is observable.
    let content = "BT 1 0 0 rg 0 0 1 RG /F1 60 Tf 2 Tr 10 30 Td (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Tj Tr=2 fill+stroke must be byte-identical off vs on");

    // The fill colour wins on the rendered glyph. Pin it explicitly so a
    // regression that swapped fill/stroke would be caught.
    let avg = average_ink_rgb(&on, 5, 30, 55, 95);
    assert!(avg.is_some(), "expected fill-coloured glyph ink under Tr=2");
    let (r, g, b) = avg.unwrap();
    assert!(
        r > 180.0 && g < 80.0 && b < 80.0,
        "Tr=2: glyph ink must be the FILL colour (red), not the stroke colour (blue), \
         got avg=({r:.1}, {g:.1}, {b:.1})"
    );
}

#[test]
fn pilot_tj_tr3_invisible_parity_pipeline_off_vs_on() {
    // Tr=3 paints nothing — the text rasteriser zeroes the alpha and the
    // glyph is invisible. Pipeline migration must remain byte-identical
    // and the page must stay at the white background.
    let content = "BT 1 0 0 rg /F1 60 Tf 3 Tr 10 30 Td (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Tj Tr=3 invisible must be byte-identical off vs on");

    // No ink anywhere on the page — Tr=3 makes glyphs invisible.
    let ink = count_ink_pixels(&on, 0, 0, 100, 100);
    assert_eq!(ink, 0, "Tr=3 invisible must paint zero pixels, got {ink} non-background pixels");
}

#[test]
fn pilot_tj_advances_text_matrix_under_tr3() {
    // Negative state-preservation pin: Tr=3 must still advance the text
    // matrix (the OCR-overlay pattern depends on this — the invisible
    // glyphs reserve space the visible run beside them paints into). A
    // regression that early-returned out of the operator arm because
    // the pipeline resolver returned `None` for Tr=3 would break this.
    //
    // Fixture: two Tj calls, the first Tr=3 invisible, the second Tr=0
    // visible. The visible glyph must paint to the RIGHT of the
    // invisible run's slot, not on top of it.
    let content = "BT /F1 24 Tf 5 50 Td 3 Tr (HHHHH) Tj 0 Tr 1 0 0 rg (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let on = render_with_pipeline(&doc, true);

    // Region of the suppressed Tr=3 run, x ∈ [5, 50] roughly — must be
    // background.
    let ink_left = count_ink_pixels(&on, 5, 30, 55, 70);
    assert_eq!(
        ink_left, 0,
        "Tr=3 region must stay background (no glyph ink), got {ink_left} pixels"
    );

    // Region to the right where the visible M must paint after the
    // advance of 5×H widths. With Helvetica /WinAnsi at size 24, 5 H's
    // span ≈ 35 pt → the M lands at x ≈ 40..55. Probe x ∈ [50, 95].
    let ink_right = count_ink_pixels(&on, 50, 30, 95, 70);
    assert!(
        ink_right > 0,
        "visible M after Tr=3 advance must paint somewhere right of x=50, got 0 pixels"
    );
}

// ---------- TJ parity test ----------

#[test]
fn pilot_tj_array_device_rgb_parity_pipeline_off_vs_on() {
    // Numeric kerning offsets in the array must not change the colour
    // routing — only the X advance between glyphs. Parity invariant
    // holds.
    let content = "BT 0 0 1 rg /F1 50 Tf 5 30 Td [(H) -200 (i)] TJ ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "TJ array DeviceRGB must be byte-identical off vs on");

    // Blue glyph ink somewhere on the page.
    let avg = average_ink_rgb(&on, 0, 20, 100, 95);
    assert!(avg.is_some(), "expected blue glyph ink from TJ array");
    let (r, g, b) = avg.unwrap();
    assert!(
        r < 80.0 && g < 80.0 && b > 180.0,
        "TJ glyph ink must be blue, got avg=({r:.1}, {g:.1}, {b:.1})"
    );
}

// ---------- Quote (') and DoubleQuote (") parity tests ----------

#[test]
fn pilot_quote_device_rgb_parity_pipeline_off_vs_on() {
    // `'` is `T* Tj`. Establish a leading via TL=30 so the line-advance
    // moves to a known PDF y; emit the `'` against DeviceRGB fill green.
    // Parity is byte-identical and the painted ink is green.
    let content = "BT 0 1 0 rg /F1 40 Tf 30 TL 10 80 Td (X) ' ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Quote (`'`) DeviceRGB must be byte-identical off vs on");

    let avg = average_ink_rgb(&on, 0, 0, 100, 100);
    assert!(avg.is_some(), "expected green glyph ink from Quote");
    let (r, g, b) = avg.unwrap();
    assert!(
        r < 80.0 && g > 180.0 && b < 80.0,
        "Quote glyph ink must be green, got avg=({r:.1}, {g:.1}, {b:.1})"
    );
}

#[test]
fn pilot_double_quote_device_rgb_parity_pipeline_off_vs_on() {
    // `"` is `aw Tw ac Tc T* Tj`. The two numeric parameters (Tw, Tc) are
    // state-only — they don't perturb the colour routing. Parity is
    // byte-identical and the painted ink is red.
    let content = "BT 1 0 0 rg /F1 40 Tf 30 TL 10 80 Td 0 0 (X) \" ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "DoubleQuote (`\"`) DeviceRGB must be byte-identical off vs on");

    let avg = average_ink_rgb(&on, 0, 0, 100, 100);
    assert!(avg.is_some(), "expected red glyph ink from DoubleQuote");
    let (r, g, b) = avg.unwrap();
    assert!(
        r > 180.0 && g < 80.0 && b < 80.0,
        "DoubleQuote glyph ink must be red, got avg=({r:.1}, {g:.1}, {b:.1})"
    );
}

// ---------- Capability tests — Type 4 Separation text fill ----------

#[test]
fn pilot_text_tj_type4_separation_fill_pipeline_resolves_correctly() {
    // The text-side mirror of the wave-1 Type 4 fill pilot. With a Type 4
    // Separation colour space on the fill side at full tint, the inline
    // text path inherits the same `1.0 - tint` fallback the path-fill arm
    // had before wave 1 (`fill_color_rgb` is populated by the `scn` op,
    // which uses the same fallback). At tint=1 the fallback resolves to
    // black; the pipeline runs the Type 4 program and the glyph paints
    // in the program's actual colour — magenta.
    let type4_program = "{ 0.0 exch 0.0 0.0 }";
    let content = "/SpotMagenta cs 1 scn \
                   BT /F1 60 Tf 10 30 Td (M) Tj ET\n";
    let resources = "/ColorSpace << /SpotMagenta [/Separation /MagentaSpot /DeviceCMYK 6 0 R] >>";
    let bytes = build_pdf_text_with_type4_separation(content, type4_program, resources);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);

    // Inline path: glyph paints near-black (`1.0 - tint = 0` fallback).
    let avg_off = average_ink_rgb(&off, 0, 20, 100, 95);
    assert!(avg_off.is_some(), "inline path: expected SOME glyph ink (fallback colour)");
    let (r_o, g_o, b_o) = avg_off.unwrap();
    assert!(
        r_o < 60.0 && g_o < 60.0 && b_o < 60.0,
        "inline path: full-tint Type 4 Separation text fill must fall back to near-black, \
         got avg=({r_o:.1}, {g_o:.1}, {b_o:.1})"
    );

    // Pipeline: glyph paints in magenta (high R, low G, high B).
    let avg_on = average_ink_rgb(&on, 0, 20, 100, 95);
    assert!(avg_on.is_some(), "pipeline: expected magenta glyph ink");
    let (r_n, g_n, b_n) = avg_on.unwrap();
    assert!(
        r_n > 180.0 && g_n < 80.0 && b_n > 180.0,
        "pipeline: Type 4 Separation text fill must resolve to magenta, got avg=({r_n:.1}, {g_n:.1}, {b_n:.1})"
    );

    // And the pixmaps must differ — the toggle has a visible effect.
    assert_ne!(
        off, on,
        "pipeline output must differ from inline output for Type 4 Separation text fill"
    );
}

#[test]
fn pilot_text_tj_type4_separation_stroke_pipeline_resolves_correctly() {
    // Tr=1 puts the stroke side in charge of the painted ink. Today's text
    // rasteriser does not yet emit per-glyph strokes (a follow-up wave),
    // so neither the inline nor the pipeline path produces visible ink
    // under Tr=1. What we CAN verify today is that:
    //   - The toggle remains parity-safe under Tr=1 (no spurious paint
    //     from the pipeline).
    //   - The stroke-side pipeline resolution is in the call graph: the
    //     spliced graphics state carries the resolved stroke colour. We
    //     do that by combining Tr=1 stroke-only with a follow-up Tr=0
    //     fill from the same Separation — the fill side proves the
    //     resolver ran, and the parity-on-Tr=1 proves stroke routing
    //     does not perturb the no-paint behaviour.
    let type4_program = "{ 0.0 exch 0.0 0.0 }";
    let content = "/SpotMagenta CS 1 SCN \
                   /SpotMagenta cs 1 scn \
                   BT /F1 50 Tf 1 Tr 10 30 Td (M) Tj 0 Tr (M) Tj ET\n";
    let resources = "/ColorSpace << /SpotMagenta [/Separation /MagentaSpot /DeviceCMYK 6 0 R] >>";
    let bytes = build_pdf_text_with_type4_separation(content, type4_program, resources);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);

    // The Tr=0 follow-up glyph paints — inline = black fallback, pipeline
    // = magenta. This proves the stroke-side intent didn't accidentally
    // overwrite the fill-side resolution.
    let avg_off = average_ink_rgb(&off, 0, 20, 100, 95);
    assert!(avg_off.is_some(), "inline path: expected glyph ink from the Tr=0 follow-up");
    let (r_o, g_o, b_o) = avg_off.unwrap();
    assert!(
        r_o < 60.0 && g_o < 60.0 && b_o < 60.0,
        "inline path: Tr=0 follow-up must be near-black (Type 4 fallback), got avg=({r_o:.1}, {g_o:.1}, {b_o:.1})"
    );

    let avg_on = average_ink_rgb(&on, 0, 20, 100, 95);
    assert!(avg_on.is_some(), "pipeline: expected magenta glyph ink");
    let (r_n, g_n, b_n) = avg_on.unwrap();
    // The glyph average includes anti-aliased halo pixels (whose RGB is a
    // blend of the painted ink and the white background), so the magenta
    // average reads lower than the pure-fill `(255, 0, 255)`. What we pin
    // is the channel SHAPE — R and B materially above G — which only
    // holds for a magenta paint, not for the inline-fallback black
    // (which averages near zero on all channels) and not for white
    // background (which would defeat the avg-ink filter entirely).
    assert!(
        r_n > 100.0 && g_n < 80.0 && b_n > 100.0 && r_n > g_n + 50.0 && b_n > g_n + 50.0,
        "pipeline: Type 4 Separation text fill (Tr=0 follow-up) must resolve to magenta, \
         got avg=({r_n:.1}, {g_n:.1}, {b_n:.1})"
    );
}

// ---------- Distinct fill/stroke under Tr=2 ----------

#[test]
fn pilot_text_tj_distinct_fill_and_stroke_colors_under_tr2() {
    // Under Tr=2 the dispatcher resolves BOTH sides through the pipeline
    // and splices them into a single transient `GraphicsState`. The text
    // rasteriser paints the fill side; the spliced stroke colour is along
    // for the ride (visible in a future wave). What we assert today:
    //   - The painted ink is the FILL colour, not the stroke colour —
    //     so the two resolutions don't contaminate each other.
    //   - Off-vs-on remains byte-identical (parity invariant: stroke side
    //     is unobservable on the rasteriser today, so the splice is a
    //     no-op on output).
    //
    // Fill DeviceRGB red, stroke DeviceRGB blue, Tr=2. Parity holds and
    // the rendered ink is unambiguously red — a regression that swapped
    // sides under Tr=2 would paint blue and fail the colour assertion.
    let content = "BT 1 0 0 rg 0 0 1 RG /F1 60 Tf 2 Tr 10 30 Td (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Tr=2 distinct fill/stroke must be byte-identical off vs on");

    let avg = average_ink_rgb(&on, 0, 20, 100, 95);
    assert!(avg.is_some(), "expected glyph ink under Tr=2");
    let (r, g, b) = avg.unwrap();
    // Strict colour pin: must be red (R high, G/B low). A swap to the
    // stroke colour would surface as B high.
    assert!(
        r > 180.0 && g < 80.0 && b < 80.0,
        "Tr=2: painted ink must be the FILL colour (red), not the stroke colour. \
         got avg=({r:.1}, {g:.1}, {b:.1})"
    );
}

// ---------- State-preservation pins ----------

#[test]
fn pilot_text_under_pipeline_preserves_font_size_and_matrix() {
    // Font size is a graphics-state field the pipeline must not perturb —
    // we splice only colour. Render the same glyph at two different sizes
    // through the pipeline path; the larger glyph must produce more ink
    // pixels than the smaller. Parity off-vs-on confirms the splice was
    // a no-op for the size dial.
    let big_content = "BT 1 0 0 rg /F1 80 Tf 5 10 Td (M) Tj ET\n";
    let small_content = "BT 1 0 0 rg /F1 20 Tf 5 10 Td (M) Tj ET\n";
    let big = PdfDocument::from_bytes(build_pdf_text(big_content, "")).unwrap();
    let small = PdfDocument::from_bytes(build_pdf_text(small_content, "")).unwrap();

    let big_on = render_with_pipeline(&big, true);
    let small_on = render_with_pipeline(&small, true);

    let big_ink = count_ink_pixels(&big_on, 0, 0, 100, 100);
    let small_ink = count_ink_pixels(&small_on, 0, 0, 100, 100);
    assert!(
        big_ink > small_ink * 2,
        "font size 80 should paint substantially more ink than size 20, got big={big_ink}, small={small_ink}"
    );

    // Parity invariant.
    let big_off = render_with_pipeline(&big, false);
    let small_off = render_with_pipeline(&small, false);
    assert_eq!(big_off, big_on, "font size 80 must round-trip through pipeline path");
    assert_eq!(small_off, small_on, "font size 20 must round-trip through pipeline path");
}

#[test]
fn pilot_text_under_pipeline_preserves_character_and_word_spacing() {
    // Tc and Tw widen the horizontal gap between glyphs. The pipeline
    // must leave them untouched. Render the same text at default
    // spacing vs. wide spacing through the pipeline; the wide-spaced
    // run must cover more horizontal extent than the tight one.
    let tight = "BT 1 0 0 rg /F1 24 Tf 5 50 Td (HHH HHH) Tj ET\n";
    let wide = "BT 1 0 0 rg /F1 24 Tf 5 Tc 2 Tw 5 50 Td (HHH HHH) Tj ET\n";
    let tight_doc = PdfDocument::from_bytes(build_pdf_text(tight, "")).unwrap();
    let wide_doc = PdfDocument::from_bytes(build_pdf_text(wide, "")).unwrap();

    let tight_on = render_with_pipeline(&tight_doc, true);
    let wide_on = render_with_pipeline(&wide_doc, true);

    // Find the rightmost ink pixel in each render. With wider spacing the
    // last glyph lands further right.
    let rightmost = |rgba: &[u8]| -> Option<u32> {
        for x in (0u32..100).rev() {
            for y in 30u32..70 {
                let off = ((y * 100 + x) * 4) as usize;
                let r = rgba[off];
                let g = rgba[off + 1];
                let b = rgba[off + 2];
                if r < 240 || g < 240 || b < 240 {
                    return Some(x);
                }
            }
        }
        None
    };
    let tight_right = rightmost(&tight_on).expect("tight render: glyph ink present");
    let wide_right = rightmost(&wide_on).expect("wide render: glyph ink present");
    assert!(
        wide_right > tight_right,
        "wider Tc/Tw must push the rightmost glyph further right; tight={tight_right}, wide={wide_right}"
    );

    // Parity off-vs-on.
    let tight_off = render_with_pipeline(&tight_doc, false);
    let wide_off = render_with_pipeline(&wide_doc, false);
    assert_eq!(tight_off, tight_on, "tight Tc/Tw must round-trip through pipeline path");
    assert_eq!(wide_off, wide_on, "wide Tc/Tw must round-trip through pipeline path");
}

#[test]
fn pilot_text_under_pipeline_preserves_horizontal_scaling() {
    // Tz (horizontal scaling) scales the glyph advance horizontally — at
    // 50 % the cluster's rightmost pixel lands further LEFT than at 100 %.
    // The pipeline must not stamp Tz out. Probe by measuring the rightmost
    // ink column and asserting it shifts left when Tz is reduced; parity
    // off-vs-on covers the negative direction.
    //
    // Use a font size small enough that Tz=100 % comfortably fits on the
    // 100-pt-wide page (Helvetica /F1 16 → "HHH" ≈ 33 pt) so the only
    // reason the right edge can move is the Tz advance dial itself.
    let normal = "BT 1 0 0 rg /F1 16 Tf 5 50 Td (HHH) Tj ET\n";
    let narrow = "BT 1 0 0 rg /F1 16 Tf 50 Tz 5 50 Td (HHH) Tj ET\n";
    let normal_doc = PdfDocument::from_bytes(build_pdf_text(normal, "")).unwrap();
    let narrow_doc = PdfDocument::from_bytes(build_pdf_text(narrow, "")).unwrap();

    let normal_on = render_with_pipeline(&normal_doc, true);
    let narrow_on = render_with_pipeline(&narrow_doc, true);

    // Rightmost ink column, restricted to the glyph band.
    let rightmost = |rgba: &[u8]| -> Option<u32> {
        for x in (0u32..100).rev() {
            for y in 30u32..70 {
                let off = ((y * 100 + x) * 4) as usize;
                let r = rgba[off];
                let g = rgba[off + 1];
                let b = rgba[off + 2];
                if r < 240 || g < 240 || b < 240 {
                    return Some(x);
                }
            }
        }
        None
    };
    let normal_right = rightmost(&normal_on).expect("Tz=100 render: glyph ink present");
    let narrow_right = rightmost(&narrow_on).expect("Tz=50 render: glyph ink present");
    assert!(
        narrow_right < normal_right,
        "Tz=50 must place the rightmost glyph LEFT of Tz=100; normal={normal_right}, narrow={narrow_right}"
    );

    let normal_off = render_with_pipeline(&normal_doc, false);
    let narrow_off = render_with_pipeline(&narrow_doc, false);
    assert_eq!(normal_off, normal_on, "Tz=100 must round-trip through pipeline path");
    assert_eq!(narrow_off, narrow_on, "Tz=50 must round-trip through pipeline path");
}

#[test]
fn pilot_text_helper_returns_none_when_pipeline_disabled() {
    // The off-toggle path must NOT clone `gs` for text painting — the
    // wave-1 invariant. We can't probe the helper directly, but we can
    // assert the byte-identical parity invariant against the inline
    // path for a Tr=2 case where the helper would otherwise clone
    // (both fill and stroke get resolved). If parity holds, no
    // observable clone happened (on the off path) AND the on path
    // splice was a no-op for fill+stroke.
    let content = "BT 0.5 g 0.2 G /F1 50 Tf 2 Tr 10 30 Td (M) Tj ET\n";
    let bytes = build_pdf_text(content, "");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "off-path must not be perturbed by Tr=2 gating; on-path Device colours must splice to no visible change"
    );
}

// =====================================================================
// Wave 3 — Image XObject (`Do`) pilot
//
// ImageMask XObjects (`/Subtype /Image`, `/ImageMask true`) paint a 1-bit
// stencil with the current fill colour. The pipeline routes that fill
// through the resolution pipeline (Type 4 Separation evaluation, ICC
// conversion, etc.). Standard images and Form XObjects pass through
// untouched.
// =====================================================================

/// Shared core for the two ImageMask-fixture builders below. Emits a
/// one-page PDF whose page references `/IM1` as an ImageMask XObject and
/// — when `type4_program` is `Some(_)` — an indirect Type 4 tint
/// transform as object 6 (the Separation colour space the page
/// resources declare via `resources_extra` is expected to reference
/// `6 0 R`).
///
/// `content_ops` runs on the page (typically sets the fill colour, a
/// CTM, then `/IM1 Do`). `resources_extra` is appended into the page's
/// `/Resources` dictionary (use it for `/ColorSpace` when a non-device
/// fill is required). `mask_data` is the raw 1-bit stencil byte stream.
fn build_pdf_image_mask_core(
    content_ops: &str,
    resources_extra: &str,
    width: u32,
    height: u32,
    mask_data: &[u8],
    type4_program: Option<&str>,
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
         /Resources << /XObject << /IM1 5 0 R >> {} >> /Contents 4 0 R >>\nendobj\n",
        resources_extra
    );
    buf.extend_from_slice(page.as_bytes());

    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xobj_off = buf.len();
    let xobj_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Image /ImageMask true \
         /Width {} /Height {} /BitsPerComponent 1 /Length {} >>\nstream\n",
        width,
        height,
        mask_data.len()
    );
    buf.extend_from_slice(xobj_hdr.as_bytes());
    buf.extend_from_slice(mask_data);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    // Optional Type 4 function as object 6 — present only when the
    // caller's Separation colour space references it.
    let func_off = if let Some(program) = type4_program {
        let off = buf.len();
        let func_hdr = format!(
            "6 0 obj\n<< /FunctionType 4 /Domain [0 1] /Range [0 1 0 1 0 1 0 1] /Length {} >>\nstream\n",
            program.len()
        );
        buf.extend_from_slice(func_hdr.as_bytes());
        buf.extend_from_slice(program.as_bytes());
        buf.extend_from_slice(b"\nendstream\nendobj\n");
        Some(off)
    } else {
        None
    };

    // xref + trailer: 6 or 7 objects depending on whether the Type 4
    // function is present.
    let xref_off = buf.len();
    let (size, header) = if func_off.is_some() {
        (7, "xref\n0 7\n0000000000 65535 f \n")
    } else {
        (6, "xref\n0 6\n0000000000 65535 f \n")
    };
    buf.extend_from_slice(header.as_bytes());
    let core_entries = [cat_off, pages_off, page_off, stream_off, xobj_off];
    for off in core_entries {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    if let Some(off) = func_off {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", size, xref_off)
            .as_bytes(),
    );
    buf
}

/// Build a one-page PDF with an ImageMask XObject `/IM1` filled with raw
/// 1-bit stencil bytes. `content_ops` runs on the page (typically sets
/// the fill colour, a CTM, then `/IM1 Do`). `resources_extra` is appended
/// into the page's `/Resources` dictionary (use it for `/ColorSpace`
/// when a non-device fill is required).
fn build_pdf_image_mask(
    content_ops: &str,
    resources_extra: &str,
    width: u32,
    height: u32,
    mask_data: &[u8],
) -> Vec<u8> {
    build_pdf_image_mask_core(content_ops, resources_extra, width, height, mask_data, None)
}

/// Build a one-page PDF with an ImageMask XObject `/IM1` *and* an
/// indirect Type 4 tint-transform function as object 6. The Separation
/// colour space sits in the page's `/Resources /ColorSpace` dictionary
/// passed via `resources_extra` and references object 6.
fn build_pdf_image_mask_with_type4(
    content_ops: &str,
    resources_extra: &str,
    width: u32,
    height: u32,
    mask_data: &[u8],
    type4_program: &str,
) -> Vec<u8> {
    build_pdf_image_mask_core(
        content_ops,
        resources_extra,
        width,
        height,
        mask_data,
        Some(type4_program),
    )
}

/// Build a one-page PDF with a standard (non-mask) Image XObject `/IM1`.
/// `cs_name` selects the colour space; `pixel_bytes` is the raw image
/// data (BPC=8). Used by the standard-image pass-through pilot.
fn build_pdf_standard_image(
    content_ops: &str,
    width: u32,
    height: u32,
    pixel_bytes: &[u8],
    cs_name: &str,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    let page_off = buf.len();
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
          /Resources << /XObject << /IM1 5 0 R >> >> /Contents 4 0 R >>\nendobj\n",
    );

    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xobj_off = buf.len();
    let xobj_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} \
         /BitsPerComponent 8 /ColorSpace /{} /Length {} >>\nstream\n",
        width,
        height,
        cs_name,
        pixel_bytes.len()
    );
    buf.extend_from_slice(xobj_hdr.as_bytes());
    buf.extend_from_slice(pixel_bytes);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off, xobj_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

/// Build a one-page PDF with a Form XObject `/F1` that contains
/// `form_content_ops` as its child content stream. The page's content
/// stream is `page_content_ops` and typically issues `q ... cm /F1 Do
/// Q`. The form has a `/BBox` of [0 0 100 100] and no `/Matrix` (so it
/// inherits the parent transform).
fn build_pdf_form_xobject(page_content_ops: &str, form_content_ops: &str) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    let page_off = buf.len();
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
          /Resources << /XObject << /Fm1 5 0 R >> >> /Contents 4 0 R >>\nendobj\n",
    );

    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", page_content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(page_content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let form_off = buf.len();
    let form_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Resources << >> /Length {} >>\nstream\n",
        form_content_ops.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(form_content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off, form_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

/// Solid 1-bit stencil — every pixel is opaque under default `/Decode
/// [0 1]`. Per ISO 32000-1 §8.9.6.4, an ImageMask sample of 0 means
/// "paint with the current colour" and 1 means "leave the destination
/// unaffected" (the polarity is reversed by `/Decode [1 0]`). So an
/// all-zero stream paints the whole stencil opaque. Row stride is
/// byte-aligned per PDF §8.9.3.
fn solid_image_mask_bytes(width: u32, height: u32) -> Vec<u8> {
    let row_bytes = (width as usize).div_ceil(8);
    vec![0x00u8; row_bytes * height as usize]
}

// ---------- ImageMask rendering-correctness tests (Device* fills) ----------
//
// These probe `render_image_mask` for each Device-family fill colour
// space. They are NOT pipeline off-vs-on parity tests despite the
// surrounding test file's theme — the wave-3 routing makes
// `pipeline_resolve_paint_gs(ImageMask)` short-circuit on Device-family
// fills (D-3: resolved RGBA equals `gs.fill_color_rgb`, helper returns
// `None`), so both toggle states pass the same `gs` to
// `render_image_mask` and execute byte-identical code. The `assert_eq!`
// between the two renders pins that byte-identical output (a regression
// for `render_image_mask` itself, not for the pipeline machinery; the
// path-fill pilot at the top of the file is what proves the operator
// pipeline machinery byte-identity). The centre-pixel assertions pin
// the correctness of each Device-family fill's pixel output.

#[test]
fn pilot_image_mask_render_device_rgb_byte_identical() {
    // DeviceRGB fill on an ImageMask. `rg` writes `gs.fill_color_rgb`
    // directly; `render_image_mask` consumes that. Both toggle states
    // exercise the same path and produce a byte-identical pixmap.
    //
    // CTM: place a 100×100 stencil over the whole page (`100 0 0 100 0 0
    // cm`). Solid stencil → the page is fully painted red.
    let mask = solid_image_mask_bytes(8, 8);
    let content = "q\n1 0 0 rg\n100 0 0 100 0 0 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_image_mask(content, "", 8, 8, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "ImageMask DeviceRGB fill must produce a byte-identical pixmap");

    // Sanity: the centre pixel is red.
    let (r, g, b, _a) = center_pixel(&on);
    assert!(r > 200 && g < 60 && b < 60, "centre pixel must be red, got ({r}, {g}, {b})");
}

#[test]
fn pilot_image_mask_render_device_gray_byte_identical() {
    // DeviceGray fill on an ImageMask. `g` writes `gs.fill_color_rgb`
    // via the gray-to-RGB expansion (`(g, g, g)`); `render_image_mask`
    // consumes that. Both toggle states are byte-identical.
    let mask = solid_image_mask_bytes(8, 8);
    let content = "q\n0.25 g\n100 0 0 100 0 0 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_image_mask(content, "", 8, 8, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "ImageMask DeviceGray fill must produce a byte-identical pixmap");

    let (r, g, b, _a) = center_pixel(&on);
    // 0.25 g → grey ≈ 64. Allow a generous tolerance for resampling/blend.
    assert!(
        (50..=90).contains(&(r as i32)) && r == g && g == b,
        "centre pixel must be mid-grey, got ({r}, {g}, {b})"
    );
}

#[test]
fn pilot_image_mask_render_device_cmyk_byte_identical() {
    // DeviceCMYK fill on an ImageMask. `k` writes `gs.fill_color_rgb`
    // via the additive-clamp `cmyk_to_rgb`. Both toggle states are
    // byte-identical. Pure magenta (CMYK 0,1,0,0) → RGB(1,0,1).
    let mask = solid_image_mask_bytes(8, 8);
    let content = "q\n0 1 0 0 k\n100 0 0 100 0 0 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_image_mask(content, "", 8, 8, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "ImageMask DeviceCMYK fill must produce a byte-identical pixmap");
}

#[test]
fn pilot_image_mask_render_indexed_byte_identical() {
    // Indexed colour space — palette of 4 RGB triplets at indices 0..3.
    // Wave-3 reality: neither the inline `sc` branch (page_renderer.rs
    // ~:974) nor the pipeline performs the palette lookup yet — both
    // fall back to `index / 255` as a gray value (the inline branch was
    // wired to match the pipeline's `resolve_indexed` gray fallback so
    // the toggle agrees). Byte-identical output covers the
    // gray-fallback agreement.
    //
    // We pick a non-zero index (128) so the fallback produces a
    // distinctive mid-gray instead of near-black — that lets us pin
    // the gray-fallback *value*, which will fail loudly the day someone
    // implements real Indexed lookup in the resolver and forces this
    // test to migrate to a true palette-lookup assertion.
    let mask = solid_image_mask_bytes(8, 8);
    let palette = "<FF0000 00FF00 0000FF FFFFFF>";
    let resources = format!("/ColorSpace << /CS1 [/Indexed /DeviceRGB 3 {}] >>", palette);
    let content = "q\n/CS1 cs\n128 sc\n100 0 0 100 0 0 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_image_mask(content, &resources, 8, 8, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "ImageMask Indexed fill must produce a byte-identical pixmap");

    // Pin the gray-fallback value. `128 / 255 ≈ 0.502 → ≈128`. If real
    // palette lookup ever lands, index 128 is out of range for the
    // 4-entry palette and the assertion will fail — the right outcome:
    // the implementer must then either clamp to hival (index 3, blue)
    // or define the behaviour explicitly, and rewrite this test.
    let (r, g, b, _a) = center_pixel(&on);
    assert!(
        r == g && g == b && (118..=138).contains(&(r as i32)),
        "Indexed gray fallback must paint ≈ index/255 mid-gray; got ({r}, {g}, {b})"
    );
}

// ---------- ImageMask capability gain (Type 4 Separation) ----------

#[test]
fn pilot_image_mask_with_type4_separation_fill_pipeline_resolves_correctly() {
    // Wave 3 capability gain. The Separation colour space's tint
    // transform is a PostScript Type 4 function the inline path falls
    // back to `1 - tint` for, painting a full-tint stencil solid black.
    // The pipeline runs the Type 4 program and paints with the
    // function's actual output.
    //
    // Program: `{ 0.0 exch 0.0 0.0 }` — at tint=1.0 this leaves
    // CMYK(0, 1, 0, 0) → RGB(1, 0, 1) magenta on the stack.
    let mask = solid_image_mask_bytes(8, 8);
    let type4 = "{ 0.0 exch 0.0 0.0 }";
    let resources = "/ColorSpace << /SpotMagenta [/Separation /MagentaSpot /DeviceCMYK 6 0 R] >>";
    let content = "q\n/SpotMagenta cs\n1 scn\n100 0 0 100 0 0 cm\n/IM1 Do\nQ\n";

    let bytes = build_pdf_image_mask_with_type4(content, resources, 8, 8, &mask, type4);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);

    // Inline path: full-tint Type 4 → solid black at the centre.
    let (r_off, g_off, b_off, _a) = center_pixel(&off);
    assert!(
        r_off < 50 && g_off < 50 && b_off < 50,
        "inline path must paint ImageMask black under full-tint Type 4 Separation, got ({r_off}, {g_off}, {b_off})"
    );

    // Pipeline: magenta — pin specific channel values (positive
    // correctness, per the wave 1+2 lesson — "differs from black" is
    // not enough; the actual channel values must be the Type 4
    // function output).
    let (r_on, g_on, b_on, a_on) = center_pixel(&on);
    assert!(
        r_on >= 250 && g_on <= 5 && b_on >= 250 && a_on == 255,
        "pipeline must paint ImageMask magenta from the Type 4 program, got ({r_on}, {g_on}, {b_on}, {a_on})"
    );

    // And the pixmaps differ overall.
    assert_ne!(
        off, on,
        "pipeline output must differ from inline output for Type 4 Separation ImageMask"
    );
}

// ---------- Pass-through proofs ----------

#[test]
fn pilot_standard_image_xobject_pass_through_byte_identical() {
    // Standard (non-mask) Image XObjects carry their colour in pixel
    // data and do not interact with the pipeline. Even with the toggle
    // on, the output must be byte-identical to the toggle-off render.
    //
    // Pin: 4×4 DeviceGray image, all pixels = 0x80 (50% grey). Stretch
    // over 80×80 of the page so resampling artefacts are confined to a
    // small band, then sample inside the body.
    let pixels = vec![0x80u8; 16];
    let content = "q\n80 0 0 80 10 10 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_standard_image(content, 4, 4, &pixels, "DeviceGray");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "standard Image XObject must pass through pipeline byte-identically");

    // Pin a body pixel: well inside the 80x80 image footprint at (50, 50).
    let (r, g, b, _a) = pixel_at(&on, 50, 50);
    assert!(
        r == g && g == b && (120..=140).contains(&(r as i32)),
        "centre of standard image must be ≈50% grey, got ({r}, {g}, {b})"
    );
}

#[test]
fn pilot_form_xobject_pass_through_byte_identical() {
    // Form XObjects are recursively rendered through page_renderer —
    // their child operators flow through whatever pipeline arms have
    // landed at each wave. The `Do` arm itself must not splice for
    // Form XObjects (no fill-time colour resolve at the Do boundary).
    //
    // Build a Form whose child stream paints a 40×40 red rectangle via
    // an inline `f`. Toggle-off vs toggle-on must be byte-identical:
    // the child `f` IS pipeline-migrated (wave 1) but DeviceRGB is a
    // no-op splice, so the result holds.
    let page_ops = "q\n/Fm1 Do\nQ\n";
    let form_ops = "1 0 0 rg\n30 30 40 40 re\nf\n";
    let bytes = build_pdf_form_xobject(page_ops, form_ops);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Form XObject pass-through must be byte-identical off vs on");

    // Sanity: the form did paint something visible.
    let (r, g, b, _a) = center_pixel(&on);
    assert!(
        r > 200 && g < 60 && b < 60,
        "Form-XObject-painted rectangle must be red at centre, got ({r}, {g}, {b})"
    );
}

// ---------- State preservation under the pipeline ----------

#[test]
fn pilot_image_mask_under_pipeline_preserves_ctm() {
    // The `Do` arm reads `gs.ctm` to position the stencil. The pipeline's
    // splice clones `gs` to override `fill_color_rgb`; if it perturbed
    // `ctm` while cloning, the stencil would land at a different pixel
    // under the toggle.
    //
    // To make this probe meaningful, the fill must take the splice
    // branch — `pipeline_resolve_paint_gs(ImageMask)` short-circuits and
    // returns `None` for Device-family fills whose resolved RGBA already
    // matches `gs.fill_color_rgb` (D-3). We use a Type 4 Separation fill
    // so the resolver returns a *different* colour (magenta) than the
    // legacy `1 - tint = 0` black `gs` field, forcing the helper to
    // build and return `Some(spliced)`. The spliced clone is what
    // `render_image_mask` then reads `ctm` from.
    //
    // CTM: rotate ~30° around the origin, scale by 60, translate to
    // (50, 25). The unit square's centre (0.5, 0.5) lands in user space
    // at (50 + 51.96*0.5 + (-30)*0.5, 25 + 30*0.5 + 51.96*0.5) ≈
    // (60.98, 65.98). At 72 dpi and the renderer's top-left origin, the
    // pixmap coordinate is (61, 100 - 66) = (61, 34).
    let mask = solid_image_mask_bytes(8, 8);
    let type4 = "{ 0.0 exch 0.0 0.0 }"; // CMYK(0, tint, 0, 0) → magenta at tint=1.
    let resources = "/ColorSpace << /SpotMagenta [/Separation /MagentaSpot /DeviceCMYK 6 0 R] >>";
    let content = "q\n/SpotMagenta cs\n1 scn\n51.96 30 -30 51.96 50 25 cm\n/IM1 Do\nQ\n";

    let bytes = build_pdf_image_mask_with_type4(content, resources, 8, 8, &mask, type4);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);

    // Inline path: full-tint Type 4 fallback paints the stencil black.
    // Pin a body pixel that the rotated-scaled stencil covers — this is
    // the same target the pipeline branch hits, so a CTM divergence in
    // the splice would shift the magenta off this pixel.
    let (r_off, g_off, b_off, _a_off) = pixel_at(&off, 61, 34);
    assert!(
        r_off < 50 && g_off < 50 && b_off < 50,
        "inline path must paint rotated/scaled stencil black under full-tint Type 4, got ({r_off}, {g_off}, {b_off})"
    );

    // Pipeline path: same pixel, but magenta. Same-pixel hit proves the
    // spliced clone's CTM equals the original's — if the splice
    // perturbed `ctm`, the magenta would have landed somewhere else and
    // this pixel would be untouched (alpha 0 / background).
    let (r_on, g_on, b_on, a_on) = pixel_at(&on, 61, 34);
    assert!(
        r_on >= 250 && g_on <= 5 && b_on >= 250 && a_on == 255,
        "spliced clone's CTM must land the magenta stencil at the same rotated/scaled position the inline path painted black; got ({r_on}, {g_on}, {b_on}, {a_on})"
    );

    // And the pixmaps must differ overall — the splice did run.
    assert_ne!(
        off, on,
        "Type 4 Separation fill must drive a visible difference between toggle-off and toggle-on"
    );
}

#[test]
fn pilot_image_mask_under_pipeline_preserves_clip() {
    // An active `W n` clipping path constrains where the stencil can
    // paint. The pipeline splice must not drop the clip — the spliced
    // clone is `gs` only; the clip state lives on the operator walker's
    // clip stack, not on `gs`. If the splice somehow leaked into the
    // clip stack, pixels outside the clip box would also receive paint.
    //
    // As with the CTM probe above, the splice branch only runs when the
    // resolved fill differs from `gs.fill_color_rgb`, so we use a
    // Type 4 Separation fill — Device-family fills would short-circuit
    // out of the splice and skip the very code path under test.
    //
    // Clip to a 40×40 box at (30, 60)-(70, 100) in PDF user space —
    // that's the upper-right region of the page. Then paint a full-page
    // (100×100 cm) stencil. Only the clipped region must carry colour.
    // In pixmap coordinates (top-left origin), the clip box maps to
    // x ∈ [30, 70], pixmap_y ∈ [0, 40].
    let mask = solid_image_mask_bytes(8, 8);
    let type4 = "{ 0.0 exch 0.0 0.0 }";
    let resources = "/ColorSpace << /SpotMagenta [/Separation /MagentaSpot /DeviceCMYK 6 0 R] >>";
    let content = "q\n30 60 40 40 re W n\n/SpotMagenta cs\n1 scn\n100 0 0 100 0 0 cm\n/IM1 Do\nQ\n";

    let bytes = build_pdf_image_mask_with_type4(content, resources, 8, 8, &mask, type4);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);

    // Inside the clip box: pipeline paints magenta from the Type 4
    // program (50, 20) lands well inside the [30..70] × [0..40] region.
    let (r_in, g_in, b_in, a_in) = pixel_at(&on, 50, 20);
    assert!(
        r_in >= 250 && g_in <= 5 && b_in >= 250 && a_in == 255,
        "inside the clip box, pipeline must paint Type-4-evaluated magenta; got ({r_in}, {g_in}, {b_in}, {a_in})"
    );

    // Outside the clip box: pixel must be the white page background.
    // If the splice had leaked into the clip stack, the full-page
    // stencil would have painted this pixel magenta on top of the
    // background. Sample (10, 90) — far outside both axes of the clip
    // box.
    let (r_out, g_out, b_out, _a_out) = pixel_at(&on, 10, 90);
    assert!(
        r_out >= 250 && g_out >= 250 && b_out >= 250,
        "outside the clip box, the page must be the white background — splice must not perturb the clip stack; got ({r_out}, {g_out}, {b_out})"
    );

    // Inline path under the same clip: stencil paints black inside the
    // clip box (full-tint Type 4 fallback).
    let (r_off, g_off, b_off, a_off) = pixel_at(&off, 50, 20);
    assert!(
        r_off < 50 && g_off < 50 && b_off < 50 && a_off == 255,
        "inline path must paint the clipped stencil black under full-tint Type 4, got ({r_off}, {g_off}, {b_off}, {a_off})"
    );

    // And the pixmaps differ overall — the splice did run.
    assert_ne!(
        off, on,
        "Type 4 Separation fill must drive a visible difference between toggle-off and toggle-on"
    );
}

// =====================================================================
// Wave 4 — shading (`sh`) operator. The pipeline pre-resolves the two
// endpoint colours `/C0` and `/C1` of an axial (Type 2) or radial
// (Type 3) gradient through the resolution pipeline, then hands the
// resolved RGBA pair to the existing tiny-skia gradient builder. The
// interpolation math (linear / radial) is untouched.
//
// The brief lays out four categories of tests:
//
//   1. Parity for shading types where both paths agree (DeviceRGB,
//      DeviceGray Type-2 shading endpoints).
//   2. Capability gain — Type 4 Separation endpoints that the inline
//      `evaluate_shading_function` reads raw and the pipeline routes
//      through the tint transform.
//   3. State preservation under the splice (CTM, clip).
//   4. Pass-through proofs for shading types the pipeline does NOT
//      migrate (Type 1 function-based, Type 4 mesh).
// =====================================================================

/// Build a one-page PDF whose page resources carry a Type 2 axial
/// shading dictionary named `/Sh1`. The shading's `/ColorSpace` is
/// `space`, `/Coords` is `[x0 y0 x1 y1]`, and `/Function` is a Type 2
/// exponential interpolation with the supplied `c0` / `c1`
/// component arrays.
///
/// `extra_resources` is appended to the page's `/Resources` (e.g. to
/// declare a Separation colour space the shading references).
/// `extra_objects` is concatenated after the shading object and the
/// xref table accounts for it. Object numbering: 1 Catalog, 2 Pages,
/// 3 Page, 4 Content, 5 Shading, 6+ extra.
fn build_pdf_axial_shading(
    content_ops: &str,
    space_str: &str,
    coords: &str,
    c0: &str,
    c1: &str,
    extra_resources: &str,
    extra_objects: &[(usize, String)],
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
         /Resources << /Shading << /Sh1 5 0 R >> {} >> /Contents 4 0 R >>\nendobj\n",
        extra_resources
    );
    buf.extend_from_slice(page.as_bytes());

    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let shading_off = buf.len();
    let shading = format!(
        "5 0 obj\n<< /ShadingType 2 /ColorSpace {} /Coords {} /Domain [0 1] \
         /Function << /FunctionType 2 /Domain [0 1] /C0 {} /C1 {} /N 1 >> >>\nendobj\n",
        space_str, coords, c0, c1
    );
    buf.extend_from_slice(shading.as_bytes());

    let mut offsets = vec![cat_off, pages_off, page_off, stream_off, shading_off];
    for (_n, body) in extra_objects {
        offsets.push(buf.len());
        buf.extend_from_slice(body.as_bytes());
    }

    let xref_off = buf.len();
    let size = offsets.len() + 1;
    buf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", size).as_bytes());
    for off in &offsets {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", size, xref_off)
            .as_bytes(),
    );
    buf
}

/// Build a one-page PDF carrying a Type 3 radial shading. Same shape
/// as `build_pdf_axial_shading` but `/ShadingType 3` and `/Coords` is
/// six numbers (`[x0 y0 r0 x1 y1 r1]`).
fn build_pdf_radial_shading(
    content_ops: &str,
    space_str: &str,
    coords_6: &str,
    c0: &str,
    c1: &str,
    extra_resources: &str,
    extra_objects: &[(usize, String)],
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
         /Resources << /Shading << /Sh1 5 0 R >> {} >> /Contents 4 0 R >>\nendobj\n",
        extra_resources
    );
    buf.extend_from_slice(page.as_bytes());

    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let shading_off = buf.len();
    let shading = format!(
        "5 0 obj\n<< /ShadingType 3 /ColorSpace {} /Coords {} /Domain [0 1] \
         /Function << /FunctionType 2 /Domain [0 1] /C0 {} /C1 {} /N 1 >> >>\nendobj\n",
        space_str, coords_6, c0, c1
    );
    buf.extend_from_slice(shading.as_bytes());

    let mut offsets = vec![cat_off, pages_off, page_off, stream_off, shading_off];
    for (_n, body) in extra_objects {
        offsets.push(buf.len());
        buf.extend_from_slice(body.as_bytes());
    }

    let xref_off = buf.len();
    let size = offsets.len() + 1;
    buf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", size).as_bytes());
    for off in &offsets {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", size, xref_off)
            .as_bytes(),
    );
    buf
}

/// Build a single Type 4 PostScript function object string with the
/// given object number. Used as an `extra_object` for shadings whose
/// `/ColorSpace` is a `[/Separation ... funcRef]` array.
fn type4_function_object(obj_num: usize, program: &str) -> String {
    let body = format!(
        "{} 0 obj\n<< /FunctionType 4 /Domain [0 1] /Range [0 1 0 1 0 1 0 1] /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        obj_num,
        program.len(),
        program
    );
    body
}

// ---------- Parity: shading types both paths agree on ----------

#[test]
fn pilot_shading_type2_axial_device_rgb_parity_pipeline_off_vs_on() {
    // DeviceRGB axial shading — the inline path reads `/C0` and `/C1`
    // raw and treats the 3-element arrays as RGB triples; the
    // pipeline routes them through `LogicalColor::Device(Rgb)` and
    // returns the same RGB. Result: byte-identical pixmaps.
    //
    // Coords run a horizontal gradient across the page: from x=0 to
    // x=100 (PDF user space), C0=red on the left, C1=blue on the
    // right. `sh /Sh1` paints the whole pixmap; the centre should
    // come out roughly halfway between red and blue.
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_axial_shading(
        content,
        "/DeviceRGB",
        "[0 50 100 50]",
        "[1 0 0]",
        "[0 0 1]",
        "",
        &[],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "DeviceRGB Type 2 axial shading must be byte-identical off vs on");

    // Sanity: the centre pixel of a left-to-right red-to-blue
    // gradient should be roughly purple — non-trivial red and blue,
    // very little green.
    let (r, g, b, _a) = center_pixel(&on);
    assert!(
        r > 80 && b > 80 && g < 80,
        "axial gradient centre must be ~purple, got ({r}, {g}, {b})"
    );
}

#[test]
fn pilot_shading_type2_axial_device_gray_parity_pipeline_off_vs_on() {
    // DeviceGray axial shading. `/C0 [0]` and `/C1 [1]` are 1-element
    // arrays. The inline `parse_color_array` expands a one-element
    // array to `(g, g, g)`; the pipeline builds
    // `LogicalColor::Device(Gray(c))` which the resolver also expands
    // to `(g, g, g)`. Both must produce byte-identical output.
    let content = "/Sh1 sh\n";
    let bytes =
        build_pdf_axial_shading(content, "/DeviceGray", "[0 50 100 50]", "[0]", "[1]", "", &[]);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "DeviceGray Type 2 axial shading must be byte-identical off vs on");

    // Sanity: centre pixel of a black-to-white gradient ≈ mid-grey.
    let (r, g, b, _a) = center_pixel(&on);
    assert!(
        r == g && g == b && (100..=160).contains(&(r as i32)),
        "axial gray gradient centre must be mid-grey, got ({r}, {g}, {b})"
    );
}

#[test]
fn pilot_shading_type2_axial_device_cmyk_parity_pipeline_off_vs_on() {
    // DeviceCMYK Type 2 axial shading. The inline path reads `/C0`
    // (a 4-element CMYK array) raw and silently truncates to the
    // first three components, treating those as RGB. The pipeline
    // reads the shading dict's `/ColorSpace /DeviceCMYK` and routes
    // the 4 components through the spec's additive-clamp CMYK→RGB
    // conversion. These two evaluations DIVERGE for any non-trivial
    // CMYK colour — this is a capability gain for the pipeline, not a
    // parity case.
    //
    // To get a true PARITY pin against the inline truncation, we
    // exploit the fact that the resolver's CMYK→RGB on
    // `(c, m, y, 0)` produces `(1-c, 1-m, 1-y)`, while the inline
    // truncation reads the literal `(c, m, y)` triple. They coincide
    // only when `c = 1-c` (i.e. 0.5), `m = 1-m`, `y = 1-y`. Picking
    // `/C0 [0.5 0.5 0.5 0]` and `/C1 [0.5 0.5 0.5 0]` gives one
    // colour both paths agree on — a constant mid-grey gradient. The
    // resulting parity pin is narrow but unambiguous: any divergence
    // in how the pipeline maps DeviceCMYK at this specific colour
    // would surface as a pixel difference.
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_axial_shading(
        content,
        "/DeviceCMYK",
        "[0 50 100 50]",
        "[0.5 0.5 0.5 0]",
        "[0.5 0.5 0.5 0]",
        "",
        &[],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "DeviceCMYK Type 2 axial shading must be byte-identical off vs on \
         at the inline-truncation/pipeline-CMYK fixed point"
    );
}

#[test]
fn pilot_shading_type3_radial_device_rgb_parity_pipeline_off_vs_on() {
    // DeviceRGB radial shading. Same parity rationale as the axial
    // case: both paths fold the same 3-element RGB array into the
    // same `(r, g, b)` triple. Inner colour red, outer blue.
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_radial_shading(
        content,
        "/DeviceRGB",
        "[50 50 0 50 50 50]",
        "[1 0 0]",
        "[0 0 1]",
        "",
        &[],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "DeviceRGB Type 3 radial shading must be byte-identical off vs on");

    // Sanity: centre pixel sits at the inner-radius=0 colour (red).
    let (r, g, b, _a) = center_pixel(&on);
    assert!(
        r > 200 && g < 60 && b < 60,
        "radial gradient centre must be red (the C0 endpoint), got ({r}, {g}, {b})"
    );
}

// ---------- Capability gain — Type 4 Separation endpoint ----------

#[test]
fn pilot_shading_type2_axial_type4_separation_pipeline_resolves_correctly() {
    // The wave-4 capability gain. `/C0` is a 1-element Separation
    // component (full tint) whose tint transform is a PostScript
    // Type 4 calculator. The inline `evaluate_shading_function`
    // reads `/C0 [1]` as a 1-element array and expands it to
    // `(1, 1, 1)` (white), so the toggle-off gradient runs from white
    // to white-ish (C1 also white).
    //
    // The pipeline routes the component through the shading dict's
    // `/ColorSpace [/Separation /MagentaSpot /DeviceCMYK funcRef]`,
    // which evaluates the Type 4 program — at tint=1 the program
    // leaves CMYK(0, 1, 0, 0) → RGB(1, 0, 1) magenta on the stack.
    //
    // Probe location: the gradient runs horizontally across the page
    // from x=0 (C0 end) to x=100 (C1 end). The pixel at x=5 is well
    // inside the C0 endpoint region; tiny-skia's interpolation hasn't
    // moved meaningfully toward C1 yet, so the colour at x=5 should
    // be nearly pure magenta under the pipeline.
    let type4 = "{ 0.0 exch 0.0 0.0 }";
    let content = "/Sh1 sh\n";
    let space_str = "[/Separation /MagentaSpot /DeviceCMYK 6 0 R]";
    let func_obj = type4_function_object(6, type4);
    let bytes = build_pdf_axial_shading(
        content,
        space_str,
        "[0 50 100 50]",
        "[1]", // full tint
        "[1]", // C1 also full tint — keeps the endpoint analysis simple
        "",
        &[(6, func_obj)],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);

    // Inline path: `parse_color_array` on `[1]` returns `(1, 1, 1)`
    // (white). The whole page should be near-white.
    let (r_off, g_off, b_off, _a) = pixel_at(&off, 5, 50);
    assert!(
        r_off > 240 && g_off > 240 && b_off > 240,
        "inline path reads /C0 [1] as white; got ({r_off}, {g_off}, {b_off})"
    );

    // Pipeline path: the Type 4 program turns tint=1 into CMYK→RGB
    // magenta. Pin the actual channel values near the C0 endpoint
    // (positive correctness, not "differs from baseline").
    let (r_on, g_on, b_on, a_on) = pixel_at(&on, 5, 50);
    assert!(
        r_on >= 250 && g_on <= 5 && b_on >= 250 && a_on == 255,
        "pipeline must paint Type-4-evaluated magenta at the gradient C0 stop; \
         got ({r_on}, {g_on}, {b_on}, {a_on})"
    );

    // And the pixmaps differ overall — the splice did run.
    assert_ne!(
        off, on,
        "Type 4 Separation shading endpoint must drive a visible \
         pipeline-vs-inline difference"
    );
}

// ---------- State preservation ----------

#[test]
fn pilot_shading_type4_separation_under_pipeline_respects_user_space_ctm() {
    // Capability-gain probe under a non-identity user-space CTM. The
    // wave-4 splice only swaps the endpoint colours; it never touches
    // the graphics state (the helper takes `&GraphicsState`, not
    // `&mut`), so a true CTM perturbation isn't structurally
    // expressible. What this test actually verifies is that a Type 4
    // Separation `/C0` resolves to spec-correct magenta through the
    // pipeline under a non-identity CTM, while the inline path keeps
    // reading the raw `[1]` array as RGB white.
    //
    // CTM: scale by 50 and translate to (10, 10). The shading's
    // `/Coords [0 0 1 0]` maps the unit-x gradient from (10, 10) to
    // (60, 10) in user space. The pixel at user (30, 10) is roughly
    // 40% along the gradient — still close enough to C0 that the
    // resolved magenta dominates the interpolation toward C1=white.
    //
    // In pixmap coordinates the renderer flips Y, so user (30, 10)
    // lands at pixmap (30, 100 - 10) = (30, 90).
    let type4 = "{ 0.0 exch 0.0 0.0 }";
    let content = "q\n50 0 0 50 10 10 cm\n/Sh1 sh\nQ\n";
    let space_str = "[/Separation /MagentaSpot /DeviceCMYK 6 0 R]";
    let func_obj = type4_function_object(6, type4);
    let bytes = build_pdf_axial_shading(
        content,
        space_str,
        "[0 0 1 0]",
        "[1]", // C0 full tint → magenta under the pipeline
        "[0]", // C1 zero tint → white (Separation at 0 is the canvas)
        "",
        &[(6, func_obj)],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);

    // Sample in tiny-skia's `SpreadMode::Pad` region — to the left
    // of the C0 endpoint, where any pixel projecting to a negative
    // gradient-t is clamped to pure C0 regardless of how far away
    // it sits from the axis. The gradient axis maps from user
    // (10, 10) → (60, 10); pixmap (5, 95) corresponds to user
    // (5, 5), which projects to t < 0 and lands in the Pad region.
    // This isolates the C0 value cleanly (no interpolation toward
    // C1), so we can pin the resolved-magenta vs raw-RGB-white
    // divergence with tight thresholds.
    let (r_on, g_on, b_on, a_on) = pixel_at(&on, 5, 95);
    assert!(
        r_on >= 250 && g_on <= 5 && b_on >= 250 && a_on == 255,
        "under a non-identity CTM, the pipeline must paint the Pad-clamped \
         C0 region pure magenta (Type-4 evaluation of /C0 [1] under the \
         Separation space); got ({r_on}, {g_on}, {b_on}, {a_on})"
    );

    // The inline path reads /C0 [1] as `(1, 1, 1)` white, so the
    // same Pad-region pixel must be pure white.
    let (r_off, g_off, b_off, _) = pixel_at(&off, 5, 95);
    assert!(
        r_off >= 250 && g_off >= 250 && b_off >= 250,
        "inline path must paint the Pad-clamped C0 region pure white \
         (inline reads /C0 [1] as RGB white); got ({r_off}, {g_off}, {b_off})"
    );

    assert_ne!(off, on, "Type 4 Separation shading endpoint must drive a visible difference");
}

#[test]
fn pilot_shading_type4_separation_under_pipeline_paints_clipped_region() {
    // Capability-gain probe under an active `W n` clipping path.
    // The wave-4 splice only swaps endpoint colours; it never
    // perturbs the clip stack (the helper takes `&GraphicsState`,
    // not `&mut`), so a true clip perturbation isn't structurally
    // expressible. What this test actually verifies is that the
    // pipeline's Type-4-Separation endpoint paints magenta only
    // inside the clip box and leaves the page background untouched
    // outside it.
    //
    // Clip: [30 60 70 100] in user space (the upper-right quarter).
    // Pixmap (top-left origin): x ∈ [30, 70], y ∈ [0, 40].
    let type4 = "{ 0.0 exch 0.0 0.0 }";
    let content = "q\n30 60 40 40 re W n\n/Sh1 sh\nQ\n";
    let space_str = "[/Separation /MagentaSpot /DeviceCMYK 6 0 R]";
    let func_obj = type4_function_object(6, type4);
    let bytes = build_pdf_axial_shading(
        content,
        space_str,
        "[0 50 100 50]", // gradient axis runs left-right across page
        "[1]",
        "[1]",
        "",
        &[(6, func_obj)],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);

    // Inside the clip box: pipeline paints magenta. Sample (50, 20).
    let (r_in, g_in, b_in, a_in) = pixel_at(&on, 50, 20);
    assert!(
        r_in >= 250 && g_in <= 5 && b_in >= 250 && a_in == 255,
        "inside the clip box, the pipeline must paint Type-4-evaluated \
         magenta; got ({r_in}, {g_in}, {b_in}, {a_in})"
    );

    // Outside the clip box: page must be the white background.
    let (r_out, g_out, b_out, _) = pixel_at(&on, 10, 90);
    assert!(
        r_out >= 250 && g_out >= 250 && b_out >= 250,
        "outside the clip box, the page must be the white background; \
         got ({r_out}, {g_out}, {b_out})"
    );

    // Pipeline must differ from inline for this Type 4 Separation
    // case — confirms the splice fired at all.
    assert_ne!(off, on, "Type 4 Separation shading must differ off vs on");

    // Inline path inside the clip: reads C0=[1] as white, so the
    // gradient runs white-to-white; the clipped patch is near-white.
    let (r_off, g_off, b_off, _) = pixel_at(&off, 50, 20);
    assert!(
        r_off > 240 && g_off > 240 && b_off > 240,
        "inline path inside the clip must be near-white (C0 read raw as RGB white); \
         got ({r_off}, {g_off}, {b_off})"
    );
}

// ---------- Pass-through proofs (Types 1 and 4 — not migrated) ----------

/// Build a Type 1 (function-based) shading PDF. The shading dict's
/// `/ColorSpace` is DeviceRGB and a Type 2 function maps `(x, y)` →
/// RGB. The pipeline does NOT migrate Type 1 — the dispatcher's
/// `if shading_type == 2 || shading_type == 3` gate keeps it on the
/// inline path. Even if the inline path can't actually paint a Type 1
/// shading (`render_shading` falls into the catch-all log-only arm),
/// the byte-equal pin holds: both toggle states do the same thing.
fn build_pdf_type1_shading(content_ops: &str) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    let page_off = buf.len();
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
          /Resources << /Shading << /Sh1 5 0 R >> >> /Contents 4 0 R >>\nendobj\n",
    );

    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    // Type 1: function-based shading. `/Domain` is `[xmin xmax ymin
    // ymax]`; `/Function` produces N component values per (x, y).
    let shading_off = buf.len();
    buf.extend_from_slice(
        b"5 0 obj\n<< /ShadingType 1 /ColorSpace /DeviceRGB /Domain [0 1 0 1] \
          /Function << /FunctionType 2 /Domain [0 1] /C0 [1 0 0] /C1 [0 1 0] /N 1 >> >>\nendobj\n",
    );

    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off, shading_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

/// Build a Type 4 (free-form Gouraud-shaded triangle mesh) shading
/// stream — a minimal shape the parser can accept even if the
/// dispatcher refuses to paint it. The point isn't to render
/// triangles; it's to prove the wave-4 dispatcher gate ignores
/// `/ShadingType 4` and the toggle is byte-identical on or off.
fn build_pdf_type4_mesh_shading(content_ops: &str) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    let page_off = buf.len();
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
          /Resources << /Shading << /Sh1 5 0 R >> >> /Contents 4 0 R >>\nendobj\n",
    );

    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    // Type 4 shading: empty mesh stream — payload length 0. The
    // parser accepts the dictionary shape; the renderer's
    // `render_shading` catch-all logs and does nothing for type 4.
    let shading_off = buf.len();
    let shading = b"5 0 obj\n<< /ShadingType 4 /ColorSpace /DeviceRGB /BitsPerCoordinate 8 \
                   /BitsPerComponent 8 /BitsPerFlag 8 /Decode [0 100 0 100 0 1 0 1 0 1] \
                   /Length 0 >>\nstream\n\nendstream\nendobj\n";
    buf.extend_from_slice(shading);

    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off, shading_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

#[test]
fn pilot_shading_type1_function_based_pass_through() {
    // Type 1 shading — pipeline does NOT migrate. The dispatcher's
    // `shading_type == 2 || shading_type == 3` gate sends Type 1
    // straight to `render_shading`'s catch-all (which logs and does
    // nothing). Toggle-off and toggle-on must render the same
    // page background.
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_type1_shading(content);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "Type 1 function-based shading must be byte-identical under both toggle states"
    );
}

#[test]
fn pilot_shading_type4_mesh_pass_through() {
    // Type 4 mesh shading — pipeline does NOT migrate. Same pass-
    // through pin as Type 1: the dispatcher refuses to splice and
    // the inline render falls through the catch-all.
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_type4_mesh_shading(content);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");

    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "Type 4 mesh shading must be byte-identical under both toggle states");
}
