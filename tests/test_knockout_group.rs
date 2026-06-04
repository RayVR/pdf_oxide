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
    let form_content = b"/GS1 gs\n1 0 0 rg\n0 0 100 100 re\nf\n0 0 1 rg\n0 0 100 100 re\nf\n";

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
/// Build a PDF whose knockout group paints a fully opaque red rect then a
/// fully opaque blue rect with `BM=Multiply`. The multiply blend reads
/// the destination, so the spec-correct outcome differs by which
/// destination is used: knockout uses the group's initial backdrop
/// (transparent), non-knockout uses the prior paint (the red).
///
/// `knockout` toggles `/K`. `BM=Multiply` is set via an ExtGState; both
/// paints stay at `ca = 1.0` so the alpha short-circuit would (wrongly)
/// fire if not gated on the blend mode.
fn build_pdf_with_opaque_multiply_in_group(knockout: bool) -> Vec<u8> {
    let page_content = b"/F1 Do\n";
    // /GS1 sets BM=Multiply only on the second fill; first fill is normal.
    let form_content = b"1 0 0 rg\n0 0 100 100 re\nf\n/GS1 gs\n0 0 1 rg\n0 0 100 100 re\nf\n";

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
    buf.extend_from_slice(b"6 0 obj\n<< /Type /ExtGState /BM /Multiply >>\nendobj\n");

    finalize_pdf(buf, offsets)
}

#[test]
fn knockout_group_opaque_non_normal_blend_redirects_to_backdrop() {
    let ko = decode_png(
        &render_page(
            &PdfDocument::from_bytes(build_pdf_with_opaque_multiply_in_group(true)).expect("parse"),
            0,
            &RenderOptions::with_dpi(72),
        )
        .expect("render")
        .data,
    );
    let nk = decode_png(
        &render_page(
            &PdfDocument::from_bytes(build_pdf_with_opaque_multiply_in_group(false))
                .expect("parse"),
            0,
            &RenderOptions::with_dpi(72),
        )
        .expect("render")
        .data,
    );

    let ko_c = ko.get_pixel(50, 50);
    let nk_c = nk.get_pixel(50, 50);

    // Knockout: blue Multiply against transparent backdrop on a white page
    // ends up pure blue — no red contribution.
    // Non-knockout: blue Multiply against the prior red layer = (255*0,
    // 0*0, 0*255) = (0, 0, 0) compositied over white → mostly black.
    // The two must diverge — the previous alpha short-circuit (which
    // fired on ca = 1.0 regardless of blend mode) silently dropped the
    // knockout dance and produced identical output for both.
    assert!(
        (ko_c[0] as i32 - nk_c[0] as i32).abs() > 30
            || (ko_c[1] as i32 - nk_c[1] as i32).abs() > 30
            || (ko_c[2] as i32 - nk_c[2] as i32).abs() > 30,
        "Opaque non-Normal blend must redirect to backdrop in knockout group; \
         knockout {ko_c:?} matched non-knockout {nk_c:?} (alpha short-circuit fired)"
    );
}

