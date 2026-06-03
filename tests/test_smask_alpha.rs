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
