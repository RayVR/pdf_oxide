//! Press-accurate OutputIntent CMYK ICC integration tests.
//!
//! Builds synthetic PDFs that declare an `/OutputIntents` array with a
//! CMYK `DestOutputProfile`, renders them through the composite path,
//! and pins that the resulting RGB values come from the qcms-driven
//! ICC conversion rather than the §10.3.5 additive-clamp fallback.
//!
//! The minimal CMYK ICC profile used here is synthesised in-test (see
//! `build_minimal_cmyk_to_rgb_lut8_profile` and the README in
//! `tests/fixtures/icc/`). It maps every CMYK input to a constant
//! `RGB(128, 128, 128)` so the pin is unambiguous: an OutputIntent-
//! driven render gives ~128 grey; an additive-clamp fallback gives the
//! §10.3.5 value for the input CMYK.

#![cfg(all(feature = "rendering", feature = "icc"))]
// Probe set grows across commits; the no-OutputIntent baseline
// builder lands ahead of its consumer.
#![allow(dead_code)]

use pdf_oxide::document::PdfDocument;
use pdf_oxide::rendering::{render_page, ImageFormat, RenderOptions};

// ===========================================================================
// QA round-1 tracking constants
// ===========================================================================
//
// Probes that lock behaviour the foundation does not yet ship are gated on
// `#[ignore = OUTPUT_INTENT_DEFER_*]` so a future engineer running the
// suite sees the open question by name instead of by silence. Each
// constant names the open question and the plan phase that will close
// it.
//
// Convention matches the wave-QA suites' `WAVE-DEFER-*` style so a
// `grep -RI 'OUTPUT_INTENT_DEFER_'` across the worktree pulls every pin
// that is currently on ice.

/// Caching of `Transform::new_srgb_target` calls. Each `k` / `K` operator
/// rebuilds the qcms transform today; the plan defers this to phase 7.
const OUTPUT_INTENT_DEFER_PHASE_7_CACHING: &str =
    "OUTPUT_INTENT_DEFER_PHASE_7_CACHING: plan phase 7 will cache compiled qcms transforms; \
     until then per-paint transform construction is the baseline";

/// Page-level `/DefaultCMYK` override (§8.6.5.6) is threaded onto the
/// `ResolutionContext` but the colour stage does not yet consume it; the
/// plan defers the consumer to phase 9. The probe lives here so the
/// future phase 9 commit deletes the `#[ignore]` rather than having to
/// invent the test from scratch.
const OUTPUT_INTENT_DEFER_PHASE_9_DEFAULT_CMYK: &str =
    "OUTPUT_INTENT_DEFER_PHASE_9_DEFAULT_CMYK: plan phase 9 will route /DefaultCMYK page-level \
     overrides ahead of the document /OutputIntents profile";

// ===========================================================================
// Minimal CMYK ICC profile synthesis
// ===========================================================================
//
// ICC v2 profile structure (per ICC.1:2004-10 §7):
//   - 128-byte header
//   - 4-byte tag count
//   - tag table: N × 12 bytes (signature, offset, size)
//   - tag data: each section 4-byte aligned
//
// Minimum tags qcms's CMYK→RGB transform path needs:
//   - A2B0 (mft1 LUT8 type): CMYK→PCS lookup
// qcms reads the LUT8 (entry-size 1, fixed 256-entry input/output tables)
// per ICC.1 §10.8. Layout inside the LUT8 tag data:
//   bytes 0..4    type signature 'mft1' (0x6d667431)
//   bytes 4..8    reserved zero
//   bytes 8       input channels (4 for CMYK)
//   bytes 9       output channels (3 for RGB)
//   bytes 10      grid points per dimension
//   bytes 11      padding
//   bytes 12..48  9 × s15Fixed16 matrix entries (identity for CMYK)
//   bytes 48..    input tables (input_channels × 256 bytes)
//   then          CLUT (grid_points^input_channels × output_channels bytes)
//   then          output tables (output_channels × 256 bytes)

/// Build a minimal valid ICC v2 CMYK→Lab profile whose A2B0 LUT8 maps
/// every CMYK input to a fixed Lab tuple. The PCS is `Lab ` rather
/// than `XYZ ` because qcms's Lab→XYZ→sRGB chain decodes the 8-bit
/// LUT8 outputs as `L = byte/255*100`, `a = byte - 128`, `b = byte -
/// 128` — easier to point at "neutral grey" than to compute the
/// matching XYZ tuple and round it into a LUT8 byte.
///
/// The constant CLUT makes the test pin unambiguous: whichever CMYK
/// quadruple the renderer feeds the profile, the qcms-converted RGB
/// is the same near-neutral grey that Lab(target_L, 0, 0) projects to
/// through sRGB. That's distinct from the §10.3.5 additive-clamp
/// value for any non-degenerate CMYK input, so a fallback to
/// additive-clamp is immediately visible.
///
/// `target_l_byte` is the LUT8 byte for the L* channel — e.g. 135 ≈
/// L*53, which projects through sRGB to roughly mid-grey
/// `RGB(~128, ~128, ~128)`. a* and b* are pinned at 128 (decoded as
/// 0, the achromatic axis).
fn build_minimal_cmyk_to_rgb_lut8_profile(target_l_byte: u8) -> Vec<u8> {
    // LUT8 tag body for in=4 out=3 grid=2.
    // Sizes:
    //   header: 48
    //   input tables: 4 * 256 = 1024
    //   CLUT: 2^4 * 3 = 48
    //   output tables: 3 * 256 = 768
    //   total: 1888 bytes
    let in_chan: u8 = 4;
    let out_chan: u8 = 3;
    let grid: u8 = 2;
    let mut lut = Vec::with_capacity(1888);

    // Type signature 'mft1'.
    lut.extend_from_slice(&0x6d66_7431u32.to_be_bytes());
    // Reserved.
    lut.extend_from_slice(&0u32.to_be_bytes());
    lut.push(in_chan);
    lut.push(out_chan);
    lut.push(grid);
    lut.push(0); // padding

    // 9 × s15Fixed16 matrix entries (identity matrix). qcms reads these
    // off the LUT8 tag header at offsets 12..48 even for CMYK inputs;
    // they only matter for RGB inputs but qcms still parses them.
    // Identity matrix: 1.0 along diagonal.
    let identity: [i32; 9] = [0x0001_0000, 0, 0, 0, 0x0001_0000, 0, 0, 0, 0x0001_0000];
    for v in identity {
        lut.extend_from_slice(&(v as u32).to_be_bytes());
    }

    // Input tables — identity 0..255 for each of 4 input channels.
    for _ in 0..in_chan {
        for i in 0..256u16 {
            lut.push(i as u8);
        }
    }

    // CLUT: 2^4 × 3 = 16 grid points × 3 output channels.
    // Every grid point outputs Lab(target_L, 0, 0) — neutral grey at the
    // requested lightness. qcms decodes LUT8 outputs through the chain
    //   L = byte/255 * 100
    //   a = byte - 128
    //   b = byte - 128
    // so target_l_byte directly controls L*; a* and b* are pinned at
    // 128 (decoded as the achromatic axis 0).
    let grid_size = (grid as usize).pow(in_chan as u32);
    for _ in 0..grid_size {
        lut.push(target_l_byte);
        lut.push(128);
        lut.push(128);
    }

    // Output tables — identity 0..255 for each of 3 output channels.
    for _ in 0..out_chan {
        for i in 0..256u16 {
            lut.push(i as u8);
        }
    }

    debug_assert_eq!(lut.len(), 1888, "LUT8 body size mismatch");

    // ICC profile envelope: 128-byte header + tag table + tag data.
    // Total profile size: 128 (header) + 4 (count) + 12 (one tag entry)
    // + 1888 (A2B0 data) = 2032 bytes, with the A2B0 data starting at
    // offset 144.
    let mut profile = vec![0u8; 128];
    let total_size: u32 = 128 + 4 + 12 + lut.len() as u32;

    // Profile size at bytes 0..4.
    profile[0..4].copy_from_slice(&total_size.to_be_bytes());
    // Preferred CMM at bytes 4..8 — left zero (no preference).
    // Profile version: 2.4.0.0 at bytes 8..12.
    profile[8..12].copy_from_slice(&0x0240_0000u32.to_be_bytes());
    // Device class: 'prtr' (output device).
    profile[12..16].copy_from_slice(b"prtr");
    // Colour space: 'CMYK'.
    profile[16..20].copy_from_slice(b"CMYK");
    // PCS: 'Lab ' — qcms's LABtoXYZ stage gives us a straightforward
    // mapping from "byte in CLUT" to "near-neutral grey at L*≈53".
    profile[20..24].copy_from_slice(b"Lab ");
    // Creation date (12 bytes) at 24..36 — all-zero.
    // Profile signature 'acsp' at 36..40.
    profile[36..40].copy_from_slice(b"acsp");
    // Primary platform at 40..44 — zero.
    // Flags / device manufacturer / model / attributes — all zero through
    // byte 100. Rendering intent at 64..68 (0 = perceptual).
    profile[64..68].copy_from_slice(&0u32.to_be_bytes());
    // Illuminant XYZ at 68..80 — D50 (0.9642, 1.0, 0.8249).
    profile[68..72].copy_from_slice(&0x0000_F6D6u32.to_be_bytes()); // X 0.9642
    profile[72..76].copy_from_slice(&0x0001_0000u32.to_be_bytes()); // Y 1.0
    profile[76..80].copy_from_slice(&0x0000_D32Du32.to_be_bytes()); // Z 0.8249
                                                                    // Creator at 80..84 — zero.

    // Tag table: count = 1, then one entry (signature, offset, size).
    profile.extend_from_slice(&1u32.to_be_bytes());
    profile.extend_from_slice(&0x4132_4230u32.to_be_bytes()); // 'A2B0'
    profile.extend_from_slice(&144u32.to_be_bytes()); // offset
    profile.extend_from_slice(&(lut.len() as u32).to_be_bytes()); // size

    // A2B0 tag data.
    profile.extend_from_slice(&lut);

    profile
}

// ===========================================================================
// PDF construction helpers
// ===========================================================================

