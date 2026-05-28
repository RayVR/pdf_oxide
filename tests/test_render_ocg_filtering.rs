//! Tests for OCG layer filtering in the rendering pipeline.
//!
//! Verifies that `RenderOptions::excluded_layers` suppresses graphical content
//! (rectangles, text) drawn inside OCG-tagged BDC scopes while preserving
//! content outside those scopes.

use std::collections::HashSet;

use pdf_oxide::document::PdfDocument;
use pdf_oxide::rendering::{ImageFormat, RenderOptions};

// ============================================================================
// Helper: build a PDF with a colored rectangle inside an OCG layer + text outside
// ============================================================================

/// Build a PDF where:
/// - A red rectangle is drawn inside BDC /OC /MC0 scope (OCG "Background")
/// - Text "HELLO" is drawn outside any layer scope
///
/// The rectangle fills a 200x200 area at (50, 550) in PDF coords.
fn build_pdf_with_ocg_rect_and_text() -> Vec<u8> {
    let mut pdf = Vec::new();
    let mut offsets: Vec<usize> = Vec::new();

    pdf.extend_from_slice(b"%PDF-1.4\n");

    // Obj 1: Catalog with OCProperties
    offsets.push(pdf.len());
    pdf.extend_from_slice(
        b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R\n\
           /OCProperties << /OCGs [6 0 R] /D << /ON [6 0 R] >> >> >>\nendobj\n\n",
    );

    // Obj 2: Pages
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\n");

    // Obj 3: Page
    offsets.push(pdf.len());
    pdf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792]\n\
           /Contents 4 0 R\n\
           /Resources << /Font << /F1 5 0 R >> /Properties << /MC0 6 0 R >> >> >>\nendobj\n\n",
    );

    // Obj 4: Content stream
    // Red rectangle inside OCG "Background", then text outside any layer
    let content = b"/OC /MC0 BDC q 1 0 0 rg 50 550 200 200 re f Q EMC \
                    BT /F1 24 Tf 300 650 Td (HELLO) Tj ET";
    offsets.push(pdf.len());
    let hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len());
    pdf.extend_from_slice(hdr.as_bytes());
    pdf.extend_from_slice(content);
    pdf.extend_from_slice(b"\nendstream\nendobj\n\n");

    // Obj 5: Font
    offsets.push(pdf.len());
    pdf.extend_from_slice(
        b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica\n\
           /Encoding /WinAnsiEncoding >>\nendobj\n\n",
    );

    // Obj 6: OCG dictionary
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"6 0 obj\n<< /Type /OCG /Name /Background >>\nendobj\n\n");

    // Xref
    let xref_offset = pdf.len();
    let n_obj = offsets.len() + 1;
    let mut xref = format!("xref\n0 {}\n", n_obj);
    xref.push_str("0000000000 65535 f \n");
    for off in &offsets {
        xref.push_str(&format!("{:010} 00000 n \n", off));
    }
    pdf.extend_from_slice(xref.as_bytes());

    let trailer = format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        n_obj, xref_offset
    );
    pdf.extend_from_slice(trailer.as_bytes());
    pdf
}

