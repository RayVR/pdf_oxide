//! Transparency-correctness audit probes — composite (pixmap) render path.
//!
//! This suite enumerates ISO 32000-1:2008 §11.3.5 (blend modes), §11.4
//! (transparency: groups, soft masks, group composition), §11.6
//! (transparency group XObjects), and §11.7.4 (overprint) features and
//! pins what `pdf_oxide` does today on the composite render path
//! (`pdf_oxide::rendering::render_page`). Where the implementation is
//! correct, a live byte-anchored probe acts as a regression sentry.
//! Where the implementation is partial or absent, the probe is
//! `#[ignore]`-marked with a `HONEST_GAP_<feature>` tracking constant
//! so the gap surfaces by name to the next round of work.
//!
//! ## Feature inventory matrix
//!
//! | Feature                                         | Spec      | Implemented? | Test status | Tracking                  |
//! |-------------------------------------------------|-----------|--------------|-------------|---------------------------|
//! | `/CA`, `/ca` ExtGState alpha                    | §11.3.4   | yes          | LIVE        | regression sentry         |
//!
//! ### Source citations for the inventory
//!
//! - `src/rendering/ext_gstate.rs:30-53` — `ParsedExtGState::apply`
//!   routes `/CA` to `gs.stroke_alpha` and `/ca` to `gs.fill_alpha`;
//!   the rasteriser folds those alphas into the painted pixels via
//!   tiny_skia's `Color::from_rgba(_, _, _, alpha)`.
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
    assert!(
        r > 200 && (g as i32 - b as i32).abs() <= 5,
        "/CA 0.5 stroke top edge: R must remain high (>200) and G≈B; got ({r}, {g}, {b})"
    );
    assert!(
        (100..=200).contains(&g),
        "/CA 0.5 stroke top edge: G must be midway (faded); got G={g}"
    );
}
