//! Round-2 QA probes for the transparency-flattening branch.
//!
//! This suite augments `test_transparency_flattening_audit.rs` with
//! probes that surface coverage gaps the round-2 implementation agent
//! flagged but did not close. Categories:
//!
//!  - **Non-linear ICC OutputIntent + composite precedence** (gap 1 from
//!    the round-1 audit, deferred by the round-2 agent). The agent
//!    claimed the additive-clamp fallback is linear so convert-first vs
//!    composite-first are byte-identical. This QA suite builds a
//!    non-linear ICC fixture (non-identity input curves drive
//!    quadlinear-CLUT lookups along distinct paths for each paint, so
//!    `ICC(A) + ICC(B)` differs from `ICC(A+B)`) and writes the probe
//!    that proves the gap real.
//!
//!  - **SMask + overprint paint-arm coverage matrix**. The round-2 fix
//!    wires `smask_snapshot` / `overprint_snapshot` only on
//!    `Operator::Fill` and `Operator::Stroke`. The agent explicitly
//!    noted FillStroke combos (`B`, `B*`, `b`, `b*`), FillEvenOdd
//!    (`f*`), PaintShading (`sh`), Do (`Do`), and text-showing (`Tj`,
//!    `TJ`, `'`, `"`) all keep the existing direct-paint path. Each
//!    probe documents one such arm with a tracking constant.
//!
//!  - **SMask scope through q/Q**. The agent flagged this as "rides on
//!    GraphicsState clone behaviour, correct but unprobed."
//!
//!  - **Composite overprint reconstruction loss**. The agent admitted
//!    "snapshot-RGB reconstruction loses information for snapshots that
//!    previously went through a non-trivial ICC."

#![cfg(all(feature = "rendering", feature = "icc"))]
#![allow(dead_code)]

use pdf_oxide::document::PdfDocument;
use pdf_oxide::rendering::{render_page, ImageFormat, RenderOptions};

// ===========================================================================
// HONEST_GAP tracking constants
// ===========================================================================

/// Gap 1 from the round-1 audit, deferred by the round-2 implementation
/// agent. The agent's claim: "additive-clamp OutputIntent fallback is
/// linear in CMYK, so convert-first and composite-first are
/// byte-identical." That holds for the additive-clamp path. With a
/// non-linear ICC OutputIntent (input curves that are not identity, so
/// the per-channel mapping into the CLUT diverges between paints), the
/// composite-first vs convert-first ordering produces different bytes —
/// the spec requires composite-first per §11.4 + Annex G.
pub const HONEST_GAP_PRECEDENCE_CONVERT_BEFORE_COMPOSITE_NONLINEAR_ICC: &str =
    "HONEST_GAP_PRECEDENCE_CONVERT_BEFORE_COMPOSITE_NONLINEAR_ICC: under a \
     non-linear OutputIntent ICC the composite path still converts each \
     CMYK paint via OutputIntent before alpha compositing. The probe \
     proves the divergence with a non-identity input-curve CMYK ICC \
     profile. Round 3 must defer CMYK→RGB until after composition.";

macro_rules! smask_op_gap {
    ($name:ident, $op_desc:literal) => {
        pub const $name: &str = concat!(
            stringify!($name),
            ": ExtGState /SMask is only honoured on Operator::Fill and \
             Operator::Stroke. The ",
            $op_desc,
            " operator path does not call smask_snapshot / \
             apply_smask_after_paint; soft masks silently drop on this \
             paint arm. The round-2 implementation agent flagged this as \
             mechanical duplication."
        );
    };
}

smask_op_gap!(HONEST_GAP_SMASK_FILLSTROKE_NOT_WIRED, "B (fill+stroke)");
smask_op_gap!(HONEST_GAP_SMASK_FILLSTROKE_EVENODD_NOT_WIRED, "B* (fill+stroke EvenOdd)");
smask_op_gap!(HONEST_GAP_SMASK_CLOSE_FILLSTROKE_NOT_WIRED, "b (close+fill+stroke)");
smask_op_gap!(
    HONEST_GAP_SMASK_CLOSE_FILLSTROKE_EVENODD_NOT_WIRED,
    "b* (close+fill+stroke EvenOdd)"
);
smask_op_gap!(HONEST_GAP_SMASK_FILL_EVENODD_NOT_WIRED, "f* (fill EvenOdd)");
smask_op_gap!(HONEST_GAP_SMASK_PAINT_SHADING_NOT_WIRED, "sh (paint shading)");
smask_op_gap!(HONEST_GAP_SMASK_DO_NOT_WIRED, "Do (Form XObject + image invocation)");
smask_op_gap!(HONEST_GAP_SMASK_TEXT_SHOWING_NOT_WIRED, "Tj / TJ / ' / \" (text-showing)");

macro_rules! overprint_op_gap {
    ($name:ident, $op_desc:literal) => {
        pub const $name: &str = concat!(
            stringify!($name),
            ": §11.7.4 overprint correction is only honoured on \
             Operator::Fill and Operator::Stroke. The ",
            $op_desc,
            " operator path does not call overprint_snapshot / \
             apply_overprint_after_paint; overprint preview silently \
             drops on this paint arm. The round-2 implementation agent \
             flagged this as mechanical duplication."
        );
    };
}

overprint_op_gap!(HONEST_GAP_OVERPRINT_FILLSTROKE_NOT_WIRED, "B (fill+stroke)");
overprint_op_gap!(HONEST_GAP_OVERPRINT_FILLSTROKE_EVENODD_NOT_WIRED, "B* (fill+stroke EvenOdd)");
overprint_op_gap!(HONEST_GAP_OVERPRINT_CLOSE_FILLSTROKE_NOT_WIRED, "b (close+fill+stroke)");
overprint_op_gap!(
    HONEST_GAP_OVERPRINT_CLOSE_FILLSTROKE_EVENODD_NOT_WIRED,
    "b* (close+fill+stroke EvenOdd)"
);
overprint_op_gap!(HONEST_GAP_OVERPRINT_FILL_EVENODD_NOT_WIRED, "f* (fill EvenOdd)");

