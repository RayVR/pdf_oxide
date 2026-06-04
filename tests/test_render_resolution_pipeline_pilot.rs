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

    // Inline stroke path falls back to the SCN gray clamp (no Separation
    // handling on the stroke side at all today). The point is the
    // declared magenta is NOT what gets painted.
    let (r_off, g_off, b_off, _a) = pixel_at(&off, 50, 80);
    let inline_is_magenta = r_off > 200 && g_off < 60 && b_off > 200;
    assert!(
        !inline_is_magenta,
        "inline stroke path must NOT render the declared magenta for a Type 4 Separation, got ({r_off}, {g_off}, {b_off})"
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
