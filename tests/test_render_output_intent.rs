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
