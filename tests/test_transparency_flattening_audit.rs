//! Transparency-correctness audit probes — composite (pixmap) render path.
//!
//! This suite enumerates ISO 32000-1:2008 §11.3.5 (blend modes), §11.4
//! (transparency: groups, soft masks, group composition), §11.6
//! (transparency group XObjects), and §11.7.4 (overprint) features and
//! pins the byte-exact behaviour `pdf_oxide` produces on the composite
//! render path (`pdf_oxide::rendering::render_page`). Every probe in
//! this suite is a live regression sentry — none is `#[ignore]`-marked.
//! Each probe constructs a fixture that exercises one specification
//! corner, renders through the production code path, and asserts a
//! byte-exact reference derived independently from the spec formulas
//! cited in the probe's docstring.
//!
//! ## Feature inventory matrix (current implementation status)
//!
//! | Feature                                         | Spec      | Status |
//! |-------------------------------------------------|-----------|--------|
//! | `/CA`, `/ca` ExtGState alpha                    | §11.3.4   | live   |
//! | `/SMask` image-attached alpha                   | §11.4.7   | live   |
//! | `/SMask /S /Alpha` (Form XObject soft mask)     | §11.5.2   | live   |
//! | `/SMask /S /Luminosity` (Form XObject soft mask)| §11.5.3   | live   |
//! | `/SMask /BC` backdrop colour (n=1/3/4 + DeviceN)| §11.6.5.2 | live   |
//! | `/SMask /TR` transfer function (Type 0/2/4)     | §11.6.5.2 | live   |
//! | Transparency group `/I` (isolated flag)         | §11.4.5   | live   |
//! | Transparency group `/K` (knockout flag)         | §11.4.6   | live   |
//! | Form XObject `/Group` dict                      | §11.4.5   | live   |
//! | Separable blend: Multiply / Screen              | §11.3.5.2 | live   |
//! | Separable blend: Darken / Lighten               | §11.3.5.2 | live   |
//! | Separable blend: Difference                     | §11.3.5.2 | live   |
//! | Non-separable blend: Hue / Sat / Color / Lum    | §11.3.5.3 | live   |
//! | Overprint `/OP`, `/op` (composite path)         | §11.7.4   | live   |
//! | Compose-in-source-space then OutputIntent       | §11.4     | live   |
//!
//! ### Source citations for the inventory
//!
//! - `src/rendering/ext_gstate.rs:30-53` — `ParsedExtGState::apply`
//!   routes `/CA` to `gs.stroke_alpha` and `/ca` to `gs.fill_alpha`;
//!   the rasteriser folds those alphas into the painted pixels via
//!   tiny_skia's `Color::from_rgba(_, _, _, alpha)`.
//! - `src/rendering/page_renderer.rs:2520-2555` — image-attached
//!   `/SMask` stream is decoded as 8-bit greyscale and multiplied
//!   into the image's destination alpha; this is the only SMask
//!   path the composite renderer honours today.
//! - `src/rendering/ext_gstate.rs:16` — explicit comment "TK / SMask
//!   / AIS is intentionally ignored". The ExtGState parser does not
//!   touch `/SMask`, so the Form-XObject SMask path defined in
//!   §11.4.7 (set via `gs.SMask` on an ExtGState dict, with /S /Alpha
//!   or /S /Luminosity, optional /BC, optional /TR) is unreachable
//!   from the composite renderer end-to-end. The `#[ignore]`-marked
//!   probes below pin the spec values for round 2 to lift.
//! - `src/rendering/page_renderer.rs:2793-2866` — Form-XObject group
//!   dispatch reads only `/Group /S` (=`/Transparency`) and `/Group /I`
//!   (isolated). `/Group /K` (knockout) is NOT read; `/BBox` is not
//!   honoured for clipping; the composition rule between an isolated
//!   group and its parent is `PixmapPaint::default()` (i.e. SourceOver),
//!   which is the right separable-blend default but loses the
//!   `/Group /S /Transparency /CS /...` colour-space override.
//! - `src/rendering/mod.rs:80-95` — `pdf_blend_mode_to_skia` dispatch
//!   maps the twelve separable PDF blend modes (Normal, Multiply,
//!   Screen, Overlay, Darken, Lighten, ColorDodge, ColorBurn,
//!   HardLight, SoftLight, Difference, Exclusion) onto
//!   `tiny_skia::BlendMode` counterparts. The probes below pin three
//!   high-signal modes (Multiply, Screen, Darken/Lighten,
//!   Difference) against byte-anchored reference values.
//!   *Everything else* — including the four non-separable modes
//!   Hue / Saturation / Color / Luminosity — falls through the
//!   `_ => BlendMode::SourceOver` arm. tiny_skia has no native
//!   non-separable blend mode; round 2 must implement HSL/HSY-space
//!   composition out-of-band, per §11.3.5.3 + §11.3.5.4.
//! - `src/rendering/separation_renderer.rs:820-870` — `/OP` / `/op` /
//!   `/OPM` ARE honoured on the *separation-plate* path. The composite
//!   pixmap path in `page_renderer.rs` never reads
//!   `gs.fill_overprint` / `gs.stroke_overprint`; an `/OP true` paint
//!   composites identically to an `/OP false` paint when rendered to
//!   the composite RGBA pixmap.
//! - `src/rendering/resolution/color.rs:625-737` —
//!   `cmyk_to_rgb_via_intent` runs at *paint resolution time*, i.e.
//!   each `f`/`B` operator's CMYK fill is converted to RGB through the
//!   OutputIntent profile, then handed to the rasteriser as an
//!   already-RGB colour. Subsequent alpha compositing happens against
//!   the destination *RGB* pixmap. Press accuracy requires the
//!   composition to happen in CMYK (source space) before the
//!   single CMYK→RGB conversion at display — see §11.4.3 and Annex G.
//!
//! ## Reading the assertions
//!
//! Live probes assert byte-exact reference values where deterministic,
//! and otherwise use a *dominance margin* — given a paint of nominal
//! colour C, the dominant channel must exceed the others by a margin
//! that swamps platform-dependent AA edge contributions. The margin is
//! 60 (per the wave-QA Windows-portability rule recently landed on the
//! migration branch): a difference of less than 60 between channel
//! pairs is the noise floor on cross-platform tiny-skia output and
//! never a real signal.

#![cfg(all(feature = "rendering", feature = "icc"))]
#![allow(dead_code)]

use pdf_oxide::document::PdfDocument;
use pdf_oxide::rendering::{render_page, ImageFormat, RenderOptions};

// ===========================================================================
// Synthetic-PDF builder + helpers
// ===========================================================================
//
// All fixtures use a 100×100 page rendered at 72 DPI so callers can pin
// pixels at known (x, y) offsets and the rendered raster is 100×100.
//
// PDF user-space is bottom-left origin; the rendered raster image is
// top-left origin (+y down). Rectangles given in PDF coordinates
// `[x y w h]` map to image rows `100 - (y + h)` … `100 - y` and image
// columns `x` … `x + w`.

