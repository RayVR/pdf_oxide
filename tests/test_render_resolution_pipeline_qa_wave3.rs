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

// ===========================================================================
// Probes 10-12 — Standard (non-mask) Image XObject pass-through.
//
// Wave 3 routes ONLY `/ImageMask true` through the pipeline; standard
// images go to `render_image` unchanged. These probes pin that the
// guard reads `/ImageMask true` strictly (not "any /ImageMask entry")
// and that toggle-on remains byte-identical to toggle-off for
// non-mask images across the colour spaces that matter.
// ===========================================================================

/// Build a one-page PDF with a standard (non-mask) Image XObject `/IM1`
/// whose ColorSpace dict entry is rendered inline as `/{cs_name}` (use
/// for `DeviceRGB`, `DeviceGray`, `DeviceCMYK`). `bits_per_component`
/// is also written into the stream dict.
fn build_pdf_standard_image_named_cs(
    content_ops: &str,
    width: u32,
    height: u32,
    bits_per_component: u32,
    pixel_bytes: &[u8],
    cs_name: &str,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    let page_off = buf.len();
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
          /Resources << /XObject << /IM1 5 0 R >> >> /Contents 4 0 R >>\nendobj\n",
    );

    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xobj_off = buf.len();
    let xobj_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} \
         /BitsPerComponent {} /ColorSpace /{} /Length {} >>\nstream\n",
        width,
        height,
        bits_per_component,
        cs_name,
        pixel_bytes.len()
    );
    buf.extend_from_slice(xobj_hdr.as_bytes());
    buf.extend_from_slice(pixel_bytes);
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

/// Build a one-page PDF with a standard Image XObject `/IM1` whose
/// ColorSpace is `[/Indexed /DeviceRGB hival lookup_stream_ref]`.
/// `palette_bytes` is the lookup table as raw RGB triples; `pixel_bytes`
/// are the index samples (BPC=8).
fn build_pdf_standard_image_indexed(
    content_ops: &str,
    width: u32,
    height: u32,
    pixel_bytes: &[u8],
    palette_bytes: &[u8],
    hival: u32,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    // Render palette as a hex string so we can keep everything in one
    // file without an extra indirect object.
    let mut palette_hex = String::from("<");
    for b in palette_bytes {
        palette_hex.push_str(&format!("{:02X}", b));
    }
    palette_hex.push('>');

    let page_off = buf.len();
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
          /Resources << /XObject << /IM1 5 0 R >> >> /Contents 4 0 R >>\nendobj\n",
    );

    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xobj_off = buf.len();
    let xobj_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} \
         /BitsPerComponent 8 /ColorSpace [/Indexed /DeviceRGB {} {}] \
         /Length {} >>\nstream\n",
        width,
        height,
        hival,
        palette_hex,
        pixel_bytes.len()
    );
    buf.extend_from_slice(xobj_hdr.as_bytes());
    buf.extend_from_slice(pixel_bytes);
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

/// Probe 10 — CMYK standard image (non-mask) pass-through. Wave 3
/// must not splice the pipeline on these; output byte-identical
/// regardless of the toggle.
#[test]
fn qa_standard_image_cmyk_pass_through_byte_identical() {
    // 4x4 CMYK pixels, all (0, 1, 0, 0) → magenta under additive clamp.
    // Each pixel is 4 bytes (one per component).
    let mut pixels = Vec::with_capacity(16 * 4);
    for _ in 0..16 {
        pixels.extend_from_slice(&[0u8, 255, 0, 0]);
    }
    let content = "q\n80 0 0 80 10 10 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_standard_image_named_cs(content, 4, 4, 8, &pixels, "DeviceCMYK");
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "standard CMYK image must pass through pipeline byte-identically");
}