/// Build a one-page PDF with the given catalog entries and content
/// stream. The catalog entries string is spliced into the catalog
/// dictionary so callers can add `/OutputIntents [...]` without
/// reconstructing the whole envelope.
///
/// MediaBox is fixed at `[0 0 100 100]`; rendering at 72 DPI gives a
/// 100×100 pixel canvas so callers can pin pixels at known offsets.
fn build_pdf_with_catalog_entries_and_content(
    catalog_entries: &str,
    content_ops: &str,
    icc_profile_bytes: Option<&[u8]>,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    let catalog =
        format!("1 0 obj\n<< /Type /Catalog /Pages 2 0 R {} >>\nendobj\n", catalog_entries);
    buf.extend_from_slice(catalog.as_bytes());

    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    let page_off = buf.len();
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << >> /Contents 4 0 R >>\nendobj\n",
    );

    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let icc_off;
    let obj_count;
    if let Some(icc) = icc_profile_bytes {
        icc_off = buf.len();
        let icc_hdr = format!("5 0 obj\n<< /N 4 /Length {} >>\nstream\n", icc.len());
        buf.extend_from_slice(icc_hdr.as_bytes());
        buf.extend_from_slice(icc);
        buf.extend_from_slice(b"\nendstream\nendobj\n");
        obj_count = 6;
    } else {
        icc_off = 0;
        obj_count = 5;
    }

    let xref_off = buf.len();
    buf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", obj_count).as_bytes());
    for off in [cat_off, pages_off, page_off, stream_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    if icc_profile_bytes.is_some() {
        buf.extend_from_slice(format!("{:010} 00000 n \n", icc_off).as_bytes());
    }
    buf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            obj_count, xref_off
        )
        .as_bytes(),
    );
    buf
}

/// Build a PDF whose page paints CMYK(0.25, 0, 0, 0) into a 60×60
/// rect centred on the canvas and whose catalog declares
/// `/OutputIntents [<< /Type /OutputIntent /S /GTS_PDFX
/// /OutputCondition (Synthetic CMYK) /DestOutputProfile 5 0 R >>]`.
fn build_pdf_cmyk_with_output_intent(icc_profile_bytes: &[u8]) -> Vec<u8> {
    let catalog_entries = "/OutputIntents [<< /Type /OutputIntent /S /GTS_PDFX /OutputCondition (Synthetic CMYK) /DestOutputProfile 5 0 R >>]";
    // PDF user space is bottom-left origin; the rect at (20, 20, 60, 60)
    // covers the canvas centre.
    let content_ops = "0.25 0 0 0 k\n20 20 60 60 re\nf\n";
    build_pdf_with_catalog_entries_and_content(
        catalog_entries,
        content_ops,
        Some(icc_profile_bytes),
    )
}

/// Same paint operator as `build_pdf_cmyk_with_output_intent` but with
/// no `/OutputIntents` in the catalog. Pins the §10.3.5 fallback.
fn build_pdf_cmyk_without_output_intent() -> Vec<u8> {
    let content_ops = "0.25 0 0 0 k\n20 20 60 60 re\nf\n";
    build_pdf_with_catalog_entries_and_content("", content_ops, None)
}

/// Build a PDF whose page paints a `/Separation` colour space (with a
/// Type-4 PostScript tint transform that produces CMYK(0, tint, 0, 0))
/// against a document-level `/OutputIntents` CMYK profile.
///
/// Object layout:
///   1 — Catalog (with /OutputIntents → 5 0 R)
///   2 — Pages
///   3 — Page (with Resources /ColorSpace /CS1 →
///       [/Separation /MagentaSpot /DeviceCMYK 6 0 R])
///   4 — Content stream
///   5 — OutputIntent profile stream
///   6 — Tint-transform Type-4 stream
///
/// The Type-4 program `{ 0.0 exch 0.0 0.0 }` lifts the input tint into
/// the M position so the alternate-space output is CMYK(0, tint, 0, 0).
fn build_pdf_separation_type4_devicecmyk_with_output_intent(
    output_intent_profile: &[u8],
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    let catalog = "1 0 obj\n<< /Type /Catalog /Pages 2 0 R /OutputIntents [<< /Type /OutputIntent /S /GTS_PDFX /OutputCondition (Synthetic CMYK) /DestOutputProfile 5 0 R >>] >>\nendobj\n";
    buf.extend_from_slice(catalog.as_bytes());

    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    let page_off = buf.len();
    let page = "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << /ColorSpace << /CS1 [/Separation /MagentaSpot /DeviceCMYK 6 0 R] >> >> /Contents 4 0 R >>\nendobj\n";
    buf.extend_from_slice(page.as_bytes());

    let stream_off = buf.len();
    // Activate the Separation colour space and paint the rect with full
    // tint (1.0). With the Type-4 program below, the tint transform
    // produces CMYK(0, 1, 0, 0).
    let content = "/CS1 cs\n1.0 scn\n20 20 60 60 re\nf\n";
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let icc_off = buf.len();
    let icc_hdr = format!("5 0 obj\n<< /N 4 /Length {} >>\nstream\n", output_intent_profile.len());
    buf.extend_from_slice(icc_hdr.as_bytes());
    buf.extend_from_slice(output_intent_profile);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let tint_off = buf.len();
    // Type 4 PostScript tint transform. Stack semantics per the
    // resolver-side test in src/rendering/resolution/color.rs:697:
    // `{ 0.0 exch 0.0 0.0 }` consumes input tint and leaves the stack
    // bottom-to-top as [0, tint, 0, 0] — i.e. CMYK output (C=0, M=tint,
    // Y=0, K=0). Domain [0 1] is the input range; Range [0 1 0 1 0 1 0 1]
    // is the four-component CMYK output range.
    let tint_program: &[u8] = b"{ 0.0 exch 0.0 0.0 }";
    let tint_hdr = format!(
        "6 0 obj\n<< /FunctionType 4 /Domain [0 1] /Range [0 1 0 1 0 1 0 1] /Length {} >>\nstream\n",
        tint_program.len()
    );
    buf.extend_from_slice(tint_hdr.as_bytes());
    buf.extend_from_slice(tint_program);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xref_off = buf.len();
    let obj_count = 7;
    buf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", obj_count).as_bytes());
    for off in [cat_off, pages_off, page_off, stream_off, icc_off, tint_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            obj_count, xref_off
        )
        .as_bytes(),
    );
    buf
}

/// Same shape as `build_pdf_separation_type4_devicecmyk_with_output_intent`
/// but the colour space is a 2-colorant `/DeviceN` whose alternate is
/// `/DeviceCMYK` and whose Type-4 tint transform consumes the two input
/// tints and emits CMYK(0, tint0, 0, 0) — i.e. only the first input
/// drives the magenta component, the second is dropped. With content
/// `[1.0 0.5] scn` the input is (tint0=1.0, tint1=0.5) and the output is
/// CMYK(0, 1, 0, 0).
fn build_pdf_devicen_type4_devicecmyk_with_output_intent(output_intent_profile: &[u8]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    let catalog = "1 0 obj\n<< /Type /Catalog /Pages 2 0 R /OutputIntents [<< /Type /OutputIntent /S /GTS_PDFX /OutputCondition (Synthetic CMYK) /DestOutputProfile 5 0 R >>] >>\nendobj\n";
    buf.extend_from_slice(catalog.as_bytes());

    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    let page_off = buf.len();
    // DeviceN colorant array: two named spot inks. The tint-transform
    // function is referenced by indirect object.
    let page = "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << /ColorSpace << /CS1 [/DeviceN [/Magenta /Cyan] /DeviceCMYK 6 0 R] >> >> /Contents 4 0 R >>\nendobj\n";
    buf.extend_from_slice(page.as_bytes());

    let stream_off = buf.len();
    // Activate the DeviceN colour space and paint with two component
    // tints (1.0, 0.5). The Type-4 tint transform drops the second tint
    // and emits CMYK(0, tint0, 0, 0) = CMYK(0, 1, 0, 0).
    let content = "/CS1 cs\n1.0 0.5 scn\n20 20 60 60 re\nf\n";
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let icc_off = buf.len();
    let icc_hdr = format!("5 0 obj\n<< /N 4 /Length {} >>\nstream\n", output_intent_profile.len());
    buf.extend_from_slice(icc_hdr.as_bytes());
    buf.extend_from_slice(output_intent_profile);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let tint_off = buf.len();
    // Type 4 program with two inputs (the two DeviceN colorant tints).
    // Stack on entry: [tint0, tint1]. Program: `{ pop 0.0 exch 0.0 0.0 }`
    // pops tint1, then `0.0 exch 0.0 0.0` leaves stack bottom-to-top as
    // [0, tint0, 0, 0] (C=0, M=tint0, Y=0, K=0).
    let tint_program: &[u8] = b"{ pop 0.0 exch 0.0 0.0 }";
    let tint_hdr = format!(
        "6 0 obj\n<< /FunctionType 4 /Domain [0 1 0 1] /Range [0 1 0 1 0 1 0 1] /Length {} >>\nstream\n",
        tint_program.len()
    );
    buf.extend_from_slice(tint_hdr.as_bytes());
    buf.extend_from_slice(tint_program);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xref_off = buf.len();
    let obj_count = 7;
    buf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", obj_count).as_bytes());
    for off in [cat_off, pages_off, page_off, stream_off, icc_off, tint_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            obj_count, xref_off
        )
        .as_bytes(),
    );
    buf
}

/// Build a PDF that declares BOTH an `/OutputIntents` CMYK profile A and
/// a page-resources `/ColorSpace /CS1 [/ICCBased <stream>]` colour space
/// whose embedded N=4 profile B is a DIFFERENT minimal CMYK profile. The
/// content stream sets fill colour space to `/CS1` and paints with
/// `0.25 0 0 0 scn`.
///
/// Object layout:
///   1 — Catalog (with /OutputIntents → 5 0 R)
///   2 — Pages
///   3 — Page (with Resources /ColorSpace /CS1 → ICCBased referencing 6 0 R)
///   4 — Content stream
///   5 — OutputIntent profile A stream
///   6 — ICCBased embedded profile B stream
fn build_pdf_embedded_iccbased_with_different_output_intent(
    output_intent_profile_a: &[u8],
    embedded_iccbased_profile_b: &[u8],
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    let catalog = "1 0 obj\n<< /Type /Catalog /Pages 2 0 R /OutputIntents [<< /Type /OutputIntent /S /GTS_PDFX /OutputCondition (Synthetic CMYK A) /DestOutputProfile 5 0 R >>] >>\nendobj\n";
    buf.extend_from_slice(catalog.as_bytes());

    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    let page_off = buf.len();
    // Resources declare an `ICCBased` colour space CS1 whose stream is
    // object 6 — the alternate profile B. Painting `0.25 0 0 0 scn`
    // against CS1 feeds the four components into the embedded profile.
    let page = "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << /ColorSpace << /CS1 [/ICCBased 6 0 R] >> >> /Contents 4 0 R >>\nendobj\n";
    buf.extend_from_slice(page.as_bytes());

    let stream_off = buf.len();
    // Set fill colour space to CS1, then paint a 60×60 rect at the centre
    // with the four CMYK components via `scn`. The integer-form fill
    // operator `cs` selects the named colour space.
    let content = "/CS1 cs\n0.25 0 0 0 scn\n20 20 60 60 re\nf\n";
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let icc_a_off = buf.len();
    let icc_a_hdr =
        format!("5 0 obj\n<< /N 4 /Length {} >>\nstream\n", output_intent_profile_a.len());
    buf.extend_from_slice(icc_a_hdr.as_bytes());
    buf.extend_from_slice(output_intent_profile_a);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let icc_b_off = buf.len();
    let icc_b_hdr =
        format!("6 0 obj\n<< /N 4 /Length {} >>\nstream\n", embedded_iccbased_profile_b.len());
    buf.extend_from_slice(icc_b_hdr.as_bytes());
    buf.extend_from_slice(embedded_iccbased_profile_b);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xref_off = buf.len();
    let obj_count = 7;
    buf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", obj_count).as_bytes());
    for off in [
        cat_off, pages_off, page_off, stream_off, icc_a_off, icc_b_off,
    ] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            obj_count, xref_off
        )
        .as_bytes(),
    );
    buf
}

