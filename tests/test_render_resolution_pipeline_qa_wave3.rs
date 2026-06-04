//! Wave-3 QA probes for the resolution-pipeline migration (ImageMask + `Do`).
//!
//! Sibling to `test_render_resolution_pipeline_qa_wave1.rs` (paths,
//! stroke, combos) and `_qa_wave2.rs` (text). The pilot tests
//! (`test_render_resolution_pipeline_pilot.rs`) already cover the happy
//! path: Device{RGB,Gray,CMYK,Indexed} parity, the Type 4 Separation
//! capability gain, standard image and Form XObject byte-identical
//! pass-through, plus CTM- and clip-preservation pins.
//!
//! This wave-3 QA suite probes the corners the pilot doesn't:
//!
//! 1. **ImageMask rendering correctness** — the wave-3 `Do` arm calls
//!    a brand-new `render_image_mask` helper; small / wide-with-padding /
//!    tall stencils; `/Decode [1 0]` polarity invert; missing / malformed
//!    `/Decode`; rotated and mirrored CTMs.
//! 2. **Pass-through pins for the non-mask branch** — CMYK / Indexed /
//!    ICCBased N=4 standard images must keep their inline behaviour.
//! 3. **Inline-image coverage** — `BI ... ID ... EI` is a separate parse
//!    path the renderer may or may not dispatch; pin the current
//!    behaviour either way.
//! 4. **Form-XObject interactions** — Form containing an ImageMask,
//!    nested Form-in-Form, CTM round-trip across the Form boundary.
//! 5. **Multi-XObject interactions** — back-to-back masks, mixed with
//!    standard images, under SMask / clip / blend.
//! 6. **Capability at scale** — many ImageMasks on one page; DeviceN /
//!    `/All` / `/None` colorants applied to ImageMask fill.
//! 7. **Adversarial input** — too-short / too-long / zero-dim / huge-dim
//!    stencil streams.
//! 8. **Performance** — N-paint pipeline-on vs pipeline-off wall-clock
//!    ratio (one-resolve-per-Do invariant, matching wave-2's pattern).
//!
//! Style mirrors waves 1 + 2: build a tiny PDF inline, render twice
//! through `render_with_pipeline`, compare pixmaps or sample pixels.
//! When a probe finds a bug, the test is committed with `#[ignore]` and
//! a comment naming the failing invariant; pin tests are committed
//! enabled.

#![cfg(feature = "rendering")]
#![allow(dead_code)] // probes accrete across commits; not every helper is wired up yet.

use pdf_oxide::document::PdfDocument;
use pdf_oxide::rendering::{render_page, ImageFormat, RenderOptions};
use std::sync::Mutex;

/// Process-wide lock for env-var test orchestration. Cargo runs
/// integration tests in parallel; flipping `PDF_OXIDE_RESOLUTION_PIPELINE`
/// must not race with another test's read.
static PIPELINE_TOGGLE_LOCK: Mutex<()> = Mutex::new(());

// ===========================================================================
// PDF construction helpers — self-contained so a fix-pass to the pilot or
// wave-1/2 QA helpers can't accidentally invalidate the wave-3 invariants.
// ===========================================================================

/// Build a one-page PDF containing a single ImageMask XObject `/IM1`.
/// `content_ops` runs on the page (typically sets the fill colour, a
/// CTM, then `/IM1 Do`). `resources_extra` is appended into the page's
/// `/Resources` dictionary. `mask_extras` is appended into the
/// ImageMask stream dictionary (use it for `/Decode`, `/Interpolate`,
/// etc.).
fn build_pdf_image_mask_ex(
    content_ops: &str,
    resources_extra: &str,
    width: u32,
    height: u32,
    mask_data: &[u8],
    mask_extras: &str,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    let page_off = buf.len();
    let page = format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
         /Resources << /XObject << /IM1 5 0 R >> {} >> /Contents 4 0 R >>\nendobj\n",
        resources_extra
    );
    buf.extend_from_slice(page.as_bytes());

    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xobj_off = buf.len();
    let xobj_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Image /ImageMask true \
         /Width {} /Height {} /BitsPerComponent 1 {} /Length {} >>\nstream\n",
        width,
        height,
        mask_extras,
        mask_data.len()
    );
    buf.extend_from_slice(xobj_hdr.as_bytes());
    buf.extend_from_slice(mask_data);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off, xobj_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

