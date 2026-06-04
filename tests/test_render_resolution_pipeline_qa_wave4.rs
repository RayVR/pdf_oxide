//! Wave-4 QA probes for the resolution-pipeline migration (shading `sh`).
//!
//! Sibling to waves 1 (paths/stroke), 2 (text), 3 (ImageMask + `Do`). The
//! pilot file (`test_render_resolution_pipeline_pilot.rs`) covers the
//! happy path: DeviceRGB / DeviceGray Type-2 axial parity, the Type-4
//! Separation capability gain, DeviceRGB Type-3 radial parity, CTM/clip
//! preservation, and Type-1 / Type-4 mesh pass-through.
//!
//! This wave-4 QA suite probes the corners the pilot doesn't:
//!
//! 1. **Type 2 axial edges** — non-horizontal `/Coords` (vertical,
//!    reversed, diagonal); non-default `/Domain`; explicit `/Extend`
//!    both ways.
//! 2. **Type 3 radial edges** — non-concentric circles; zero-radius
//!    inner circle (origin point); `/Extend`.
//! 3. **Colour space stress** — inline Separation/CMYK/Type-4,
//!    ICCBased, Indexed, DeviceN with multi-colorant Type-4.
//! 4. **Function-shape edges** — Type 3 stitching with 3+ sub-functions,
//!    sub-functions with different domains, Type 2 with `N != 1`.
//! 5. **State interaction** — `q ... Q`, SMask, clip path, blend mode.
//! 6. **Pass-through types** — Type 1 function-based, Type 4-7 meshes
//!    must stay byte-identical under the toggle.
//! 7. **Adversarial input** — missing `/ColorSpace`, missing `/Function`,
//!    missing `/C0` / `/C1`, empty `/Functions` array; no-panic
//!    invariant.
//! 8. **Performance** — N-shading pipeline-on vs pipeline-off wall-clock
//!    ratio; one large shading should be O(pixmap size).
//!
//! Style mirrors waves 1-3: build a tiny PDF inline, render twice
//! through `render_with_pipeline`, compare pixmaps or sample pixels.
//! When a probe finds a bug, the test is committed with `#[ignore]` and
//! a comment naming the failing invariant; pin tests are committed
//! enabled.

#![cfg(feature = "rendering")]
#![allow(dead_code)] // probes accrete across commits; not every helper is wired up yet.

use pdf_oxide::document::PdfDocument;
use pdf_oxide::rendering::{render_page, ImageFormat, RenderOptions};
use std::sync::Mutex;
use std::time::Instant;

/// Process-wide lock for env-var test orchestration. Cargo runs
/// integration tests in parallel; flipping `PDF_OXIDE_RESOLUTION_PIPELINE`
/// must not race with another test's read.
static PIPELINE_TOGGLE_LOCK: Mutex<()> = Mutex::new(());

// ===========================================================================
// PDF construction helpers — self-contained so a fix-pass to the pilot or
// wave-1/2/3 QA helpers can't accidentally invalidate the wave-4 invariants.
//
// All builders produce a 100×100 user-space page with a single shading
// resource named `/Sh1`. The content stream drives `/Sh1 sh` (or `q ...
// /Sh1 sh Q` for graphics-state probes). Extra resources / objects let
// individual probes declare per-page colour spaces and stand-alone
// function objects referenced from inline Separation / ICCBased arrays.
// ===========================================================================

/// Build a one-page PDF whose page resources carry a shading dict at
/// `/Sh1`. The shading dict body is the caller's responsibility — the
/// builder substitutes it verbatim into object 5. This is the most
/// flexible form: probes that need arbitrary `/Extend`, `/Domain`,
/// stitching `/Functions` arrays, or alternate `/FunctionType` values
/// build the dict text themselves.
///
/// Object numbering: 1 Catalog, 2 Pages, 3 Page, 4 Content, 5 Shading,
/// 6+ extra. `extra_objects` is `[(obj_num, body_with_obj_header)]`;
/// the obj_num must agree with the body's leading `N 0 obj`.
fn build_pdf_shading_raw(
    content_ops: &str,
    shading_body: &str,
    extra_resources: &str,
    extra_objects: &[(usize, String)],
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
         /Resources << /Shading << /Sh1 5 0 R >> {} >> /Contents 4 0 R >>\nendobj\n",
        extra_resources
    );
    buf.extend_from_slice(page.as_bytes());

    let stream_off = buf.len();
    let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content_ops.len());
    buf.extend_from_slice(stream_hdr.as_bytes());
    buf.extend_from_slice(content_ops.as_bytes());
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    let shading_off = buf.len();
    buf.extend_from_slice(format!("5 0 obj\n{}\nendobj\n", shading_body).as_bytes());

    let mut offsets = vec![cat_off, pages_off, page_off, stream_off, shading_off];
    for (_n, body) in extra_objects {
        offsets.push(buf.len());
        buf.extend_from_slice(body.as_bytes());
    }

    let xref_off = buf.len();
    let size = offsets.len() + 1;
    buf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", size).as_bytes());
    for off in &offsets {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", size, xref_off)
            .as_bytes(),
    );
    buf
}