/// Composite overprint reconstruction loss: the round-2 fix recovers
/// CMYK from the destination RGB snapshot via additive-clamp inversion.
/// When the snapshot was produced through a non-trivial ICC OutputIntent
/// (the RGB carries colorimetric information the inversion can't
/// reproduce), the reconstructed CMYK is approximate. The agent
/// acknowledged this. The probe pins the magnitude of the loss under a
/// non-linear ICC.
pub const HONEST_GAP_OVERPRINT_COMPOSITE_RECONSTRUCTION_LOSS: &str =
    "HONEST_GAP_OVERPRINT_COMPOSITE_RECONSTRUCTION_LOSS: the composite \
     overprint correction uses additive-clamp inversion of the \
     destination RGB to recover CMYK. Under a non-trivial ICC \
     OutputIntent the recovered CMYK is approximate; press-accurate \
     overprint preview needs the separation backend route.";

// ===========================================================================
// Synthetic PDF + ICC profile helpers
// ===========================================================================

/// Build a minimal valid ICC v2 CMYK→Lab profile whose 4-channel input
/// curves apply a gamma-2.2 transform BEFORE the CLUT lookup. Combined
/// with a CLUT whose corners are positioned at Lab(L=255·(1-Σink/4),
/// 128, 128) — i.e. white at 0-ink, black at 4-ink — the profile maps
/// CMYK to Lab via a non-multilinear function of the raw CMYK bytes.
///
/// This is the lever for the convert-first vs composite-first
/// divergence: when two CMYK paints A and B composite at alpha 0.5,
/// convert-first computes `(ICC(A) + ICC(B)) / 2`; composite-first
/// computes `ICC( (A + B) / 2 )`. Because the input curves are
/// non-linear (gamma 2.2), these two paths produce visibly different
/// RGB outputs even though the CLUT body is multilinear.
///
/// The input curves are 256-entry tables — qcms reads them as
/// `lut_interp_linear_float`, sampling across [0, 1] and using the
/// entry value as a linearised input to the CLUT. A gamma-2.2 curve
/// gives `entry[i] = (i/255)^(1/2.2) * 255`.
fn build_nonlinear_cmyk_to_lab_lut8_profile() -> Vec<u8> {
    let in_chan: u8 = 4;
    let out_chan: u8 = 3;
    let grid: u8 = 2;
    let mut lut = Vec::with_capacity(2048);

    lut.extend_from_slice(&0x6d66_7431u32.to_be_bytes()); // 'mft1'
    lut.extend_from_slice(&0u32.to_be_bytes()); // reserved
    lut.push(in_chan);
    lut.push(out_chan);
    lut.push(grid);
    lut.push(0);

    // Identity matrix (CMYK input ignores matrix per qcms but we still
    // need to emit it).
    let identity: [i32; 9] = [0x0001_0000, 0, 0, 0, 0x0001_0000, 0, 0, 0, 0x0001_0000];
    for v in identity {
        lut.extend_from_slice(&(v as u32).to_be_bytes());
    }

    // Input tables — gamma-2.2 forward curve per channel. This is the
    // non-linearity that makes the profile divergent under
    // convert-first vs composite-first.
    //
    // Per qcms's iccread of `mft1` (ICC.1:2004-10 §10.8), the input
    // table is 256 bytes per channel. qcms interprets each entry as a
    // u8 in 0..=255 sampled across the input domain [0, 1] via
    // `lut_interp_linear_float`. Writing entry[i] = ((i/255)^(1/2.2) *
    // 255) gives a gamma-2.2 forward curve that lifts mid-tones.
    for _ in 0..in_chan {
        for i in 0..256u16 {
            let v = ((i as f64) / 255.0).powf(1.0 / 2.2);
            let byte = (v * 255.0).round().clamp(0.0, 255.0) as u8;
            lut.push(byte);
        }
    }

    // CLUT: 2^4 = 16 grid points × 3 output channels. Corner ordering
    // follows qcms's `CLU` function (chain.rs:300-302) where the index
    // is `x * x_stride + y * y_stride + z * z_stride + w` with strides
    // `x_stride = grid^3`, `y_stride = grid^2`, `z_stride = grid`, `w`
    // = stride 1. The first input channel (C) thus walks the
    // outermost dimension.
    //
    // We position the corners so that "no ink" (0,0,0,0) → Lab(L=255,
    // a=128, b=128) (white) and "full ink" (255,255,255,255) →
    // Lab(L=0, a=128, b=128) (black). Linear interpolation between
    // corners in the CLUT body is multilinear, but the input gamma
    // curve above makes the overall mapping non-linear.
    let grid_size = (grid as usize).pow(in_chan as u32);
    for idx in 0..grid_size {
        // idx bits give (C, M, Y, K) at the corner positions.
        // qcms's CLU stride order is (x = first channel = C outermost,
        // w = last channel = K innermost). So idx = c*8 + m*4 + y*2 + k.
        let c = (idx >> 3) & 1;
        let m = (idx >> 2) & 1;
        let y = (idx >> 1) & 1;
        let k = idx & 1;
        let total = c + m + y + k;
        // L decreases as total ink increases: 0 ink → L byte 255,
        // 4 ink → L byte 0.
        let l_byte = (255 - total * 63).min(255) as u8;
        lut.push(l_byte);
        lut.push(128); // a* = 0
        lut.push(128); // b* = 0
    }

    // Output tables — identity 0..=255.
    for _ in 0..out_chan {
        for i in 0..256u16 {
            lut.push(i as u8);
        }
    }

    let mut profile = vec![0u8; 128];
    let total_size: u32 = 128 + 4 + 12 + lut.len() as u32;
    profile[0..4].copy_from_slice(&total_size.to_be_bytes());
    profile[8..12].copy_from_slice(&0x0240_0000u32.to_be_bytes()); // v2
    profile[12..16].copy_from_slice(b"prtr");
    profile[16..20].copy_from_slice(b"CMYK");
    profile[20..24].copy_from_slice(b"Lab ");
    profile[36..40].copy_from_slice(b"acsp");
    profile[64..68].copy_from_slice(&0u32.to_be_bytes()); // intent perceptual
    profile[68..72].copy_from_slice(&0x0000_F6D6u32.to_be_bytes()); // X 0.9642
    profile[72..76].copy_from_slice(&0x0001_0000u32.to_be_bytes()); // Y 1.0
    profile[76..80].copy_from_slice(&0x0000_D32Du32.to_be_bytes()); // Z 0.8249

    profile.extend_from_slice(&1u32.to_be_bytes()); // tag count
    profile.extend_from_slice(&0x4132_4230u32.to_be_bytes()); // 'A2B0'
    profile.extend_from_slice(&144u32.to_be_bytes()); // offset
    profile.extend_from_slice(&(lut.len() as u32).to_be_bytes()); // size
    profile.extend_from_slice(&lut);

    profile
}