fn render_rgba(doc: &PdfDocument) -> Vec<u8> {
    let opts = RenderOptions::with_dpi(72).as_raw();
    let img = render_page(doc, 0, &opts).expect("render_page");
    assert_eq!(img.format, ImageFormat::RawRgba8);
    img.data
}

fn pixel_at(rgba: &[u8], x: u32, y: u32) -> (u8, u8, u8, u8) {
    let w = 100u32;
    let h = 100u32;
    assert_eq!(rgba.len() as u32, w * h * 4);
    assert!(x < w && y < h);
    let off = ((y * w + x) * 4) as usize;
    (rgba[off], rgba[off + 1], rgba[off + 2], rgba[off + 3])
}

// ===========================================================================
// Phase 2 positive test
// ===========================================================================

/// Pin that a /DeviceCMYK fill on a page whose document declares a
/// CMYK `/OutputIntents` profile is rendered via the qcms-driven ICC
/// path rather than ISO 32000-1:2008 §10.3.5's additive-clamp formula.
///
/// Fixture details:
///   - CMYK input: (0.25, 0, 0, 0) — modest cyan tint.
///   - Profile: minimal in-test CMYK→RGB LUT8 that maps every CMYK input
///     to constant `RGB(128, 128, 128)`. With the OutputIntent path
///     live, every pixel inside the rect must be ~128 grey on every
///     channel. With the additive-clamp fallback the pixel would be
///     `(191, 255, 255)` — `1 - (C + K)`, `1 - (M + K)`, `1 - (Y + K)`
///     scaled to bytes.
#[test]
fn device_cmyk_paint_with_output_intent_renders_via_icc_not_additive_clamp() {
    // L*53 maps roughly to sRGB(128, 128, 128) — a clear non-additive-
    // clamp anchor for CMYK(0.25, 0, 0, 0).
    let target_l_byte: u8 = 135;
    let icc = build_minimal_cmyk_to_rgb_lut8_profile(target_l_byte);
    // First sanity-check the synthesised profile compiles into a real
    // qcms transform — otherwise the test would silently degrade to
    // the §10.3.5 fallback and the assertion below would fail for the
    // wrong reason. The transform-build path is the same one the
    // composite renderer will exercise on this profile.
    {
        use pdf_oxide::color::{IccProfile, RenderingIntent, Transform};
        use std::sync::Arc;
        let prof = Arc::new(
            IccProfile::parse(icc.clone(), 4)
                .expect("synthesised profile parses through IccProfile::parse"),
        );
        let t = Transform::new_srgb_target(prof, RenderingIntent::RelativeColorimetric);
        assert!(
            t.has_cmm(),
            "synthesised profile must compile into a real qcms transform; \
             without it the OutputIntent test degrades to the additive-clamp \
             fallback and asserts the wrong thing"
        );
        // Sanity-pin the constant CLUT actually drives qcms: with this
        // profile every CMYK input must produce roughly (128, 128, 128).
        // qcms tetra-CLUT interpolation on a 2^4 grid with constant
        // output should be exact to within rounding.
        let rgb = t.convert_cmyk_pixel(64, 0, 0, 0);
        // Lab(53, 0, 0) → sRGB ≈ (128, 128, 128) within rounding. Tolerate
        // ±10 per channel — Lab→XYZ→sRGB through the qcms pipeline rounds
        // at multiple steps and ICC v2 Lab encoding has its own scale
        // quantisation.
        let near = |a: u8, b: u8| (a as i32 - b as i32).abs() <= 10;
        assert!(
            near(rgb[0], 128) && near(rgb[1], 128) && near(rgb[2], 128),
            "qcms must drive the constant CLUT: got {rgb:?}, want ~(128, 128, 128) \
             ±10 (Lab(53,0,0) → sRGB grey)"
        );
    }

    let pdf = build_pdf_cmyk_with_output_intent(&icc);
    let doc = PdfDocument::from_bytes(pdf).expect("open synthetic PDF");
    // Re-confirm the document accessor finds the OutputIntent. If this
    // returns None the test isn't actually probing the OutputIntent
    // path — it'd just probe the no-OutputIntent baseline.
    let oi = doc
        .output_intent_cmyk_profile()
        .expect("synthetic catalog declares a CMYK OutputIntent");
    assert_eq!(oi.n_components(), 4, "OutputIntent must be /N=4");

    let rgba = render_rgba(&doc);
    let (r, g, b, _a) = pixel_at(&rgba, 50, 50);

    // Additive-clamp value for CMYK(0.25, 0, 0, 0) is RGB(0.75, 1.0, 1.0)
    // = (191, 255, 255). The qcms-converted value is ~(128, 128, 128).
    // Tolerance ±10 absorbs Lab → XYZ → sRGB rounding through the chain.
    let near_const = |v: u8| (v as i32 - 128).abs() <= 10;
    assert!(
        near_const(r) && near_const(g) && near_const(b),
        "OutputIntent /DeviceCMYK paint expected qcms-converted RGB ~(128, 128, 128); \
         got ({r}, {g}, {b}). RGB(191, 255, 255) would mean the §10.3.5 additive-clamp \
         fallback fired — the resolver is not consulting ctx.output_intent_cmyk."
    );
}

// ===========================================================================
// Negative pin: no OutputIntent → §10.3.5 additive-clamp preserved
// ===========================================================================

/// Pin that a /DeviceCMYK fill on a page whose document declares no
/// `/OutputIntents` array is rendered through ISO 32000-1:2008
/// §10.3.5's additive-clamp formula, byte-for-byte, as it shipped
/// before OutputIntent threading landed.
///
/// This is the contrapositive of the positive test: when
/// `ctx.output_intent_cmyk` is `None`, the resolver MUST fall through
/// to the shipped behaviour. A bug that unconditionally consulted
/// some other ICC profile (or that flipped the precedence rules) would
/// surface here as the wrong colour.
#[test]
fn device_cmyk_paint_without_output_intent_renders_additive_clamp() {
    let pdf = build_pdf_cmyk_without_output_intent();
    let doc = PdfDocument::from_bytes(pdf).expect("open synthetic PDF");
    // Cross-check the catalog has no OutputIntent — if it did, this
    // test would conflate "no OI" with "OI that happens to produce
    // additive-clamp values" and could pass for the wrong reason.
    assert!(
        doc.output_intent_cmyk_profile().is_none(),
        "fixture must declare no /OutputIntents in catalog"
    );

    let rgba = render_rgba(&doc);
    let (r, g, b, _a) = pixel_at(&rgba, 50, 50);

    // CMYK(0.25, 0, 0, 0) → additive-clamp:
    //   R = 1 - (0.25 + 0) = 0.75 → 191
    //   G = 1 - (0.00 + 0) = 1.00 → 255
    //   B = 1 - (0.00 + 0) = 1.00 → 255
    assert_eq!(
        (r, g, b),
        (191, 255, 255),
        "without /OutputIntents the §10.3.5 additive-clamp fallback must \
         be preserved byte-for-byte; got ({r}, {g}, {b})"
    );
}

// ===========================================================================
// QA: byte-exact Lab→sRGB pin (replaces the ±10 hand-wave)
// ===========================================================================

/// Byte-exact pin of the qcms reference value the synthesised
/// `target_l_byte=135` profile yields.
///
/// The existing positive test (`device_cmyk_paint_with_output_intent_*`)
/// asserts the rendered pixel falls within `(128, 128, 128) ± 10` per
/// channel — that's a hand-wave that hides up to a ~9-byte channel-by-
/// channel drift. Derived against qcms 0.3.0 (the version pinned in
/// Cargo.lock at this commit), the byte-exact reference for
/// `target_l_byte=135` + CMYK(64,0,0,0) at `RelativeColorimetric` is
/// `(126, 126, 126)`. The rendered pixel at (50, 50) through the
/// composite pipeline is `(126, 126, 126, 255)`. We pin both — any
/// drift in the qcms chain (Lab→XYZ→sRGB), the LUT8 tetra-interp, or
/// the resolver's 8-bit round-trip surfaces here byte-for-byte.
///
/// If a future qcms upgrade shifts the reference, the right answer is
/// to re-derive the value here, not to widen the tolerance — `±10` was
/// the impl-agent's tolerance for an unmeasured target; this probe pins
/// the actual measured target.
#[test]
fn output_intent_render_pixel_is_byte_exact_against_qcms_reference() {
    use pdf_oxide::color::{IccProfile, RenderingIntent, Transform};
    use std::sync::Arc;

    let target_l_byte: u8 = 135;
    let icc = build_minimal_cmyk_to_rgb_lut8_profile(target_l_byte);

    // Standalone transform: pin the qcms output byte-for-byte against
    // the derived reference. CMYK(64, 0, 0, 0) is the input the
    // positive integration test feeds for its sanity check.
    {
        let prof = Arc::new(IccProfile::parse(icc.clone(), 4).expect("parse"));
        let t = Transform::new_srgb_target(prof, RenderingIntent::RelativeColorimetric);
        let rgb = t.convert_cmyk_pixel(64, 0, 0, 0);
        assert_eq!(
            rgb,
            [126u8, 126, 126],
            "qcms 0.3.0 byte-exact reference for target_l_byte=135 + CMYK(64,0,0,0): \
             expected (126, 126, 126); got {rgb:?}. Re-derive (see plan errata) if qcms \
             ever changes its Lab→sRGB chain — do not widen tolerance."
        );
    }

    // Through the composite renderer: pin the rendered pixel at the
    // centre of the painted rect byte-for-byte.
    let pdf = build_pdf_cmyk_with_output_intent(&icc);
    let doc = PdfDocument::from_bytes(pdf).expect("open");
    let rgba = render_rgba(&doc);
    let (r, g, b, a) = pixel_at(&rgba, 50, 50);
    assert_eq!(
        (r, g, b, a),
        (126u8, 126, 126, 255),
        "rendered pixel must match the qcms reference byte-for-byte; got ({r},{g},{b},{a}). \
         (191,255,255,_) means the §10.3.5 fallback fired."
    );
}