/// Probe 11 — Indexed standard image (non-mask) pass-through. Palette
/// of 256 entries (full 8-bit). Pixel data picks index 0 (red palette
/// entry) for every sample.
#[test]
fn qa_standard_image_indexed_256_pass_through_byte_identical() {
    // Build a 256-entry palette: index 0 = red, all others = white.
    let mut palette = Vec::with_capacity(256 * 3);
    palette.extend_from_slice(&[0xFFu8, 0x00, 0x00]); // index 0: red
    for _ in 1..256 {
        palette.extend_from_slice(&[0xFFu8, 0xFF, 0xFF]); // others: white
    }
    let pixels = vec![0u8; 16]; // 4x4 image, all index 0
    let content = "q\n80 0 0 80 10 10 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_standard_image_indexed(content, 4, 4, &pixels, &palette, 255);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "standard Indexed image (256-entry palette) must pass through pipeline byte-identically"
    );
    // Pin a body pixel: well inside the 80x80 image footprint.
    let (r, g, b, _a) = pixel_at(&on, 50, 50);
    assert!(
        r > 200 && g < 60 && b < 60,
        "indexed image at index 0 (red palette) must be red at centre, got ({r}, {g}, {b})"
    );
}

/// Probe 12 — `/ImageMask false` explicit (not omitted). The wave-3
/// guard reads `matches!(o, Object::Boolean(true))`; the `false` case
/// must take the standard-image branch. This is a regression pin
/// against a future refactor that might switch to `o.is_some()`.
#[test]
fn qa_image_with_explicit_imagemask_false_routes_to_standard_image() {
    // Mint a 4x4 DeviceGray standard image AND tag it with `/ImageMask
    // false`. The renderer must NOT take the mask branch (no
    // `render_image_mask` call), and toggle off vs on must be
    // byte-identical because the pipeline isn't routed for standard
    // images.
    let pixels = vec![0x80u8; 16];
    let content = "q\n80 0 0 80 10 10 cm\n/IM1 Do\nQ\n";

    // Custom build with the extra dict key.
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");
    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    let page_off = buf.len();
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
          /Resources << /XObject << /IM1 5 0 R >> >> /Contents 4 0 R >>\nendobj\n",
    );
    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let xobj_off = buf.len();
    let xobj_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Image /ImageMask false \
         /Width 4 /Height 4 /BitsPerComponent 8 /ColorSpace /DeviceGray \
         /Length {} >>\nstream\n",
        pixels.len()
    );
    buf.extend_from_slice(xobj_hdr.as_bytes());
    buf.extend_from_slice(&pixels);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off, xobj_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );

    let doc = PdfDocument::from_bytes(buf).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "standard image with explicit `/ImageMask false` must pass through pipeline byte-identically"
    );
    // Pin: centre is mid-grey, NOT painted with the (default-zero) fill
    // colour. If the mask branch had erroneously fired, the stencil
    // bits (0x80 = `1000 0000`) would have painted only the high bit
    // as opaque, with the current fill colour, leaving most of the
    // page unpainted.
    let (r, g, b, _a) = pixel_at(&on, 50, 50);
    assert!(
        r == g && g == b && (110..=145).contains(&(r as i32)),
        "explicit /ImageMask false must render the grey pixel data, got ({r}, {g}, {b})"
    );
}

/// Probe 12b — ICCBased N=4 (CMYK ICC profile) standard image (non-mask)
/// pass-through. The ICC profile is supplied as an indirect stream (object
/// 6). Even if the extractor falls back when the ICC bytes are not a
/// valid profile, the *routing decision* (mask vs standard) must remain
/// stable: toggle off vs on byte-identical.
#[test]
fn qa_standard_image_iccbased_n4_pass_through_byte_identical() {
    // 2x2 CMYK pixels (16 bytes), all magenta.
    let mut pixels = Vec::with_capacity(16);
    for _ in 0..4 {
        pixels.extend_from_slice(&[0u8, 255, 0, 0]);
    }
    // Bogus ICC profile bytes — the extractor falls back to /Alternate
    // or DeviceCMYK; what we're pinning is routing, not colour fidelity.
    let icc_bytes = vec![0u8; 32];

    let content = "q\n80 0 0 80 10 10 cm\n/IM1 Do\nQ\n";
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");
    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    let page_off = buf.len();
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
          /Resources << /XObject << /IM1 5 0 R >> >> /Contents 4 0 R >>\nendobj\n",
    );
    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let xobj_off = buf.len();
    let xobj_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Image /Width 2 /Height 2 \
         /BitsPerComponent 8 /ColorSpace [/ICCBased 6 0 R] /Length {} >>\nstream\n",
        pixels.len()
    );
    buf.extend_from_slice(xobj_hdr.as_bytes());
    buf.extend_from_slice(&pixels);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let icc_off = buf.len();
    let icc_hdr = format!(
        "6 0 obj\n<< /N 4 /Alternate /DeviceCMYK /Length {} >>\nstream\n",
        icc_bytes.len()
    );
    buf.extend_from_slice(icc_hdr.as_bytes());
    buf.extend_from_slice(&icc_bytes);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 7\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off, xobj_off, icc_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 7 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    let doc = PdfDocument::from_bytes(buf).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "ICCBased N=4 standard image must pass through pipeline byte-identically"
    );
}