/// Convenience wrapper for a Type 2 (axial) shading with a Type 2
/// (exponential) function. Builds the shading dict from the parts.
fn build_pdf_axial_shading(
    content_ops: &str,
    space_str: &str,
    coords: &str,
    c0: &str,
    c1: &str,
    extra_shading_keys: &str,
    extra_function_keys: &str,
    extra_resources: &str,
    extra_objects: &[(usize, String)],
) -> Vec<u8> {
    let shading_body = format!(
        "<< /ShadingType 2 /ColorSpace {} /Coords {} /Domain [0 1] {} \
         /Function << /FunctionType 2 /Domain [0 1] /C0 {} /C1 {} /N 1 {} >> >>",
        space_str, coords, extra_shading_keys, c0, c1, extra_function_keys
    );
    build_pdf_shading_raw(content_ops, &shading_body, extra_resources, extra_objects)
}

/// Convenience wrapper for a Type 3 (radial) shading with a Type 2
/// (exponential) function.
fn build_pdf_radial_shading(
    content_ops: &str,
    space_str: &str,
    coords_6: &str,
    c0: &str,
    c1: &str,
    extra_shading_keys: &str,
    extra_resources: &str,
    extra_objects: &[(usize, String)],
) -> Vec<u8> {
    let shading_body = format!(
        "<< /ShadingType 3 /ColorSpace {} /Coords {} /Domain [0 1] {} \
         /Function << /FunctionType 2 /Domain [0 1] /C0 {} /C1 {} /N 1 >> >>",
        space_str, coords_6, extra_shading_keys, c0, c1
    );
    build_pdf_shading_raw(content_ops, &shading_body, extra_resources, extra_objects)
}

/// Build a Type 4 PostScript-function object string referencing the
/// supplied program (the contents between the `{ }` braces are also
/// supplied; the helper wraps them in the brace pair).
fn type4_function_object(obj_num: usize, program: &str, range: &str) -> String {
    let stream = format!("{{ {} }}", program);
    format!(
        "{} 0 obj\n<< /FunctionType 4 /Domain [0 1] /Range {} /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        obj_num,
        range,
        stream.len(),
        stream
    )
}

/// Build a Type 4 PostScript-function object with a multi-input domain.
fn type4_function_object_multi(obj_num: usize, program: &str, domain: &str, range: &str) -> String {
    let stream = format!("{{ {} }}", program);
    format!(
        "{} 0 obj\n<< /FunctionType 4 /Domain {} /Range {} /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        obj_num,
        domain,
        range,
        stream.len(),
        stream
    )
}

// ===========================================================================
// Render orchestration — toggle the env var around a render call. Shared
// mutex serialises env access across parallel-test runs.
// ===========================================================================

/// Render holding the toggle to `enabled` for the call's duration.
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

// ===========================================================================
// Probes 1-5 — Type 2 axial edges.
//
// `render_axial_shading` parses `/Coords [x0 y0 x1 y1]` and hands the
// transformed endpoints to a `tiny_skia::LinearGradient` with two stops
// at t=0 and t=1. The wave-4 splice only changes the two stop *colours*;
// the geometry (Coords, Domain, Extend, SpreadMode) is untouched. So
// every probe here keeps the colour fixed under a Device family (off-vs-on
// parity must hold byte-for-byte) and varies only the geometric parameter.
// A mismatch flags a renderer state that the pipeline accidentally
// perturbs.
// ===========================================================================

