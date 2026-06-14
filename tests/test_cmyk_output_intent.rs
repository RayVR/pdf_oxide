//! A caller-supplied fallback CMYK profile colour-manages uncalibrated
//! DeviceCMYK content (a PDF with no `/OutputIntents`). Without it, DeviceCMYK
//! uses the §10.3.5 naive additive-clamp (oversaturated); with it, the `k`
//! operator's fill is converted through the supplied profile — matching what a
//! print-oriented renderer does for uncalibrated content.

#![cfg(all(
    feature = "rendering",
    any(feature = "icc-qcms", feature = "icc-lcms2")
))]

use pdf_oxide::document::PdfDocument;
use pdf_oxide::rendering::{render_page, ImageFormat, RenderOptions};

/// Single 100×100pt page, no OutputIntent, one full-page DeviceCMYK fill
/// (`c m y k k` operator) — a saturated blue (C=0.8, M=0.95).
fn build_cmyk_fill_pdf() -> Vec<u8> {
    let content = b"0.8 0.95 0.0 0.0 k\n0 0 100 100 re f";
    let mut buf = Vec::new();
    let mut offs = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");
    offs.push(buf.len());
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    offs.push(buf.len());
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    offs.push(buf.len());
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Contents 4 0 R /Resources << >> >>\nendobj\n",
    );
    offs.push(buf.len());
    buf.extend_from_slice(format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len()).as_bytes());
    buf.extend_from_slice(content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let xref = buf.len();
    buf.extend_from_slice(b"xref\n0 5\n0000000000 65535 f \n");
    for o in &offs {
        buf.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes(),
    );
    buf
}

/// Minimal constant-Lab `mft1` CMYK→Lab ICC profile: every CMYK input maps to
/// Lab(L=l_byte, a=128, b=128) — i.e. a single neutral grey regardless of the
/// CMYK value. That makes the "profile was applied" assertion unambiguous.
fn build_constant_cmyk_icc(l_byte: u8) -> Vec<u8> {
    let (in_chan, out_chan, grid): (u8, u8, u8) = (4, 3, 2);
    let mut lut = Vec::new();
    lut.extend_from_slice(&0x6d66_7431u32.to_be_bytes());
    lut.extend_from_slice(&0u32.to_be_bytes());
    lut.push(in_chan);
    lut.push(out_chan);
    lut.push(grid);
    lut.push(0);
    let identity: [i32; 9] = [0x0001_0000, 0, 0, 0, 0x0001_0000, 0, 0, 0, 0x0001_0000];
    for v in identity {
        lut.extend_from_slice(&(v as u32).to_be_bytes());
    }
    for _ in 0..in_chan {
        for i in 0..256u16 {
            lut.push(i as u8);
        }
    }
    for _ in 0..(grid as usize).pow(in_chan as u32) {
        lut.push(l_byte);
        lut.push(128);
        lut.push(128);
    }
    for _ in 0..out_chan {
        for i in 0..256u16 {
            lut.push(i as u8);
        }
    }
    let mut profile = vec![0u8; 128];
    let total: u32 = 128 + 4 + 12 + lut.len() as u32;
    profile[0..4].copy_from_slice(&total.to_be_bytes());
    profile[8..12].copy_from_slice(&0x0240_0000u32.to_be_bytes());
    profile[12..16].copy_from_slice(b"prtr");
    profile[16..20].copy_from_slice(b"CMYK");
    profile[20..24].copy_from_slice(b"Lab ");
    profile[36..40].copy_from_slice(b"acsp");
    profile.extend_from_slice(&1u32.to_be_bytes());
    profile.extend_from_slice(&0x4132_4230u32.to_be_bytes());
    profile.extend_from_slice(&144u32.to_be_bytes());
    profile.extend_from_slice(&(lut.len() as u32).to_be_bytes());
    profile.extend_from_slice(&lut);
    profile
}

/// Single 100×100pt page with a full-page DeviceCMYK fill AND an embedded
/// `/OutputIntents` CMYK profile (object 5).
fn build_cmyk_fill_pdf_with_output_intent(icc: &[u8]) -> Vec<u8> {
    let content = b"0.8 0.95 0.0 0.0 k\n0 0 100 100 re f";
    let mut buf = Vec::new();
    let mut offs = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");
    offs.push(buf.len());
    buf.extend_from_slice(
        b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R /OutputIntents [<< /Type /OutputIntent /S /GTS_PDFX /OutputCondition (Synthetic) /DestOutputProfile 5 0 R >>] >>\nendobj\n",
    );
    offs.push(buf.len());
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    offs.push(buf.len());
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Contents 4 0 R /Resources << >> >>\nendobj\n",
    );
    offs.push(buf.len());
    buf.extend_from_slice(format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len()).as_bytes());
    buf.extend_from_slice(content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    offs.push(buf.len());
    buf.extend_from_slice(
        format!("5 0 obj\n<< /N 4 /Length {} >>\nstream\n", icc.len()).as_bytes(),
    );
    buf.extend_from_slice(icc);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let xref = buf.len();
    buf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
    for o in &offs {
        buf.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes(),
    );
    buf
}

fn render_center_rgb(doc: &PdfDocument) -> (u8, u8, u8) {
    let mut opts = RenderOptions::with_dpi(72);
    opts.format = ImageFormat::RawRgba8;
    let img = render_page(doc, 0, &opts).expect("render");
    let (x, y) = (img.width / 2, img.height / 2);
    let i = ((y * img.width + x) * 4) as usize;
    let a = img.data[i + 3] as f32 / 255.0;
    let over = |c: u8| ((c as f32 * a) + 255.0 * (1.0 - a)).round() as u8;
    (over(img.data[i]), over(img.data[i + 1]), over(img.data[i + 2]))
}