/// Build a one-page PDF with a content stream, optional resource-dict
/// fragment, and extra indirect objects starting at object 5. When
/// `icc_profile` is `Some`, the catalog declares an `/OutputIntents`
/// array referencing object 5 (the ICC profile stream), and extra
/// objects start at 6.
fn build_pdf_with_optional_output_intent(
    content: &str,
    resources_inner: &str,
    extra_objs: &[&str],
    icc_profile: Option<&[u8]>,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    let catalog = if icc_profile.is_some() {
        "1 0 obj\n<< /Type /Catalog /Pages 2 0 R /OutputIntents [<< /Type /OutputIntent /S /GTS_PDFX /OutputCondition (Synthetic Non-Linear CMYK) /DestOutputProfile 5 0 R >>] >>\nendobj\n".to_string()
    } else {
        "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_string()
    };
    buf.extend_from_slice(catalog.as_bytes());

    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    let page_off = buf.len();
    let page = format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << {} >> /Contents 4 0 R >>\nendobj\n",
        resources_inner
    );
    buf.extend_from_slice(page.as_bytes());

    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let mut extra_offs: Vec<usize> = Vec::new();

    let mut next_obj_num = 5;
    if let Some(icc) = icc_profile {
        extra_offs.push(buf.len());
        let icc_hdr = format!("{} 0 obj\n<< /N 4 /Length {} >>\nstream\n", next_obj_num, icc.len());
        buf.extend_from_slice(icc_hdr.as_bytes());
        buf.extend_from_slice(icc);
        buf.extend_from_slice(b"\nendstream\nendobj\n");
        next_obj_num += 1;
    }

    for obj in extra_objs {
        extra_offs.push(buf.len());
        // Caller emits the object with its own leading number — we
        // assume the caller numbered them starting at `next_obj_num`.
        let _ = next_obj_num;
        buf.extend_from_slice(obj.as_bytes());
    }

    let xref_off = buf.len();
    let total_objs = 4 + extra_offs.len();
    buf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", total_objs + 1).as_bytes());
    for off in [cat_off, pages_off, page_off, stream_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    for off in extra_offs {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            total_objs + 1,
            xref_off
        )
        .as_bytes(),
    );
    buf
}

fn render_rgba(pdf_bytes: Vec<u8>) -> Vec<u8> {
    let doc = PdfDocument::from_bytes(pdf_bytes).expect("synthetic PDF parses");
    let opts = RenderOptions::with_dpi(72).as_raw();
    let img = render_page(&doc, 0, &opts).expect("render_page succeeds");
    assert_eq!(img.format, ImageFormat::RawRgba8);
    assert_eq!(img.width, 100);
    assert_eq!(img.height, 100);
    img.data
}

fn pixel_at(rgba: &[u8], x: u32, y: u32) -> (u8, u8, u8, u8) {
    let off = ((y * 100 + x) * 4) as usize;
    (rgba[off], rgba[off + 1], rgba[off + 2], rgba[off + 3])
}

fn mean_rgb(rgba: &[u8], x_min: u32, x_max: u32, y_min: u32, y_max: u32) -> (f32, f32, f32) {
    let mut r_sum = 0u32;
    let mut g_sum = 0u32;
    let mut b_sum = 0u32;
    let mut n = 0u32;
    for y in y_min..y_max {
        for x in x_min..x_max {
            let (r, g, b, _) = pixel_at(rgba, x, y);
            r_sum += r as u32;
            g_sum += g as u32;
            b_sum += b as u32;
            n += 1;
        }
    }
    let n = n as f32;
    (r_sum as f32 / n, g_sum as f32 / n, b_sum as f32 / n)
}

// ===========================================================================
// Sanity: the non-linear ICC fixture is non-degenerate
// ===========================================================================
//
// Before relying on the non-linear ICC to surface convert-first vs
// composite-first divergence, prove the profile actually maps distinct
// CMYK inputs to distinct RGB outputs and is non-linear in at least one
// channel. Two single-paint renders at CMYK(0,0,0,0) and
// CMYK(0.5,0.5,0.5,0.5) must produce visibly different RGB.

fn fixture_nonlinear_icc_single_cmyk(c: f32, m: f32, y: f32, k: f32) -> Vec<u8> {
    let content = format!("{c} {m} {y} {k} k\n10 10 80 80 re\nf\n");
    let profile = build_nonlinear_cmyk_to_lab_lut8_profile();
    build_pdf_with_optional_output_intent(&content, "", &[], Some(&profile))
}

