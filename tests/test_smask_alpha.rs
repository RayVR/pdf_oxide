//! ExtGState soft-mask (`/SMask /S /Alpha`) tests for the page renderer.
//!
//! ISO 32000-1 §11.6.5.2: an ExtGState `/SMask` modulates subsequent paint
//! operations through a transparency group rendered into its own buffer.
//! For subtype `/Alpha` the *alpha channel* of the rendered group is the
//! mask — opaque (α = 1) pixels in the group let paint through, transparent
//! (α = 0) pixels block it.
//!
//! Test plan: render a synthetic PDF whose SMask group paints an opaque
//! rectangle in the top half of the page in PDF user space. With the mask
//! correctly applied, a full-page black fill under that ExtGState produces
//! black in the top half of the rendered image and the white background in
//! the bottom half. Without SMask handling the mask is ignored and the
//! whole image is black.

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

/// Build a 100×100 PDF that paints a black full-page rectangle through an
/// ExtGState whose `/SMask` group fills `0 50 100 50` (the top half in PDF
/// user space). Mask subtype is `/S /Alpha`.
fn build_pdf_with_alpha_smask() -> Vec<u8> {
    let page_content = b"q\n/GS1 gs\n0 g\n0 0 100 100 re\nf\nQ\n";
    let group_content = b"0 50 100 50 re\nf\n";

    let mut buf = Vec::new();
    let mut offsets = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    // 1: Catalog
    offsets.push(buf.len());
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // 2: Pages
    offsets.push(buf.len());
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    // 3: Page (with a /Group dict so the page is a transparency root)
    offsets.push(buf.len());
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
           /Contents 4 0 R \
           /Resources << /ExtGState << /GS1 5 0 R >> >> \
           /Group << /Type /Group /S /Transparency >> >>\nendobj\n",
    );

    // 4: Page content stream
    offsets.push(buf.len());
    let hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", page_content.len());
    buf.extend_from_slice(hdr.as_bytes());
    buf.extend_from_slice(page_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    // 5: ExtGState referencing the soft-mask dict
    offsets.push(buf.len());
    buf.extend_from_slice(b"5 0 obj\n<< /Type /ExtGState /SMask 6 0 R >>\nendobj\n");

    // 6: Soft-mask dict (/S /Alpha, /G is the form XObject)
    offsets.push(buf.len());
    buf.extend_from_slice(b"6 0 obj\n<< /Type /Mask /S /Alpha /G 7 0 R >>\nendobj\n");

    // 7: Form XObject (transparency group) — paints opaque black in top half
    offsets.push(buf.len());
    let form_hdr = format!(
        "7 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency >> \
         /Resources << >> /Length {} >>\nstream\n",
        group_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(group_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    finalize_pdf(buf, offsets)
}

fn decode_png(bytes: &[u8]) -> image::RgbaImage {
    let cursor = std::io::Cursor::new(bytes);
    image::load(cursor, image::ImageFormat::Png)
        .expect("decode rendered PNG")
        .to_rgba8()
}

/// Build a PDF that paints two black rectangles. The first is painted under
/// `/GS1` (alpha mask: top half opaque, bottom half transparent). The
/// second is painted after `/SMask /None` clears the mask — so the second
/// fill should land everywhere regardless of what the first mask blocked.
fn build_pdf_with_smask_then_none() -> Vec<u8> {
    let page_content =
        b"q\n/GS1 gs\n0 g\n0 0 100 100 re\nf\n/GS2 gs\n0 0 100 50 re\nf\nQ\n";
    let group_content = b"0 50 100 50 re\nf\n";

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
           /Resources << /ExtGState << /GS1 5 0 R /GS2 8 0 R >> >> \
           /Group << /Type /Group /S /Transparency >> >>\nendobj\n",
    );
    offsets.push(buf.len());
    let hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", page_content.len());
    buf.extend_from_slice(hdr.as_bytes());
    buf.extend_from_slice(page_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"5 0 obj\n<< /Type /ExtGState /SMask 6 0 R >>\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"6 0 obj\n<< /Type /Mask /S /Alpha /G 7 0 R >>\nendobj\n");
    offsets.push(buf.len());
    let form_hdr = format!(
        "7 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency >> \
         /Resources << >> /Length {} >>\nstream\n",
        group_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(group_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"8 0 obj\n<< /Type /ExtGState /SMask /None >>\nendobj\n");

    finalize_pdf(buf, offsets)
}

#[test]
fn ext_gstate_smask_none_clears_active_soft_mask() {
    let doc = PdfDocument::from_bytes(build_pdf_with_smask_then_none()).expect("parse");
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    let rgba = decode_png(&img.data);

    // First fill (under /GS1's mask) lands in the top half — PNG rows
    // 0..50. Without the first mask installing at all (the regression mode
    // the standalone basic test guards against), the FIRST fill would have
    // painted the whole page black and the post-/None second fill would be
    // a no-op — the bottom-half assertion alone wouldn't distinguish that
    // failure from the correct path.
    let top = rgba.get_pixel(50, 25);
    assert!(
        top[0] < 60,
        "first fill under /GS1 must still land on the top half; \
         got R={} G={} B={} A={}",
        top[0], top[1], top[2], top[3]
    );

    // After `/SMask /None`, the second fill (PDF y = 0..50, the bottom half
    // in user space — PNG rows 50..100) lands. Without /None handling the
    // first mask would still be active and the bottom half would stay white.
    let bottom = rgba.get_pixel(50, 75);
    assert!(
        bottom[0] < 60,
        "after /SMask /None, second fill should paint the bottom half; \
         got R={} G={} B={} A={}",
        bottom[0], bottom[1], bottom[2], bottom[3]
    );
}

/// Same fixture as the basic alpha test, but the page paints inside a
/// nested `q`/`Q` block AND paints a second rectangle after the `Q`.
/// The soft mask installed before `q` must be in effect inside, and must
/// still be in effect after `Q` (because the stack pop must not lose the
/// outer-level mask). Pins the soft-mask stack push/pop in lockstep with
/// the clip stack.
fn build_pdf_with_smask_through_q_save_restore() -> Vec<u8> {
    let page_content = b"/GS1 gs\nq\n0 g\n0 0 100 100 re\nf\nQ\n0 0 100 100 re\nf\n";
    let group_content = b"0 50 100 50 re\nf\n";

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
    buf.extend_from_slice(b"5 0 obj\n<< /Type /ExtGState /SMask 6 0 R >>\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"6 0 obj\n<< /Type /Mask /S /Alpha /G 7 0 R >>\nendobj\n");
    offsets.push(buf.len());
    let form_hdr = format!(
        "7 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency >> \
         /Resources << >> /Length {} >>\nstream\n",
        group_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(group_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    finalize_pdf(buf, offsets)
}

#[test]
fn ext_gstate_smask_survives_q_save_restore() {
    let doc = PdfDocument::from_bytes(build_pdf_with_smask_through_q_save_restore()).expect("parse");
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    let rgba = decode_png(&img.data);

    // Same expectation as the basic test: the mask installed before `q`
    // applies to the paint inside the `q`/`Q` block.
    let top = rgba.get_pixel(50, 25);
    let bottom = rgba.get_pixel(50, 75);
    assert!(top[0] < 60, "top under mask should be black; got {top:?}");
    assert!(bottom[0] > 200, "bottom under mask should be white; got {bottom:?}");
}

/// Build a fixture where the page applies a scale CTM *before* installing
/// the SMask. The form's `/BBox` is small (`0..2`) but a `50 0 0 50 0 0 cm`
/// runs before `/GS1 gs`, so per §11.6.5.2 the mask group must be rendered
/// at the CTM that was current at install time — i.e. scaled 50× so the
/// form's `0..2` BBox spans `0..100` device pixels and covers the whole
/// painted area. Without that, the mask renders at identity into a 2×2-px
/// region and blocks 99% of the paint.
fn build_pdf_with_smask_under_active_ctm() -> Vec<u8> {
    let page_content = b"q\n50 0 0 50 0 0 cm\n/GS1 gs\n0 g\n0 0 2 2 re\nf\nQ\n";
    let group_content = b"0 1 2 1 re\nf\n";

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
    buf.extend_from_slice(b"5 0 obj\n<< /Type /ExtGState /SMask 6 0 R >>\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"6 0 obj\n<< /Type /Mask /S /Alpha /G 7 0 R >>\nendobj\n");
    offsets.push(buf.len());
    let form_hdr = format!(
        "7 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 2 2] \
         /Group << /Type /Group /S /Transparency >> \
         /Resources << >> /Length {} >>\nstream\n",
        group_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(group_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    finalize_pdf(buf, offsets)
}

/// Adversarial fixture: the SMask `/G` form's `/Resources` declares the same
/// `/GS1` ExtGState (which has the same `/SMask /G` pointing back at the
/// form), and the form's content stream invokes `/GS1 gs`. Every gs in the
/// chain triggers another mask rasterisation, which renders the form again,
/// which invokes gs again. Without a depth cap this stack-overflows.
fn build_pdf_with_cyclic_smask() -> Vec<u8> {
    // Page content paints a black square through /GS1.
    let page_content = b"q\n/GS1 gs\n0 g\n0 0 100 100 re\nf\nQ\n";
    // Form's content paints the top half *and* re-invokes /GS1, which
    // closes the cycle.
    let group_content = b"/GS1 gs\n0 50 100 50 re\nf\n";

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
    buf.extend_from_slice(b"5 0 obj\n<< /Type /ExtGState /SMask 6 0 R >>\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"6 0 obj\n<< /Type /Mask /S /Alpha /G 7 0 R >>\nendobj\n");
    offsets.push(buf.len());
    // The form's /Resources references the same /GS1 → 5 0 R, closing the cycle.
    let form_hdr = format!(
        "7 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency >> \
         /Resources << /ExtGState << /GS1 5 0 R >> >> /Length {} >>\nstream\n",
        group_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(group_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    finalize_pdf(buf, offsets)
}

#[test]
fn ext_gstate_smask_cyclic_g_does_not_stack_overflow() {
    let doc = PdfDocument::from_bytes(build_pdf_with_cyclic_smask()).expect("parse");
    // Should render without panic or stack overflow. The depth guard kicks
    // in after MAX_SMASK_DEPTH (32) levels of nested SMask materialisation,
    // logs a warning, and drops the mask on overflow — subsequent paints
    // land normally. The page output is non-empty PNG.
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    assert!(img.data.len() > 200, "cyclic SMask render produced empty PNG");
}

#[test]
fn ext_gstate_alpha_smask_honours_install_time_ctm() {
    let doc = PdfDocument::from_bytes(build_pdf_with_smask_under_active_ctm()).expect("parse");
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    let rgba = decode_png(&img.data);

    // With the scaled CTM applied to the SMask rasterisation, the form's
    // `0 1 2 1 re f` fills the *top* half of the 100×100 device region.
    // The paint covers the whole 100×100, so the top half lands black and
    // the bottom stays white.
    let top = rgba.get_pixel(50, 25);
    let bottom = rgba.get_pixel(50, 75);
    assert!(
        top[0] < 60,
        "scaled-CTM SMask: top of paint area should be black; got {top:?}"
    );
    assert!(
        bottom[0] > 200,
        "scaled-CTM SMask: bottom of paint area should be background white; \
         got {bottom:?}"
    );
}

/// Build a fixture exercising `/S /Luminosity`. The mask `/G` paints a
/// mid-grey rectangle (`0.5 g … 0 50 100 50 re f`) in the top half of its
/// BBox and leaves the bottom half unpainted. Under Luminosity:
///   - top half luminance ≈ 128 → mask passes paint at ~50% intensity, so a
///     full-page black fill composites to mid-grey.
///   - bottom half luminance = 0 (default black backdrop on unpainted
///     pixels) → mask blocks paint; the white page background stays.
///
/// This distinguishes from `/S /Alpha` (which would see alpha = 255 in the
/// top half and paint fully black) and from "no mask installed" (which
/// would paint the whole page fully black).
fn build_pdf_with_luminosity_smask() -> Vec<u8> {
    let page_content = b"q\n/GS1 gs\n0 g\n0 0 100 100 re\nf\nQ\n";
    let group_content = b"0.5 g\n0 50 100 50 re\nf\n";

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
    buf.extend_from_slice(b"5 0 obj\n<< /Type /ExtGState /SMask 6 0 R >>\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"6 0 obj\n<< /Type /Mask /S /Luminosity /G 7 0 R >>\nendobj\n");
    offsets.push(buf.len());
    let form_hdr = format!(
        "7 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency >> \
         /Resources << >> /Length {} >>\nstream\n",
        group_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(group_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    finalize_pdf(buf, offsets)
}

#[test]
fn ext_gstate_luminosity_smask_modulates_paint_by_group_luma() {
    let doc = PdfDocument::from_bytes(build_pdf_with_luminosity_smask()).expect("parse");
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    let rgba = decode_png(&img.data);

    let top = rgba.get_pixel(50, 25);
    let bottom = rgba.get_pixel(50, 75);

    // Top half: mid-grey luminance ≈ 128 modulates black-over-white
    // SourceOver to ~128. Allow a generous tolerance window for JPEG-free
    // rasteriser rounding. Must not be full black (Alpha would give that)
    // and must not be near white (no mask would not).
    assert!(
        top[0] > 80 && top[0] < 200,
        "Luminosity SMask top should composite mid-grey; got R={} (G={} B={} A={})",
        top[0], top[1], top[2], top[3]
    );

    // Bottom half: unpainted /G → luminance 0 → paint blocked → background white.
    assert!(
        bottom[0] > 200,
        "Luminosity SMask bottom should be the white background; got R={} (G={} B={} A={})",
        bottom[0], bottom[1], bottom[2], bottom[3]
    );
}

/// `/BC` (backdrop colour) test: an empty `/G` form (no paint at all) under
/// `/S /Luminosity`. With `/BC [1 1 1]` the unpainted backdrop is white, so
/// luminance = 255 and the page paint passes through. Without `/BC`
/// (default black), luminance = 0 and the page paint is fully blocked.
fn build_pdf_with_luminosity_smask_backdrop_white() -> Vec<u8> {
    let page_content = b"q\n/GS1 gs\n0 g\n0 0 100 100 re\nf\nQ\n";
    // /G paints nothing — the entire mask area is "backdrop only".
    let group_content = b"";

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
    buf.extend_from_slice(b"5 0 obj\n<< /Type /ExtGState /SMask 6 0 R >>\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(
        b"6 0 obj\n<< /Type /Mask /S /Luminosity /G 7 0 R /BC [1 1 1] >>\nendobj\n",
    );
    offsets.push(buf.len());
    let form_hdr = format!(
        "7 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency /CS /DeviceRGB >> \
         /Resources << >> /Length {} >>\nstream\n",
        group_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(group_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    finalize_pdf(buf, offsets)
}

#[test]
fn ext_gstate_luminosity_smask_bc_white_backdrop_passes_paint() {
    let doc = PdfDocument::from_bytes(build_pdf_with_luminosity_smask_backdrop_white())
        .expect("parse");
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    let rgba = decode_png(&img.data);

    let centre = rgba.get_pixel(50, 50);
    assert!(
        centre[0] < 60,
        "/BC [1 1 1] backdrop should give luma 255 → paint passes (black); \
         got R={} (G={} B={} A={})",
        centre[0], centre[1], centre[2], centre[3]
    );
}

/// `/TR` transfer function test: Luminosity SMask whose /G paints mid-grey
/// (luma ≈ 128) over its entire BBox. With `/TR Identity` (or no /TR) the
/// page paint composites at ~50%. With a Type 2 exponential `/TR
/// {N: 2}`, the mask is squared: 0.5² = 0.25, so the paint composites at
/// ~25%, leaving the rendered pixel substantially closer to white.
fn build_pdf_with_luminosity_smask_squared_transfer() -> Vec<u8> {
    let page_content = b"q\n/GS1 gs\n0 g\n0 0 100 100 re\nf\nQ\n";
    let group_content = b"0.5 g\n0 0 100 100 re\nf\n";

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
    buf.extend_from_slice(b"5 0 obj\n<< /Type /ExtGState /SMask 6 0 R >>\nendobj\n");
    offsets.push(buf.len());
    // /TR is an indirect reference to a Type 2 exponential function:
    //   Domain [0 1], Range [0 1], C0 [0], C1 [1], N 2  →  y = x²
    buf.extend_from_slice(b"6 0 obj\n<< /Type /Mask /S /Luminosity /G 7 0 R /TR 8 0 R >>\nendobj\n");
    offsets.push(buf.len());
    let form_hdr = format!(
        "7 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency >> \
         /Resources << >> /Length {} >>\nstream\n",
        group_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(group_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(
        b"8 0 obj\n<< /FunctionType 2 /Domain [0 1] /Range [0 1] \
            /C0 [0] /C1 [1] /N 2 >>\nendobj\n",
    );

    finalize_pdf(buf, offsets)
}

#[test]
fn ext_gstate_luminosity_smask_tr_type2_squared_attenuates_paint() {
    let doc = PdfDocument::from_bytes(build_pdf_with_luminosity_smask_squared_transfer())
        .expect("parse");
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    let rgba = decode_png(&img.data);

    let centre = rgba.get_pixel(50, 50);
    // Without /TR: 0.5 luma → 50% black over white → R ≈ 128.
    // With /TR squaring: 0.25 luma → 25% black over white → R ≈ 192.
    // Use a wide-but-positioned assertion so the test fails closed if /TR
    // is silently ignored (R stays ~128).
    assert!(
        centre[0] > 160,
        "/TR squaring should attenuate the paint (expect R ≳ 192); got R={} \
         (without /TR this lands ~128)",
        centre[0]
    );
}

/// `/Group /CS /DeviceGray` test: the SMask group declares a single-component
/// blend space. Luma calculation should treat the rendered gray channel as
/// luma directly. For a uniform 50%-gray /G, the resulting mask should
/// produce ~50% composited paint regardless of /CS (since gray's BT.601
/// reduces to the gray value itself), but the impl must not crash or
/// silently drop the mask when /CS is non-RGB.
fn build_pdf_with_luminosity_smask_gray_cs() -> Vec<u8> {
    let page_content = b"q\n/GS1 gs\n0 g\n0 0 100 100 re\nf\nQ\n";
    let group_content = b"0.5 g\n0 0 100 100 re\nf\n";

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
    buf.extend_from_slice(b"5 0 obj\n<< /Type /ExtGState /SMask 6 0 R >>\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"6 0 obj\n<< /Type /Mask /S /Luminosity /G 7 0 R >>\nendobj\n");
    offsets.push(buf.len());
    let form_hdr = format!(
        "7 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency /CS /DeviceGray >> \
         /Resources << >> /Length {} >>\nstream\n",
        group_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(group_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    finalize_pdf(buf, offsets)
}

/// Pins the current behaviour for a malformed group: `/CS /DeviceGray`
/// declared but the form's content uses an RGB paint operator (`1 0 0 rg`,
/// pure red). pdf_oxide's renderer rasterises into an RGB pixmap regardless
/// of the declared `/CS`, and `materialise_soft_mask_alpha` runs BT.601
/// luma unconditionally — there is no `/CS` dispatch for the luma path.
///
/// For valid gray content (`R = G = B`) BT.601 reduces to the gray value,
/// so a hypothetical "DeviceGray → read channel 0" shortcut would behave
/// identically. For non-gray content the two paths diverge: BT.601 gives a
/// weighted blend (red → luma 76); single-channel R-as-luma would give 255.
/// This test fails on the single-channel interpretation, locking in the
/// BT.601-unconditional choice so a future refactor that adds /CS dispatch
/// is forced to think about the malformed case.
fn build_pdf_with_devicegray_group_painting_red() -> Vec<u8> {
    let page_content = b"q\n/GS1 gs\n0 g\n0 0 100 100 re\nf\nQ\n";
    // Pure red paint inside a group declared as /CS /DeviceGray.
    let group_content = b"1 0 0 rg\n0 0 100 100 re\nf\n";

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
    buf.extend_from_slice(b"5 0 obj\n<< /Type /ExtGState /SMask 6 0 R >>\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"6 0 obj\n<< /Type /Mask /S /Luminosity /G 7 0 R >>\nendobj\n");
    offsets.push(buf.len());
    let form_hdr = format!(
        "7 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency /CS /DeviceGray >> \
         /Resources << >> /Length {} >>\nstream\n",
        group_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(group_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    finalize_pdf(buf, offsets)
}

#[test]
fn ext_gstate_luminosity_smask_malformed_devicegray_with_rgb_paint_uses_bt601() {
    let doc = PdfDocument::from_bytes(build_pdf_with_devicegray_group_painting_red())
        .expect("parse");
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    let rgba = decode_png(&img.data);

    let centre = rgba.get_pixel(50, 50);
    // BT.601 luma of pure red is 0.299·255 ≈ 76, so a black fill composites
    // at ~30% over the white background: R ≈ 0.7·255 + 0.3·0 ≈ 179.
    // A single-channel "DeviceGray reads R" interpretation would instead
    // give luma 255 → full black → R ≈ 0.
    assert!(
        centre[0] > 130 && centre[0] < 220,
        "Malformed DeviceGray group painting red: BT.601 luma ≈ 76 should \
         compose to R ≈ 179; got R={} (single-channel R-as-luma would give R ≈ 0)",
        centre[0]
    );
}

#[test]
fn ext_gstate_luminosity_smask_group_cs_devicegray_yields_50pct_paint() {
    let doc =
        PdfDocument::from_bytes(build_pdf_with_luminosity_smask_gray_cs()).expect("parse");
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    let rgba = decode_png(&img.data);

    let centre = rgba.get_pixel(50, 50);
    // 50% gray luma → 50% black over white → R ≈ 128.
    assert!(
        centre[0] > 80 && centre[0] < 200,
        "Luminosity SMask with /CS /DeviceGray should still composite ~mid-grey; \
         got R={}",
        centre[0]
    );
}

/// Build a PDF that paints two rows of text — one at PDF y=70 (top half,
/// mask passes), one at y=20 (bottom half, mask blocks) — under an Alpha
/// SMask. The text rasterizer calls into the same `effective_clip`
/// machinery as paths and images, so the bottom-row glyphs should be
/// blocked and the top-row glyphs should render.
fn build_pdf_with_text_under_smask() -> Vec<u8> {
    // Text at y=70 (top half, PNG row ~30) and y=20 (bottom half, PNG row ~80).
    let page_content = b"q\n/GS1 gs\nBT\n/F1 18 Tf\n0 g\n1 0 0 1 10 70 Tm\n(TOP) Tj\n1 0 0 1 10 20 Tm\n(BOTTOM) Tj\nET\nQ\n";
    let group_content = b"0 50 100 50 re\nf\n";

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
           /Resources << /ExtGState << /GS1 5 0 R >> \
                        /Font << /F1 8 0 R >> >> \
           /Group << /Type /Group /S /Transparency >> >>\nendobj\n",
    );
    offsets.push(buf.len());
    let hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", page_content.len());
    buf.extend_from_slice(hdr.as_bytes());
    buf.extend_from_slice(page_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"5 0 obj\n<< /Type /ExtGState /SMask 6 0 R >>\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"6 0 obj\n<< /Type /Mask /S /Alpha /G 7 0 R >>\nendobj\n");
    offsets.push(buf.len());
    let form_hdr = format!(
        "7 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency >> \
         /Resources << >> /Length {} >>\nstream\n",
        group_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(group_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(
        b"8 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
    );

    finalize_pdf(buf, offsets)
}

#[test]
fn smask_clips_text_paint() {
    let doc = PdfDocument::from_bytes(build_pdf_with_text_under_smask()).expect("parse");
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    let rgba = decode_png(&img.data);

    // PNG rows 0..50 = top half (mask passes) — TOP text should produce
    // dark pixels among the white background. Scan the band where the
    // 18-pt baseline-at-y=70 glyphs land (PDF y 70..82 → PNG rows 18..30).
    let mut top_has_dark = false;
    for y in 18..40 {
        for x in 10..50 {
            if rgba.get_pixel(x, y)[0] < 100 {
                top_has_dark = true;
                break;
            }
        }
        if top_has_dark {
            break;
        }
    }
    assert!(top_has_dark, "TOP text should leave dark pixels (mask passes)");

    // PNG rows 50..100 = bottom half (mask blocks). Scan the band where
    // the BOTTOM glyphs would have landed (PDF y 20..32 → PNG rows 68..80)
    // and assert no dark pixels appear.
    for y in 65..85 {
        for x in 10..70 {
            let p = rgba.get_pixel(x, y);
            assert!(
                p[0] > 200,
                "BOTTOM text leaked through the masked region at ({x}, {y}); \
                 got {p:?}"
            );
        }
    }
}

/// Build a PDF that paints a black-filled image under an Alpha SMask whose
/// `/G` paints the top half opaque. The mask is applied via
/// `effective_clip` at the `Do` paint site, so the image's pixels in the
/// bottom half (mask alpha = 0) should be transparent and reveal the
/// white background.
fn build_pdf_with_image_under_smask() -> Vec<u8> {
    let page_content = b"q\n/GS1 gs\n50 0 0 50 25 25 cm\n/Im1 Do\nQ\n";
    let group_content = b"0 50 100 50 re\nf\n";

    // 1×1 fully-opaque solid black image: DeviceRGB, 1 pixel = (0, 0, 0).
    let img_bytes: &[u8] = &[0u8, 0u8, 0u8];

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
           /Resources << /ExtGState << /GS1 5 0 R >> \
                        /XObject << /Im1 8 0 R >> >> \
           /Group << /Type /Group /S /Transparency >> >>\nendobj\n",
    );
    offsets.push(buf.len());
    let hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", page_content.len());
    buf.extend_from_slice(hdr.as_bytes());
    buf.extend_from_slice(page_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"5 0 obj\n<< /Type /ExtGState /SMask 6 0 R >>\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"6 0 obj\n<< /Type /Mask /S /Alpha /G 7 0 R >>\nendobj\n");
    offsets.push(buf.len());
    let form_hdr = format!(
        "7 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency >> \
         /Resources << >> /Length {} >>\nstream\n",
        group_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(group_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    offsets.push(buf.len());
    let img_hdr = format!(
        "8 0 obj\n<< /Type /XObject /Subtype /Image /Width 1 /Height 1 \
         /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length {} >>\nstream\n",
        img_bytes.len()
    );
    buf.extend_from_slice(img_hdr.as_bytes());
    buf.extend_from_slice(img_bytes);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    finalize_pdf(buf, offsets)
}

#[test]
fn smask_clips_image_paint() {
    let doc = PdfDocument::from_bytes(build_pdf_with_image_under_smask()).expect("parse");
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    let rgba = decode_png(&img.data);

    // Image bbox in PDF user space is (25..75, 25..75) — PNG rows 25..75.
    // Mask covers top half of page (PDF y 50..100 → PNG rows 0..50).
    // So inside the image bbox:
    //   - PNG row 30 (top, mask passes): image's black should land.
    //   - PNG row 70 (bottom, mask blocks): white background visible.
    let top = rgba.get_pixel(50, 30);
    let bottom = rgba.get_pixel(50, 70);
    assert!(
        top[0] < 60,
        "SMask should let image black through in the top half; got {top:?}"
    );
    assert!(
        bottom[0] > 200,
        "SMask should block image paint in the bottom half; got {bottom:?}"
    );
}

/// Build a Luminosity-SMask fixture where `/G` paints a full-page
/// uniform colour given by `(r, g, b)` (each in 0..=255). The page then
/// fills full black through `/GS1`; the rendered centre pixel's R should
/// reflect the BT.601 luminance of `(r,g,b)` composited over the white
/// page background as `255 - luma`.
fn build_pdf_with_luminosity_uniform_color_smask(r: u8, g: u8, b: u8) -> Vec<u8> {
    let page_content = b"q\n/GS1 gs\n0 g\n0 0 100 100 re\nf\nQ\n";
    let group_content = format!(
        "{r:.3} {g:.3} {b:.3} rg\n0 0 100 100 re\nf\n",
        r = r as f32 / 255.0,
        g = g as f32 / 255.0,
        b = b as f32 / 255.0
    );
    let group_content = group_content.as_bytes();

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
    buf.extend_from_slice(b"5 0 obj\n<< /Type /ExtGState /SMask 6 0 R >>\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"6 0 obj\n<< /Type /Mask /S /Luminosity /G 7 0 R >>\nendobj\n");
    offsets.push(buf.len());
    let form_hdr = format!(
        "7 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency >> \
         /Resources << >> /Length {} >>\nstream\n",
        group_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(group_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    finalize_pdf(buf, offsets)
}

/// BT.601 weight pinning — pure red. Luma = 0.299·255 ≈ 76; composite
/// black over white = 255 − 76 ≈ 179. A weight bug that gave luma 150
/// (green-dominant) would land at R ≈ 105; a swap to blue-dominant would
/// give R ≈ 226. The tight band ±6 catches both.
#[test]
fn bt601_luma_pure_red_yields_r_approx_179() {
    let doc = PdfDocument::from_bytes(build_pdf_with_luminosity_uniform_color_smask(255, 0, 0))
        .expect("parse");
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    let r = decode_png(&img.data).get_pixel(50, 50)[0] as i32;
    assert!(
        (r - 179).abs() <= 6,
        "Pure red under BT.601 luma must give R ≈ 179; got {r}"
    );
}

/// BT.601 weight pinning — pure green. Luma = 0.587·255 ≈ 150; composite
/// black over white = 255 − 150 ≈ 105.
#[test]
fn bt601_luma_pure_green_yields_r_approx_105() {
    let doc = PdfDocument::from_bytes(build_pdf_with_luminosity_uniform_color_smask(0, 255, 0))
        .expect("parse");
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    let r = decode_png(&img.data).get_pixel(50, 50)[0] as i32;
    assert!(
        (r - 105).abs() <= 6,
        "Pure green under BT.601 luma must give R ≈ 105; got {r}"
    );
}

/// BT.601 weight pinning — pure blue. Luma = 0.114·255 ≈ 29; composite
/// black over white = 255 − 29 ≈ 226.
#[test]
fn bt601_luma_pure_blue_yields_r_approx_226() {
    let doc = PdfDocument::from_bytes(build_pdf_with_luminosity_uniform_color_smask(0, 0, 255))
        .expect("parse");
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    let r = decode_png(&img.data).get_pixel(50, 50)[0] as i32;
    assert!(
        (r - 226).abs() <= 6,
        "Pure blue under BT.601 luma must give R ≈ 226; got {r}"
    );
}

/// Build a Luminosity-SMask fixture where `/G` paints uniform gray at the
/// given level (`/<gray> g`), and the SMask carries a Type-2 exponential
/// `/TR` with the given `N`. Used to multi-sample the transfer function.
fn build_pdf_with_luminosity_smask_tr_type2(gray: f32, n: f32) -> Vec<u8> {
    let page_content = b"q\n/GS1 gs\n0 g\n0 0 100 100 re\nf\nQ\n";
    let group_content = format!("{gray:.3} g\n0 0 100 100 re\nf\n");
    let group_content = group_content.as_bytes();

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
    buf.extend_from_slice(b"5 0 obj\n<< /Type /ExtGState /SMask 6 0 R >>\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"6 0 obj\n<< /Type /Mask /S /Luminosity /G 7 0 R /TR 8 0 R >>\nendobj\n");
    offsets.push(buf.len());
    let form_hdr = format!(
        "7 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency >> \
         /Resources << >> /Length {} >>\nstream\n",
        group_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(group_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    offsets.push(buf.len());
    let tr = format!(
        "8 0 obj\n<< /FunctionType 2 /Domain [0 1] /Range [0 1] \
            /C0 [0] /C1 [1] /N {n} >>\nendobj\n"
    );
    buf.extend_from_slice(tr.as_bytes());

    finalize_pdf(buf, offsets)
}

/// `/TR` Type-2 N=2 pin at three luma input points: 0.25, 0.5, 0.75.
/// Mask after transfer: 0.0625, 0.25, 0.5625. Composite over white:
/// R ≈ 239, 191, 112. A bug that uses x¹, x³, or x⁴ would fail at least
/// one assertion (different curvature shows up at the endpoints).
#[test]
fn tr_type2_squared_pins_exponent_at_three_points() {
    let cases = [
        (0.25_f32, 239_i32),
        (0.50_f32, 191_i32),
        (0.75_f32, 112_i32),
    ];
    for (gray, expected) in cases {
        let doc = PdfDocument::from_bytes(build_pdf_with_luminosity_smask_tr_type2(gray, 2.0))
            .expect("parse");
        let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
        let r = decode_png(&img.data).get_pixel(50, 50)[0] as i32;
        assert!(
            (r - expected).abs() <= 8,
            "/TR Type 2 N=2 at gray={gray} must give R ≈ {expected}; got {r}"
        );
    }
}

/// `/TR` Type-2 with `N <= 0` must be rejected (§7.10.3). The mask should
/// then come straight from BT.601 luma with no transfer applied: 0.5 g →
/// luma ≈ 128 → R ≈ 127. A regression that drops the `N > 0 && is_finite`
/// guard would let `0_f64.powf(0.0) = 1.0` invert the mask everywhere
/// and produce R ≈ 0.
#[test]
fn tr_type2_invalid_n_falls_through_to_identity() {
    let doc = PdfDocument::from_bytes(build_pdf_with_luminosity_smask_tr_type2(0.5, -1.0))
        .expect("parse");
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    let r = decode_png(&img.data).get_pixel(50, 50)[0] as i32;
    assert!(
        (r - 127).abs() <= 8,
        "/TR with N=-1 must be rejected, falling through to identity (R ≈ 127); got {r}"
    );
}

#[test]
fn ext_gstate_alpha_smask_blocks_paint_under_transparent_mask() {
    let pdf = build_pdf_with_alpha_smask();
    let doc = PdfDocument::from_bytes(pdf).expect("parse PDF");
    let img = render_page(&doc, 0, &RenderOptions::with_dpi(72)).expect("render");
    let rgba = decode_png(&img.data);

    // PDF y goes up; PNG rows go top-to-bottom. The SMask group fills PDF
    // y = 50..100 (the *top* half in user space) which lands in PNG rows
    // 0..50 (top of the rendered image). The bottom half of the PNG
    // corresponds to the *transparent* region of the mask.
    let top = rgba.get_pixel(50, 25);
    let bottom = rgba.get_pixel(50, 75);

    // Top half: mask α = 1 → black fill should land. The R channel should be
    // close to 0 (pure black on a white background).
    assert!(
        top[0] < 60,
        "top-half pixel should be black under opaque mask region; got R={} G={} B={} A={}",
        top[0], top[1], top[2], top[3]
    );

    // Bottom half: mask α = 0 → the fill should be blocked, leaving the
    // white background visible (R close to 255).
    assert!(
        bottom[0] > 200,
        "bottom-half pixel should be white where the mask is transparent; \
         got R={} G={} B={} A={}",
        bottom[0], bottom[1], bottom[2], bottom[3]
    );
}