/// With no fallback profile, a DeviceCMYK fill uses the naive §10.3.5
/// conversion: C=0.8,M=0.95,Y=0,K=0 → roughly (51, 13, 255), a saturated blue.
#[test]
fn devicecmyk_without_fallback_is_naive_saturated() {
    let doc = PdfDocument::from_bytes(build_cmyk_fill_pdf()).expect("parse");
    let (r, g, b) = render_center_rgb(&doc);
    assert!(
        b > 200 && r < 90 && g < 90,
        "naive DeviceCMYK should be a saturated blue (~51,13,255); got ({r},{g},{b})"
    );
}

/// Installing a fallback CMYK profile colour-manages the same fill: the
/// constant-Lab profile maps every CMYK value to a neutral grey, so the fill
/// becomes grey (R≈G≈B, not blue) — proving the vector `k` fill was routed
/// through the supplied profile rather than the naive conversion.
#[test]
fn devicecmyk_with_fallback_profile_is_colour_managed() {
    let doc = PdfDocument::from_bytes(build_cmyk_fill_pdf()).expect("parse");
    let naive = render_center_rgb(&doc);

    let doc2 = PdfDocument::from_bytes(build_cmyk_fill_pdf()).expect("parse");
    doc2.set_fallback_cmyk_profile_from_bytes(build_constant_cmyk_icc(128))
        .expect("constant CMYK profile must parse");
    let managed = render_center_rgb(&doc2);

    let (r, g, b) = managed;
    assert!(
        (r as i32 - g as i32).abs() < 24 && (g as i32 - b as i32).abs() < 24,
        "fallback-managed DeviceCMYK should be neutral grey (R≈G≈B); got ({r},{g},{b})"
    );
    assert!(
        b < 200,
        "fallback-managed fill must not be the naive saturated blue; got ({r},{g},{b})"
    );
    assert_ne!(naive, managed, "fallback profile must change the rendered colour");
}

/// A non-CMYK (or junk) profile is rejected rather than silently mis-applied.
#[test]
fn non_cmyk_profile_is_rejected() {
    let doc = PdfDocument::from_bytes(build_cmyk_fill_pdf()).expect("parse");
    assert!(
        doc.set_fallback_cmyk_profile_from_bytes(vec![0u8; 64])
            .is_err(),
        "garbage bytes must not be accepted as a CMYK profile"
    );
}

fn lum((r, g, b): (u8, u8, u8)) -> f32 {
    (r as f32 + g as f32 + b as f32) / 3.0
}

/// An OVERRIDE profile must win over the document's own embedded
/// `/OutputIntents` (soft-proofing). The doc embeds a light constant-Lab
/// profile; overriding with a dark one must render dark — proving the override
/// replaced the embedded profile rather than the embedded one winning.
#[test]
fn override_profile_beats_embedded_output_intent() {
    let doc = PdfDocument::from_bytes(build_cmyk_fill_pdf_with_output_intent(
        &build_constant_cmyk_icc(200),
    ))
    .expect("parse");
    let embedded = render_center_rgb(&doc);

    let doc2 = PdfDocument::from_bytes(build_cmyk_fill_pdf_with_output_intent(
        &build_constant_cmyk_icc(200),
    ))
    .expect("parse");
    doc2.set_override_cmyk_profile_from_bytes(build_constant_cmyk_icc(80))
        .expect("override profile must parse");
    let overridden = render_center_rgb(&doc2);

    assert!(
        lum(overridden) < lum(embedded) - 30.0,
        "override profile must replace the embedded OutputIntent: embedded \
         lum {:.0} (light), overridden lum {:.0} (should be clearly darker)",
        lum(embedded),
        lum(overridden)
    );
}

/// Clearing the override restores the embedded OutputIntent.
#[test]
fn clearing_override_restores_embedded() {
    let doc = PdfDocument::from_bytes(build_cmyk_fill_pdf_with_output_intent(
        &build_constant_cmyk_icc(200),
    ))
    .expect("parse");
    let embedded = render_center_rgb(&doc);
    doc.set_override_cmyk_profile_from_bytes(build_constant_cmyk_icc(80))
        .expect("parse");
    let overridden = render_center_rgb(&doc);
    doc.set_override_cmyk_profile(None);
    let restored = render_center_rgb(&doc);
    assert_ne!(embedded, overridden, "override should change the colour");
    assert_eq!(embedded, restored, "clearing override should restore embedded");
}

/// The CMYK rendering-intent override round-trips through the accessor and a
/// render under each intent succeeds (plumbing for an Acrobat-style intent
/// selector; the colorimetric effect depends on the profile/gamut).
#[test]
fn cmyk_rendering_intent_override_roundtrips() {
    use pdf_oxide::color::RenderingIntent;
    let doc = PdfDocument::from_bytes(build_cmyk_fill_pdf()).expect("parse");
    assert_eq!(doc.cmyk_rendering_intent(), None, "default: no intent override");
    doc.set_fallback_cmyk_profile_from_bytes(build_constant_cmyk_icc(128))
        .expect("parse");
    for intent in [
        RenderingIntent::Perceptual,
        RenderingIntent::RelativeColorimetric,
        RenderingIntent::Saturation,
        RenderingIntent::AbsoluteColorimetric,
    ] {
        doc.set_cmyk_rendering_intent(Some(intent));
        assert_eq!(doc.cmyk_rendering_intent(), Some(intent));
        let _ = render_center_rgb(&doc); // must not panic under any intent
    }
    doc.set_cmyk_rendering_intent(None);
    assert_eq!(doc.cmyk_rendering_intent(), None, "intent override clears");
}