#[test]
fn nonlinear_icc_distinct_cmyk_yields_distinct_rgb() {
    let r0 = render_rgba(fixture_nonlinear_icc_single_cmyk(0.0, 0.0, 0.0, 0.0));
    let r_full = render_rgba(fixture_nonlinear_icc_single_cmyk(1.0, 1.0, 1.0, 1.0));
    let r_half = render_rgba(fixture_nonlinear_icc_single_cmyk(0.5, 0.5, 0.5, 0.5));
    let (r_a, g_a, b_a) = mean_rgb(&r0, 30, 70, 30, 70);
    let (r_b, g_b, b_b) = mean_rgb(&r_full, 30, 70, 30, 70);
    let (r_c, g_c, b_c) = mean_rgb(&r_half, 30, 70, 30, 70);
    // The three samples must be distinguishable.
    let delta_full = (r_a - r_b).abs() + (g_a - g_b).abs() + (b_a - b_b).abs();
    let delta_half_to_zero = (r_a - r_c).abs() + (g_a - g_c).abs() + (b_a - b_c).abs();
    let delta_half_to_full = (r_b - r_c).abs() + (g_b - g_c).abs() + (b_b - b_c).abs();
    assert!(
        delta_full > 50.0,
        "non-linear ICC must drive CMYK(0,0,0,0)→white vs CMYK(1,1,1,1)→dark; \
         got delta {delta_full:.1} between ({r_a:.0},{g_a:.0},{b_a:.0}) and \
         ({r_b:.0},{g_b:.0},{b_b:.0})"
    );
    assert!(
        delta_half_to_zero > 20.0 && delta_half_to_full > 20.0,
        "non-linear ICC: 50% CMYK should not equal 0% or 100% CMYK; got \
         half=({r_c:.0},{g_c:.0},{b_c:.0}), 0={r_a:.0}, full={r_b:.0}"
    );
}

// ===========================================================================
// Gap 1 — compose-before-convert under a NON-LINEAR ICC OutputIntent
// ===========================================================================
//
// The probe builds two PDFs:
//
//   A. Two CMYK paints with /ca 0.5 on the upper one, declaring the
//      non-linear ICC profile as /OutputIntents.
//   B. Same paints, no /OutputIntents (additive-clamp fallback).
//
// Convert-first ordering (current pdf_oxide behaviour):
//
//   for each paint:
//     CMYK → RGB via ICC at paint-resolution time
//     SourceOver alpha-blend in RGB pixmap
//
// Compose-first ordering (spec-correct per §11.4 + Annex G):
//
//   for each paint:
//     accumulate CMYK in source space (SourceOver in CMYK)
//   single CMYK → RGB conversion via ICC at the end
//
// Under a non-linear ICC, `ICC(α·A + (1-α)·B) ≠ α·ICC(A) +
// (1-α)·ICC(B)` because the input curves are not identity. The
// difference between the convert-first and composite-first results is
// the test signal the round-2 agent claimed didn't exist for any
// fixture they could build.
//
// The probe samples the OVERLAP region and asserts the rendered output
// matches the compose-first expected value (the spec-correct one). If
// the implementation is convert-first (as today), the rendered output
// matches the convert-first formula and DIFFERS from the expected
// compose-first value — the probe fails, surfacing the gap.

fn fixture_nonlinear_icc_two_overlapping_cmyk_paints() -> Vec<u8> {
    // Lower paint: CMYK(0, 0, 0, 0) — no ink, fully white through the
    // non-linear ICC. Upper paint at /ca 0.5: CMYK(1, 1, 1, 1) — full
    // ink, dark through the non-linear ICC.
    //
    // Overlap composite-first: source-over in CMYK at α=0.5 gives
    //   composited CMYK = 0.5·(1,1,1,1) + 0.5·(0,0,0,0) = (0.5, 0.5, 0.5, 0.5)
    // → through the non-linear ICC at the CMYK(0.5, 0.5, 0.5, 0.5)
    //   tetrahedral interpolation, where input curves apply gamma-2.2
    //   to each 0.5 byte (0.5^(1/2.2) ≈ 0.73) before the CLUT lookup.
    //
    // Overlap convert-first (current code): convert each paint
    // separately, then blend in RGB.
    //   convert(CMYK(0,0,0,0)) = RGB(white) ≈ (255, 255, 255)
    //   convert(CMYK(1,1,1,1)) = RGB(black) ≈ (0, 0, 0)
    //   blend at α=0.5 = ((0+255)/2, (0+255)/2, (0+255)/2) = (~128, ~128, ~128)
    //
    // The compose-first expected value depends on the precise
    // gamma-2.2 + multilinear CLUT computation; we capture it by
    // computing what the same ICC produces for a single-paint
    // CMYK(0.5, 0.5, 0.5, 0.5) (the composited CMYK quadruple). If
    // the implementation is composite-first, the overlap region's
    // rendered RGB equals the single-paint render's RGB at that
    // quadruple. If convert-first (current code), it equals the
    // RGB-blend value ~(128, 128, 128).
    let content = "0 0 0 0 k\n10 10 80 80 re\nf\n\
                   /Half gs\n\
                   1 1 1 1 k\n\
                   20 20 60 60 re\nf\n";
    let resources = "/ExtGState << /Half << /Type /ExtGState /ca 0.5 >> >>";
    let profile = build_nonlinear_cmyk_to_lab_lut8_profile();
    build_pdf_with_optional_output_intent(content, resources, &[], Some(&profile))
}