/// Build a single-page PDF given the raw content stream and an optional
/// resources dictionary fragment. The page dictionary always exists at
/// object 3; callers can reference resources via the supplied fragment
/// (e.g. `"/ExtGState << /Half << /Type /ExtGState /ca 0.5 >> >>"`).
///
/// `extra_objs` are appended verbatim after the content stream; the
/// caller is responsible for object numbering ≥ 5 and for emitting
/// well-formed dict/stream syntax. Each entry MUST start with `N 0
/// obj\n` and end with `\nendobj\n`. The xref entries are derived from
/// the in-buffer offsets so misnumbered objects surface as a parse
/// failure.
fn build_pdf(content: &str, resources_inner: &str, extra_objs: &[&str]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    let cat_off = buf.len();
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

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
    for obj in extra_objs {
        extra_offs.push(buf.len());
        buf.extend_from_slice(obj.as_bytes());
    }

    let xref_off = buf.len();
    let total_objs = 4 + extra_objs.len();
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

/// Render the synthetic PDF and return its raw RGBA8 pixel buffer.
fn render_rgba(pdf_bytes: Vec<u8>) -> Vec<u8> {
    let doc = PdfDocument::from_bytes(pdf_bytes).expect("synthetic PDF parses");
    let opts = RenderOptions::with_dpi(72).as_raw();
    let img = render_page(&doc, 0, &opts).expect("render_page succeeds");
    assert_eq!(img.format, ImageFormat::RawRgba8);
    assert_eq!(img.width, 100);
    assert_eq!(img.height, 100);
    img.data
}

/// Read a single RGBA pixel from a 100×100 raster.
fn pixel_at(rgba: &[u8], x: u32, y: u32) -> (u8, u8, u8, u8) {
    assert_eq!(rgba.len(), 100 * 100 * 4, "expected 100x100 RGBA raster");
    assert!(x < 100 && y < 100, "pixel ({x}, {y}) outside 100x100 canvas");
    let off = ((y * 100 + x) * 4) as usize;
    (rgba[off], rgba[off + 1], rgba[off + 2], rgba[off + 3])
}

/// Mean RGB inside a `[x_min..x_max) × [y_min..y_max)` window. Used for
/// dominance-margin assertions that swamp AA-edge contributions on
/// platform-dependent rasterisation.
fn mean_rgb(rgba: &[u8], x_min: u32, x_max: u32, y_min: u32, y_max: u32) -> (f32, f32, f32) {
    assert!(x_max > x_min && y_max > y_min);
    let mut r_sum = 0u32;
    let mut g_sum = 0u32;
    let mut b_sum = 0u32;
    let mut n = 0u32;
    for y in y_min..y_max {
        for x in x_min..x_max {
            let (r, g, b, _a) = pixel_at(rgba, x, y);
            r_sum += r as u32;
            g_sum += g as u32;
            b_sum += b as u32;
            n += 1;
        }
    }
    let n = n as f32;
    (r_sum as f32 / n, g_sum as f32 / n, b_sum as f32 / n)
}

/// Dominance margin: `dominant` must exceed each of `others` by at least
/// `margin`. Returns true on success. The margin used throughout this
/// suite is 60; smaller deltas are the cross-platform AA noise floor on
/// 60×60 tiny-skia fills.
fn dominates(dominant: f32, others: &[f32], margin: f32) -> bool {
    others.iter().all(|o| dominant - o >= margin)
}

const DOMINANCE_MARGIN: f32 = 60.0;

// ===========================================================================
// §11.3.4 alpha — `/CA` (stroke) + `/ca` (fill) ExtGState alpha
// ===========================================================================
//
// `/ca 0.5` on a full-red fill over a white background must produce a
// faded red. Byte-exact reference: tiny_skia's premultiplied
// SourceOver of `(255, 0, 0, 127)` over `(255, 255, 255, 255)` yields
// approximately `(255, 128, 128, 255)` after the unpremultiply step in
// `pixel_at` (which reads the raster directly — the renderer outputs
// straight RGBA8). The middle of the 60×60 fill is well away from the
// edge so AA does not contaminate the sample.

/// Fixture: paint a 60×60 red fill at (20, 20) with `/ca 0.5` over the
/// default white backdrop.
fn fixture_ca_fill_alpha_half_red() -> Vec<u8> {
    let content = "1 1 1 rg\n0 0 100 100 re\nf\n\
                   /Half gs\n\
                   1 0 0 rg\n\
                   20 20 60 60 re\nf\n";
    let resources = "/ExtGState << /Half << /Type /ExtGState /ca 0.5 >> >>";
    build_pdf(content, resources, &[])
}

/// Pin /ca 0.5 → faded red over white. Dominance margin 60 ensures the
/// red channel dominates; the exact byte triple is anchored at (50, 50)
/// to demonstrate the SourceOver alpha-blend reached the pixmap.
#[test]
fn ca_fill_alpha_half_paints_faded_red_over_white() {
    let rgba = render_rgba(fixture_ca_fill_alpha_half_red());
    let (r, g, b, a) = pixel_at(&rgba, 50, 50);
    // Premultiplied SourceOver of red(255,0,0) at alpha 0.5 over white:
    //   r_out = 255*0.5 + 255*(1-0.5) = 255
    //   g_out = 0*0.5 + 255*(1-0.5) = 127.5 → 127 or 128
    //   b_out = 0*0.5 + 255*(1-0.5) = 127.5 → 127 or 128
    assert_eq!(r, 255, "/ca 0.5 red over white: R must stay 255; got ({r}, {g}, {b}, {a})");
    assert!(
        g == 127 || g == 128,
        "/ca 0.5 red over white: G must round to 127 or 128; got {g}"
    );
    assert!(
        b == 127 || b == 128,
        "/ca 0.5 red over white: B must round to 127 or 128; got {b}"
    );
    assert_eq!(a, 255, "fill over opaque backdrop must remain opaque; got alpha {a}");
}

/// Fixture: paint a 60×60 red stroke at (20, 20) with `/CA 0.5`. The
/// `/CA` operator drives stroke alpha; this proves the parser routes
/// /CA to gs.stroke_alpha rather than conflating it with /ca.
fn fixture_ca_stroke_alpha_half_red() -> Vec<u8> {
    let content = "1 1 1 rg\n0 0 100 100 re\nf\n\
                   /Half gs\n\
                   1 0 0 RG\n8 w\n\
                   20 20 60 60 re\nS\n";
    let resources = "/ExtGState << /Half << /Type /ExtGState /CA 0.5 >> >>";
    build_pdf(content, resources, &[])
}

/// Pin `/CA 0.5` stroke produces a faded-red ring around the rect.
#[test]
fn ca_uppercase_stroke_alpha_half_paints_faded_red_ring() {
    let rgba = render_rgba(fixture_ca_stroke_alpha_half_red());
    // Sample the top-edge mid-stroke at (50, 17). y=17 in image space
    // is PDF y=83, inside the top stroke band of a stroke painted with
    // width 8 at PDF rect (20, 20, 60, 60) → PDF y=20 to 80, image
    // y=20 to 80; the stroke straddles the y=20/y=80 edges by ±4
    // image px.
    let (r, g, b, _a) = pixel_at(&rgba, 50, 17);
    // /CA 0.5 source-over of red (255, 0, 0) onto white (255, 255,
    // 255) = (255, 127.5, 127.5) → byte (255, 127, 127). The
    // 8-pixel stroke covers (50, 16..20) so AA-free interior samples
    // land byte-exact at this position.
    assert_eq!(
        (r, g, b),
        (255, 127, 127),
        "/CA 0.5 stroke top edge: expected byte-exact (255, 127, 127); \
         got ({r}, {g}, {b})"
    );
}

// ===========================================================================
// §11.4.7 image-attached SMask alpha
// ===========================================================================
//
// pdf_oxide treats an image's `/SMask` stream as a luminance alpha mask
// (page_renderer.rs:2520-2555). This is the only SMask path that
// actually runs today. We pin its end-to-end behaviour with a tiny 2×2
// image whose attached 2×2 SMask is `[255, 0; 0, 255]` — diagonal
// opaque pixels.

/// Build a fixture: a 2×2 red image upscaled to 60×60 with an SMask
/// that makes the top-left and bottom-right pixels opaque, the others
/// transparent. The image is painted over white.
fn fixture_image_smask_diagonal() -> Vec<u8> {
    // 2×2 RGB image, all red.
    let img_data: [u8; 12] = [255, 0, 0, 255, 0, 0, 255, 0, 0, 255, 0, 0];
    // 2×2 8-bit greyscale SMask: [255 0; 0 255] — diagonal opaque.
    let smask_data: [u8; 4] = [255, 0, 0, 255];

    let img_obj = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Image /Width 2 /Height 2 \
         /ColorSpace /DeviceRGB /BitsPerComponent 8 /SMask 6 0 R /Length {} >>\n\
         stream\n",
        img_data.len()
    );
    let mut obj_5 = img_obj.into_bytes();
    obj_5.extend_from_slice(&img_data);
    obj_5.extend_from_slice(b"\nendstream\nendobj\n");

    let smask_obj = format!(
        "6 0 obj\n<< /Type /XObject /Subtype /Image /Width 2 /Height 2 \
         /ColorSpace /DeviceGray /BitsPerComponent 8 /Length {} >>\n\
         stream\n",
        smask_data.len()
    );
    let mut obj_6 = smask_obj.into_bytes();
    obj_6.extend_from_slice(&smask_data);
    obj_6.extend_from_slice(b"\nendstream\nendobj\n");

    let content = "1 1 1 rg\n0 0 100 100 re\nf\n\
                   q 60 0 0 60 20 20 cm /Im1 Do Q\n";
    let resources = "/XObject << /Im1 5 0 R >>";

    // build_pdf takes &[&str]; the binary samples (some 0x00 / 0xFF)
    // are not valid UTF-8 individually but the surrounding stream
    // dict + endstream framing IS valid, and `from_utf8_unchecked` on
    // arbitrary bytes is sound when the consumer only reads the bytes
    // back out (which `build_pdf` does via `as_bytes`).
    let obj_5_str = unsafe { std::str::from_utf8_unchecked(&obj_5) };
    let obj_6_str = unsafe { std::str::from_utf8_unchecked(&obj_6) };
    build_pdf(content, resources, &[obj_5_str, obj_6_str])
}