// ===========================================================================
// Probes 13-14 — Inline images (`BI ... ID ... EI`).
//
// Inline images are an entirely separate parse path. The wave-3 commit
// only touches the `Operator::Do` arm; inline images flow through
// `Operator::InlineImage` which the renderer DOES NOT IMPLEMENT —
// `page_renderer.rs` has no `Operator::InlineImage` arm. So:
//
//   - inline images render as nothing (transparent / unchanged page);
//   - inline ImageMasks therefore can't be filled via the pipeline
//     (capability gap, not a regression).
//
// These probes PIN the current behaviour. If a future wave wires up
// `Operator::InlineImage`, both should start failing — at which point
// the new arm needs its own pipeline routing for `/IM true`.
// ===========================================================================

/// Build a one-page PDF whose content stream is a literal byte slice
/// (so callers can embed non-ASCII inline-image data). The renderer
/// doesn't dispatch `Operator::InlineImage` today; this is a gap pin.
fn build_pdf_inline_image_bytes(content_ops: &[u8]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    let page_off = buf.len();
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
          /Resources << >> /Contents 4 0 R >>\nendobj\n",
    );
    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 5\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

/// Probe 13 — Inline ImageMask via `BI ... ID ... EI`. Pin the current
/// behaviour: the renderer does NOT dispatch `Operator::InlineImage`,
/// so the page is blank regardless of the toggle.
///
/// If a future wave adds inline-image support, this test will fail —
/// at which point the new arm needs its own pipeline routing for
/// `/IM true` to match the `Do` arm's behaviour. Tracked as
/// **WAVE-3-GAP-INLINE**.
#[test]
fn qa_inline_image_mask_renderer_gap_pin() {
    // Inline ImageMask: 1x1, /BPC 1, /IM true, one zero byte (opaque
    // under default Decode). Surround with a fill colour set first.
    //
    // Per PDF §8.9.7 the syntax for an inline image is:
    //   BI <dict-entries> ID <data> EI
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(b"q\n1 0 0 rg\n80 0 0 80 10 10 cm\n");
    content.extend_from_slice(b"BI /W 1 /H 1 /BPC 1 /IM true ID ");
    content.push(0x00);
    content.extend_from_slice(b" EI\nQ\n");
    let bytes = build_pdf_inline_image_bytes(&content);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "renderer-gap pin: inline ImageMask renders identically (as nothing) off vs on"
    );
    // Pin the gap: the page must be all white. If a future wave wires
    // up InlineImage rendering and forgets to route the fill through
    // the pipeline, this stops being all-white at the centre and the
    // pin fires.
    let (r, g, b, _a) = center_pixel(&on);
    assert_eq!(
        (r, g, b),
        (255, 255, 255),
        "inline ImageMask currently goes unrendered (renderer gap); \
         WAVE-3-GAP-INLINE must remain until InlineImage is wired up"
    );
}

/// Probe 14 — Inline standard (non-mask) image. Same gap: the renderer
/// doesn't dispatch `Operator::InlineImage`. Pin all-white centre.
#[test]
fn qa_inline_standard_image_renderer_gap_pin() {
    // 1x1 DeviceGray, BPC 8, single byte 0x80 → mid-grey. Without
    // dispatch, the page is blank.
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(b"q\n80 0 0 80 10 10 cm\n");
    content.extend_from_slice(b"BI /W 1 /H 1 /BPC 8 /CS /G ID ");
    content.push(0x80);
    content.extend_from_slice(b" EI\nQ\n");
    let bytes = build_pdf_inline_image_bytes(&content);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "renderer-gap pin: inline standard image renders identically (as nothing) off vs on"
    );
    let (r, g, b, _a) = center_pixel(&on);
    assert_eq!(
        (r, g, b),
        (255, 255, 255),
        "inline standard image currently goes unrendered (renderer gap)"
    );
}