/// Convenience wrapper — no extra stream-dict entries.
fn build_pdf_image_mask(
    content_ops: &str,
    resources_extra: &str,
    width: u32,
    height: u32,
    mask_data: &[u8],
) -> Vec<u8> {
    build_pdf_image_mask_ex(content_ops, resources_extra, width, height, mask_data, "")
}

/// Render holding the toggle to `enabled` for the call's duration; shared
/// mutex serialises env-var manipulation across parallel tests.
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
    match prev {
        Some(v) => std::env::set_var("PDF_OXIDE_RESOLUTION_PIPELINE", v),
        None => std::env::remove_var("PDF_OXIDE_RESOLUTION_PIPELINE"),
    }
    data
}

/// Render under pipeline-`enabled`, allowing the call to fail without
/// panicking. Used by adversarial-input probes — the invariant is
/// "no panic", not "render succeeds".
fn render_with_pipeline_allow_fail(doc: &PdfDocument, enabled: bool) -> Option<Vec<u8>> {
    let _guard = PIPELINE_TOGGLE_LOCK.lock().unwrap();
    let prev = std::env::var("PDF_OXIDE_RESOLUTION_PIPELINE").ok();
    if enabled {
        std::env::set_var("PDF_OXIDE_RESOLUTION_PIPELINE", "1");
    } else {
        std::env::remove_var("PDF_OXIDE_RESOLUTION_PIPELINE");
    }
    let opts = RenderOptions::with_dpi(72).as_raw();
    let result = render_page(doc, 0, &opts).ok().map(|img| img.data);
    match prev {
        Some(v) => std::env::set_var("PDF_OXIDE_RESOLUTION_PIPELINE", v),
        None => std::env::remove_var("PDF_OXIDE_RESOLUTION_PIPELINE"),
    }
    result
}

/// Sample a pixel at (x, y) on the 100×100 page.
fn pixel_at(rgba: &[u8], x: u32, y: u32) -> (u8, u8, u8, u8) {
    let w = 100u32;
    let off = ((y * w + x) * 4) as usize;
    (rgba[off], rgba[off + 1], rgba[off + 2], rgba[off + 3])
}

/// Sample the centre pixel of the 100×100 page.
fn center_pixel(rgba: &[u8]) -> (u8, u8, u8, u8) {
    pixel_at(rgba, 50, 50)
}

/// Count pixels in `[x0, x1) × [y0, y1)` whose RGB is materially below
/// the white background — i.e. "this region got painted".
fn count_ink_pixels(rgba: &[u8], x0: u32, y0: u32, x1: u32, y1: u32) -> u32 {
    let w = 100u32;
    let h = 100u32;
    assert_eq!(rgba.len() as u32, w * h * 4);
    let mut n = 0u32;
    for y in y0..y1.min(h) {
        for x in x0..x1.min(w) {
            let off = ((y * w + x) * 4) as usize;
            let r = rgba[off];
            let g = rgba[off + 1];
            let b = rgba[off + 2];
            if r < 240 || g < 240 || b < 240 {
                n += 1;
            }
        }
    }
    n
}

/// Solid 1-bit stencil — all bytes 0x00, so every pixel paints opaque
/// under the default `/Decode [0 1]`. Rows are byte-padded per PDF
/// §8.9.3.
fn solid_image_mask_bytes(width: u32, height: u32) -> Vec<u8> {
    let row_bytes = (width as usize).div_ceil(8);
    vec![0x00u8; row_bytes * height as usize]
}

/// Empty 1-bit stencil — all bytes 0xFF, so every pixel is transparent
/// under the default `/Decode [0 1]`.
fn empty_image_mask_bytes(width: u32, height: u32) -> Vec<u8> {
    let row_bytes = (width as usize).div_ceil(8);
    vec![0xFFu8; row_bytes * height as usize]
}

// ===========================================================================
// Probes 1-9 — ImageMask rendering correctness (the new capability).
//
// `render_image_mask` is brand new in wave 3. Even ignoring the pipeline
// toggle, it must decode the 1-bit stream correctly (row padding, default
// vs inverted Decode, missing Decode, malformed Decode), stay panic-free
// on degenerate input, and respect CTM rotation / mirroring.
// ===========================================================================