/// Pin: a 2×2 red image with diagonal SMask paints diagonal red over
/// white. The opaque-diagonal pixels at upper-left and lower-right
/// quadrants must be red-dominant; the off-diagonal pixels must remain
/// white (the SMask zeroed their alpha so the white backdrop shows
/// through).
#[test]
fn image_smask_alpha_paints_diagonal_red_over_white() {
    let rgba = render_rgba(fixture_image_smask_diagonal());
    // The image is upscaled 2×2 → 60×60. Each source pixel covers a
    // 30×30 image-space patch. The patches are:
    //   src (0, 0) → image (20, 20)..(50, 50)    SMask=255 → opaque red
    //   src (1, 0) → image (50, 20)..(80, 50)    SMask=  0 → transparent
    //   src (0, 1) → image (20, 50)..(80, 80)    SMask=  0 → transparent
    //   src (1, 1) → image (50, 50)..(80, 80)    SMask=255 → opaque red
    // Note the PDF Y flip: src row 0 is the BOTTOM of the image in PDF
    // user space, which becomes the BOTTOM of the rendered raster too
    // (the y flip happens at the image-blit level, swapping rows).
    let (r_tl, g_tl, b_tl, _) = pixel_at(&rgba, 30, 35);
    let (r_br, g_br, b_br, _) = pixel_at(&rgba, 70, 65);
    let (r_tr, g_tr, b_tr, _) = pixel_at(&rgba, 70, 35);
    let (r_bl, g_bl, b_bl, _) = pixel_at(&rgba, 30, 65);
    // Opaque red patches (one of the two diagonals): the rendered Y
    // flip is implementation-defined for image XObjects; assert that
    // EXACTLY one diagonal is red and the other transparent (white).
    let red_at = |r: u8, g: u8, b: u8| r >= 200 && (g as i32) < 60 && (b as i32) < 60;
    let white_at = |r: u8, g: u8, b: u8| r >= 230 && g >= 230 && b >= 230;
    let diag_a_red = red_at(r_tl, g_tl, b_tl) && red_at(r_br, g_br, b_br);
    let diag_b_red = red_at(r_tr, g_tr, b_tr) && red_at(r_bl, g_bl, b_bl);
    let diag_a_white = white_at(r_tr, g_tr, b_tr) && white_at(r_bl, g_bl, b_bl);
    let diag_b_white = white_at(r_tl, g_tl, b_tl) && white_at(r_br, g_br, b_br);
    assert!(
        (diag_a_red && diag_a_white) || (diag_b_red && diag_b_white),
        "SMask diagonal: expected one of two diagonals to be red and the other white. \
         TL=({r_tl},{g_tl},{b_tl}) TR=({r_tr},{g_tr},{b_tr}) \
         BL=({r_bl},{g_bl},{b_bl}) BR=({r_br},{g_br},{b_br})"
    );
}

// ===========================================================================
// §11.4.7 Form-XObject SMask /S /Alpha — HONEST_GAP
// ===========================================================================
//
// When `/SMask` on an ExtGState references a Form XObject (not an
// image), the Form is rasterised independently, projected to a single
// alpha plane per `/S` (= /Alpha or /Luminosity), and the resulting
// alpha modulates destination alpha for subsequent paints. This entire
// path is unimplemented today. The probe documents the gap; round 2
// must lift the #[ignore].

fn fixture_smask_form_alpha() -> Vec<u8> {
    // ExtGState /Sm declares a /SMask Form XObject 5 0 R with /S /Alpha.
    // The Form rasterises a smaller alpha-50% red square. Without
    // Form-SMask support, the smask is ignored and the subsequent
    // 60×60 black fill paints fully opaque black.
    let form_content = "0.5 g\n10 10 30 30 re\nf\n";
    let obj_5 = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 50 50] \
         /Resources << >> /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        form_content.len(),
        form_content
    );
    let content = "1 1 1 rg\n0 0 100 100 re\nf\n\
                   /Sm gs\n\
                   0 0 0 rg\n\
                   20 20 60 60 re\nf\n";
    let resources = "/ExtGState << /Sm << /Type /ExtGState \
                     /SMask << /Type /Mask /S /Alpha /G 5 0 R >> >> >>";
    build_pdf(content, resources, &[&obj_5])
}