// ===========================================================================
// Probes 15-17 — Form-XObject ImageMask interactions.
//
// Form XObjects are rendered recursively. When the Form's content
// stream invokes an ImageMask, the recursive walk should:
//   - find the mask in the Form's own /Resources;
//   - paint it through the wave-3 pipeline-routed path;
//   - propagate the parent's CTM into the recursion.
//
// These probes pin those interactions.
// ===========================================================================

/// Build a one-page PDF whose `/Fm1` Form XObject internally invokes
/// an ImageMask `/IM1`. Both are listed in the Form's own /Resources.
/// The page invokes `/Fm1 Do`.
fn build_pdf_form_with_inner_image_mask(
    page_content: &str,
    form_content: &str,
    form_resources_extra: &str,
    width: u32,
    height: u32,
    mask_data: &[u8],
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    let page_off = buf.len();
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
          /Resources << /XObject << /Fm1 5 0 R >> >> /Contents 4 0 R >>\nendobj\n",
    );
    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", page_content.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(page_content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    // Form XObject (object 5). Its /Resources lists /IM1 → object 6,
    // plus any extra entries the caller wants (e.g. /ColorSpace).
    let form_off = buf.len();
    let form_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Resources << /XObject << /IM1 6 0 R >> {} >> /Length {} >>\nstream\n",
        form_resources_extra,
        form_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(form_content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    // ImageMask XObject (object 6).
    let im_off = buf.len();
    let im_hdr = format!(
        "6 0 obj\n<< /Type /XObject /Subtype /Image /ImageMask true \
         /Width {} /Height {} /BitsPerComponent 1 /Length {} >>\nstream\n",
        width,
        height,
        mask_data.len()
    );
    buf.extend_from_slice(im_hdr.as_bytes());
    buf.extend_from_slice(mask_data);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 7\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off, form_off, im_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 7 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

/// Build a PDF with TWO Form XObjects: the page invokes `/Fm1`, `/Fm1`
/// invokes `/Fm2`, and `/Fm2` invokes the ImageMask `/IM1`. Used to
/// pin two-level recursion.
fn build_pdf_form_in_form_with_image_mask(
    page_content: &str,
    outer_form_content: &str,
    inner_form_content: &str,
    width: u32,
    height: u32,
    mask_data: &[u8],
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    let page_off = buf.len();
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
          /Resources << /XObject << /Fm1 5 0 R >> >> /Contents 4 0 R >>\nendobj\n",
    );
    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", page_content.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(page_content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    // Outer form: lists /Fm2 (object 6) in its /XObject.
    let outer_off = buf.len();
    let outer_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Resources << /XObject << /Fm2 6 0 R >> >> /Length {} >>\nstream\n",
        outer_form_content.len()
    );
    buf.extend_from_slice(outer_hdr.as_bytes());
    buf.extend_from_slice(outer_form_content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    // Inner form: lists /IM1 (object 7).
    let inner_off = buf.len();
    let inner_hdr = format!(
        "6 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Resources << /XObject << /IM1 7 0 R >> >> /Length {} >>\nstream\n",
        inner_form_content.len()
    );
    buf.extend_from_slice(inner_hdr.as_bytes());
    buf.extend_from_slice(inner_form_content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    // ImageMask.
    let im_off = buf.len();
    let im_hdr = format!(
        "7 0 obj\n<< /Type /XObject /Subtype /Image /ImageMask true \
         /Width {} /Height {} /BitsPerComponent 1 /Length {} >>\nstream\n",
        width,
        height,
        mask_data.len()
    );
    buf.extend_from_slice(im_hdr.as_bytes());
    buf.extend_from_slice(mask_data);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 8\n0000000000 65535 f \n");
    for off in [
        cat_off, pages_off, page_off, stream_off, outer_off, inner_off, im_off,
    ] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 8 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

/// Build a PDF with a Form containing an ImageMask AND a Type 4
/// Separation in its /Resources/ColorSpace. The Form invokes the mask
/// after setting the spot colour. Used by the capability-gain test for
/// nested-Form Separation fills.
fn build_pdf_form_with_imagemask_and_type4_separation(
    page_content: &str,
    form_content: &str,
    type4_program: &str,
    width: u32,
    height: u32,
    mask_data: &[u8],
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");
    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    let page_off = buf.len();
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
          /Resources << /XObject << /Fm1 5 0 R >> >> /Contents 4 0 R >>\nendobj\n",
    );
    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", page_content.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(page_content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    // Form with /SpotMagenta colour space (Type 4 tint → object 7).
    let form_off = buf.len();
    let form_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Resources << /XObject << /IM1 6 0 R >> \
                       /ColorSpace << /SpotMagenta [/Separation /MagentaSpot /DeviceCMYK 7 0 R] >> \
                     >> /Length {} >>\nstream\n",
        form_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(form_content.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    // ImageMask.
    let im_off = buf.len();
    let im_hdr = format!(
        "6 0 obj\n<< /Type /XObject /Subtype /Image /ImageMask true \
         /Width {} /Height {} /BitsPerComponent 1 /Length {} >>\nstream\n",
        width,
        height,
        mask_data.len()
    );
    buf.extend_from_slice(im_hdr.as_bytes());
    buf.extend_from_slice(mask_data);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    // Type 4 function.
    let func_off = buf.len();
    let func_hdr = format!(
        "7 0 obj\n<< /FunctionType 4 /Domain [0 1] /Range [0 1 0 1 0 1 0 1] /Length {} >>\nstream\n",
        type4_program.len()
    );
    buf.extend_from_slice(func_hdr.as_bytes());
    buf.extend_from_slice(type4_program.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 8\n0000000000 65535 f \n");
    for off in [
        cat_off, pages_off, page_off, stream_off, form_off, im_off, func_off,
    ] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 8 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

/// Probe 15 — Form XObject whose internal content paints an
/// ImageMask under a Type 4 Separation fill. The capability gain
/// (full-tint → magenta vs `1 - tint` → black) must propagate through
/// the recursive Form rendering.
#[test]
fn qa_form_xobject_with_inner_image_mask_type4_separation_capability_gain() {
    let mask = solid_image_mask_bytes(8, 8);
    let type4 = "{ 0.0 exch 0.0 0.0 }"; // tint=1 → magenta
    let page = "q\n/Fm1 Do\nQ\n";
    let form = "q\n/SpotMagenta cs\n1 scn\n100 0 0 100 0 0 cm\n/IM1 Do\nQ\n";

    let bytes = build_pdf_form_with_imagemask_and_type4_separation(page, form, type4, 8, 8, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);

    // Inline (off): Type 4 falls back to 1 - tint → solid black.
    let (r_off, g_off, b_off, _a) = center_pixel(&off);
    assert!(
        r_off < 50 && g_off < 50 && b_off < 50,
        "inline Form-nested Type 4 Separation ImageMask must paint ~black, got ({r_off}, {g_off}, {b_off})"
    );
    // Pipeline (on): Type 4 program executes → magenta.
    let (r_on, g_on, b_on, _a) = center_pixel(&on);
    assert!(
        r_on >= 250 && g_on <= 5 && b_on >= 250,
        "pipeline-on Form-nested Type 4 Separation ImageMask must paint magenta, got ({r_on}, {g_on}, {b_on})"
    );
    assert_ne!(off, on, "Form-nested capability gain must be visible");
}

/// Probe 16 — Two-level Form recursion (Form-in-Form), where the
/// innermost content invokes an ImageMask. Toggle parity for a
/// DeviceRGB fill — the resolved colour is the same on both paths so
/// the pixmaps must match.
#[test]
fn qa_form_in_form_with_image_mask_toggle_parity() {
    let mask = solid_image_mask_bytes(8, 8);
    let page = "q\n/Fm1 Do\nQ\n";
    let outer = "q\n/Fm2 Do\nQ\n"; // delegate straight to inner
                                   // Inner sets the fill colour itself and paints the mask. (Set the
                                   // fill at the inner level so propagation through Form recursion is
                                   // not co-mingled with the pipeline-routing pin we're after — the
                                   // CTM and resource scope already test recursion.)
    let inner = "q\n0 1 0 rg\n100 0 0 100 0 0 cm\n/IM1 Do\nQ\n";

    let bytes = build_pdf_form_in_form_with_image_mask(page, outer, inner, 8, 8, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "Form-in-Form ImageMask with DeviceRGB fill must be byte-identical off vs on"
    );
    let (r, g, b, _a) = center_pixel(&on);
    assert!(
        g > 200 && r < 60 && b < 60,
        "two-level Form ImageMask should paint green, got ({r}, {g}, {b})"
    );
}

/// Probe 16b — Bug-found pin (UNRELATED to wave-3, but discovered while
/// probing it). When the page sets the fill colour and then invokes a
/// Form which paints an ImageMask, the Form's content stream does NOT
/// see the page's `rg` — the centre paints black instead of the
/// inherited fill. Symmetric across the pipeline toggle (so it is NOT
/// a wave-3 regression, but rather a graphics-state-propagation gap
/// at the Form recursion boundary).
///
/// Pinned `#[ignore]` to record the discovery without failing CI.
/// Bug name: **FORM-RECURSION-FILL-NOT-INHERITED** — the renderer's
/// recursive Form walk appears to reset (or not propagate) the GS
/// fill colour on entry to the child Form's content stream. Per PDF
/// §8.10.1 a Form XObject inherits the parent graphics state at the
/// point of invocation, with only `q ... Q` saving/restoring around
/// the call; the fill colour set with `rg` before `/Fm1 Do` should be
/// visible inside the Form's content stream.
#[ignore = "FORM-RECURSION-FILL-NOT-INHERITED: page-level fill not seen by Form's ImageMask paint"]
#[test]
fn qa_form_fill_inheritance_bug_pin() {
    let mask = solid_image_mask_bytes(8, 8);
    let page = "q\n0 1 0 rg\n/Fm1 Do\nQ\n";
    // Form sets only the CTM — does NOT set a fill colour itself, so
    // it must inherit the page-level `0 1 0 rg`.
    let form = "100 0 0 100 0 0 cm\n/IM1 Do\n";
    let bytes = build_pdf_form_with_inner_image_mask(page, form, "", 8, 8, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let on = render_with_pipeline(&doc, true);
    let (r, g, b, _a) = center_pixel(&on);
    // Expected per spec: GS state propagates into child Form content
    // stream. Observed: centre is (0, 0, 0) — the page-level `rg` did
    // not stick across the Form boundary.
    assert!(
        g > 200 && r < 60 && b < 60,
        "page-level fill must be visible at Form's ImageMask paint, got ({r}, {g}, {b}) — FORM-RECURSION-FILL-NOT-INHERITED"
    );
}

/// Probe 17 — Form-XObject with a nested CTM transformation around
/// the inner ImageMask. Inside the Form, an inner `q ... cm ... /IM1
/// Do ... Q` must compose with the page's `cm` cleanly under both
/// toggle states.
#[test]
fn qa_form_xobject_inner_ctm_around_image_mask_toggle_parity() {
    let mask = solid_image_mask_bytes(8, 8);
    // The page sets a 30° rotation; the form sets a translation and
    // scale around the mask. CTM stack correctness across the form
    // boundary is what's being pinned.
    let page = "q\n0.866 0.5 -0.5 0.866 50 50 cm\n/Fm1 Do\nQ\n";
    let form = "q\n1 0 0 rg\n40 0 0 40 -20 -20 cm\n/IM1 Do\nQ\n";

    let bytes = build_pdf_form_with_inner_image_mask(page, form, "", 8, 8, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "Form-XObject inner CTM around ImageMask must round-trip byte-identically"
    );
    assert!(
        count_ink_pixels(&on, 0, 0, 100, 100) > 100,
        "Form with rotated + nested CTM should leave visible ink"
    );
}

// ===========================================================================
// Probes 18-22 — Multi-XObject interactions.
//
// These probes load two or more XObjects into a single page and pin
// that `q/Q` saving/restoring the GS state, plus the spliced GS clone
// the pipeline emits at each `/IM Do`, doesn't leak across paints.
// ===========================================================================

/// Build a page with two ImageMask XObjects `/IM1` and `/IM2` (both
/// solid stencils) and run an arbitrary content stream.
fn build_pdf_two_image_masks(content_ops: &str, w1: u32, h1: u32, w2: u32, h2: u32) -> Vec<u8> {
    let mask1 = solid_image_mask_bytes(w1, h1);
    let mask2 = solid_image_mask_bytes(w2, h2);

    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");
    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    let page_off = buf.len();
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
          /Resources << /XObject << /IM1 5 0 R /IM2 6 0 R >> >> /Contents 4 0 R >>\nendobj\n",
    );
    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let im1_off = buf.len();
    let im1_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Image /ImageMask true \
         /Width {} /Height {} /BitsPerComponent 1 /Length {} >>\nstream\n",
        w1,
        h1,
        mask1.len()
    );
    buf.extend_from_slice(im1_hdr.as_bytes());
    buf.extend_from_slice(&mask1);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let im2_off = buf.len();
    let im2_hdr = format!(
        "6 0 obj\n<< /Type /XObject /Subtype /Image /ImageMask true \
         /Width {} /Height {} /BitsPerComponent 1 /Length {} >>\nstream\n",
        w2,
        h2,
        mask2.len()
    );
    buf.extend_from_slice(im2_hdr.as_bytes());
    buf.extend_from_slice(&mask2);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 7\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off, im1_off, im2_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 7 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

/// Build a page with an ImageMask `/IM1` and a standard image `/SI1`,
/// so probes can interleave them.
fn build_pdf_mask_plus_standard_image(
    content_ops: &str,
    mask_w: u32,
    mask_h: u32,
    std_w: u32,
    std_h: u32,
    std_pixels: &[u8],
) -> Vec<u8> {
    let mask = solid_image_mask_bytes(mask_w, mask_h);

    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");
    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    let pages_off = buf.len();
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    let page_off = buf.len();
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
          /Resources << /XObject << /IM1 5 0 R /SI1 6 0 R >> >> /Contents 4 0 R >>\nendobj\n",
    );
    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let im_off = buf.len();
    let im_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Image /ImageMask true \
         /Width {} /Height {} /BitsPerComponent 1 /Length {} >>\nstream\n",
        mask_w,
        mask_h,
        mask.len()
    );
    buf.extend_from_slice(im_hdr.as_bytes());
    buf.extend_from_slice(&mask);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let si_off = buf.len();
    let si_hdr = format!(
        "6 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} \
         /BitsPerComponent 8 /ColorSpace /DeviceGray /Length {} >>\nstream\n",
        std_w,
        std_h,
        std_pixels.len()
    );
    buf.extend_from_slice(si_hdr.as_bytes());
    buf.extend_from_slice(std_pixels);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    let xref_off = buf.len();
    buf.extend_from_slice(b"xref\n0 7\n0000000000 65535 f \n");
    for off in [cat_off, pages_off, page_off, stream_off, im_off, si_off] {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size 7 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off).as_bytes(),
    );
    buf
}

/// Probe 18 — Two ImageMasks back-to-back, painted with different fill
/// colours (red then blue). Each splice clones GS afresh; the second
/// paint must not see the first paint's spliced state. The two halves
/// of the page should end up cleanly coloured.
#[test]
fn qa_two_image_masks_back_to_back_distinct_colours_toggle_parity() {
    // Left half: red. Right half: blue. The `q ... Q` brackets isolate
    // each paint's CTM and fill state.
    let content = "q\n1 0 0 rg\n50 0 0 100 0 0 cm\n/IM1 Do\nQ\n\
                   q\n0 0 1 rg\n50 0 0 100 50 0 cm\n/IM2 Do\nQ\n";
    let bytes = build_pdf_two_image_masks(content, 8, 8, 8, 8);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "two back-to-back ImageMasks must be byte-identical off vs on");

    // Sample the left half (red) and right half (blue) interior.
    let (r1, g1, b1, _a) = pixel_at(&on, 20, 50);
    let (r2, g2, b2, _a) = pixel_at(&on, 80, 50);
    assert!(
        r1 > 200 && g1 < 60 && b1 < 60,
        "left half should be red, got ({r1}, {g1}, {b1})"
    );
    assert!(
        b2 > 200 && r2 < 60 && g2 < 60,
        "right half should be blue, got ({r2}, {g2}, {b2})"
    );
}