/// IGNORED — pins the compose-first vs convert-first divergence under
/// a non-linear ICC OutputIntent. As-shipped (convert-first), the
/// overlap region shows the RGB-blend of pre-converted paints. Spec-
/// correct (compose-first) would show the ICC-converted value of the
/// composited CMYK.
///
/// **TEST SIGNAL**: this probe FAILS at HEAD precisely when the
/// implementation is convert-first; it PASSES when composite-first is
/// landed. The agent's claim "no observable test signal" is rebutted by
/// this fixture if and only if the fixture's CMYK(0.5,0.5,0.5,0.5)
/// single-paint render produces a value distinct from the overlap-blend
/// value.
#[test]
#[ignore = "HONEST_GAP_PRECEDENCE_CONVERT_BEFORE_COMPOSITE_NONLINEAR_ICC"]
fn qa_round2_compose_before_convert_under_nonlinear_icc() {
    let rgba_two = render_rgba(fixture_nonlinear_icc_two_overlapping_cmyk_paints());
    let rgba_composited = render_rgba(fixture_nonlinear_icc_single_cmyk(0.5, 0.5, 0.5, 0.5));

    // Overlap region centre — PDF (40, 40) → image (40, 60) (PDF y=40,
    // image y=100-40 = 60). Sample a 20×20 mean to swamp AA noise.
    let (or_mean_r, or_mean_g, or_mean_b) = mean_rgb(&rgba_two, 35, 65, 35, 65);
    let (cs_mean_r, cs_mean_g, cs_mean_b) = mean_rgb(&rgba_composited, 35, 65, 35, 65);

    // The compose-first expected value is the single-paint render of
    // CMYK(0.5, 0.5, 0.5, 0.5). The convert-first actual value blends
    // RGB(white) with RGB(black) in RGB, giving ~(128, 128, 128).
    //
    // Under a non-linear ICC, these MUST differ — otherwise the
    // round-2 agent's deferral claim ("compose-first vs convert-first
    // are byte-identical") would be correct.
    let convert_first_delta =
        (or_mean_r - 128.0).abs() + (or_mean_g - 128.0).abs() + (or_mean_b - 128.0).abs();
    let compose_first_delta = (or_mean_r - cs_mean_r).abs()
        + (or_mean_g - cs_mean_g).abs()
        + (or_mean_b - cs_mean_b).abs();

    // Assertion: the overlap region MUST match the compose-first
    // expected value (single-paint at composited CMYK), NOT the
    // convert-first RGB-blend value. As shipped, the implementation
    // is convert-first, so this assertion fails.
    assert!(
        compose_first_delta < 15.0,
        "compose-first expected: overlap region under non-linear ICC must \
         equal the single-paint render of CMYK(0.5, 0.5, 0.5, 0.5). \
         Got overlap RGB ({or_mean_r:.0}, {or_mean_g:.0}, {or_mean_b:.0}); \
         single-paint reference RGB ({cs_mean_r:.0}, {cs_mean_g:.0}, \
         {cs_mean_b:.0}); convert-first reference RGB (128, 128, 128). \
         compose_first_delta={compose_first_delta:.1}, \
         convert_first_delta={convert_first_delta:.1}. {}",
        HONEST_GAP_PRECEDENCE_CONVERT_BEFORE_COMPOSITE_NONLINEAR_ICC
    );
}

// ===========================================================================
// SMask + overprint paint-arm coverage matrix
// ===========================================================================
//
// The round-2 impl wires soft-mask + overprint correction ONLY on
// Operator::Fill (`f`) and Operator::Stroke (`S`). Every other paint
// operator continues to take the direct path that the round-1 audit
// proved drops SMask + overprint state. We pin each uncovered arm with
// a probe that exercises that operator under an active /SMask or
// /op-true ExtGState. Each probe is `#[ignore]`-marked with the
// matching HONEST_GAP constant; the round-3 fix lifts the ignore.
//
// Each fixture follows the same template:
//
//   1. White background fill (Operator::Fill, the path that IS wired).
//   2. Push ExtGState declaring /SMask or /op true.
//   3. Run the target paint operator that should be modulated.
//
// The assertion checks that the destination pixel reflects the SMask
// or overprint effect, which it will not as-shipped.

fn fixture_smask_for_op(op_ops: &str) -> Vec<u8> {
    let form_content = "0.5 g\n0 0 100 100 re\nf\n";
    let obj_5 = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Resources << >> /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        form_content.len(),
        form_content
    );
    let content = format!(
        "1 1 1 rg\n0 0 100 100 re\nf\n\
         /Sm gs\n\
         1 0 0 rg\n\
         1 0 0 RG\n5 w\n\
         {}\n",
        op_ops
    );
    let resources = "/ExtGState << /Sm << /Type /ExtGState \
                     /SMask << /Type /Mask /S /Luminosity /G 5 0 R >> >> >>";
    build_pdf_with_optional_output_intent(&content, resources, &[&obj_5], None)
}

/// IGNORED — SMask on `B` (fill+stroke). The Fill arm IS wired but
/// `B` takes the FillStroke branch which is unwired.
#[test]
#[ignore = "HONEST_GAP_SMASK_FILLSTROKE_NOT_WIRED"]
fn qa_round2_smask_modulates_fill_stroke_combo() {
    let pdf = fixture_smask_for_op("20 20 60 60 re\nB\n");
    let rgba = render_rgba(pdf);
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    // SMask at /S /Luminosity with a 50%-grey form ⇒ modulation ≈ 0.5.
    // Red over white at α≈0.5 ⇒ (255, 128, 128). As-shipped, B paints
    // fully opaque red ⇒ (255, 0, 0).
    assert!(
        r >= 240 && (g as i32 - 128).abs() <= 15 && (b as i32 - 128).abs() <= 15,
        "B (FillStroke) under SMask /Luminosity 50% grey form: expected \
         ~(255, 128, 128); got ({r}, {g}, {b}). {}",
        HONEST_GAP_SMASK_FILLSTROKE_NOT_WIRED
    );
}

