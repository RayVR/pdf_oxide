//! Pins ISO 32000-1 §11.3.5.3 non-separable blend modes
//! (`Hue`, `Saturation`, `Color`, `Luminosity`) in the page renderer.
//!
//! Until this commit `pdf_blend_mode_to_skia` silently degraded all four
//! to `SourceOver` — a layer rendered with `BM=Luminosity` over a colored
//! backdrop would lose its blend math and simply overwrite. The four
//! modes are native variants in `tiny_skia::BlendMode` since 0.12, so
//! the fix is a four-arm match-table extension. This test asserts the
//! mapping yields a *non-Normal* blend result for `BM=Luminosity`.

#![cfg(feature = "rendering")]

use pdf_oxide::rendering::{render_page, RenderOptions};
use pdf_oxide::PdfDocument;

fn finalize_pdf(mut buf: Vec<u8>, offsets: Vec<usize>) -> Vec<u8> {
    let xref_offset = buf.len();
    buf.extend_from_slice(b"xref\n");
    buf.extend_from_slice(format!("0 {}\n", offsets.len() + 1).as_bytes());
    buf.extend_from_slice(b"0000000000 65535 f \n");
    for off in &offsets {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            offsets.len() + 1,
            xref_offset
        )
        .as_bytes(),
    );
    buf
}

/// Build a PDF that paints a pure-red full-page rectangle, then paints a
/// pure-blue full-page rectangle on top using ExtGState `/BM /Luminosity`.
///
/// Under `SourceOver` (the silent-fallback bug) the blue overwrites the
/// red entirely, so the resulting pixel is RGB ≈ (0, 0, 255).
///
/// Under proper `Luminosity` semantics, the *luminance* of the source
/// (blue, Y ≈ 29) replaces the destination's luminance while keeping the
/// destination's hue (red). The composited pixel ends up as a very dark
/// red — high R relative to G/B, but each channel small enough that the
/// SourceOver outcome (huge B, zero R) is impossible.
fn build_pdf_with_luminosity_blend() -> Vec<u8> {
    let page_content = b"q\n1 0 0 rg\n0 0 100 100 re\nf\n/GS1 gs\n0 0 1 rg\n0 0 100 100 re\nf\nQ\n";

    let mut buf = Vec::new();
    let mut offsets = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    offsets.push(buf.len());
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
           /Contents 4 0 R \
           /Resources << /ExtGState << /GS1 5 0 R >> >> \
           /Group << /Type /Group /S /Transparency >> >>\nendobj\n",
    );
    offsets.push(buf.len());
    let hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", page_content.len());
    buf.extend_from_slice(hdr.as_bytes());
    buf.extend_from_slice(page_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"5 0 obj\n<< /Type /ExtGState /BM /Luminosity >>\nendobj\n");

    finalize_pdf(buf, offsets)
}

fn decode_png(bytes: &[u8]) -> image::RgbaImage {
    let cursor = std::io::Cursor::new(bytes);
    image::load(cursor, image::ImageFormat::Png)
        .expect("decode PNG")
        .to_rgba8()
}

#[test]
fn luminosity_blend_mode_does_not_overwrite_with_source() {
    let doc = PdfDocument::from_bytes(build_pdf_with_luminosity_blend()).expect("parse");
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    let rgba = decode_png(&img.data);

    let centre = rgba.get_pixel(50, 50);
    // SourceOver fallback would give B ≈ 255 (the blue source overwrites
    // the red backdrop). Luminosity must NOT produce that — it keeps
    // some hue/saturation of the red backdrop.
    assert!(
        centre[2] < 200,
        "Luminosity blend collapsed to SourceOver (blue overwrote red); \
         got R={} G={} B={} A={}",
        centre[0],
        centre[1],
        centre[2],
        centre[3]
    );
}