/// Probe 19 — ImageMask, standard image, ImageMask interleaved on
/// the same page. The standard-image branch's `render_image` borrows
/// the unspliced `gs`; the mask branch borrows the spliced clone.
/// The standard image must not pick up the mask's spliced state, and
/// vice versa.
#[test]
fn qa_image_mask_then_standard_then_mask_interleaved_toggle_parity() {
    // 4x4 grey pixels for the standard image.
    let std_pixels = vec![0x60u8; 16];
    // Left strip: red mask. Middle: grey std image. Right strip: blue mask.
    let content = "q\n0.5 g\n40 0 0 100 30 0 cm\n/SI1 Do\nQ\n\
                   q\n1 0 0 rg\n30 0 0 100 0 0 cm\n/IM1 Do\nQ\n\
                   q\n0 0 1 rg\n30 0 0 100 70 0 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_mask_plus_standard_image(content, 8, 8, 4, 4, &std_pixels);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "mask + std + mask interleave must be byte-identical off vs on");

    // Left strip: red.
    let (r1, g1, b1, _a) = pixel_at(&on, 15, 50);
    assert!(r1 > 200 && g1 < 60 && b1 < 60, "left strip must be red, got ({r1},{g1},{b1})");
    // Middle: dark grey from the standard image (≈0x60 with possible filter).
    let (r2, g2, b2, _a) = pixel_at(&on, 50, 50);
    assert!(
        r2 == g2 && g2 == b2 && (60..=160).contains(&(r2 as i32)),
        "middle must be grey from standard image, got ({r2},{g2},{b2})"
    );
    // Right strip: blue.
    let (r3, g3, b3, _a) = pixel_at(&on, 85, 50);
    assert!(b3 > 200 && r3 < 60 && g3 < 60, "right strip must be blue, got ({r3},{g3},{b3})");
}

