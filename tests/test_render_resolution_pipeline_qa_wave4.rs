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