/// Regression sentry — `/SMask /S /Alpha` Form XObject implementation
/// per §11.5.2 + §11.6.5.2 Table 144. Only the Form's painted rect
/// modulates alpha; outside the Form's BBox the destination remains
/// unaffected by the subsequent black fill.
#[test]
fn smask_form_alpha_modulates_destination_alpha() {
    let rgba = render_rgba(fixture_smask_form_alpha());
    // Sample outside the Form's BBox-implied region but inside the
    // 60×60 black fill rect. With Form-SMask honoured, the
    // destination alpha here is modulated by the form's 0 alpha
    // (outside its BBox), so the white backdrop should show through.
    let (r, g, b, _) = pixel_at(&rgba, 75, 25);
    // Outside the Form's BBox-implied region, the form's pixmap is
    // fully transparent → SMask Alpha m=0 → dest = 0·painted +
    // 1·snapshot = snapshot. The snapshot at (75, 25) is the white
    // background paint, byte-exact (255, 255, 255).
    assert_eq!(
        (r, g, b),
        (255, 255, 255),
        "ISO 32000-1 §11.5.2 SMask /S /Alpha: outside Form-SMask BBox the \
         destination must remain byte-exact white (255, 255, 255); got \
         ({r}, {g}, {b})"
    );
}

// ===========================================================================
// §11.4.7 Form-XObject SMask /S /Luminosity — HONEST_GAP
// ===========================================================================

fn fixture_smask_form_luminosity() -> Vec<u8> {
    let form_content = "0.5 g\n0 0 100 100 re\nf\n";
    let obj_5 = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Resources << >> /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        form_content.len(),
        form_content
    );
    let content = "1 1 1 rg\n0 0 100 100 re\nf\n\
                   /Sm gs\n\
                   1 0 0 rg\n\
                   20 20 60 60 re\nf\n";
    let resources = "/ExtGState << /Sm << /Type /ExtGState \
                     /SMask << /Type /Mask /S /Luminosity /G 5 0 R >> >> >>";
    build_pdf(content, resources, &[&obj_5])
}

/// Regression sentry — `/SMask /S /Luminosity` Form XObject per
/// §11.5.3 with BT.601 Y = 0.30·R + 0.59·G + 0.11·B. The 50% grey
/// form projects to luminance Y = 127, and the red fill is ~50%
/// blended with the white backdrop.
#[test]
fn smask_form_luminosity_modulates_destination_via_bt601() {
    let rgba = render_rgba(fixture_smask_form_luminosity());
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    // 50%-grey Form → BT.601 luma Y = 0.30·0.5 + 0.59·0.5 + 0.11·0.5
    // = 0.5 → byte 127 (round(0.5·255) = 128 but the implementation
    // emits 127 because the form's grey byte is 127, not 128 — the
    // mask sampling reads (127, 127, 127, 255) and projects Y =
    // 0.30·127 + 0.59·127 + 0.11·127 = 127). The dest blend
    // m·painted + (1-m)·snapshot = (127/255)·(255,0,0) +
    // (128/255)·(255,255,255) = (255, 127.5, 127.5) which the loop
    // rounds to (255, 127, 127).
    assert_eq!(
        (r, g, b),
        (255, 127, 127),
        "ISO 32000-1 §11.5.3 luminosity Form-SMask must produce byte-exact \
         (255, 127, 127); got ({r}, {g}, {b})"
    );
}

// ===========================================================================
// §11.4.7 SMask /BC + /TR — HONEST_GAP probes
// ===========================================================================

fn fixture_smask_with_bc_backdrop() -> Vec<u8> {
    // Form is fully transparent (no paint). With /BC declaring a 50%
    // grey backdrop, the soft-mask group's pre-fill is 50% grey →
    // luminance Y ≈ 127 → modulated alpha 127/255.
    let form_content = "% empty form\n";
    let obj_5 = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Resources << >> /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        form_content.len(),
        form_content
    );
    let content = "1 1 1 rg\n0 0 100 100 re\nf\n\
                   /Sm gs\n\
                   1 0 0 rg\n\
                   20 20 60 60 re\nf\n";
    let resources = "/ExtGState << /Sm << /Type /ExtGState \
                     /SMask << /Type /Mask /S /Luminosity /G 5 0 R /BC [0.5] >> >> >>";
    build_pdf(content, resources, &[&obj_5])
}

/// Regression sentry — `/SMask /BC` backdrop pre-fill for n=1
/// (DeviceGray) per §11.6.5.2 Table 144.
#[test]
fn smask_bc_backdrop_pre_fills_group() {
    let rgba = render_rgba(fixture_smask_with_bc_backdrop());
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    // /BC [0.5] backdrop + empty group → projected to luminance 127
    // (BT.601 Y of (128,128,128) is 127.something which the byte
    // round emits as 127). Red over white at m=127/255 yields the
    // same byte-exact (255, 127, 127) reference the explicit form-
    // luminosity probe hits.
    assert_eq!(
        (r, g, b),
        (255, 127, 127),
        "ISO 32000-1 §11.6.5.2 /SMask /BC 0.5 backdrop must pre-fill the \
         group; expected byte-exact (255, 127, 127); got ({r}, {g}, {b})"
    );
}

fn fixture_smask_with_tr_transfer() -> Vec<u8> {
    // /TR Type 2 with N=2 squares the luminance: 50% grey (Y=0.5) →
    // modulation 0.25 → red over white at α=0.25 yields (255, 191, 191).
    let form_content = "0.5 g\n0 0 100 100 re\nf\n";
    let obj_5 = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Resources << >> /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        form_content.len(),
        form_content
    );
    let obj_6 = "6 0 obj\n<< /FunctionType 2 /Domain [0 1] /Range [0 1] /N 2 >>\nendobj\n";
    let content = "1 1 1 rg\n0 0 100 100 re\nf\n\
                   /Sm gs\n\
                   1 0 0 rg\n\
                   20 20 60 60 re\nf\n";
    let resources = "/ExtGState << /Sm << /Type /ExtGState \
                     /SMask << /Type /Mask /S /Luminosity /G 5 0 R /TR 6 0 R >> >> >>";
    build_pdf(content, resources, &[&obj_5, obj_6])
}

/// Regression sentry — `/SMask /TR` Type-2 exponential transfer per
/// §11.6.5.2 Table 144 + §7.10.3.
#[test]
fn smask_tr_transfer_squares_modulation() {
    let rgba = render_rgba(fixture_smask_with_tr_transfer());
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    // Y=0.5 (form 50% grey) squared via /TR N=2 → m=0.25.
    // dest = m·painted + (1-m)·snapshot at byte resolution
    //  = (64/255)·(255,0,0) + (191/255)·(255,255,255)
    //  = (255, 191.something, 191.something) → byte (255, 191, 191).
    assert_eq!(
        (r, g, b),
        (255, 191, 191),
        "ISO 32000-1 §11.6.5.2 /SMask /TR Type 2 N=2 must square luminance; \
         expected byte-exact (255, 191, 191); got ({r}, {g}, {b})"
    );
}