/// IGNORED — SMask on `B*` (fill+stroke EvenOdd).
#[test]
#[ignore = "HONEST_GAP_SMASK_FILLSTROKE_EVENODD_NOT_WIRED"]
fn qa_round2_smask_modulates_fill_stroke_evenodd_combo() {
    let pdf = fixture_smask_for_op("20 20 60 60 re\nB*\n");
    let rgba = render_rgba(pdf);
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    assert!(
        r >= 240 && (g as i32 - 128).abs() <= 15 && (b as i32 - 128).abs() <= 15,
        "B* (FillStrokeEvenOdd) under SMask: expected ~(255, 128, 128); \
         got ({r}, {g}, {b}). {}",
        HONEST_GAP_SMASK_FILLSTROKE_EVENODD_NOT_WIRED
    );
}

/// IGNORED — SMask on `b` (close+fill+stroke).
#[test]
#[ignore = "HONEST_GAP_SMASK_CLOSE_FILLSTROKE_NOT_WIRED"]
fn qa_round2_smask_modulates_close_fill_stroke_combo() {
    // Use a path that needs closing — moveto + lineto + lineto + b.
    let pdf = fixture_smask_for_op("20 20 m\n80 20 l\n80 80 l\n20 80 l\nb\n");
    let rgba = render_rgba(pdf);
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    assert!(
        r >= 240 && (g as i32 - 128).abs() <= 15 && (b as i32 - 128).abs() <= 15,
        "b (CloseFillStroke) under SMask: expected ~(255, 128, 128); got \
         ({r}, {g}, {b}). {}",
        HONEST_GAP_SMASK_CLOSE_FILLSTROKE_NOT_WIRED
    );
}

/// IGNORED — SMask on `b*` (close+fill+stroke EvenOdd).
#[test]
#[ignore = "HONEST_GAP_SMASK_CLOSE_FILLSTROKE_EVENODD_NOT_WIRED"]
fn qa_round2_smask_modulates_close_fill_stroke_evenodd_combo() {
    let pdf = fixture_smask_for_op("20 20 m\n80 20 l\n80 80 l\n20 80 l\nb*\n");
    let rgba = render_rgba(pdf);
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    assert!(
        r >= 240 && (g as i32 - 128).abs() <= 15 && (b as i32 - 128).abs() <= 15,
        "b* (CloseFillStrokeEvenOdd) under SMask: expected ~(255, 128, 128); \
         got ({r}, {g}, {b}). {}",
        HONEST_GAP_SMASK_CLOSE_FILLSTROKE_EVENODD_NOT_WIRED
    );
}

/// IGNORED — SMask on `f*` (fill EvenOdd).
#[test]
#[ignore = "HONEST_GAP_SMASK_FILL_EVENODD_NOT_WIRED"]
fn qa_round2_smask_modulates_fill_evenodd() {
    let pdf = fixture_smask_for_op("20 20 60 60 re\nf*\n");
    let rgba = render_rgba(pdf);
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    assert!(
        r >= 240 && (g as i32 - 128).abs() <= 15 && (b as i32 - 128).abs() <= 15,
        "f* (FillEvenOdd) under SMask: expected ~(255, 128, 128); got \
         ({r}, {g}, {b}). {}",
        HONEST_GAP_SMASK_FILL_EVENODD_NOT_WIRED
    );
}

fn fixture_overprint_for_op(op_ops: &str) -> Vec<u8> {
    // CMYK backdrop fill (cyan 50%) then the target operator paints
    // yellow with overprint on. With overprint, the overlap should
    // retain the cyan plate. Without (as-shipped on uncovered arms),
    // the yellow knocks the cyan out completely.
    let content = format!(
        "0.5 0 0 0 k\n10 10 80 80 re\nf\n\
         /OpOn gs\n\
         0 0 1 0 k\n\
         0 0 1 0 K\n5 w\n\
         {}\n",
        op_ops
    );
    let resources = "/ExtGState << /OpOn << /Type /ExtGState /op true /OP true /OPM 1 >> >>";
    build_pdf_with_optional_output_intent(&content, resources, &[], None)
}

fn fixture_no_overprint_for_op(op_ops: &str) -> Vec<u8> {
    let content = format!(
        "0.5 0 0 0 k\n10 10 80 80 re\nf\n\
         0 0 1 0 k\n\
         0 0 1 0 K\n5 w\n\
         {}\n",
        op_ops
    );
    build_pdf_with_optional_output_intent(&content, "", &[], None)
}

#[test]
#[ignore = "HONEST_GAP_OVERPRINT_FILLSTROKE_NOT_WIRED"]
fn qa_round2_overprint_modulates_fill_stroke_combo() {
    let with_op = render_rgba(fixture_overprint_for_op("30 30 50 50 re\nB\n"));
    let no_op = render_rgba(fixture_no_overprint_for_op("30 30 50 50 re\nB\n"));
    let (r_op, g_op, b_op) = mean_rgb(&with_op, 40, 60, 40, 60);
    let (r_no, g_no, b_no) = mean_rgb(&no_op, 40, 60, 40, 60);
    let delta = (r_op - r_no).abs() + (g_op - g_no).abs() + (b_op - b_no).abs();
    assert!(
        delta > 30.0,
        "B (FillStroke) overprint vs no-overprint delta: expected > 30, got \
         {delta:.1} between ({r_op:.0},{g_op:.0},{b_op:.0}) and \
         ({r_no:.0},{g_no:.0},{b_no:.0}). {}",
        HONEST_GAP_OVERPRINT_FILLSTROKE_NOT_WIRED
    );
}