/// Probe 1 — Vertical `/Coords` (`[x0 y0 x0 y1]`). The axis runs
/// top-to-bottom rather than left-to-right. Pre-existing behaviour;
/// toggle parity must hold since DeviceRGB short-circuits at the
/// Device-family arm of `build_logical_color` and folds the same RGBA
/// triple into the stop as the inline path's `parse_color_array`.
#[test]
fn qa_axial_vertical_coords_toggle_parity() {
    // Red at top (y=10) → blue at bottom (y=90) in user space. PDF
    // user-space y axis points up, so y=10 is near the bottom of the
    // page in pixmap coordinates and y=90 is near the top.
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_axial_shading(
        content,
        "/DeviceRGB",
        "[50 10 50 90]",
        "[1 0 0]",
        "[0 0 1]",
        "",
        "",
        "",
        &[],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "DeviceRGB vertical-axis axial shading must be byte-identical off vs on"
    );

    // Positive correctness — top of pixmap (small y) corresponds to
    // large user-y, which is the `/C1 [0 0 1]` (blue) end.
    let (r_top, g_top, b_top, _) = pixel_at(&on, 50, 15);
    let (r_bot, g_bot, b_bot, _) = pixel_at(&on, 50, 85);
    assert!(
        b_top > 200 && r_top < 60 && g_top < 60,
        "top of vertical gradient should be ~blue (C1), got ({r_top}, {g_top}, {b_top})"
    );
    assert!(
        r_bot > 200 && g_bot < 60 && b_bot < 60,
        "bottom of vertical gradient should be ~red (C0), got ({r_bot}, {g_bot}, {b_bot})"
    );
}

/// Probe 2 — Reversed `/Coords` (`[x1 y x0 y]`, i.e. coordinate-1 is
/// to the left of coordinate-0). The gradient runs right-to-left in
/// user space. The colour at the "high-x" end of the page should be
/// C0 (since C0 sits at the *first* listed point, which is at the
/// right). Toggle parity must hold.
#[test]
fn qa_axial_reversed_coords_toggle_parity() {
    // /C0 at (90, 50), /C1 at (10, 50). The "high x" side of the page
    // is C0 = green; the "low x" side is C1 = white.
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_axial_shading(
        content,
        "/DeviceRGB",
        "[90 50 10 50]",
        "[0 1 0]", // C0 = green
        "[1 1 1]", // C1 = white
        "",
        "",
        "",
        &[],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "DeviceRGB reversed-axis axial shading must be byte-identical off vs on"
    );

    // Sample inside the SpreadMode::Pad region near the right edge
    // (x=95 projects beyond t=0 → clamps to pure C0 green).
    let (r_r, g_r, b_r, _) = pixel_at(&on, 95, 50);
    assert!(
        g_r > 230 && r_r < 40 && b_r < 40,
        "right edge under reversed coords must clamp to C0=green; got ({r_r}, {g_r}, {b_r})"
    );
    // Left edge (x=5 projects past t=1 → clamps to pure C1 white).
    let (r_l, g_l, b_l, _) = pixel_at(&on, 5, 50);
    assert!(
        r_l > 230 && g_l > 230 && b_l > 230,
        "left edge under reversed coords must clamp to C1=white; got ({r_l}, {g_l}, {b_l})"
    );
}