/// Build a PDF with an OCMD-referenced rectangle.
fn build_pdf_with_ocmd_rect() -> Vec<u8> {
    let mut pdf = Vec::new();
    let mut offsets: Vec<usize> = Vec::new();

    pdf.extend_from_slice(b"%PDF-1.4\n");

    offsets.push(pdf.len());
    pdf.extend_from_slice(
        b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R\n\
           /OCProperties << /OCGs [7 0 R] /D << /ON [7 0 R] >> >> >>\nendobj\n\n",
    );
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\n");
    offsets.push(pdf.len());
    pdf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792]\n\
           /Contents 4 0 R\n\
           /Resources << /Font << /F1 5 0 R >> /Properties << /MC0 6 0 R >> >> >>\nendobj\n\n",
    );

    // Blue rectangle inside OCMD scope, green rectangle outside
    let content = b"/OC /MC0 BDC 0 0 1 rg 50 550 200 200 re f EMC \
                    0 1 0 rg 300 550 200 200 re f";
    offsets.push(pdf.len());
    let hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len());
    pdf.extend_from_slice(hdr.as_bytes());
    pdf.extend_from_slice(content);
    pdf.extend_from_slice(b"\nendstream\nendobj\n\n");

    offsets.push(pdf.len());
    pdf.extend_from_slice(
        b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica\n\
           /Encoding /WinAnsiEncoding >>\nendobj\n\n",
    );
    // Obj 6: OCMD referencing OCG
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"6 0 obj\n<< /Type /OCMD /OCGs [7 0 R] /P /AllOn >>\nendobj\n\n");
    // Obj 7: OCG
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"7 0 obj\n<< /Type /OCG /Name /Watermark >>\nendobj\n\n");

    let xref_offset = pdf.len();
    let n_obj = offsets.len() + 1;
    let mut xref = format!("xref\n0 {}\n", n_obj);
    xref.push_str("0000000000 65535 f \n");
    for off in &offsets {
        xref.push_str(&format!("{:010} 00000 n \n", off));
    }
    pdf.extend_from_slice(xref.as_bytes());
    let trailer = format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        n_obj, xref_offset
    );
    pdf.extend_from_slice(trailer.as_bytes());
    pdf
}

/// Build a PDF with OCG-tagged content inside a Form XObject.
fn build_pdf_with_ocg_in_form_xobject() -> Vec<u8> {
    let mut pdf = Vec::new();
    let mut offsets: Vec<usize> = Vec::new();

    pdf.extend_from_slice(b"%PDF-1.4\n");

    offsets.push(pdf.len());
    pdf.extend_from_slice(
        b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R\n\
           /OCProperties << /OCGs [7 0 R] /D << /ON [7 0 R] >> >> >>\nendobj\n\n",
    );
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\n");
    offsets.push(pdf.len());
    pdf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792]\n\
           /Contents 4 0 R\n\
           /Resources << /Font << /F1 6 0 R >> /XObject << /Fm0 5 0 R >> >> >>\nendobj\n\n",
    );

    // Page content: just invoke Form XObject, then draw green rect outside
    let page_content = b"/Fm0 Do 0 1 0 rg 300 550 200 200 re f";
    offsets.push(pdf.len());
    let hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", page_content.len());
    pdf.extend_from_slice(hdr.as_bytes());
    pdf.extend_from_slice(page_content);
    pdf.extend_from_slice(b"\nendstream\nendobj\n\n");

    // Obj 5: Form XObject with OCG-tagged red rect
    let form_stream = b"/OC /MC0 BDC 1 0 0 rg 50 550 200 200 re f EMC";
    offsets.push(pdf.len());
    let form_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 612 792]\n\
            /Resources << /Font << /F1 6 0 R >> /Properties << /MC0 7 0 R >> >>\n\
            /Length {} >>\nstream\n",
        form_stream.len()
    );
    pdf.extend_from_slice(form_hdr.as_bytes());
    pdf.extend_from_slice(form_stream);
    pdf.extend_from_slice(b"\nendstream\nendobj\n\n");

    // Obj 6: Font
    offsets.push(pdf.len());
    pdf.extend_from_slice(
        b"6 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica\n\
           /Encoding /WinAnsiEncoding >>\nendobj\n\n",
    );

    // Obj 7: OCG dictionary
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"7 0 obj\n<< /Type /OCG /Name /Background >>\nendobj\n\n");

    let xref_offset = pdf.len();
    let n_obj = offsets.len() + 1;
    let mut xref = format!("xref\n0 {}\n", n_obj);
    xref.push_str("0000000000 65535 f \n");
    for off in &offsets {
        xref.push_str(&format!("{:010} 00000 n \n", off));
    }
    pdf.extend_from_slice(xref.as_bytes());
    let trailer = format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        n_obj, xref_offset
    );
    pdf.extend_from_slice(trailer.as_bytes());
    pdf
}