/// Pin the qcms reference value is intent-independent for the synthesised
/// constant-CLUT profile.
///
/// The constant-CLUT shape of the synthesised profile means a CMM whose
/// gamut compression depends on rendering intent (which is the whole
/// point of having intents) should still produce the same value — there's
/// no out-of-gamut excursion to compress. If qcms ever starts producing
/// different values per intent on a constant CLUT that's a CMM bug
/// worth surfacing.
#[test]
fn output_intent_constant_clut_is_invariant_across_rendering_intents() {
    use pdf_oxide::color::{IccProfile, RenderingIntent, Transform};
    use std::sync::Arc;

    let icc = build_minimal_cmyk_to_rgb_lut8_profile(135);
    let prof = Arc::new(IccProfile::parse(icc, 4).expect("parse"));
    let mut last: Option<[u8; 3]> = None;
    for intent in [
        RenderingIntent::Perceptual,
        RenderingIntent::RelativeColorimetric,
        RenderingIntent::Saturation,
        RenderingIntent::AbsoluteColorimetric,
    ] {
        let t = Transform::new_srgb_target(Arc::clone(&prof), intent);
        let rgb = t.convert_cmyk_pixel(64, 0, 0, 0);
        if let Some(prev) = last {
            assert_eq!(
                prev, rgb,
                "constant-CLUT qcms output must be identical across rendering intents; \
                 first intent yielded {prev:?}, intent={intent:?} yielded {rgb:?}"
            );
        }
        last = Some(rgb);
    }
}

// ===========================================================================
// QA: qcms validation fragility — bad-profile fall-through
// ===========================================================================

/// Pin that a syntactically-shaped but tag-table-truncated CMYK profile
/// declared on `/OutputIntents` does not crash the renderer and produces
/// the §10.3.5 fallback colour byte-for-byte.
///
/// This is the impl-agent's open-question #1 surfaced as a probe: when
/// qcms refuses to compile the OutputIntent profile, `Transform::
/// convert_cmyk_pixel` devolves internally — but the renderer-level
/// behaviour must be (a) no panic and (b) the same RGB the no-
/// OutputIntent fixture produces, so a malformed `/OutputIntents`
/// degrades gracefully.
#[test]
fn output_intent_with_unparseable_profile_falls_through_to_additive_clamp() {
    // Header-only profile: parses through `IccProfile::parse` (which
    // only validates the 128-byte header), but qcms refuses at build
    // time because there's no tag table. Mirrors the stub the in-source
    // unit test in color.rs uses but reaches the rasteriser end-to-end.
    let mut header_only = vec![0u8; 128];
    header_only[8..12].copy_from_slice(&0x0400_0000u32.to_be_bytes());
    header_only[12..16].copy_from_slice(b"prtr");
    header_only[16..20].copy_from_slice(b"CMYK");
    header_only[20..24].copy_from_slice(b"Lab ");
    header_only[36..40].copy_from_slice(b"acsp");

    let pdf = build_pdf_cmyk_with_output_intent(&header_only);
    let doc = PdfDocument::from_bytes(pdf).expect("open");

    // Sanity-pin: the document-level accessor still hands back the
    // parsed-header profile, so the renderer DOES see a Some on
    // `ctx.output_intent_cmyk` — the fall-through has to happen inside
    // `convert_cmyk_pixel`, not by the accessor returning None.
    assert!(
        doc.output_intent_cmyk_profile().is_some(),
        "header-only stub must parse through IccProfile::parse; fall-through must \
         happen inside Transform::convert_cmyk_pixel, not at the accessor"
    );

    let rgba = render_rgba(&doc);
    let (r, g, b, a) = pixel_at(&rgba, 50, 50);
    assert_eq!(
        (r, g, b, a),
        (191u8, 255, 255, 255),
        "unparseable OutputIntent profile must fall through to §10.3.5 byte-exact; \
         got ({r},{g},{b},{a})"
    );
}

/// Pin that an OutputIntent profile whose ICC header declares a non-CMYK
/// colour space (`RGB `, `GRAY`, `Lab `) is filtered out by
/// `IccProfile::parse`'s cross-check, even though the stream dict's
/// `/N 4` would otherwise let it through the accessor.
///
/// `IccProfile::parse(bytes, declared_n)` at `src/color.rs:159` requires
/// that the ICC header's implied component count match the stream
/// dict's `/N`. An `RGB ` header implies `n=3`; `declared_n=4` → reject.
/// `output_intent_cmyk_profile` then returns `None`, and the renderer
/// falls back to §10.3.5 byte-for-byte.
///
/// This is the strongest gate: a malformed profile that lied about
/// colour space in the ICC header gets rejected before reaching qcms.
/// A regression that loosened the cross-check would let the qcms layer
/// see CMYK bytes through an RGB profile — at best garbage, at worst a
/// panic in the CMM.
#[test]
fn output_intent_with_mismatched_icc_header_colour_space_is_rejected_at_parse() {
    let mut header_only = vec![0u8; 128];
    header_only[8..12].copy_from_slice(&0x0400_0000u32.to_be_bytes());
    header_only[12..16].copy_from_slice(b"prtr");
    header_only[16..20].copy_from_slice(b"RGB "); // intentionally mismatched
    header_only[20..24].copy_from_slice(b"Lab ");
    header_only[36..40].copy_from_slice(b"acsp");

    let pdf = build_pdf_cmyk_with_output_intent(&header_only);
    let doc = PdfDocument::from_bytes(pdf).expect("open");
    // IccProfile::parse rejects the mismatch (header→n=3 vs declared_n=4);
    // the accessor surfaces None.
    assert!(
        doc.output_intent_cmyk_profile().is_none(),
        "IccProfile::parse must reject when ICC header colour-space \
         tag implies a different component count than the stream's /N"
    );
    // Renderer falls through to §10.3.5 byte-for-byte.
    let rgba = render_rgba(&doc);
    let (r, g, b, a) = pixel_at(&rgba, 50, 50);
    assert_eq!(
        (r, g, b, a),
        (191u8, 255, 255, 255),
        "mismatched-header OutputIntent must fall through to §10.3.5; got ({r},{g},{b},{a})"
    );
}

// ===========================================================================
// QA: helper-level consistency (§10.3.5 source-of-truth probe)
// ===========================================================================

/// Pin that `crate::extractors::images::cmyk_pixel_to_rgb` and the
/// resolver helper's no-OutputIntent arm produce the same RGB bytes on
/// the same CMYK quadruple.
///
/// This is the HONEST_GAP the impl agent flagged in
/// `cmyk_to_rgb_via_intent_falls_back_when_profile_has_no_cmm`. Verified
/// here at the public-API level by routing both paths through a known
/// CMYK input and comparing byte-for-byte. If a future refactor diverges
/// the two §10.3.5 implementations, the fallback path inside qcms's
/// no-CMM arm could disagree with the resolver's bare-fallback arm even
/// though both intend the spec formula.
///
/// The probe iterates over a handful of representative inputs — pure
/// process inks, the test fixture's input, and a few interior CMYK
/// quadruples. Every input must agree.
#[test]
fn additive_clamp_consistency_between_extractors_helper_and_no_output_intent_arm() {
    use pdf_oxide::color::{IccProfile, RenderingIntent, Transform};
    use std::sync::Arc;

    // Build a header-only stub: qcms refuses, Transform::convert_cmyk_pixel
    // devolves to crate::extractors::images::cmyk_pixel_to_rgb internally
    // (verified at src/color.rs:301). That's the reference "no-CMM
    // fallback" path.
    let mut header_only = vec![0u8; 128];
    header_only[8..12].copy_from_slice(&0x0400_0000u32.to_be_bytes());
    header_only[12..16].copy_from_slice(b"prtr");
    header_only[16..20].copy_from_slice(b"CMYK");
    header_only[20..24].copy_from_slice(b"Lab ");
    header_only[36..40].copy_from_slice(b"acsp");
    let prof = Arc::new(IccProfile::parse(header_only, 4).expect("parse"));
    let t = Transform::new_srgb_target(prof, RenderingIntent::RelativeColorimetric);

    // The §10.3.5 formula in plain Rust — re-derived here so we don't
    // import the crate-private helper. Both the Transform no-CMM arm
    // and the resolver fallback must agree with this.
    fn spec_additive_clamp(c: u8, m: u8, y: u8, k: u8) -> [u8; 3] {
        let cf = c as f32 / 255.0;
        let mf = m as f32 / 255.0;
        let yf = y as f32 / 255.0;
        let kf = k as f32 / 255.0;
        let r = ((1.0 - (cf + kf).min(1.0)) * 255.0).round() as u8;
        let g = ((1.0 - (mf + kf).min(1.0)) * 255.0).round() as u8;
        let b = ((1.0 - (yf + kf).min(1.0)) * 255.0).round() as u8;
        [r, g, b]
    }

    for (c, m, y, k) in [
        (0u8, 0, 0, 0),
        (255, 0, 0, 0),
        (0, 255, 0, 0),
        (0, 0, 255, 0),
        (0, 0, 0, 255),
        (64, 0, 0, 0), // fixture input
        (128, 128, 128, 128),
        (200, 100, 50, 25),
    ] {
        let from_transform = t.convert_cmyk_pixel(c, m, y, k);
        let from_spec = spec_additive_clamp(c, m, y, k);
        assert_eq!(
            from_transform, from_spec,
            "Transform no-CMM fallback must agree with §10.3.5 spec on CMYK({c},{m},{y},{k}); \
             transform={from_transform:?}, spec={from_spec:?}"
        );
    }
}

// ===========================================================================
// QA: foundation coverage probes (q/Q, alpha edges, deferred placeholders)
// ===========================================================================

/// Pin that DeviceCMYK paint inside a `q ... Q` save-restore bracket
/// still routes through the OutputIntent ICC.
///
/// `q`/`Q` push/pop the graphics state; a regression that re-built the
/// resolution context inside the bracket without re-attaching the
/// OutputIntent borrow would lose the ICC routing on the inner paint
/// even though it's the same page.
#[test]
fn output_intent_survives_graphics_state_save_restore() {
    let icc = build_minimal_cmyk_to_rgb_lut8_profile(135);
    // q / fill / Q bracket performing the CMYK paint inside a fresh
    // graphics-state scope. The inner paint must still hit ICC.
    let catalog_entries =
        "/OutputIntents [<< /Type /OutputIntent /S /GTS_PDFX /OutputCondition (S) /DestOutputProfile 5 0 R >>]";
    let content = "q\n0.25 0 0 0 k\n20 20 60 60 re\nf\nQ\n";
    let pdf = build_pdf_with_catalog_entries_and_content(catalog_entries, content, Some(&icc));
    let doc = PdfDocument::from_bytes(pdf).expect("open");
    let rgba = render_rgba(&doc);
    let (r, g, b, a) = pixel_at(&rgba, 50, 50);
    assert_eq!(
        (r, g, b, a),
        (126u8, 126, 126, 255),
        "DeviceCMYK paint inside q/Q must still route through OutputIntent ICC; got ({r},{g},{b},{a})"
    );
}