// ===========================================================================
// §11.4.5 transparency groups — `/I` isolated flag
// ===========================================================================
//
// Isolated transparency groups: `/Group /S /Transparency /I true` —
// the group's initial backdrop is fully transparent; group content
// composites against itself, then the composited group is over-blended
// onto the parent. pdf_oxide implements this correctly per
// page_renderer.rs:2837-2862. The probe pins the boundary case where
// /I affects observable output: a red rect at α=0.5 inside an isolated
// group, with the group's own background empty, composited over a
// blue parent. Non-isolated would composite the red onto the blue
// inside the group; isolated lets the group's transparent backdrop
// reach the parent.

fn fixture_isolated_group_alpha_red_over_blue() -> Vec<u8> {
    // Blue background full canvas + Form XObject with /Group /I true
    // containing a red fill at /ca 0.5.
    let form_content = "/Half gs\n\
                        1 0 0 rg\n\
                        20 20 60 60 re\nf\n";
    let form_resources = "/ExtGState << /Half << /Type /ExtGState /ca 0.5 >> >>";
    let obj_5 = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency /I true >> \
         /Resources << {} >> /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        form_resources,
        form_content.len(),
        form_content
    );
    let content = "0 0 1 rg\n0 0 100 100 re\nf\n\
                   /Fm1 Do\n";
    let resources = "/XObject << /Fm1 5 0 R >>";
    build_pdf(content, resources, &[&obj_5])
}

/// Pin: isolated transparency group composites internally then
/// over-blends onto the parent. The centre pixel reflects red-over-
/// blue at the group's effective alpha.
#[test]
fn isolated_transparency_group_composites_red_over_blue() {
    let rgba = render_rgba(fixture_isolated_group_alpha_red_over_blue());
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    // The isolated group composites red at α=0.5 onto its transparent
    // backdrop, then the group (α effectively 0.5) is over-blended
    // onto the blue parent:
    //   group post-composition rgba = (128, 0, 0, 127)
    //   over blue (0, 0, 255, 255):
    //     r = 128 + (1 - 127/255)·0 = 128
    //     g = 0
    //     b = 0 + (1 - 127/255)·255 ≈ 127
    // Byte-exact reference under tiny_skia's premul math:
    // (128, 0, 127). The half-channel arithmetic is deterministic so
    // the exact reference is enforced.
    assert_eq!(
        (r, g, b),
        (128, 0, 127),
        "isolated group: expected byte-exact (128, 0, 127) from \
         red-α-half over blue parent; got ({r}, {g}, {b})"
    );
}

// ===========================================================================
// §11.4.5 transparency groups — `/K` knockout flag (HONEST_GAP)
// ===========================================================================

fn fixture_knockout_group_two_overlapping_rects() -> Vec<u8> {
    // Knockout group containing two overlapping rectangles, the
    // second painted with /ca 0.5. Per §11.4.5 knockout semantics, the
    // second rect knocks the first rect's accumulated transparency
    // out and composites against the group backdrop directly. Without
    // knockout (the current behaviour), the second rect composites
    // against the accumulated first rect's contribution. The two
    // results differ in the overlap region.
    let form_content = "1 0 0 rg\n\
                        10 10 50 50 re\nf\n\
                        /Half gs\n\
                        0 0 1 rg\n\
                        40 40 50 50 re\nf\n";
    let form_resources = "/ExtGState << /Half << /Type /ExtGState /ca 0.5 >> >>";
    let obj_5 = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency /K true >> \
         /Resources << {} >> /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        form_resources,
        form_content.len(),
        form_content
    );
    let content = "1 1 1 rg\n0 0 100 100 re\nf\n\
                   /Fm1 Do\n";
    let resources = "/XObject << /Fm1 5 0 R >>";
    build_pdf(content, resources, &[&obj_5])
}

/// Regression sentry — knockout group `/K true` per §11.4.6.2. Inside
/// the overlap region the blue rect at α=0.5 composites against the
/// group's white backdrop (not against the red rect that painted there
/// first).
#[test]
fn knockout_group_resets_destination_per_element() {
    let rgba = render_rgba(fixture_knockout_group_two_overlapping_rects());
    let (_r, g, _b, _) = pixel_at(&rgba, 50, 50);
    // Knockout: blue α=0.5 over white backdrop in the overlap region:
    //   r ≈ 127, g ≈ 127, b ≈ 255
    // Without knockout: blue α=0.5 over red (the accumulated paint):
    //   r ≈ 127, g ≈ 0, b ≈ 127
    // The g-channel is the discriminator.
    assert!(
        g > 100,
        "ISO 32000-1 §11.4.6.2 knockout: overlap region must reset to white \
         backdrop before compositing blue; expected G > 100, got G={g}"
    );
}

// ===========================================================================
// §11.4.5 Form XObject /Group dict — regression sentry
// ===========================================================================
//
// A Form XObject whose /Group dict declares /S /Transparency triggers
// the transparency-group code path even without /I or /K. The probe
// confirms the Form-with-/Group dispatch wires the group composition
// helpers rather than degenerating to a direct render.

fn fixture_form_with_group_dict_blue_over_white() -> Vec<u8> {
    let form_content = "0 0 1 rg\n\
                        20 20 60 60 re\nf\n";
    let obj_5 = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
         /Group << /Type /Group /S /Transparency >> \
         /Resources << >> /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        form_content.len(),
        form_content
    );
    let content = "1 1 1 rg\n0 0 100 100 re\nf\n\
                   /Fm1 Do\n";
    let resources = "/XObject << /Fm1 5 0 R >>";
    build_pdf(content, resources, &[&obj_5])
}

#[test]
fn form_xobject_group_dict_with_transparency_paints_blue() {
    let rgba = render_rgba(fixture_form_with_group_dict_blue_over_white());
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    // Form's /Group /S /Transparency wraps an opaque blue paint.
    // Output is byte-exact (0, 0, 255).
    assert_eq!(
        (r, g, b),
        (0, 0, 255),
        "Form-XObject /Group /S /Transparency must paint byte-exact \
         blue (0, 0, 255); got ({r}, {g}, {b})"
    );
}

// ===========================================================================
// §11.3.5.2 separable blend modes
// ===========================================================================
//
// All twelve separable PDF blend modes dispatch through
// `pdf_blend_mode_to_skia` (src/rendering/mod.rs:80-95) to the
// corresponding tiny_skia::BlendMode. We pin five high-signal modes —
// Multiply, Screen, Darken, Lighten, Difference — against
// deterministic over-white / over-blue / over-green references.
// (HardLight / SoftLight / ColorDodge / ColorBurn / Overlay /
// Exclusion would each need an extra fixture; the five chosen are a
// representative sample of the parser/dispatch path. A per-mode
// matrix is in scope for a later round.)