/// Probe 3 — Non-default `/Domain` (`[-0.5 1.5]`). Per ISO 32000-1
/// §8.7.4.5.3 the gradient maps device-space t into the function's
/// domain by `x = D0 + t*(D1 - D0)`. The two endpoint colours that the
/// stops carry should be the function evaluated at the *axis*
/// endpoints (t=0 and t=1 along the geometric axis, which under
/// non-default Domain means evaluation at x=D0 and x=D1 inside the
/// function, NOT at x=0 and x=1).
///
/// The wave-4 helper reads `/C0` and `/C1` raw and hands them through
/// — it does NOT consult Domain — so for `N=1` exponential
/// interpolation the result at the axis endpoints is `C0` (for any
/// Domain that includes 0) and `C1` (for any Domain that includes 1)
/// only if Domain == [0 1].
///
/// **#[ignore]** — The current implementer ignores `/Domain` on the
/// shading dict; both paths share the same bug. This probe pins the
/// observed behaviour so a future Domain fix can flip the assertion
/// in a single line.
///
/// Bug name: WAVE4-SHADING-DOMAIN-NOT-APPLIED-TO-ENDPOINTS.
#[test]
#[ignore = "WAVE4-SHADING-DOMAIN-NOT-APPLIED-TO-ENDPOINTS: /Domain other than [0 1] should remap t into function input"]
fn qa_axial_domain_other_than_unit_interval() {
    // /Domain [-0.5 1.5]. With N=1 exponential, C0=red, C1=blue, the
    // colour at the geometric C0 end of the axis should be the
    // function evaluated at x=-0.5 (which exponential extrapolates to
    // red + (-0.5)^1 * (blue-red) = 1.5*red - 0.5*blue, i.e.
    // (1.5, 0, -0.5) clamped → (1, 0, 0) red but slightly shifted).
    // The renderer currently pins the geometric C0 endpoint stop to
    // `/C0` verbatim regardless of Domain; this probe asserts that
    // pinned value to lock in the current behaviour.
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_axial_shading(
        content,
        "/DeviceRGB",
        "[10 50 90 50]",
        "[1 0 0]", // C0
        "[0 0 1]", // C1
        "/Domain [-0.5 1.5]",
        "",
        "",
        &[],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let on = render_with_pipeline(&doc, true);
    let (r, g, b, _) = pixel_at(&on, 5, 50);
    // Under proper Domain handling at x=-0.5 the colour should NOT be
    // pure red. Pin: a spec-compliant renderer would paint something
    // notably off-red. The current renderer paints pure red because
    // /Domain is dropped.
    assert!(
        r < 245 || g > 10 || b > 10,
        "with /Domain [-0.5 1.5] the C0 endpoint should evaluate the function at x=-0.5, \
         not paint raw C0 red; got ({r}, {g}, {b})"
    );
}

/// Probe 3b — Companion pin: with the current (buggy) implementation,
/// /Domain is dropped; the toggle parity must still hold. This is the
/// regression pin that catches a future Domain implementation that
/// breaks off/on parity while only fixing one side.
#[test]
fn qa_axial_domain_other_than_unit_interval_toggle_parity() {
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_axial_shading(
        content,
        "/DeviceRGB",
        "[10 50 90 50]",
        "[1 0 0]",
        "[0 0 1]",
        "/Domain [-0.5 1.5]",
        "",
        "",
        &[],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(
        off, on,
        "non-default /Domain DeviceRGB axial shading must keep off-vs-on parity"
    );
}

/// Probe 4 — Explicit `/Extend [true true]`. Per spec, when true the
/// shading extends past the geometric endpoints with the endpoint
/// colour, exactly what tiny-skia's `SpreadMode::Pad` already does.
/// So the renderer's hard-coded Pad happens to do the right thing for
/// `[true true]` — pin the parity and the visible-extension behaviour.
#[test]
fn qa_axial_extend_true_true_toggle_parity() {
    // Axis is the middle 20% of the page (x=40 → x=60). With Extend
    // [true true], pixels at x=5 and x=95 should still be painted —
    // clamped to C0 and C1 respectively.
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_axial_shading(
        content,
        "/DeviceRGB",
        "[40 50 60 50]",
        "[1 0 0]", // C0 red
        "[0 0 1]", // C1 blue
        "/Extend [true true]",
        "",
        "",
        &[],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "/Extend [true true] axial shading must keep off-vs-on parity");

    // Inside the extension region at x=5 (well left of x=40), the
    // colour must still be ~C0 red. SpreadMode::Pad already gives us
    // this.
    let (r_l, g_l, b_l, _) = pixel_at(&on, 5, 50);
    assert!(
        r_l > 200 && g_l < 60 && b_l < 60,
        "/Extend [true true] must paint past C0 endpoint with C0 colour; got ({r_l}, {g_l}, {b_l})"
    );
    let (r_r, g_r, b_r, _) = pixel_at(&on, 95, 50);
    assert!(
        b_r > 200 && r_r < 60 && g_r < 60,
        "/Extend [true true] must paint past C1 endpoint with C1 colour; got ({r_r}, {g_r}, {b_r})"
    );
}

/// Probe 5 — Explicit `/Extend [false false]`. Per spec, when false
/// the shading must NOT paint beyond the geometric endpoints. tiny-skia's
/// `SpreadMode::Pad` clamps to the endpoint colour, which is the
/// wrong behaviour for `[false false]` — anything outside the axis
/// projection should be the page background, not the endpoint colour.
///
/// **#[ignore]** — The current `render_axial_shading` hard-codes
/// `SpreadMode::Pad` and ignores `/Extend` entirely. Both paths share
/// the bug; this probe pins it.
///
/// Bug name: WAVE4-SHADING-EXTEND-NOT-HONOURED.
#[test]
#[ignore = "WAVE4-SHADING-EXTEND-NOT-HONOURED: /Extend [false false] should NOT paint past geometric endpoints"]
fn qa_axial_extend_false_false_does_not_paint_past_endpoints() {
    // Axis the middle 20% of the page. With Extend [false false] the
    // pixel at x=5 (far left of axis) should be the page background,
    // not C0 red.
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_axial_shading(
        content,
        "/DeviceRGB",
        "[40 50 60 50]",
        "[1 0 0]",
        "[0 0 1]",
        "/Extend [false false]",
        "",
        "",
        &[],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let on = render_with_pipeline(&doc, true);
    let (r, g, b, _) = pixel_at(&on, 5, 50);
    // A spec-compliant renderer leaves this pixel as the white page
    // background. The current renderer paints it C0 red.
    assert!(
        r > 230 && g > 230 && b > 230,
        "with /Extend [false false], pixels outside the axis must be page background; \
         got ({r}, {g}, {b})"
    );
}

/// Probe 5b — Companion parity pin: even though `/Extend` is dropped,
/// off-vs-on must still match. Locks in the pre-existing bug while
/// pinning the wave-4 invariant (toggle parity for Device families).
#[test]
fn qa_axial_extend_false_false_toggle_parity() {
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_axial_shading(
        content,
        "/DeviceRGB",
        "[40 50 60 50]",
        "[1 0 0]",
        "[0 0 1]",
        "/Extend [false false]",
        "",
        "",
        &[],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "/Extend [false false] axial shading must keep off-vs-on parity");
}

// ===========================================================================
// Probes 6-8 — Type 3 radial edges.
//
// `render_radial_shading` parses `/Coords [x0 y0 r0 x1 y1 r1]` but
// **discards** `x0`, `y0`, and `r0` entirely — the start of the radial
// is hard-coded to centre `(x1, y1)` with radius 0. This pre-existing
// bug means non-concentric circles can't be rendered correctly and
// non-zero `r0` is silently dropped. The wave-4 splice doesn't touch
// this geometry — it only changes the two stop colours — but the QA
// brief explicitly asks for these probes so we pin the current
// behaviour and document the bug for a follow-up.
// ===========================================================================

/// Probe 6 — Non-concentric circles. `/Coords [x0 y0 r0 x1 y1 r1]`
/// where (x0, y0) != (x1, y1). The PDF spec defines the gradient as
/// a family of circles interpolating between the two, but
/// `render_radial_shading` ignores x0/y0/r0 and centres both
/// start and end on (x1, y1). Pin the toggle parity (the splice
/// doesn't perturb the geometry) and the visible bug.
///
/// **#[ignore]** for the geometric probe — the visible result with
/// non-concentric input matches concentric input around (x1, y1).
///
/// Bug name: WAVE4-RADIAL-NON-CONCENTRIC-COORDS-IGNORED.
#[test]
#[ignore = "WAVE4-RADIAL-NON-CONCENTRIC-COORDS-IGNORED: render_radial_shading discards x0/y0/r0 from /Coords"]
fn qa_radial_non_concentric_circles_uses_x0_y0() {
    // Inner circle at (20, 50) r=5; outer at (80, 50) r=30. A
    // spec-compliant renderer would paint C0 red around (20, 50) and
    // C1 white around (80, 50). The current renderer paints both
    // around (80, 50). Pin: pixel at (20, 50) should be near-red if
    // the gradient honoured x0/y0.
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_radial_shading(
        content,
        "/DeviceRGB",
        "[20 50 5 80 50 30]",
        "[1 0 0]",
        "[1 1 1]",
        "",
        "",
        &[],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let on = render_with_pipeline(&doc, true);
    let (r, g, b, _) = pixel_at(&on, 20, 50);
    assert!(
        r > 200 && g < 60 && b < 60,
        "non-concentric radial: pixel at the inner-circle centre (20, 50) should be ~C0 red; \
         got ({r}, {g}, {b}) — renderer is dropping x0/y0"
    );
}

/// Probe 6b — Companion parity pin: the splice must preserve the
/// (buggy) pre-existing geometry off vs on.
#[test]
fn qa_radial_non_concentric_circles_toggle_parity() {
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_radial_shading(
        content,
        "/DeviceRGB",
        "[20 50 5 80 50 30]",
        "[1 0 0]",
        "[1 1 1]",
        "",
        "",
        &[],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "non-concentric radial DeviceRGB must keep off-vs-on parity");
}

/// Probe 7 — One zero-radius circle (origin point). `/Coords
/// [x0 y0 0 x1 y1 r1]` defines a gradient growing from a point at
/// (x0, y0) outward to a circle at (x1, y1) of radius r1. This is
/// the most common radial-shading shape in real PDFs (highlight
/// gradients, spotlight effects). The current renderer ignores
/// (x0, y0, r0) and paints centred on (x1, y1) — the parity pin
/// holds, the geometric correctness probe doesn't.
///
/// **#[ignore]** for the geometric pin.
#[test]
#[ignore = "WAVE4-RADIAL-NON-CONCENTRIC-COORDS-IGNORED: zero-radius inner circle at distinct (x0, y0) is dropped"]
fn qa_radial_zero_radius_inner_at_distinct_point() {
    // Inner point at (30, 30); outer circle at (60, 60) r=40. A
    // correct renderer paints C0 only at (30, 30) and lerps outward.
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_radial_shading(
        content,
        "/DeviceRGB",
        "[30 30 0 60 60 40]",
        "[1 0 0]",
        "[1 1 1]",
        "",
        "",
        &[],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let on = render_with_pipeline(&doc, true);
    // Pixel close to the inner-circle origin should be near-C0 red
    // under correct rendering. In pixmap coords y is flipped, so
    // user (30, 30) maps to pixmap (30, 70).
    let (r, g, b, _) = pixel_at(&on, 30, 70);
    assert!(
        r > 200 && g < 80 && b < 80,
        "zero-r0 radial: pixel at the inner point should be ~C0; got ({r}, {g}, {b})"
    );
}

/// Probe 7b — Companion parity pin for the zero-radius inner point.
#[test]
fn qa_radial_zero_radius_inner_at_distinct_point_toggle_parity() {
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_radial_shading(
        content,
        "/DeviceRGB",
        "[30 30 0 60 60 40]",
        "[1 0 0]",
        "[1 1 1]",
        "",
        "",
        &[],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "zero-r0 radial DeviceRGB must keep off-vs-on parity");
}

/// Probe 8 — `/Extend [true true]`. The end-circle radius is small
/// (r=20), centred at (50, 50). With Extend [true true] pixels
/// outside the larger circle should be C1, pixels inside the
/// degenerate inner point should be C0. The current renderer uses
/// `SpreadMode::Pad` which happens to match the spec here — pin the
/// parity and the visible behaviour.
#[test]
fn qa_radial_extend_true_true_toggle_parity_and_correctness() {
    let content = "/Sh1 sh\n";
    let bytes = build_pdf_radial_shading(
        content,
        "/DeviceRGB",
        "[50 50 0 50 50 20]",
        "[1 0 0]", // C0 red (centre)
        "[0 0 1]", // C1 blue (edge)
        "/Extend [true true]",
        "",
        &[],
    );
    let doc = PdfDocument::from_bytes(bytes).expect("PDF parses");
    let off = render_with_pipeline(&doc, false);
    let on = render_with_pipeline(&doc, true);
    assert_eq!(off, on, "concentric radial /Extend [true true] must keep off-vs-on parity");

    // Centre should be ~C0 red.
    let (r_c, g_c, b_c, _) = pixel_at(&on, 50, 50);
    assert!(
        r_c > 200 && g_c < 80 && b_c < 80,
        "radial centre should be ~C0 red; got ({r_c}, {g_c}, {b_c})"
    );
    // Corner of page (outside r=20 from centre): should be C1 blue
    // under Pad/Extend-true.
    let (r_e, g_e, b_e, _) = pixel_at(&on, 5, 5);
    assert!(
        b_e > 200 && r_e < 80 && g_e < 80,
        "/Extend [true true] should fill outside radius with C1 blue; got ({r_e}, {g_e}, {b_e})"
    );
}