// ============================================================================
// Helper: render a PDF and get raw RGBA pixel data
// ============================================================================

fn render_raw(doc: &PdfDocument, excluded: HashSet<String>) -> (Vec<u8>, u32, u32) {
    let mut options = RenderOptions::with_dpi(72);
    options.format = ImageFormat::RawRgba8;
    options.excluded_layers = excluded;
    let img = pdf_oxide::rendering::render_page(doc, 0, &options).expect("render");
    (img.data, img.width, img.height)
}

/// Sample the average color in a rectangular pixel region.
/// Returns (r, g, b, a) as floats in 0..255.
fn sample_region(data: &[u8], width: u32, x: u32, y: u32, w: u32, h: u32) -> (f32, f32, f32, f32) {
    let mut r_sum = 0u64;
    let mut g_sum = 0u64;
    let mut b_sum = 0u64;
    let mut a_sum = 0u64;
    let mut count = 0u64;
    for py in y..(y + h).min(width) {
        for px in x..(x + w).min(width) {
            let idx = ((py * width + px) * 4) as usize;
            if idx + 3 < data.len() {
                // tiny-skia stores premultiplied RGBA; un-premultiply for comparison
                let a = data[idx + 3] as f32;
                if a > 0.0 {
                    let scale = 255.0 / a;
                    r_sum += (data[idx] as f32 * scale).round().min(255.0) as u64;
                    g_sum += (data[idx + 1] as f32 * scale).round().min(255.0) as u64;
                    b_sum += (data[idx + 2] as f32 * scale).round().min(255.0) as u64;
                } else {
                    r_sum += data[idx] as u64;
                    g_sum += data[idx + 1] as u64;
                    b_sum += data[idx + 2] as u64;
                }
                a_sum += data[idx + 3] as u64;
                count += 1;
            }
        }
    }
    if count == 0 {
        return (0.0, 0.0, 0.0, 0.0);
    }
    (
        r_sum as f32 / count as f32,
        g_sum as f32 / count as f32,
        b_sum as f32 / count as f32,
        a_sum as f32 / count as f32,
    )
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_render_ocg_rect_visible_without_filter() {
    let pdf_bytes = build_pdf_with_ocg_rect_and_text();
    let doc = PdfDocument::from_bytes(pdf_bytes).expect("parse PDF");

    let (data, width, _height) = render_raw(&doc, HashSet::new());

    // The red rectangle region (PDF coords 50,550 to 250,750 mapped to 72 DPI pixels)
    // At 72 DPI, 1pt = 1px. PDF y=550..750 → pixel y = 792-750..792-550 = 42..242
    let (r, g, b, a) = sample_region(&data, width, 100, 80, 50, 50);
    assert!(r > 200.0, "Red channel should be high in rect region, got {r}");
    assert!(g < 50.0, "Green should be low, got {g}");
    assert!(b < 50.0, "Blue should be low, got {b}");
    assert!(a > 200.0, "Alpha should be high, got {a}");
}

#[test]
fn test_render_ocg_rect_hidden_with_filter() {
    let pdf_bytes = build_pdf_with_ocg_rect_and_text();
    let doc = PdfDocument::from_bytes(pdf_bytes).expect("parse PDF");

    let excluded = HashSet::from(["Background".to_string()]);
    let (data, width, _height) = render_raw(&doc, excluded);

    // The red rectangle region should now be white (background)
    let (r, g, b, a) = sample_region(&data, width, 100, 80, 50, 50);
    assert!(
        r > 250.0 && g > 250.0 && b > 250.0,
        "Rect region should be white background when layer excluded, got ({r}, {g}, {b})"
    );
    assert!(a > 250.0, "Alpha should still be high (white bg), got {a}");
}

#[test]
fn test_render_unrelated_layer_filter_preserves_content() {
    let pdf_bytes = build_pdf_with_ocg_rect_and_text();
    let doc = PdfDocument::from_bytes(pdf_bytes).expect("parse PDF");

    // Exclude a layer that doesn't exist — should not affect rendering
    let excluded = HashSet::from(["NonExistentLayer".to_string()]);
    let (data_filtered, _width, _) = render_raw(&doc, excluded);
    let (data_unfiltered, _, _) = render_raw(&doc, HashSet::new());

    // Both renders should be identical
    assert_eq!(
        data_filtered, data_unfiltered,
        "Filtering a non-existent layer must not change the rendered output"
    );
}

#[test]
fn test_render_ocmd_rect_hidden_with_filter() {
    let pdf_bytes = build_pdf_with_ocmd_rect();
    let doc = PdfDocument::from_bytes(pdf_bytes).expect("parse PDF");

    // Without filter: blue rect at (50,550) should be visible
    let (data_unfiltered, width, _) = render_raw(&doc, HashSet::new());
    let (r, g, b, _a) = sample_region(&data_unfiltered, width, 100, 80, 50, 50);
    assert!(b > 200.0, "Blue rect should be visible without filter, got b={b}");
    assert!(r < 50.0 && g < 50.0, "Should be blue, got r={r} g={g}");

    // Green rect at (300,550) should be visible
    let (_r, g, _b, _a) = sample_region(&data_unfiltered, width, 350, 80, 50, 50);
    assert!(g > 200.0, "Green rect should be visible, got g={g}");

    // With filter: exclude "Watermark" → blue rect gone, green rect remains
    let excluded = HashSet::from(["Watermark".to_string()]);
    let (data_filtered, width, _) = render_raw(&doc, excluded);

    // Blue rect region should now be white
    let (r, g, b, _) = sample_region(&data_filtered, width, 100, 80, 50, 50);
    assert!(
        r > 250.0 && g > 250.0 && b > 250.0,
        "Blue rect should be hidden, got ({r}, {g}, {b})"
    );

    // Green rect should still be visible
    let (_r, g, _b, _) = sample_region(&data_filtered, width, 350, 80, 50, 50);
    assert!(g > 200.0, "Green rect should survive filter, got g={g}");
}

#[test]
fn test_render_ocg_in_form_xobject_filtered() {
    let pdf_bytes = build_pdf_with_ocg_in_form_xobject();
    let doc = PdfDocument::from_bytes(pdf_bytes).expect("parse PDF");

    // Without filter: red rect from Form XObject should be visible
    let (data_unfiltered, width, _) = render_raw(&doc, HashSet::new());
    let (r, _g, _b, _a) = sample_region(&data_unfiltered, width, 100, 80, 50, 50);
    assert!(r > 200.0, "Red rect from XObject should be visible, got r={r}");

    // With filter: exclude "Background" → red rect gone, green rect remains
    let excluded = HashSet::from(["Background".to_string()]);
    let (data_filtered, width, _) = render_raw(&doc, excluded);

    // Red rect region should now be white
    let (r, g, b, _) = sample_region(&data_filtered, width, 100, 80, 50, 50);
    assert!(
        r > 250.0 && g > 250.0 && b > 250.0,
        "Red rect from XObject should be hidden, got ({r}, {g}, {b})"
    );

    // Green rect at (300,550) should still be visible
    let (_r, g, _b, _) = sample_region(&data_filtered, width, 350, 80, 50, 50);
    assert!(g > 200.0, "Green rect should survive filter, got g={g}");
}

#[test]
fn test_render_empty_excluded_layers_matches_default() {
    let pdf_bytes = build_pdf_with_ocg_rect_and_text();
    let doc = PdfDocument::from_bytes(pdf_bytes).expect("parse PDF");

    let (data_default, _, _) = render_raw(&doc, HashSet::new());

    let mut options = RenderOptions::with_dpi(72);
    options.format = ImageFormat::RawRgba8;
    // excluded_layers defaults to empty
    let img = pdf_oxide::rendering::render_page(&doc, 0, &options).expect("render");

    assert_eq!(
        data_default, img.data,
        "Default excluded_layers (empty) should match explicit empty HashSet"
    );
}