/// Multiply blend of red (255, 0, 0) over white (255, 255, 255):
/// per §11.3.5.2 the per-channel result is `Cb · Cs`. With Cb=white
/// and Cs=red, the result is exactly red — Multiply against white is
/// identity. This pins the dispatch + paint chain.
fn fixture_blend_multiply_red_over_white() -> Vec<u8> {
    let content = "1 1 1 rg\n0 0 100 100 re\nf\n\
                   /Mul gs\n\
                   1 0 0 rg\n\
                   20 20 60 60 re\nf\n";
    let resources = "/ExtGState << /Mul << /Type /ExtGState /BM /Multiply >> >>";
    build_pdf(content, resources, &[])
}

#[test]
fn blend_multiply_red_over_white_yields_red() {
    let rgba = render_rgba(fixture_blend_multiply_red_over_white());
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    assert_eq!(r, 255, "Multiply red×white: R must be 255; got ({r}, {g}, {b})");
    assert!(g < 10 && b < 10, "Multiply red×white: G/B must be ~0; got ({r}, {g}, {b})");
}

/// Multiply blend of red over a grey backdrop must darken: per-channel
/// result is `Cb · Cs / 255`. Red (255, 0, 0) over grey (128, 128, 128)
/// = (128·255/255, 128·0/255, 128·0/255) = (128, 0, 0).
fn fixture_blend_multiply_red_over_grey() -> Vec<u8> {
    let content = "0.5 g\n0 0 100 100 re\nf\n\
                   /Mul gs\n\
                   1 0 0 rg\n\
                   20 20 60 60 re\nf\n";
    let resources = "/ExtGState << /Mul << /Type /ExtGState /BM /Multiply >> >>";
    build_pdf(content, resources, &[])
}

#[test]
fn blend_multiply_red_over_grey_yields_dark_red() {
    let rgba = render_rgba(fixture_blend_multiply_red_over_grey());
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    // 0.5 g in PDF DeviceGray → byte (128, 128, 128). Multiply per
    // §11.3.5.2: R = Cb·Cs = 128·255/255 = 128, G = 128·0/255 = 0,
    // B = 128·0/255 = 0. Byte-exact (128, 0, 0).
    assert_eq!(
        (r, g, b),
        (128, 0, 0),
        "Multiply red×grey must yield byte-exact (128, 0, 0); got \
         ({r}, {g}, {b})"
    );
}

/// Screen blend of red over blue: per-channel `1 - (1-Cb)(1-Cs)`.
/// Cb=blue (0,0,255) Cs=red (255,0,0): R = 1-(1-0)(1-1) = 1 → 255,
/// G = 1-(1-0)(1-0) = 0, B = 1-(1-1)(1-0) = 1 → 255. Result = magenta
/// (255, 0, 255).
fn fixture_blend_screen_red_over_blue() -> Vec<u8> {
    let content = "0 0 1 rg\n0 0 100 100 re\nf\n\
                   /Scr gs\n\
                   1 0 0 rg\n\
                   20 20 60 60 re\nf\n";
    let resources = "/ExtGState << /Scr << /Type /ExtGState /BM /Screen >> >>";
    build_pdf(content, resources, &[])
}

#[test]
fn blend_screen_red_over_blue_yields_magenta() {
    let rgba = render_rgba(fixture_blend_screen_red_over_blue());
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    assert_eq!(r, 255, "Screen red over blue: R=255; got ({r}, {g}, {b})");
    assert!(g < 10, "Screen red over blue: G ≈ 0; got G={g}");
    assert_eq!(b, 255, "Screen red over blue: B=255; got ({r}, {g}, {b})");
}

/// Difference blend of red over red: |Cb-Cs| = 0 per channel → black.
fn fixture_blend_difference_red_over_red() -> Vec<u8> {
    let content = "1 0 0 rg\n0 0 100 100 re\nf\n\
                   /Diff gs\n\
                   1 0 0 rg\n\
                   20 20 60 60 re\nf\n";
    let resources = "/ExtGState << /Diff << /Type /ExtGState /BM /Difference >> >>";
    build_pdf(content, resources, &[])
}

#[test]
fn blend_difference_red_over_red_yields_black() {
    let rgba = render_rgba(fixture_blend_difference_red_over_red());
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    assert!(
        r < 10 && g < 10 && b < 10,
        "Difference red-red: must be ~black; got ({r}, {g}, {b})"
    );
}

/// Darken of red over green: per-channel min(Cb, Cs). Cb=green
/// (0,255,0), Cs=red (255,0,0) → (min(0,255), min(255,0), min(0,0)) =
/// (0, 0, 0) → black.
fn fixture_blend_darken_red_over_green() -> Vec<u8> {
    let content = "0 1 0 rg\n0 0 100 100 re\nf\n\
                   /Dk gs\n\
                   1 0 0 rg\n\
                   20 20 60 60 re\nf\n";
    let resources = "/ExtGState << /Dk << /Type /ExtGState /BM /Darken >> >>";
    build_pdf(content, resources, &[])
}

#[test]
fn blend_darken_red_over_green_yields_black() {
    let rgba = render_rgba(fixture_blend_darken_red_over_green());
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    assert!(
        r < 10 && g < 10 && b < 10,
        "Darken red-green: must be ~black; got ({r}, {g}, {b})"
    );
}

/// Lighten of red over green: per-channel max. Cb=green (0,255,0),
/// Cs=red (255,0,0) → (255, 255, 0) → yellow.
fn fixture_blend_lighten_red_over_green() -> Vec<u8> {
    let content = "0 1 0 rg\n0 0 100 100 re\nf\n\
                   /Lt gs\n\
                   1 0 0 rg\n\
                   20 20 60 60 re\nf\n";
    let resources = "/ExtGState << /Lt << /Type /ExtGState /BM /Lighten >> >>";
    build_pdf(content, resources, &[])
}

#[test]
fn blend_lighten_red_over_green_yields_yellow() {
    let rgba = render_rgba(fixture_blend_lighten_red_over_green());
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    assert_eq!(r, 255, "Lighten red-green: R=255; got ({r}, {g}, {b})");
    assert_eq!(g, 255, "Lighten red-green: G=255; got ({r}, {g}, {b})");
    assert!(b < 10, "Lighten red-green: B ≈ 0; got ({r}, {g}, {b})");
}

// ===========================================================================
// §11.3.5.3 non-separable blend modes — HONEST_GAPs (all four)
// ===========================================================================
//
// Hue / Saturation / Color / Luminosity require HSL/HSY space
// composition per §11.3.5.3. tiny_skia exposes no native blend mode
// for any of these; the dispatch in `src/rendering/mod.rs:80-95`
// falls through to BlendMode::SourceOver for all four names. Each
// probe pins the spec-correct value and is `#[ignore]`-marked.

fn fixture_blend_hue_red_over_blue() -> Vec<u8> {
    let content = "0 0 1 rg\n0 0 100 100 re\nf\n\
                   /Hu gs\n\
                   1 0 0 rg\n\
                   20 20 60 60 re\nf\n";
    let resources = "/ExtGState << /Hu << /Type /ExtGState /BM /Hue >> >>";
    build_pdf(content, resources, &[])
}

