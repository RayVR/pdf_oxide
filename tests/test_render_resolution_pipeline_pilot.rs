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