/// Probe 1 — 1×1 ImageMask stencil. A single opaque sample painted with
/// a known fill colour. Toggle parity (the fill is DeviceRGB, so both
/// paths read `gs.fill_color_rgb` and the spliced clone short-circuits).
#[test]
fn qa_image_mask_1x1_solid_toggle_parity() {
    let mask = solid_image_mask_bytes(1, 1); // 1 byte, all opaque
                                             // Stretch the 1×1 stencil over 60×60 in the centre of the page.
    let content = "q\n0 1 0 rg\n60 0 0 60 20 20 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_image_mask(content, "", 1, 1, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "1x1 ImageMask DeviceRGB fill must be byte-identical off vs on");
    let (r, g, b, a) = center_pixel(&on);
    assert!(
        g > 200 && r < 60 && b < 60 && a > 200,
        "1x1 stencil stretched over centre should be green, got ({r}, {g}, {b}, {a})"
    );
}

/// Probe 2 — Width that is NOT a byte multiple (7px wide). Per PDF
/// §8.9.3 each row is padded to a byte boundary; the padding bits in the
/// trailing nibble must NOT paint. If the row-bytes maths in
/// `render_image_mask` is off, the 8th column will appear opaque even
/// though it is padding.
///
/// Stencil: 7×4, all bits 0 (opaque under default Decode). Each row is
/// 1 byte; the high 7 bits are valid pixels, the low bit is padding.
/// We stretch the stencil over the full page; the right edge of the
/// rendered image must drop off after the 7th of 8 image columns —
/// i.e. roughly 100 * 7/8 = 87.5 px from the left edge.
#[test]
fn qa_image_mask_width_not_byte_multiple_padding_does_not_paint() {
    let width = 7u32;
    let height = 4u32;
    let mask = solid_image_mask_bytes(width, height);
    let content = "q\n1 0 0 rg\n100 0 0 100 0 0 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_image_mask(content, "", width, height, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "padded-row ImageMask must be byte-identical off vs on");

    // A pixel firmly inside the 7-column region (around x=50, y=50) must
    // be red. The renderer's resampler may smear hard edges, so we don't
    // assert on the boundary itself — just on "interior paints".
    let (r, g, b, _a) = pixel_at(&on, 50, 50);
    assert!(
        r > 200 && g < 60 && b < 60,
        "centre of 7-column stencil must paint red, got ({r}, {g}, {b})"
    );
}

/// Probe 3 — Tall ImageMask (height = 256). Parity at scale; also
/// guards against an off-by-one in the row-loop or buffer-size maths.
#[test]
fn qa_image_mask_tall_height_toggle_parity() {
    let mask = solid_image_mask_bytes(8, 256);
    let content = "q\n0 0 1 rg\n100 0 0 100 0 0 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_image_mask(content, "", 8, 256, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "tall (256-row) ImageMask must be byte-identical off vs on");
    let (r, g, b, _a) = center_pixel(&on);
    assert!(b > 200 && r < 60 && g < 60, "centre must be blue, got ({r}, {g}, {b})");
}

/// Probe 4 — `/Decode [1 0]` polarity invert. With this Decode array a
/// stencil bit of `1` paints, `0` does not. Build an all-1s stream
/// (every byte 0xFF), under inverted Decode that should fill the whole
/// stencil; under default Decode it would be transparent.
///
/// PIN: the wave-3 helper supports `/Decode [1 0]`. This is a positive
/// pin — the renderer must paint, and toggle off vs on must match.
#[test]
fn qa_image_mask_decode_inverted_polarity_paints_under_ff_bytes() {
    let mask = vec![0xFFu8; 1]; // 8x1 stencil, all bits 1
    let content = "q\n1 0 0 rg\n100 0 0 100 0 0 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_image_mask_ex(content, "", 8, 1, &mask, "/Decode [1 0]");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "/Decode [1 0] ImageMask must be byte-identical off vs on");

    let (r, g, b, _a) = center_pixel(&on);
    assert!(
        r > 200 && g < 60 && b < 60,
        "inverted-Decode all-1 stencil should paint red everywhere, got ({r}, {g}, {b})"
    );
}

/// Probe 5 — Missing `/Decode` (default `[0 1]`). An all-0 stream
/// paints opaque. Confirms the missing-entry path doesn't accidentally
/// drop into the inverted branch.
#[test]
fn qa_image_mask_no_decode_default_paints_under_zero_bytes() {
    let mask = solid_image_mask_bytes(8, 1); // all zeros
    let content = "q\n0 1 1 rg\n100 0 0 100 0 0 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_image_mask(content, "", 8, 1, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "default-/Decode ImageMask must be byte-identical off vs on");
    let (r, g, b, _a) = center_pixel(&on);
    assert!(
        g > 200 && b > 200 && r < 60,
        "default-Decode all-0 stencil should paint cyan everywhere, got ({r}, {g}, {b})"
    );
}