/// Pin that a fully-opaque DeviceCMYK paint at the alpha=1 edge resolves
/// to the qcms reference without any zero-coverage shortcut intercepting
/// the conversion before it reaches the helper.
///
/// The composite path has multiple alpha-aware shortcuts (zero-alpha
/// skip, fully-opaque skip, etc.). A regression that bypassed the
/// colour stage on the opaque edge would silently produce the
/// uncomposited additive-clamp value.
#[test]
fn output_intent_renders_at_alpha_one_edge() {
    let icc = build_minimal_cmyk_to_rgb_lut8_profile(135);
    // Default content stream has no explicit alpha — that's alpha=1.
    let pdf = build_pdf_cmyk_with_output_intent(&icc);
    let doc = PdfDocument::from_bytes(pdf).expect("open");
    let rgba = render_rgba(&doc);
    let (r, g, b, a) = pixel_at(&rgba, 50, 50);
    assert_eq!(a, 255, "alpha=1 paint must produce fully-opaque pixel");
    assert_eq!(
        (r, g, b),
        (126u8, 126, 126),
        "alpha=1 paint must still route through OutputIntent ICC; got ({r},{g},{b})"
    );
}

/// Pin that a subsequent opaque RGB over-paint obscures the prior CMYK
/// ICC paint cleanly — the OutputIntent path doesn't leak ICC-converted
/// pixels into a later non-CMYK paint scope.
#[test]
fn output_intent_does_not_leak_into_subsequent_rgb_overpaint() {
    let icc = build_minimal_cmyk_to_rgb_lut8_profile(135);
    let catalog_entries =
        "/OutputIntents [<< /Type /OutputIntent /S /GTS_PDFX /OutputCondition (S) /DestOutputProfile 5 0 R >>]";
    // CMYK paint, then white RGB paint covering the same rect.
    let content = "0.25 0 0 0 k\n20 20 60 60 re\nf\n1 1 1 rg\n20 20 60 60 re\nf\n";
    let pdf = build_pdf_with_catalog_entries_and_content(catalog_entries, content, Some(&icc));
    let doc = PdfDocument::from_bytes(pdf).expect("open");
    let rgba = render_rgba(&doc);
    let (r, g, b, a) = pixel_at(&rgba, 50, 50);
    assert_eq!(
        (r, g, b, a),
        (255u8, 255, 255, 255),
        "white RGB over-paint must obscure the CMYK paint regardless of OutputIntent; \
         got ({r},{g},{b},{a})"
    );
}

/// Pin that DeviceCMYK painted inside a Form XObject inherits the
/// document-level OutputIntent. Form XObjects share the document's
/// colour-policy state by spec (§14.8.3) — a regression that built a
/// fresh resolution context for the XObject scope without re-threading
/// the OutputIntent borrow would lose the ICC routing on every spot
/// CMYK paint nested inside the XObject.
///
/// Currently `#[ignore]`-ed pending a Form-XObject test-fixture helper;
/// the marker captures the gap so a follow-up audit picks it up.
#[test]
#[ignore = "OUTPUT_INTENT_DEFER_PHASE_9_DEFAULT_CMYK"]
fn output_intent_inherited_by_form_xobject_paint() {
    panic!("placeholder: needs a Form XObject test-fixture helper");
}

/// Pin the page-level `/DefaultCMYK` override precedence. With the field
/// threaded onto `ResolutionContext` but no consumer yet, this probe is
/// deferred. The marker exists so the phase 9 commit knows where to
/// turn the probe on.
#[test]
#[ignore = "OUTPUT_INTENT_DEFER_PHASE_9_DEFAULT_CMYK"]
fn page_level_default_cmyk_takes_precedence_over_output_intent() {
    panic!("placeholder: not yet implemented — phase 9 consumer pending");
}

/// Document the per-paint qcms-transform construction cost so the phase 7
/// caching PR can show a measurable win. This probe is `#[ignore]`-ed in
/// the default suite; running it with `--ignored` produces a baseline
/// duration that phase 7 can compare against.
///
/// The probe paints 1000 same-colour `k`+`re`+`f` operators on a single
/// page. Without caching the renderer builds 1000 qcms transforms;
/// caching should reduce that to one.
#[test]
#[ignore = "OUTPUT_INTENT_DEFER_PHASE_7_CACHING"]
fn output_intent_thousand_cmyk_paints_baseline_cost() {
    let icc = build_minimal_cmyk_to_rgb_lut8_profile(135);
    let mut ops = String::new();
    for i in 0..1000 {
        let y = i % 100;
        ops.push_str(&format!("0.25 0 0 0 k\n0 {y} 1 1 re\nf\n"));
    }
    let catalog_entries =
        "/OutputIntents [<< /Type /OutputIntent /S /GTS_PDFX /OutputCondition (S) /DestOutputProfile 5 0 R >>]";
    let pdf = build_pdf_with_catalog_entries_and_content(catalog_entries, &ops, Some(&icc));
    let doc = PdfDocument::from_bytes(pdf).expect("open");
    let start = std::time::Instant::now();
    let _ = render_rgba(&doc);
    let elapsed = start.elapsed();
    eprintln!(
        "OUTPUT_INTENT_PHASE_7_BASELINE: 1000 same-colour DeviceCMYK paints took {:?} \
         (each rebuilds the qcms transform; phase 7 caches)",
        elapsed
    );
    // No assertion — baseline-measurement probe.
}

// ===========================================================================
// QA: TDD-discipline verification report (inline docstring)
// ===========================================================================

/// TDD-discipline verification report for round-1 OutputIntent foundation.
///
/// Verified by checking out the round-1 commit graph in a throwaway
/// worktree and re-running the failing/passing tests at the relevant
/// SHAs. Captured here so a future reader has the audit trail without
/// having to re-do the bisect.
///
/// **Failing test commit `eab4040`:**
/// Planting `tests/test_render_output_intent.rs` from `eab4040` onto
/// its parent `65063ba` (last `feat` commit before the impl landed)
/// produced:
///
/// ```text
/// thread 'device_cmyk_paint_with_output_intent_renders_via_icc_not_additive_clamp'
///   panicked at tests/test_render_output_intent.rs:365:5:
/// OutputIntent /DeviceCMYK paint expected qcms-converted RGB ~(128, 128, 128);
/// got (191, 255, 255). RGB(191, 255, 255) would mean the §10.3.5 additive-clamp
/// fallback fired — the resolver is not consulting ctx.output_intent_cmyk.
/// test result: FAILED. 0 passed; 1 failed
/// ```
///
/// Checking out the impl commit `656c119` then produced:
///
/// ```text
/// test device_cmyk_paint_with_output_intent_renders_via_icc_not_additive_clamp ... ok
/// test result: ok. 1 passed; 0 failed
/// ```
///
/// **Negative-pin commit `fda9b6f`:**
/// The negative pin (`*_without_output_intent_renders_additive_clamp`)
/// is a regression guard, not a failing test. Verified by planting the
/// commit's test on its parent `656c119`: it passed even there because
/// the no-OutputIntent fallback was the shipped behaviour. The impl
/// agent's report categorised this honestly as a "negative pin", and
/// the actual test categorisation matches.
///
/// **Conclusion:** TDD discipline was followed for the positive ICC
/// path. The negative pin is correctly described as a regression guard.
#[test]
fn qa_tdd_discipline_verification_report() {
    // Marker test — its docstring carries the verification narrative;
    // the body just confirms the integration suite is still compilable
    // by referencing the two test functions whose behaviour the report
    // describes.
    let _ = device_cmyk_paint_with_output_intent_renders_via_icc_not_additive_clamp;
    let _ = device_cmyk_paint_without_output_intent_renders_additive_clamp;
}

// ===========================================================================
// Phase 4: embedded /ICCBased N=4 trumps document /OutputIntents
// ===========================================================================
//
// ISO 32000-1:2008 §8.6.5.5 (and §14.11.5): an `/ICCBased` colour space
// carries its own `DestOutputProfile`-equivalent stream; that stream IS
// the conversion source, and the document-level `/OutputIntents` profile
// is only the default for `/DeviceCMYK` paint that lacks any embedded
// override. Embedded ICC always wins.
//
// The byte-exact references baked into the assertions below come from
// the discovery harness (run once, output captured) — see the plan
// errata. They are intent-invariant because the synthesised LUT8
// profile uses a constant CLUT.

/// Byte-exact qcms 0.3.0 reference for the `target_l_byte=200` profile
/// at CMYK(64,0,0,0) under RelativeColorimetric (intent-invariant by
/// construction). Distinct from the round-1 profile A reference of
/// (126,126,126) so the precedence assertion is unambiguous.
const PROFILE_B_TARGET_L_BYTE: u8 = 200;
const PROFILE_B_RGB_AT_FIXTURE_INPUT: (u8, u8, u8) = (194, 194, 194);