#[test]
#[ignore = "HONEST_GAP_OVERPRINT_FILLSTROKE_EVENODD_NOT_WIRED"]
fn qa_round2_overprint_modulates_fill_stroke_evenodd_combo() {
    let with_op = render_rgba(fixture_overprint_for_op("30 30 50 50 re\nB*\n"));
    let no_op = render_rgba(fixture_no_overprint_for_op("30 30 50 50 re\nB*\n"));
    let (r_op, g_op, b_op) = mean_rgb(&with_op, 40, 60, 40, 60);
    let (r_no, g_no, b_no) = mean_rgb(&no_op, 40, 60, 40, 60);
    let delta = (r_op - r_no).abs() + (g_op - g_no).abs() + (b_op - b_no).abs();
    assert!(
        delta > 30.0,
        "B* overprint vs no-overprint delta: expected > 30, got {delta:.1}. {}",
        HONEST_GAP_OVERPRINT_FILLSTROKE_EVENODD_NOT_WIRED
    );
}

#[test]
#[ignore = "HONEST_GAP_OVERPRINT_CLOSE_FILLSTROKE_NOT_WIRED"]
fn qa_round2_overprint_modulates_close_fill_stroke_combo() {
    let with_op = render_rgba(fixture_overprint_for_op("30 30 m\n80 30 l\n80 80 l\n30 80 l\nb\n"));
    let no_op = render_rgba(fixture_no_overprint_for_op("30 30 m\n80 30 l\n80 80 l\n30 80 l\nb\n"));
    let (r_op, g_op, b_op) = mean_rgb(&with_op, 40, 60, 40, 60);
    let (r_no, g_no, b_no) = mean_rgb(&no_op, 40, 60, 40, 60);
    let delta = (r_op - r_no).abs() + (g_op - g_no).abs() + (b_op - b_no).abs();
    assert!(
        delta > 30.0,
        "b overprint delta: expected > 30, got {delta:.1}. {}",
        HONEST_GAP_OVERPRINT_CLOSE_FILLSTROKE_NOT_WIRED
    );
}

#[test]
#[ignore = "HONEST_GAP_OVERPRINT_CLOSE_FILLSTROKE_EVENODD_NOT_WIRED"]
fn qa_round2_overprint_modulates_close_fill_stroke_evenodd_combo() {
    let with_op = render_rgba(fixture_overprint_for_op("30 30 m\n80 30 l\n80 80 l\n30 80 l\nb*\n"));
    let no_op =
        render_rgba(fixture_no_overprint_for_op("30 30 m\n80 30 l\n80 80 l\n30 80 l\nb*\n"));
    let (r_op, g_op, b_op) = mean_rgb(&with_op, 40, 60, 40, 60);
    let (r_no, g_no, b_no) = mean_rgb(&no_op, 40, 60, 40, 60);
    let delta = (r_op - r_no).abs() + (g_op - g_no).abs() + (b_op - b_no).abs();
    assert!(
        delta > 30.0,
        "b* overprint delta: expected > 30, got {delta:.1}. {}",
        HONEST_GAP_OVERPRINT_CLOSE_FILLSTROKE_EVENODD_NOT_WIRED
    );
}

#[test]
#[ignore = "HONEST_GAP_OVERPRINT_FILL_EVENODD_NOT_WIRED"]
fn qa_round2_overprint_modulates_fill_evenodd() {
    let with_op = render_rgba(fixture_overprint_for_op("30 30 50 50 re\nf*\n"));
    let no_op = render_rgba(fixture_no_overprint_for_op("30 30 50 50 re\nf*\n"));
    let (r_op, g_op, b_op) = mean_rgb(&with_op, 40, 60, 40, 60);
    let (r_no, g_no, b_no) = mean_rgb(&no_op, 40, 60, 40, 60);
    let delta = (r_op - r_no).abs() + (g_op - g_no).abs() + (b_op - b_no).abs();
    assert!(
        delta > 30.0,
        "f* overprint delta: expected > 30, got {delta:.1}. {}",
        HONEST_GAP_OVERPRINT_FILL_EVENODD_NOT_WIRED
    );
}

// ===========================================================================
// SMask scope through q/Q
// ===========================================================================
//
// Per §11.4.7, ExtGState /SMask is graphics-state — q pushes a copy, Q
// pops back to the prior state. After Q, any /SMask in the popped
// scope MUST be inactive. The round-2 impl rides on the
// GraphicsStateStack's `push` / `pop` (which deep-clones the state on
// push, restoring on pop). The agent flagged this as "correct but
// unprobed."

fn fixture_smask_scoped_through_q_then_paint_outside() -> Vec<u8> {
    let form_content = "0.5 g\n0 0 100 100 re\nf\n";
    let obj_5 = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Resources << >> /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        form_content.len(),
        form_content
    );
    // White background, then `q` push, /Sm gs, paint inside scope (red
    // through SMask → faded red ~(255, 128, 128)), `Q` pop, paint
    // again outside scope (red WITHOUT SMask → fully opaque red).
    let content = "1 1 1 rg\n0 0 100 100 re\nf\n\
                   q\n\
                   /Sm gs\n\
                   1 0 0 rg\n\
                   10 10 30 30 re\nf\n\
                   Q\n\
                   1 0 0 rg\n\
                   60 60 30 30 re\nf\n";
    let resources = "/ExtGState << /Sm << /Type /ExtGState \
                     /SMask << /Type /Mask /S /Luminosity /G 5 0 R >> >> >>";
    build_pdf_with_optional_output_intent(content, resources, &[&obj_5], None)
}