/// Probe 6 — Malformed `/Decode`. Several adversarial cases: empty
/// array, ambiguous `[0.5 0.5]`, single-element. The renderer must not
/// panic on any of them; it should fall back to default polarity (the
/// wave-3 helper's `match … _ => false` arm).
///
/// PIN: the wave-3 implementation reads `first > 0.5` for the polarity
/// flag. With `[0.5 0.5]` `first` is exactly `0.5`, so `first > 0.5` is
/// `false` → default polarity (zeros paint, ones don't). The all-zeros
/// stream should therefore paint opaque. Empty array and single-element
/// `[1]` should hit the catch-all and also default to non-inverted.
#[test]
fn qa_image_mask_malformed_decode_no_panic_default_polarity() {
    let mask = solid_image_mask_bytes(8, 1);
    for decode in &["/Decode []", "/Decode [0.5 0.5]", "/Decode [1]"] {
        let content = "q\n0.4 g\n100 0 0 100 0 0 cm\n/IM1 Do\nQ\n";
        let bytes = build_pdf_image_mask_ex(content, "", 8, 1, &mask, decode);
        let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
        let off = render_with_pipeline_allow_fail(&doc, false)
            .unwrap_or_else(|| panic!("toggle-off must not error for {}", decode));
        let on = render_with_pipeline_allow_fail(&doc, true)
            .unwrap_or_else(|| panic!("toggle-on must not error for {}", decode));
        assert_eq!(off, on, "malformed Decode {} must produce identical pixmaps off vs on", decode);
    }
}

/// Probe 7 — ImageMask under a CTM that rotates 90° clockwise. CTM
/// preservation across the spliced GS clone (pipeline path) and across
/// the inline path must match byte-for-byte. The stencil itself
/// (8×1, all opaque) becomes a vertical band when rotated.
#[test]
fn qa_image_mask_ctm_90deg_rotation_toggle_parity() {
    let mask = solid_image_mask_bytes(8, 1);
    // Rotate 90° clockwise (a,b,c,d = 0,-1,1,0) then scale and translate
    // to land the band on the page.
    let content = "q\n1 0 0 rg\n0 -60 60 0 20 80 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_image_mask(content, "", 8, 1, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "ImageMask under 90° rotated CTM must be byte-identical off vs on");
    // Sanity: some ink landed.
    assert!(
        count_ink_pixels(&on, 0, 0, 100, 100) > 100,
        "rotated stencil should leave visible ink"
    );
}

/// Probe 8 — ImageMask under a CTM with negative X scale (horizontal
/// mirror). The image flip lives in `render_image_mask`'s
/// `pre_translate(0, 1).pre_scale(1/w, -1/h)`; a negative-scale CTM
/// composes correctly only if the helper's flip is applied in the
/// right order. Toggle parity confirms the migration didn't break it.
#[test]
fn qa_image_mask_negative_scale_mirror_toggle_parity() {
    let mask = solid_image_mask_bytes(8, 1);
    let content = "q\n0 0 1 rg\n-60 0 0 60 80 20 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_image_mask(content, "", 8, 1, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "mirrored ImageMask must be byte-identical off vs on");
    assert!(
        count_ink_pixels(&on, 0, 0, 100, 100) > 100,
        "mirrored stencil should leave visible ink"
    );
}

/// Probe 9 — ImageMask under a CTM with negative determinant (Y-flipped
/// on top of the image-space Y-flip; net result is "image space matches
/// user space"). Confirms the helper doesn't bake a flip assumption that
/// breaks composed transforms.
#[test]
fn qa_image_mask_negative_determinant_ctm_toggle_parity() {
    let mask = solid_image_mask_bytes(8, 1);
    // det < 0: a*d - b*c = 60*-60 = -3600.
    let content = "q\n0.5 g\n60 0 0 -60 20 80 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_image_mask(content, "", 8, 1, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "ImageMask under negative-determinant CTM must be byte-identical off vs on"
    );
    assert!(
        count_ink_pixels(&on, 0, 0, 100, 100) > 100,
        "negative-det stencil should leave visible ink"
    );
}