/// Build a fixture whose knockout group paints `extra_setup` (a content
/// stream snippet) followed by a 50%-alpha blue rect. When `extra_setup`
/// is empty, only the blue is painted; when it contains a 50%-alpha red
/// rect, the knockout semantics demand that the blue's contribution be
/// *identical* either way — the red is fully knocked out where the blue
/// covers.
fn build_pdf_knockout_extra_then_blue(extra_setup: &str) -> Vec<u8> {
    let page_content = b"/F1 Do\n";
    let form_content =
        format!("/GS1 gs\n{extra_setup}0 0 1 rg\n0 0 100 100 re\nf\n", extra_setup = extra_setup,);
    let form_content = form_content.as_bytes();

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

    offsets.push(buf.len());
    let form_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency /I true /K true >> \
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

/// Pixel-exact knockout invariant (§11.6.6.2): in a knockout group, a
/// prior shape that the later shape completely covers leaves *no trace*
/// in the final composited output. Render two scenes that differ only in
/// whether the red layer is present; assert every pixel in the overlap
/// region is byte-identical.
///
/// A half-implemented knockout (e.g. reset on first paint only, or merge
/// that uses the accumulating buffer instead of the backdrop) produces
/// channel deltas under the +30 thresholds elsewhere; byte equality does
/// not let those through.
#[test]
fn knockout_group_pixel_exact_replacement() {
    let with_red = decode_png(
        &render_page(
            &PdfDocument::from_bytes(build_pdf_knockout_extra_then_blue(
                "1 0 0 rg\n0 0 100 100 re\nf\n",
            ))
            .expect("parse"),
            0,
            &RenderOptions::with_dpi(72),
        )
        .expect("render")
        .data,
    );
    let only_blue = decode_png(
        &render_page(
            &PdfDocument::from_bytes(build_pdf_knockout_extra_then_blue("")).expect("parse"),
            0,
            &RenderOptions::with_dpi(72),
        )
        .expect("render")
        .data,
    );

    // The two scenes must produce byte-identical output everywhere the
    // blue covers (the full page in this fixture). One mismatched pixel
    // anywhere = a knockout bug.
    let mut mismatches = 0;
    for y in 0..100 {
        for x in 0..100 {
            if with_red.get_pixel(x, y) != only_blue.get_pixel(x, y) {
                mismatches += 1;
            }
        }
    }
    assert_eq!(
        mismatches,
        0,
        "Knockout must replace covered prior shapes pixel-exactly; \
         {mismatches} pixels differ between with-red and only-blue scenes. \
         Sample at (50, 50): with_red={:?}, only_blue={:?}",
        with_red.get_pixel(50, 50),
        only_blue.get_pixel(50, 50)
    );
}

/// Same fixture as `knockout_group_opaque_non_normal_blend_redirects_to_backdrop`
/// but parameterised across all four non-separable blend modes. The
/// `knockout_paint_alpha` helper returns 0.0 for any non-Normal mode and
/// the backdrop-redirect dance must fire for each. A bug that only
/// matches some mode names — e.g. lowercasing or substring matching
/// "Multiply" but missing "Hue" — fails closed here.
fn build_pdf_with_opaque_blend_in_knockout(blend_mode: &str, knockout: bool) -> Vec<u8> {
    let page_content = b"/F1 Do\n";
    let form_content = b"1 0 0 rg\n0 0 100 100 re\nf\n/GS1 gs\n0 0 1 rg\n0 0 100 100 re\nf\n";

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
    let ext = format!("6 0 obj\n<< /Type /ExtGState /BM /{blend_mode} >>\nendobj\n");
    buf.extend_from_slice(ext.as_bytes());

    finalize_pdf(buf, offsets)
}

#[test]
fn knockout_redirects_all_four_non_separable_blend_modes() {
    for mode in ["Hue", "Saturation", "Color", "Luminosity"] {
        let ko = decode_png(
            &render_page(
                &PdfDocument::from_bytes(build_pdf_with_opaque_blend_in_knockout(mode, true))
                    .expect("parse"),
                0,
                &RenderOptions::with_dpi(72),
            )
            .expect("render")
            .data,
        );
        let nk = decode_png(
            &render_page(
                &PdfDocument::from_bytes(build_pdf_with_opaque_blend_in_knockout(mode, false))
                    .expect("parse"),
                0,
                &RenderOptions::with_dpi(72),
            )
            .expect("render")
            .data,
        );
        let ko_c = ko.get_pixel(50, 50);
        let nk_c = nk.get_pixel(50, 50);
        let diverged = (ko_c[0] as i32 - nk_c[0] as i32).abs() > 20
            || (ko_c[1] as i32 - nk_c[1] as i32).abs() > 20
            || (ko_c[2] as i32 - nk_c[2] as i32).abs() > 20;
        assert!(
            diverged,
            "BM={mode}: opaque non-Normal blend must redirect to backdrop in \
             knockout group; knockout {ko_c:?} matched non-knockout {nk_c:?}"
        );
    }
}

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