/// Pin that an `/ICCBased` N=4 colour space paint operator routes through
/// the colour-space-embedded profile B and NOT through the document-level
/// `/OutputIntents` profile A.
///
/// Fixture geometry:
///   - Catalog declares /OutputIntents → profile A (target_l_byte=135 →
///     qcms reference RGB(126,126,126)).
///   - Page Resources /ColorSpace /CS1 → [/ICCBased <stream B>] where
///     profile B has target_l_byte=200 → qcms reference RGB(194,194,194).
///   - Content stream: `/CS1 cs   0.25 0 0 0 scn   20 20 60 60 re   f`.
///
/// Spec rule: §8.6.5.5 — the ICCBased colour space carries the conversion
/// source and overrides any document-level default. The renderer must
/// route the four `scn` components through profile B's qcms transform.
///
/// What this test catches:
///   - If the rendered pixel is (126,126,126), profile A won — the
///     embedded ICC route is being shadowed by the OutputIntent route
///     (the spec-precedence bug this phase exists to fix).
///   - If the rendered pixel is (191,255,255), neither profile was
///     consulted and §10.3.5 additive-clamp fired (an even worse
///     regression).
///   - If the rendered pixel is (194,194,194), profile B's CMM
///     compiled-and-ran through `Transform::convert_cmyk_pixel` and the
///     precedence is correct.
#[test]
fn embedded_iccbased_n4_trumps_document_output_intent() {
    let profile_a = build_minimal_cmyk_to_rgb_lut8_profile(135);
    let profile_b = build_minimal_cmyk_to_rgb_lut8_profile(PROFILE_B_TARGET_L_BYTE);

    // Sanity-pin both profiles compile through qcms and produce the
    // expected byte-exact references. Without this gate a regression
    // that broke profile B's transform would make the integration
    // assertion below fire for the wrong reason.
    {
        use pdf_oxide::color::{IccProfile, RenderingIntent, Transform};
        use std::sync::Arc;
        let prof_a = Arc::new(IccProfile::parse(profile_a.clone(), 4).expect("parse A"));
        let prof_b = Arc::new(IccProfile::parse(profile_b.clone(), 4).expect("parse B"));
        let t_a = Transform::new_srgb_target(prof_a, RenderingIntent::RelativeColorimetric);
        let t_b = Transform::new_srgb_target(prof_b, RenderingIntent::RelativeColorimetric);
        assert_eq!(
            t_a.convert_cmyk_pixel(64, 0, 0, 0),
            [126u8, 126, 126],
            "profile A reference must be (126,126,126); fixture is invalid otherwise"
        );
        assert_eq!(
            t_b.convert_cmyk_pixel(64, 0, 0, 0),
            [194u8, 194, 194],
            "profile B reference must be (194,194,194); fixture is invalid otherwise"
        );
    }

    let pdf = build_pdf_embedded_iccbased_with_different_output_intent(&profile_a, &profile_b);
    let doc = PdfDocument::from_bytes(pdf).expect("open synthetic PDF");
    // Cross-check the OutputIntent accessor sees profile A. If it didn't
    // the test would conflate "OI not seen" with "OI seen but bypassed
    // for embedded ICC" — both produce the expected pixel but only the
    // latter actually probes the precedence we care about.
    assert!(
        doc.output_intent_cmyk_profile().is_some(),
        "fixture must declare a CMYK OutputIntent so the precedence is actually contested"
    );

    let rgba = render_rgba(&doc);
    let (r, g, b, a) = pixel_at(&rgba, 50, 50);
    let (br, bg, bb) = PROFILE_B_RGB_AT_FIXTURE_INPUT;
    assert_eq!(
        (r, g, b, a),
        (br, bg, bb, 255),
        "embedded /ICCBased profile B must take precedence over /OutputIntents \
         profile A on CMYK paint through the ICCBased space; expected B's qcms \
         reference {:?}; got ({r},{g},{b},{a}). (126,126,126,_) means profile A won \
         — the spec precedence (§8.6.5.5) is inverted. (191,255,255,_) means neither \
         profile was consulted and §10.3.5 fired.",
        (br, bg, bb, 255u8)
    );
}

// ===========================================================================
// Phase 5: Separation / DeviceN with DeviceCMYK alternate routes through OutputIntent
// ===========================================================================
//
// ISO 32000-1:2008 §8.6.6.3 (Separation) and §8.6.6.4 (DeviceN): when the
// device lacks the named colorant plate, the colour is approximated via
// the alternate colour space and the tint transform. When the alternate
// is /DeviceCMYK, the alternate's CMYK quadruple is then converted to
// RGB for the composite output path — and that conversion MUST honour
// the document /OutputIntents profile, since composite output is the
// "viewer's screen" surface the OutputIntent describes.
//
// Today (post-round-1) the resolver's
// `resolve_separation_or_devicen` arm dispatches a CMYK-alternate
// result through `four_as_cmyk(&altspace_values, alpha, ctx)`, which
// itself calls `cmyk_to_rgb_via_intent` — the same OutputIntent-aware
// helper the bare /DeviceCMYK paint path consumes. So the routing is
// already correct, but the probes below pin it byte-for-byte so a
// regression that detoured Separation/DeviceN through a non-context-
// aware CMYK→RGB path would surface immediately.
//
// These probes are categorised as REGRESSION GUARDS in the TDD-discipline
// sense (they pass at HEAD without code changes) because the routing
// landed during round-1 phase 2. The TDD-failing-test→implementation
// pair for this behaviour is documented at fa1b947's prior history
// (round-1 phase 2). The probes here lock the routing for the
// specifically named Separation Type-4 and DeviceN Type-4 cases the
// plan body called out.
//
// Discrimination audit: before committing the probes, the impl agent
// temporarily flipped `four_as_cmyk` in src/rendering/resolution/color.rs
// to bypass `cmyk_to_rgb_via_intent` and call bare `cmyk_to_rgb` (the
// §10.3.5 helper) instead. With that flip, both
// `*_composite_routes_through_output_intent` probes failed with the
// expected (255, 0, 255, 255) value, demonstrating they actively
// discriminate between "OutputIntent honoured" and "additive-clamp
// fallback". The flip was reverted before the commit landed; the audit
// confirms the probes do what their names say.

/// Pin that a `/Separation /MagentaSpot /DeviceCMYK <Type-4 tint
/// transform>` paint operator's composite-side RGBA is the document
/// `/OutputIntents` profile's conversion of the tint-transform's CMYK
/// output — NOT the §10.3.5 additive-clamp of that CMYK quadruple.
///
/// Fixture: tint transform `{ 0.0 exch 0.0 0.0 }` produces CMYK(0, tint,
/// 0, 0). At tint=1.0 the alternate-CMYK value is (0, 1, 0, 0); §10.3.5
/// of that is RGB(255, 0, 255) (magenta). The OutputIntent profile
/// (constant-CLUT, target_l_byte=135) maps every CMYK input to
/// RGB(126, 126, 126), so an OutputIntent-honouring composite pixel is
/// (126, 126, 126).
///
/// Three observable outcomes:
///   - (126, 126, 126, 255): composite routed through OutputIntent — pass.
///   - (255, 0, 255, 255): composite ran §10.3.5 directly — fail
///     (alt-CMYK projection bypassed `cmyk_to_rgb_via_intent`).
///   - any other RGB: tint transform or qcms behaviour drifted.
#[test]
fn separation_type4_alt_devicecmyk_composite_routes_through_output_intent() {
    let icc = build_minimal_cmyk_to_rgb_lut8_profile(135);
    // Sanity-pin the OutputIntent reference for CMYK(0, 255, 0, 0) —
    // intent-invariant by construction (constant CLUT) so a single
    // intent is enough.
    {
        use pdf_oxide::color::{IccProfile, RenderingIntent, Transform};
        use std::sync::Arc;
        let prof = Arc::new(IccProfile::parse(icc.clone(), 4).expect("parse"));
        let t = Transform::new_srgb_target(prof, RenderingIntent::RelativeColorimetric);
        let rgb = t.convert_cmyk_pixel(0, 255, 0, 0);
        assert_eq!(
            rgb,
            [126u8, 126, 126],
            "OutputIntent profile must map CMYK(0,255,0,0) to (126,126,126); \
             fixture is invalid otherwise (got {rgb:?})"
        );
    }

    let pdf = build_pdf_separation_type4_devicecmyk_with_output_intent(&icc);
    let doc = PdfDocument::from_bytes(pdf).expect("open synthetic PDF");
    assert!(
        doc.output_intent_cmyk_profile().is_some(),
        "fixture must declare a CMYK OutputIntent for the routing to be probed"
    );

    let rgba = render_rgba(&doc);
    let (r, g, b, a) = pixel_at(&rgba, 50, 50);
    assert_eq!(
        (r, g, b, a),
        (126u8, 126, 126, 255),
        "Separation Type-4 /DeviceCMYK alternate must route the alt-CMYK \
         quadruple through the document /OutputIntents profile on the \
         composite path; expected (126,126,126,255); got ({r},{g},{b},{a}). \
         (255,0,255,_) means the §10.3.5 additive-clamp of CMYK(0,1,0,0) \
         fired — the resolver bypassed cmyk_to_rgb_via_intent for the \
         Separation alt-CMYK projection."
    );
}

/// Counter-pin: with no `/OutputIntents` declared, the same Separation
/// Type-4 alt-CMYK paint MUST produce the §10.3.5 additive-clamp value
/// for CMYK(0, 1, 0, 0) = RGB(255, 0, 255).
///
/// The positive pin above asserts "OutputIntent wins on composite when
/// present"; this counter-pin asserts "no-OutputIntent → §10.3.5
/// preserved byte-for-byte" — i.e. the OutputIntent route doesn't leak
/// into a no-OutputIntent fixture (which would imply some hard-coded
/// CMM hung around the renderer rather than the configured route).
#[test]
fn separation_type4_alt_devicecmyk_without_output_intent_renders_additive_clamp() {
    // Inline-build a PDF identical to
    // `build_pdf_separation_type4_devicecmyk_with_output_intent` but
    // without /OutputIntents. Object IDs shift down by one because the
    // ICC stream is dropped: catalog → pages → page → content → tint
    // (obj 5 instead of obj 6).
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    let page_off = buf.len();
    let page = "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << /ColorSpace << /CS1 [/Separation /MagentaSpot /DeviceCMYK 5 0 R] >> >> /Contents 4 0 R >>\nendobj\n";
    buf.extend_from_slice(page.as_bytes());

    let stream_off = buf.len();
    let content = "/CS1 cs\n1.0 scn\n20 20 60 60 re\nf\n";
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let tint_off = buf.len();
    let tint_program: &[u8] = b"{ 0.0 exch 0.0 0.0 }";
    let tint_hdr = format!(
        "5 0 obj\n<< /FunctionType 4 /Domain [0 1] /Range [0 1 0 1 0 1 0 1] /Length {} >>\nstream\n",
        tint_program.len()
    );
    buf.extend_from_slice(tint_hdr.as_bytes());
    buf.extend_from_slice(tint_program);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xref_off = buf.len();
    let obj_count = 6;
    buf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", obj_count).as_bytes());
    for off in [cat_off, pages_off, page_off, stream_off, tint_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            obj_count, xref_off
        )
        .as_bytes(),
    );

    let doc = PdfDocument::from_bytes(buf).expect("open synthetic PDF");
    assert!(
        doc.output_intent_cmyk_profile().is_none(),
        "fixture must declare no /OutputIntents for the counter-pin to actually contest the route"
    );

    let rgba = render_rgba(&doc);
    let (r, g, b, a) = pixel_at(&rgba, 50, 50);
    assert_eq!(
        (r, g, b, a),
        (255u8, 0, 255, 255),
        "Separation Type-4 /DeviceCMYK alternate without /OutputIntents must \
         fall through to §10.3.5 additive-clamp of CMYK(0,1,0,0) = (255,0,255); \
         got ({r},{g},{b},{a})"
    );
}

