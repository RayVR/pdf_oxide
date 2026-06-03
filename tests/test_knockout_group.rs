//! Knockout transparency-group tests (ISO 32000-1 §11.6.6.2).
//!
//! In a knockout group each painted element composites against the group's
//! *initial backdrop* rather than against the accumulating result of
//! preceding paints. Where shapes overlap, the later shape entirely
//! replaces the earlier — no blend math, no carry-forward.
//!
//! Implementation: per-paint-operator backdrop redirect, gated on
//! `effective_alpha < 1.0` so fully opaque paints short-circuit to the
//! normal paint path (visually identical to non-knockout for opaque
//! content; spec-aligned).

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

fn decode_png(bytes: &[u8]) -> image::RgbaImage {
    let cursor = std::io::Cursor::new(bytes);
    image::load(cursor, image::ImageFormat::Png)
        .expect("decode PNG")
        .to_rgba8()
}

/// Build a PDF whose page invokes a Form XObject. The form is an isolated
/// transparency group with `/K knockout`; its content paints a 50%-alpha
/// red rectangle followed by a 50%-alpha blue rectangle covering the same
/// area. With knockout, the blue replaces the red (composited against the
/// group's transparent backdrop); without knockout, the blue blends with
/// the red and the centre pixel retains a red component.
///
/// `knockout` controls the `/K` flag emitted on the form's `/Group` dict
/// so the same fixture can produce both the "with" and "without" cases.
fn build_pdf_with_overlapping_translucent_rects(knockout: bool) -> Vec<u8> {
    let page_content = b"/F1 Do\n";
    let form_content =
        b"/GS1 gs\n1 0 0 rg\n0 0 100 100 re\nf\n0 0 1 rg\n0 0 100 100 re\nf\n";

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
           /Resources << /XObject << /F1 5 0 R >> >> \
           /Group << /Type /Group /S /Transparency >> >>\nendobj\n",
    );
    offsets.push(buf.len());
    let hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", page_content.len());
    buf.extend_from_slice(hdr.as_bytes());
    buf.extend_from_slice(page_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    // Form XObject with /K knockout (when `knockout` is true) and an
    // ExtGState resource for the half-alpha fill.
    offsets.push(buf.len());
    let k_entry = if knockout { " /K true" } else { "" };
    let form_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency /I true{k_entry} >> \
         /Resources << /ExtGState << /GS1 6 0 R >> >> /Length {} >>\nstream\n",
        form_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(form_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    offsets.push(buf.len());
    buf.extend_from_slice(b"6 0 obj\n<< /Type /ExtGState /ca 0.5 >>\nendobj\n");

    finalize_pdf(buf, offsets)
}

/// Two 50%-alpha overlapping rectangles in a knockout group: the blue
/// fill must replace the red fill rather than blending with it. The
/// centre pixel should therefore have very little red — significantly
/// less than the same scene without `/K`.
#[test]
fn knockout_group_blue_replaces_red() {
    let knockout = render_page(
        &PdfDocument::from_bytes(build_pdf_with_overlapping_translucent_rects(true))
            .expect("parse"),
        0,
        &RenderOptions::with_dpi(72),
    )
    .expect("render");
    let blend = render_page(
        &PdfDocument::from_bytes(build_pdf_with_overlapping_translucent_rects(false))
            .expect("parse"),
        0,
        &RenderOptions::with_dpi(72),
    )
    .expect("render");
    let ko = decode_png(&knockout.data);
    let nb = decode_png(&blend.data);

    let ko_centre = ko.get_pixel(50, 50);
    let nb_centre = nb.get_pixel(50, 50);

    // With knockout the centre is pure-blue-over-white: roughly
    // (127, 127, 255). Without knockout the red layer blends in and pulls
    // green and blue down: roughly (127, 63, 191). The R channel
    // coincidentally lands at 127 in both because white's R = 255 is the
    // dominant source over a 50%-alpha blue tip; the meaningful
    // distinguishers are G and B.
    assert!(
        (ko_centre[1] as i32) > (nb_centre[1] as i32) + 30,
        "Knockout should leave more green from the white background \
         (expect G_ko ≳ G_nb + 30): knockout {ko_centre:?} vs non-knockout {nb_centre:?}"
    );
    assert!(
        (ko_centre[2] as i32) > (nb_centre[2] as i32) + 30,
        "Knockout should leave the blue layer more saturated \
         (expect B_ko ≳ B_nb + 30): knockout {ko_centre:?} vs non-knockout {nb_centre:?}"
    );
}