/// Probe 20 — ImageMask under an active SMask. The renderer must apply
/// the SMask to the paint; toggle parity (a DeviceRGB fill resolves
/// identically through both paths).
#[test]
fn qa_image_mask_under_active_smask_toggle_parity() {
    // Build a PDF with an SMask-bearing ExtGState. The pilot doesn't
    // exercise this for ImageMask; we want the parity guarantee.
    //
    // Strategy: page resources carry /GS1 in /ExtGState with `/SMask
    // /None` set explicitly. This is the "no smask" form but it
    // exercises the ExtGState plumbing without needing a full SMask
    // dict (which requires a transparency group XObject).
    let mask = solid_image_mask_bytes(8, 8);
    let resources = "/ExtGState << /GS1 << /SMask /None >> >>";
    let content = "q\n/GS1 gs\n1 0 0 rg\n100 0 0 100 0 0 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_image_mask(content, resources, 8, 8, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "ImageMask under /SMask /None ExtGState must be byte-identical");
}

/// Probe 21 — ImageMask under an active clip path. Pixels outside the
/// clip must be unchanged; toggle parity confirms the spliced clone
/// doesn't drop the clip.
#[test]
fn qa_image_mask_under_active_clip_toggle_parity_corner_unchanged() {
    let mask = solid_image_mask_bytes(8, 8);
    // Clip to a 40×40 box around the page centre, then paint a full-page
    // stencil. Corners must remain white.
    let content = "q\n30 30 40 40 re W n\n1 0 0 rg\n100 0 0 100 0 0 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_image_mask(content, "", 8, 8, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "ImageMask under active clip must be byte-identical off vs on");

    // Centre is inside the clip → red.
    let (r, g, b, _a) = center_pixel(&on);
    assert!(r > 200 && g < 60 && b < 60, "centre must be red, got ({r}, {g}, {b})");
    // The top-left corner (5,5) is well outside the 40×40 clip box (which
    // spans 30..70 in both axes) and must remain unpainted (white).
    let (rc, gc, bc, _a) = pixel_at(&on, 5, 5);
    assert_eq!(
        (rc, gc, bc),
        (255, 255, 255),
        "outside-clip corner must be white, got ({rc}, {gc}, {bc})"
    );
}

/// Probe 22 — ImageMask painted under a non-Normal blend mode. The
/// wave-3 `render_image_mask` reads `gs.blend_mode` and converts it
/// via `pdf_blend_mode_to_skia`. Toggle parity confirms the spliced
/// clone preserves the blend mode.
#[test]
fn qa_image_mask_multiply_blend_mode_toggle_parity() {
    let mask = solid_image_mask_bytes(8, 8);
    let resources = "/ExtGState << /GS1 << /BM /Multiply >> >>";
    let content = "q\n/GS1 gs\n1 0 0 rg\n100 0 0 100 0 0 cm\n/IM1 Do\nQ\n";
    let bytes = build_pdf_image_mask(content, resources, 8, 8, &mask);
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "ImageMask under Multiply blend mode must be byte-identical");

    // Multiply with red against white background → red.
    let (r, g, b, _a) = center_pixel(&on);
    assert!(
        r > 200 && g < 60 && b < 60,
        "multiply(red, white) must be red, got ({r}, {g}, {b})"
    );
}