/// Pin that a 2-colorant `/DeviceN [/Magenta /Cyan] /DeviceCMYK
/// <Type-4 tint transform>` paint operator's composite-side RGBA is
/// also routed through the document `/OutputIntents` profile when the
/// tint transform's alternate-CMYK output lands in the resolver.
///
/// Fixture: tint transform `{ pop 0.0 exch 0.0 0.0 }` consumes the two
/// colorant tints, drops the second, and emits CMYK(0, tint0, 0, 0).
/// Content `1.0 0.5 scn` provides (tint0=1.0, tint1=0.5) → alternate
/// CMYK(0, 1, 0, 0). The OutputIntent profile maps that to
/// RGB(126, 126, 126); the §10.3.5 additive-clamp value would be
/// RGB(255, 0, 255).
#[test]
fn devicen_type4_alt_devicecmyk_composite_routes_through_output_intent() {
    let icc = build_minimal_cmyk_to_rgb_lut8_profile(135);
    let pdf = build_pdf_devicen_type4_devicecmyk_with_output_intent(&icc);
    let doc = PdfDocument::from_bytes(pdf).expect("open synthetic PDF");
    assert!(
        doc.output_intent_cmyk_profile().is_some(),
        "fixture must declare a CMYK OutputIntent for the routing to be probed"
    );

    let rgba = render_rgba(&doc);
    let (r, g, b, a) = pixel_at(&rgba, 50, 50);
    assert_eq!(
        (r, g, b, a),
        (126u8, 126, 126, 255),
        "DeviceN Type-4 /DeviceCMYK alternate must route the alt-CMYK \
         quadruple through the document /OutputIntents profile on the \
         composite path; expected (126,126,126,255); got ({r},{g},{b},{a}). \
         (255,0,255,_) means §10.3.5 additive-clamp fired."
    );
}

// ===========================================================================
// QA round-2 edge probes
// ===========================================================================
//
// Round-2 phase 4 changed `resolve_iccbased` for N=4 with parseable
// embedded profile to emit `ResolvedColor::Rgba` directly (bypassing
// OutputIntent). Edge cases the impl probes did not cover:
//
//   1. Embedded profile parses but qcms refuses to build a CMM
//      (`has_cmm() == false`) — fallback path must kick in.
//   2. Embedded profile is malformed bytes (`IccProfile::parse` returns
//      None) — fallback path must kick in.
//   3. ICCBased N=3 (RGB) with a document CMYK /OutputIntents — no
//      interaction; RGB paint stays untouched.
//   4. ICCBased N=1 (gray) with a document CMYK /OutputIntents — same.
//   5. ICCBased N=4 paint inside a Form XObject — precedence survives
//      the Form scope.
//   6. **Per-plate regression**: the fix changes ICCBased N=4 from
//      `ResolvedColor::Cmyk` to `ResolvedColor::Rgba`; per-plate
//      consumers route by participating channels and `Rgba` produces an
//      empty participating list. Probe what happens when the renderer
//      is invoked for separations on the same fixture.

/// Build a minimal "valid header but no usable tags" ICC profile: passes
/// `IccProfile::parse`'s header / `acsp` / `/N` cross-check but qcms's
/// `Profile::new_from_slice` rejects it (no `A2B0`, no matrix/curve
/// tags), so `Transform::has_cmm()` returns false. Used to verify the
/// fallback path in `resolve_iccbased` kicks in cleanly.
fn build_iccbased_header_only_cmyk_profile() -> Vec<u8> {
    let mut profile = vec![0u8; 128];
    // Profile size at bytes 0..4. Header-only + 4-byte tag count of 0.
    let total: u32 = 128 + 4;
    profile[0..4].copy_from_slice(&total.to_be_bytes());
    profile[8..12].copy_from_slice(&0x0240_0000u32.to_be_bytes());
    profile[12..16].copy_from_slice(b"prtr");
    profile[16..20].copy_from_slice(b"CMYK");
    profile[20..24].copy_from_slice(b"Lab ");
    profile[36..40].copy_from_slice(b"acsp");
    profile[64..68].copy_from_slice(&0u32.to_be_bytes());
    profile[68..72].copy_from_slice(&0x0000_F6D6u32.to_be_bytes());
    profile[72..76].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    profile[76..80].copy_from_slice(&0x0000_D32Du32.to_be_bytes());
    // Tag count = 0 — no tags at all.
    profile.extend_from_slice(&0u32.to_be_bytes());
    profile
}

/// Sanity-pin: the header-only profile parses through `IccProfile::parse`
/// but produces a transform with no CMM. This is the precondition that
/// makes the `resolve_iccbased` fallback path observable: if either
/// branch flipped (parse failed OR has_cmm became true) the edge probes
/// below would conflate two failure modes.
#[test]
fn qa_round2_header_only_cmyk_profile_parses_without_cmm() {
    use pdf_oxide::color::{IccProfile, RenderingIntent, Transform};
    use std::sync::Arc;
    let bytes = build_iccbased_header_only_cmyk_profile();
    let prof = IccProfile::parse(bytes, 4).expect(
        "header-only profile should pass IccProfile::parse — only IccHeader::parse and /N \
         cross-check run there",
    );
    let prof = Arc::new(prof);
    let t = Transform::new_srgb_target(prof, RenderingIntent::RelativeColorimetric);
    assert!(
        !t.has_cmm(),
        "header-only profile must NOT compile to a usable qcms CMM; otherwise the fallback \
         path in resolve_iccbased can't be probed"
    );
}

/// Embedded /ICCBased N=4 whose profile parses through
/// `IccProfile::parse` but is rejected by qcms (`has_cmm() == false`).
/// `resolve_iccbased` must fall through to the device-family hint, which
/// emits `ResolvedColor::Cmyk` for N=4, which the composite projection
/// then runs through `cmyk_to_rgb_via_intent` against the document
/// /OutputIntents profile. Expected pixel: (126, 126, 126, 255) — the
/// OutputIntent profile A's constant CLUT.
#[test]
fn qa_round2_iccbased_n4_no_cmm_falls_through_to_output_intent() {
    let profile_a = build_minimal_cmyk_to_rgb_lut8_profile(135);
    let profile_b_no_cmm = build_iccbased_header_only_cmyk_profile();
    let pdf =
        build_pdf_embedded_iccbased_with_different_output_intent(&profile_a, &profile_b_no_cmm);
    let doc = PdfDocument::from_bytes(pdf).expect("open synthetic PDF");
    let rgba = render_rgba(&doc);
    let (r, g, b, a) = pixel_at(&rgba, 50, 50);
    assert_eq!(
        (r, g, b, a),
        (126u8, 126, 126, 255),
        "embedded ICCBased N=4 whose profile parses but has no CMM must fall through to \
         the device-family path → ResolvedColor::Cmyk → cmyk_to_rgb_via_intent → \
         document /OutputIntents profile A reference (126,126,126,255); got \
         ({r},{g},{b},{a}). (191,255,255,_) means §10.3.5 additive-clamp fired (fallback \
         path bypassed OutputIntent). (194,194,194,_) means the embedded profile's CMM \
         compiled (precondition pin was wrong)."
    );
}

/// Embedded /ICCBased N=4 with garbage bytes (no valid `acsp` header).
/// `IccProfile::parse` returns None, so the fallback path emits
/// `ResolvedColor::Cmyk` → routed through `cmyk_to_rgb_via_intent` →
/// document /OutputIntents.
#[test]
fn qa_round2_iccbased_n4_unparseable_bytes_fall_through_to_output_intent() {
    let profile_a = build_minimal_cmyk_to_rgb_lut8_profile(135);
    // 128 zero bytes — no `acsp` signature at bytes 36..40 → parse fails.
    let garbage = vec![0u8; 128];
    let pdf = build_pdf_embedded_iccbased_with_different_output_intent(&profile_a, &garbage);
    let doc = PdfDocument::from_bytes(pdf).expect("open synthetic PDF");
    let rgba = render_rgba(&doc);
    let (r, g, b, a) = pixel_at(&rgba, 50, 50);
    assert_eq!(
        (r, g, b, a),
        (126u8, 126, 126, 255),
        "embedded ICCBased N=4 with unparseable bytes must fall through to the \
         device-family path → ResolvedColor::Cmyk → document /OutputIntents \
         (126,126,126,255); got ({r},{g},{b},{a})."
    );
}

/// **Per-plate regression probe.** The phase-4 fix changes
/// `resolve_iccbased` for N=4 with parseable embedded profile to emit
/// `ResolvedColor::Rgba`. The per-plate `OverprintResolver` produces an
/// empty `participating` list for `Rgba`, and the `InkRouter` returns
/// `InkAction::Skip` for every plate when `participating` is empty. So
/// rendering the embedded-ICC fixture to separations produces NO ink
/// coverage on any plate — even though the fixture's `0.25 0 0 0 scn`
/// paint is logically 25% cyan.
///
/// This pin captures the regression vector the impl agent flagged in
/// the round-2 report. The outcome it pins (all plates zero at the
/// painted-rect centre) IS the current behaviour after phase 4; the pin
/// is here so a future engineer fixing the per-plate path doesn't
/// silently flip it without surfacing the design trade-off.
///
/// The trade-off: §8.6.5.5 says the embedded ICCBased profile is the
/// conversion source. For composite output that means "use it for
/// CMYK→RGB". For separations the question is "should we still emit
/// per-plate ink coverage values, or should we treat the ICC-converted
/// RGB as authoritative and skip the plate decomposition?". The current
/// answer is the second; this probe pins it so the design choice is
/// visible and overridable.
#[test]
#[ignore = "QA_ROUND2_OPEN_QUESTION_PER_PLATE_ROUTING_OF_ICCBASED_N4: phase-4 fix \
            emits ResolvedColor::Rgba for ICCBased N=4 with parseable embedded \
            profile; per-plate path consumes that as 'no ink coverage on any \
            plate'. Design intent: §8.6.5.5 trumps per-plate channel \
            decomposition. Pin here so a future engineer sees the design call \
            instead of debugging silent zero-output plates."]
fn qa_round2_iccbased_n4_with_embedded_profile_emits_no_separation_coverage() {
    use pdf_oxide::rendering::render_separations;
    let profile_a = build_minimal_cmyk_to_rgb_lut8_profile(135);
    let profile_b = build_minimal_cmyk_to_rgb_lut8_profile(200);
    let pdf = build_pdf_embedded_iccbased_with_different_output_intent(&profile_a, &profile_b);
    let doc = PdfDocument::from_bytes(pdf).expect("open synthetic PDF");
    let plates = render_separations(&doc, 0, 72).expect("render_separations");
    // Process plates always emit per the API contract — we just check
    // ink coverage at the painted rect centre is zero on EVERY plate.
    let sample = |p: &pdf_oxide::rendering::SeparationPlate| {
        let w = p.width as usize;
        p.data[50 * w + 50]
    };
    for p in &plates {
        assert_eq!(
            sample(p),
            0,
            "plate {} should carry ZERO ink coverage at the painted-rect centre because \
             ICCBased N=4 with parseable embedded profile now produces ResolvedColor::Rgba \
             on composite, which the per-plate path consumes as 'no participating channels' \
             → InkAction::Skip on every plate. If this fails the per-plate path was \
             updated to honour ICCBased N=4 channel decomposition — update the design \
             documentation accordingly.",
            p.ink_name
        );
    }
}