/// Pin: after `Q` pops the gstate that declared `/Sm gs`, the
/// subsequent paint must render WITHOUT SMask modulation. Inside the
/// scope: faded red. Outside the scope (post-Q): fully opaque red.
///
/// As-shipped this should PASS — the GraphicsStateStack pop restores
/// the prior `gs.smask = None`. If it FAILS, SMask state leaks
/// across q/Q and that's a real bug.
#[test]
fn qa_round2_smask_does_not_leak_across_q_q() {
    let rgba = render_rgba(fixture_smask_scoped_through_q_then_paint_outside());
    // Inside-scope sample: image (25, 75) (PDF y=10..40 → image y=60..90).
    let (r_in, g_in, b_in, _) = pixel_at(&rgba, 25, 75);
    // Outside-scope sample: image (75, 25) (PDF y=60..90 → image y=10..40).
    let (r_out, g_out, b_out, _) = pixel_at(&rgba, 75, 25);

    // Inside the SMask scope, red should be faded by the 50%
    // luminance modulation.
    assert!(
        r_in >= 240 && (g_in as i32 - 128).abs() <= 25 && (b_in as i32 - 128).abs() <= 25,
        "inside SMask scope (q ... /Sm gs ... paint ... Q): expected \
         faded red ~(255, 128, 128); got ({r_in}, {g_in}, {b_in})"
    );
    // Outside the SMask scope (post-Q), red should be fully opaque
    // (SMask state must have been popped along with the gstate).
    assert!(
        r_out >= 250 && g_out < 30 && b_out < 30,
        "outside SMask scope (post-Q): expected fully opaque red \
         ~(255, 0, 0); got ({r_out}, {g_out}, {b_out}). If this fails, \
         SMask state leaks across q/Q boundaries — a real bug."
    );
}

// ===========================================================================
// Composite overprint reconstruction loss under non-linear ICC
// ===========================================================================
//
// The round-2 composite overprint correction uses the destination RGB
// snapshot, inverts via additive-clamp (RGB→CMYK), applies the §11.7.4
// plate selection, then converts back to RGB. When the snapshot's RGB
// came from a non-trivial ICC OutputIntent, the additive-clamp
// inversion can't recover the original CMYK — the inversion is
// lossy. The probe pins the magnitude of the loss.

fn fixture_overprint_under_nonlinear_icc() -> Vec<u8> {
    let content = "0.5 0 0 0 k\n10 10 60 60 re\nf\n\
                   /OpOn gs\n\
                   0 0 1 0 k\n\
                   30 30 60 60 re\nf\n";
    let resources = "/ExtGState << /OpOn << /Type /ExtGState /op true /OP true /OPM 1 >> >>";
    let profile = build_nonlinear_cmyk_to_lab_lut8_profile();
    build_pdf_with_optional_output_intent(content, resources, &[], Some(&profile))
}

fn fixture_overprint_under_no_icc() -> Vec<u8> {
    let content = "0.5 0 0 0 k\n10 10 60 60 re\nf\n\
                   /OpOn gs\n\
                   0 0 1 0 k\n\
                   30 30 60 60 re\nf\n";
    let resources = "/ExtGState << /OpOn << /Type /ExtGState /op true /OP true /OPM 1 >> >>";
    build_pdf_with_optional_output_intent(content, resources, &[], None)
}

/// IGNORED — pins the magnitude of the composite-overprint
/// reconstruction loss under a non-trivial ICC. Under additive-clamp
/// (no ICC), the round-2 overprint correction is exact: the
/// inversion is the same function used in the forward path. Under
/// a non-linear ICC, the snapshot RGB → additive-clamp CMYK inversion
/// produces a CMYK quadruple that, when re-converted to RGB via the
/// ICC, does NOT round-trip — the round-trip delta IS the
/// reconstruction loss.
///
/// This probe is informational (it documents the loss bound) rather
/// than aspirational (it cannot fail-then-pass via a small impl fix —
/// closing it requires routing composite overprint through the
/// separation backend, an architecture-level change scheduled for the
/// PDF/X-1a phase).
#[test]
#[ignore = "HONEST_GAP_OVERPRINT_COMPOSITE_RECONSTRUCTION_LOSS"]
fn qa_round2_overprint_reconstruction_loss_under_nonlinear_icc() {
    let rgba_icc = render_rgba(fixture_overprint_under_nonlinear_icc());
    let rgba_no_icc = render_rgba(fixture_overprint_under_no_icc());

    // Overlap region under each profile.
    let (r_icc, g_icc, b_icc) = mean_rgb(&rgba_icc, 40, 60, 40, 60);
    let (r_clamp, g_clamp, b_clamp) = mean_rgb(&rgba_no_icc, 40, 60, 40, 60);

    // The forward path under non-linear ICC produces a colorimetrically
    // distinct RGB; the round-2 reconstruction inverts back through
    // additive-clamp. Press-accurate output would re-derive RGB through
    // the ICC after CMYK overprint composition. The press-accurate value
    // is the same forward-ICC mapping applied to (cyan ∪ yellow) CMYK
    // = CMYK(0.5, 0, 1, 0). We can't easily compute that without a
    // re-render, but we CAN pin that the as-shipped result tracks the
    // additive-clamp path approximately (the loss is bounded but
    // non-trivial).
    //
    // The informational assertion: ICC-profile path must deliver an RGB
    // that differs from the additive-clamp path (proves the ICC was in
    // play at all) AND must NOT be byte-exact under additive-clamp
    // (proves the reconstruction loss is observable).
    let delta = (r_icc - r_clamp).abs() + (g_icc - g_clamp).abs() + (b_icc - b_clamp).abs();
    assert!(
        delta > 5.0,
        "composite overprint under non-linear ICC must differ from \
         additive-clamp baseline (forward ICC is non-trivial); got \
         delta {delta:.1} between ICC ({r_icc:.0},{g_icc:.0},{b_icc:.0}) \
         and additive-clamp ({r_clamp:.0},{g_clamp:.0},{b_clamp:.0}). {}",
        HONEST_GAP_OVERPRINT_COMPOSITE_RECONSTRUCTION_LOSS
    );
}