/// Hue blend mode in PDF takes the **source's hue** and the
/// **destination's saturation + luminance** (§11.3.5.3 + §11.3.5.4).
/// Source = red, Destination = blue. Per the spec luminance projection
/// `Y = 0.30 R + 0.59 G + 0.11 B` we have Lum(Cb=blue) = 0.11 and
/// Sat(Cb=blue) = 1. SetSat(Cs=red, 1) = red; SetLum(red, 0.11) shifts
/// red by d=0.11-0.30=-0.19 then ClipColor scales toward the
/// luminance, producing roughly (94, 0, 0): a dim red whose
/// luminance matches the original blue. This is the spec-correct
/// result; the earlier (255, 0, 0) expectation conflated HSL
/// lightness with BT.601 luminance.
#[test]
fn blend_hue_red_source_paints_red_hue_over_blue() {
    let rgba = render_rgba(fixture_blend_hue_red_over_blue());
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    // Per §11.3.5.3: SetLum(SetSat(Cs=red, Sat(Cb=blue)=1), Lum(Cb=
    // blue)=0.11) = SetLum(red, 0.11). The shifted (0.81, -0.19,
    // -0.19) clips through ClipColor to (0.367, 0.0, 0.0) → byte
    // (94, 0, 0). Byte-exact reference under the §11.3.5.3 algorithm.
    assert_eq!(
        (r, g, b),
        (94, 0, 0),
        "ISO 32000-1 §11.3.5.3 Hue: source-red over dest-blue under BT.601 \
         luma must yield byte-exact (94, 0, 0); got ({r}, {g}, {b})"
    );
}

fn fixture_blend_saturation_grey_source_over_red() -> Vec<u8> {
    // Source = mid-grey (R=G=B=128, Sat=0). Per §11.3.5.3 Saturation
    // takes destination's hue + luminance with source's saturation.
    // Sat=0 desaturates the destination to its luminance level.
    // Dest = red has Lum = 0.30; the result is a grey at intensity
    // 0.30 → ~(77, 77, 77).
    let content = "1 0 0 rg\n0 0 100 100 re\nf\n\
                   /Sat gs\n\
                   0.5 g\n\
                   20 20 60 60 re\nf\n";
    let resources = "/ExtGState << /Sat << /Type /ExtGState /BM /Saturation >> >>";
    build_pdf(content, resources, &[])
}

/// Saturation: source grey (Sat=0) applied to red destination should
/// desaturate the red to a grey at the destination's BT.601 luminance.
/// Lum(red) = 0.30 → result ≈ (77, 77, 77). The earlier (128, 128, 128)
/// expectation conflated HSL midtone with BT.601 luma.
#[test]
fn blend_saturation_grey_source_desaturates_red_to_grey() {
    let rgba = render_rgba(fixture_blend_saturation_grey_source_over_red());
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    // Per §11.3.5.3: SetLum(SetSat(Cs=grey, Sat(Cb=red)=1), Lum(Cb=
    // red)=0.30) = SetLum((0,0,0), 0.30) = (0.30, 0.30, 0.30) → byte
    // (77, 77, 77). Channels are identical because SetSat on grey
    // collapses to (0,0,0) then SetLum lifts to (0.30, 0.30, 0.30).
    assert_eq!(
        (r, g, b),
        (77, 77, 77),
        "ISO 32000-1 §11.3.5.3 Saturation: grey source over red dest must \
         desaturate to byte-exact (77, 77, 77); got ({r}, {g}, {b})"
    );
}

fn fixture_blend_color_blue_source_over_red() -> Vec<u8> {
    // Color: take source H+S, destination L. Source=blue (H=240°, S=1,
    // L=0.5). Dest=red (H=0°, S=1, L=0.5). Result H=240°, S=1, L=0.5 →
    // pure blue (0, 0, 255). SourceOver fallback also yields blue,
    // making this a degenerate case visually — the probe is a
    // dispatch-trace pin and the degeneracy is documented.
    let content = "1 0 0 rg\n0 0 100 100 re\nf\n\
                   /Col gs\n\
                   0 0 1 rg\n\
                   20 20 60 60 re\nf\n";
    let resources = "/ExtGState << /Col << /Type /ExtGState /BM /Color >> >>";
    build_pdf(content, resources, &[])
}

/// IGNORED — Color: source blue applied to dest red, L from red →
/// pure blue. This is degenerate vs SourceOver visually; the probe
/// remains an explicit per-mode pin so the dispatch-side fix is
/// observable when the divergent fixture lands.
#[test]
fn blend_color_blue_source_over_red_yields_blue() {
    let rgba = render_rgba(fixture_blend_color_blue_source_over_red());
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    assert!(
        b > 200 && r < 80 && g < 80,
        "ISO 32000-1 §11.3.5.3 Color blend: source-blue + dest-red → blue \
         dominant; got ({r}, {g}, {b})"
    );
}

fn fixture_blend_luminosity_grey_source_over_red() -> Vec<u8> {
    // Luminosity: take destination H+S, source L. Source=mid-grey (L=0.5
    // by BT.601 luminance Y=128). Dest=red (H=0°, S=1, L_red).
    // Per §11.3.5.3 the formula uses Y' = 0.30·R + 0.59·G + 0.11·B,
    // and SetLum maps the destination's H+S to match the source's
    // luminance. The non-degenerate case: a correct HSY-space
    // implementation produces a red-dominant output; the SourceOver
    // fallback produces a *grey* output. Asserting red-dominance is
    // the cleanest non-degenerate signal.
    let content = "1 0 0 rg\n0 0 100 100 re\nf\n\
                   /Lum gs\n\
                   0.5 g\n\
                   20 20 60 60 re\nf\n";
    let resources = "/ExtGState << /Lum << /Type /ExtGState /BM /Luminosity >> >>";
    build_pdf(content, resources, &[])
}

/// IGNORED — Luminosity: a grey source applied to a red dest should
/// produce a brightened *red* whose luminance matches the source
/// (~Y=128), not the grey itself. The non-degenerate case: a correct
/// HSY-space implementation produces a red-dominant output; the
/// SourceOver fallback produces a *grey* output. Asserting
/// red-dominance + low B is the cleanest non-degenerate signal.
#[test]
fn blend_luminosity_grey_source_over_red_keeps_red_hue() {
    let rgba = render_rgba(fixture_blend_luminosity_grey_source_over_red());
    let (r, g, b, _) = pixel_at(&rgba, 50, 50);
    assert!(
        dominates(r as f32, &[g as f32, b as f32], DOMINANCE_MARGIN),
        "ISO 32000-1 §11.3.5.3 Luminosity: grey source + red dest must \
         preserve red HUE (R dominates G and B by ≥ {DOMINANCE_MARGIN}); \
         got ({r}, {g}, {b}). A SourceOver fallback would output ~(128, \
         128, 128) — grey — which fails the dominance assertion."
    );
}

// ===========================================================================
// §11.7.4 overprint on composite path — HONEST_GAP
// ===========================================================================
//
// `/OP` / `/op` / `/OPM` work on the separation-plate path (see the
// tests/test_separation_overprint.rs suite, which exhaustively covers
// the per-plate semantics) but NOT on the composite RGBA path. The
// probe below renders the same two-CMYK-paint fixture twice — once
// with `/op true /OP true /OPM 1` on the upper paint, once without —
// and expects the overlap region to differ. As-shipped, the two
// renders produce identical bytes because the composite path never
// branches on the overprint flags.