/// Counter-pin: bare /DeviceCMYK paint with no embedded ICC override
/// continues to produce per-plate coverage as before. This guards
/// against a regression where the round-2 fix accidentally widened to
/// the bare /DeviceCMYK arm too.
#[test]
fn qa_round2_bare_devicecmyk_paint_still_produces_separation_coverage() {
    use pdf_oxide::rendering::render_separations;
    let pdf = build_pdf_cmyk_without_output_intent();
    let doc = PdfDocument::from_bytes(pdf).expect("open synthetic PDF");
    let plates = render_separations(&doc, 0, 72).expect("render_separations");
    let sample = |p: &pdf_oxide::rendering::SeparationPlate| {
        let w = p.width as usize;
        p.data[50 * w + 50]
    };
    let by_name = |name: &str| {
        plates
            .iter()
            .find(|p| p.ink_name == name)
            .map(sample)
            .unwrap_or(0)
    };
    // The renderer's f32→u8 path produces 63 for tint=0.25; the
    // important point is non-zero ink coverage at the painted pixel —
    // proving the bare DeviceCMYK arm still routes through the
    // per-plate `ResolvedColor::Cmyk` decomposition.
    let cyan = by_name("Cyan");
    assert!(
        (60..=68).contains(&cyan),
        "Cyan plate should carry the ~0.25 tint from `0.25 0 0 0 k` (renderer quantises \
         to ~63). Got {cyan}. If zero the bare DeviceCMYK arm regressed too."
    );
    assert_eq!(by_name("Magenta"), 0, "Magenta should be zero");
    assert_eq!(by_name("Yellow"), 0, "Yellow should be zero");
    assert_eq!(by_name("Black"), 0, "Black should be zero");
}

/// ICCBased **N=3** (RGB) with a document CMYK /OutputIntents declared:
/// the OutputIntent applies only to CMYK conversion paths per §8.6.5.5;
/// an RGB ICCBased space neither consults nor cares about the document
/// OutputIntent. Pixel at the painted rect = direct sRGB-like
/// pass-through of the 3 components (fallback path; the device-family
/// hint at /N=3 emits `three_as_rgb`).
#[test]
fn qa_round2_iccbased_n3_with_cmyk_output_intent_ignores_output_intent() {
    // Build a one-page PDF that declares a CMYK OutputIntent and paints
    // a 3-component ICCBased rectangle. We use the existing builder for
    // embedded-ICCBased fixtures but swap the colour-space dict's /N to
    // 3 and use a 3-component `scn`. Easier: reuse
    // `build_pdf_with_catalog_entries_and_content` and inline an
    // ICCBased[3] resource via a custom catalog.
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");
    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R /OutputIntents [<< /Type /OutputIntent /S /GTS_PDFX /OutputCondition (Synthetic CMYK) /DestOutputProfile 5 0 R >>] >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    let page_off = buf.len();
    buf.extend_from_slice(b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << /ColorSpace << /CS1 [/ICCBased 6 0 R] >> >> /Contents 4 0 R >>\nendobj\n");
    let stream_off = buf.len();
    // Paint with RGB(0.5, 0.25, 0.75) via the 3-component ICCBased.
    let content = "/CS1 cs\n0.5 0.25 0.75 scn\n20 20 60 60 re\nf\n";
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let icc_a = build_minimal_cmyk_to_rgb_lut8_profile(135);
    let icc_a_off = buf.len();
    let icc_a_hdr = format!("5 0 obj\n<< /N 4 /Length {} >>\nstream\n", icc_a.len());
    buf.extend_from_slice(icc_a_hdr.as_bytes());
    buf.extend_from_slice(&icc_a);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    // ICCBased N=3 stream — we don't need a valid qcms-compilable profile
    // for the N=3 case because the device-family hint at N=3 in
    // resolve_iccbased emits three_as_rgb directly (the embedded-ICC
    // branch is gated on N=4). Just declare /N 3 with empty stream
    // bytes; parse will fail (no acsp) and the fallback path fires.
    let icc_b_off = buf.len();
    let bogus = vec![0u8; 128];
    let icc_b_hdr = format!("6 0 obj\n<< /N 3 /Length {} >>\nstream\n", bogus.len());
    buf.extend_from_slice(icc_b_hdr.as_bytes());
    buf.extend_from_slice(&bogus);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let xref_off = buf.len();
    let obj_count = 7;
    buf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", obj_count).as_bytes());
    for off in [
        cat_off, pages_off, page_off, stream_off, icc_a_off, icc_b_off,
    ] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            obj_count, xref_off
        )
        .as_bytes(),
    );
    let doc = PdfDocument::from_bytes(buf).expect("open synthetic PDF");
    let rgba = render_rgba(&doc);
    let (r, g, b, a) = pixel_at(&rgba, 50, 50);
    // The renderer's f32→u8 round produces 128 / 64 / 191 for the
    // (0.5, 0.25, 0.75) triple.
    assert_eq!(
        (r, g, b, a),
        (128u8, 64, 191, 255),
        "ICCBased N=3 with document CMYK /OutputIntents declared must pass the three \
         components through unchanged — the OutputIntent applies only to CMYK \
         conversion paths. Got ({r},{g},{b},{a})."
    );
}

/// ICCBased **N=1** (gray) with a document CMYK /OutputIntents declared:
/// same as N=3 — no spec interaction. Fallback path at N=1 emits
/// `first_as_gray`, so a single-component paint of 0.5 produces
/// RGB(128,128,128).
#[test]
fn qa_round2_iccbased_n1_with_cmyk_output_intent_ignores_output_intent() {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");
    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R /OutputIntents [<< /Type /OutputIntent /S /GTS_PDFX /OutputCondition (Synthetic CMYK) /DestOutputProfile 5 0 R >>] >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    let page_off = buf.len();
    buf.extend_from_slice(b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << /ColorSpace << /CS1 [/ICCBased 6 0 R] >> >> /Contents 4 0 R >>\nendobj\n");
    let stream_off = buf.len();
    let content = "/CS1 cs\n0.5 scn\n20 20 60 60 re\nf\n";
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let icc_a = build_minimal_cmyk_to_rgb_lut8_profile(135);
    let icc_a_off = buf.len();
    let icc_a_hdr = format!("5 0 obj\n<< /N 4 /Length {} >>\nstream\n", icc_a.len());
    buf.extend_from_slice(icc_a_hdr.as_bytes());
    buf.extend_from_slice(&icc_a);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let icc_b_off = buf.len();
    let bogus = vec![0u8; 128];
    let icc_b_hdr = format!("6 0 obj\n<< /N 1 /Length {} >>\nstream\n", bogus.len());
    buf.extend_from_slice(icc_b_hdr.as_bytes());
    buf.extend_from_slice(&bogus);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let xref_off = buf.len();
    let obj_count = 7;
    buf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", obj_count).as_bytes());
    for off in [
        cat_off, pages_off, page_off, stream_off, icc_a_off, icc_b_off,
    ] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            obj_count, xref_off
        )
        .as_bytes(),
    );
    let doc = PdfDocument::from_bytes(buf).expect("open synthetic PDF");
    let rgba = render_rgba(&doc);
    let (r, g, b, a) = pixel_at(&rgba, 50, 50);
    assert_eq!(
        (r, g, b, a),
        (128u8, 128, 128, 255),
        "ICCBased N=1 with document CMYK /OutputIntents declared must produce a neutral \
         grey from the single component (0.5 → 128); OutputIntent is not consulted. \
         Got ({r},{g},{b},{a})."
    );
}

/// /ICCBased N=4 paint **inside a Form XObject**: precedence survives
/// the Form scope. The embedded ICCBased CS1 is declared on the page,
/// the Form XObject's content paints `/CS1 cs 0.25 0 0 0 scn ... f`,
/// and the page invokes the Form with `q /Fm1 Do Q`. Expected pixel:
/// profile B's reference (194, 194, 194, 255).
#[test]
fn qa_round2_iccbased_n4_precedence_survives_form_xobject_scope() {
    let profile_a = build_minimal_cmyk_to_rgb_lut8_profile(135);
    let profile_b = build_minimal_cmyk_to_rgb_lut8_profile(PROFILE_B_TARGET_L_BYTE);

    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");
    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R /OutputIntents [<< /Type /OutputIntent /S /GTS_PDFX /OutputCondition (Synthetic CMYK A) /DestOutputProfile 5 0 R >>] >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    // Page declares ICCBased CS1 + form Fm1.
    let page_off = buf.len();
    buf.extend_from_slice(b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << /ColorSpace << /CS1 [/ICCBased 6 0 R] >> /XObject << /Fm1 7 0 R >> >> /Contents 4 0 R >>\nendobj\n");
    // Page content invokes the Form inside a q/Q scope.
    let stream_off = buf.len();
    let content = "q\n/Fm1 Do\nQ\n";
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let icc_a_off = buf.len();
    let icc_a_hdr = format!("5 0 obj\n<< /N 4 /Length {} >>\nstream\n", profile_a.len());
    buf.extend_from_slice(icc_a_hdr.as_bytes());
    buf.extend_from_slice(&profile_a);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let icc_b_off = buf.len();
    let icc_b_hdr = format!("6 0 obj\n<< /N 4 /Length {} >>\nstream\n", profile_b.len());
    buf.extend_from_slice(icc_b_hdr.as_bytes());
    buf.extend_from_slice(&profile_b);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    // Form XObject Fm1: BBox 0..100, identity matrix, inherits page
    // resources via /Resources <<>> + content paints CS1.
    let form_content = "/CS1 cs\n0.25 0 0 0 scn\n20 20 60 60 re\nf\n";
    let form_off = buf.len();
    let form_hdr = format!(
        "7 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] /Resources << /ColorSpace << /CS1 [/ICCBased 6 0 R] >> >> /Length {} >>\nstream\n",
        form_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(form_content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let xref_off = buf.len();
    let obj_count = 8;
    buf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", obj_count).as_bytes());
    for off in [
        cat_off, pages_off, page_off, stream_off, icc_a_off, icc_b_off, form_off,
    ] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            obj_count, xref_off
        )
        .as_bytes(),
    );
    let doc = PdfDocument::from_bytes(buf).expect("open synthetic PDF");
    let rgba = render_rgba(&doc);
    let (r, g, b, a) = pixel_at(&rgba, 50, 50);
    let (br, bg, bb) = PROFILE_B_RGB_AT_FIXTURE_INPUT;
    assert_eq!(
        (r, g, b, a),
        (br, bg, bb, 255),
        "embedded /ICCBased N=4 precedence must survive Form XObject scope — Form paint \
         routed through the page-declared CS1's embedded profile B, not the document \
         /OutputIntents A. Expected ({br},{bg},{bb},255); got ({r},{g},{b},{a}). \
         (126,126,126,_) means Form scope dropped the embedded-ICC routing and the \
         document OutputIntent fired. (191,255,255,_) means neither profile was \
         consulted."
    );
}