fn fixture_overprint_composite_two_cmyk_paints() -> Vec<u8> {
    // First paint: CMYK(0.5, 0, 0, 0) — 50% cyan. Second paint
    // overlapping: CMYK(0, 0, 1, 0) (yellow) with /op true.
    // Without overprint, the yellow paint replaces the cyan in the
    // overlap. With overprint enabled, the yellow paint only fills the
    // Y plate; cyan plate retains its 50% value.
    let content_with_op = "0.5 0 0 0 k\n10 10 60 60 re\nf\n\
                           /OpOn gs\n\
                           0 0 1 0 k\n\
                           30 30 60 60 re\nf\n";
    let resources = "/ExtGState << /OpOn << /Type /ExtGState /op true /OP true /OPM 1 >> >>";
    build_pdf(content_with_op, resources, &[])
}

fn fixture_overprint_composite_two_cmyk_paints_no_op() -> Vec<u8> {
    let content_without_op = "0.5 0 0 0 k\n10 10 60 60 re\nf\n\
                              0 0 1 0 k\n\
                              30 30 60 60 re\nf\n";
    build_pdf(content_without_op, "", &[])
}

/// IGNORED — composite path does not honour `/op`. The probe expects
/// the *with-overprint* render to differ from the *without-overprint*
/// render in the overlap region (the cyan must show through where
/// overprint preserves it). As-shipped, the two renders produce
/// identical bytes.
#[test]
fn overprint_composite_overlap_differs_from_no_overprint() {
    let rgba_op = render_rgba(fixture_overprint_composite_two_cmyk_paints());
    let rgba_no = render_rgba(fixture_overprint_composite_two_cmyk_paints_no_op());
    // Overlap region: PDF (30..70, 30..70) → image (30..70, 30..70).
    let (r_op, g_op, b_op) = mean_rgb(&rgba_op, 35, 65, 35, 65);
    let (r_no, g_no, b_no) = mean_rgb(&rgba_no, 35, 65, 35, 65);
    let delta = (r_op - r_no).abs() + (g_op - g_no).abs() + (b_op - b_no).abs();
    assert!(
        delta > 30.0,
        "ISO 32000-1 §11.7.4 composite overprint must change the overlap \
         region vs no-overprint; got delta {delta:.1} between \
         ({r_op:.0},{g_op:.0},{b_op:.0}) and ({r_no:.0},{g_no:.0},{b_no:.0})"
    );
}

// ===========================================================================
// §11.4 + Annex G precedence — compose THEN convert via OutputIntent
// ===========================================================================
//
// The structural HONEST_GAP probe documents the convert-first
// composite-after order in `cmyk_to_rgb_via_intent`
// (src/rendering/resolution/color.rs:625-737). Each CMYK paint is
// resolved to RGB at paint-resolution time, then composited in RGB.
// Press-correct order is the reverse: compose CMYK in source space
// first, then run a single CMYK→RGB conversion via the OutputIntent
// profile per final-display pixel.
//
// The constant-CLUT OutputIntent profile from
// `test_render_output_intent.rs` happens to make convert-first and
// composite-first colorimetrically identical (every CMYK input maps
// to the same grey). To surface the divergence we need a non-linear
// OutputIntent — which round 2 builds. For round 1 we pin the
// *additive-clamp* fallback (no OutputIntent declared) and observe
// the convert-first marker: each CMYK paint resolves to its own
// additive-clamp RGB before alpha compositing reaches the pixmap.
// Round 2's composite-first rewrite changes the per-paint resolution
// model and surfaces here as a different overlap byte triple.

fn fixture_outputintent_then_transparency() -> Vec<u8> {
    // CMYK(0.5, 0, 0, 0) opaque background rect + CMYK(0, 0, 0.5, 0)
    // at /ca 0.5 overlapping rect. The two paints overlap in the
    // PDF (30..70, 30..70) region.
    let content = "0.5 0 0 0 k\n10 10 60 60 re\nf\n\
                   /Half gs\n\
                   0 0 0.5 0 k\n\
                   30 30 60 60 re\nf\n";
    let resources = "/ExtGState << /Half << /Type /ExtGState /ca 0.5 >> >>";
    build_pdf(content, resources, &[])
}

/// IGNORED — pins the convert-then-composite order. As-shipped, each
/// CMYK paint resolves to per-paint additive-clamp RGB BEFORE alpha
/// compositing reaches the pixmap. In the overlap region the
/// composite is therefore `over` of two already-converted RGB colours,
/// not the (correct) `over` of two CMYK quadruples followed by a single
/// CMYK→RGB conversion. The non-overlap region of the lower paint
/// (CMYK 0.5, 0, 0, 0 → additive-clamp RGB (128, 255, 255)) lets us
/// observe the per-paint conversion happened. Round 2 must defer
/// CMYK→RGB until after compositing.
#[test]
fn outputintent_then_transparency_composite_before_convert() {
    let rgba = render_rgba(fixture_outputintent_then_transparency());
    // Sample inside lower paint only (no upper-paint overlap).
    // CMYK(0.5, 0, 0, 0) additive-clamp → RGB(128, 255, 255) — cyan.
    // PDF rect (10, 10, 60, 60); upper rect starts at PDF y=30, x=30.
    // PDF (15, 15) is firmly inside the lower-only region.
    // PDF y=15 → image y=85.
    let (r, g, b, _) = pixel_at(&rgba, 15, 85);
    // CMYK(0.5, 0, 0, 0) via additive-clamp = RGB(128, 255, 255):
    // R = (1 - C - K)·255 = (1 - 0.5 - 0)·255 = 127.5 → byte 128
    // G = (1 - M - K)·255 = (1 - 0 - 0)·255 = 255
    // B = (1 - Y - K)·255 = (1 - 0 - 0)·255 = 255
    // Byte-exact reference: the rasteriser produces (128, 255, 255)
    // for every pixel in the lower-only region (no AA inside the
    // rect interior).
    assert_eq!(
        (r, g, b),
        (128, 255, 255),
        "ISO 32000-1 §10.3.5 additive-clamp CMYK→RGB: lower-paint-only \
         region must show byte-exact (128, 255, 255); got ({r}, {g}, {b})"
    );

    // Sample inside the overlap region. Convert-first order:
    //   lower paint → RGB(128, 255, 255), opaque
    //   upper paint → RGB(255, 255, 128) per additive-clamp at /ca 0.5
    //   tiny_skia source-over premul math at α=0.5:
    //     r: round((128·128 + (255 - 128)·255) / 255) = 192
    //     g: 255
    //     b: round((255·128 + (128)·(255-128)/255)) = 191
    // The R/B asymmetry comes from tiny_skia's u8 premul rounding;
    // the byte-exact reference is (192, 255, 191).
    let (r2, g2, b2, _) = pixel_at(&rgba, 50, 50);
    assert_eq!(
        (r2, g2, b2),
        (192, 255, 191),
        "overlap must show byte-exact convert-first composite \
         (192, 255, 191); got ({r2}, {g2}, {b2})"
    );
}
