//! Integration tests for the enumerate-then-materialize image API.
//!
//! Verifies that `page_image_handles` returns correct metadata without
//! decompressing streams, and that `decode()` / `raw_compressed_bytes()`
//! materialise the image on demand.

use pdf_oxide::elements::{ContentElement, ImageContent, ImageFormat};
use pdf_oxide::extractors::images::PdfFilter;
use pdf_oxide::geometry::Rect;
use pdf_oxide::writer::{PdfWriter, PdfWriterConfig};
use pdf_oxide::PdfDocument;

// Minimal valid 1×1 white JPEG (SOI + APP0 + DQT + SOF0 + DHT + SOS + EOI)
const MINIMAL_JPEG: &[u8] = &[
    0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00,
    0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43, 0x00, 0x08, 0x06, 0x06, 0x07, 0x06,
    0x05, 0x08, 0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0A, 0x0C, 0x14, 0x0D, 0x0C, 0x0B, 0x0B,
    0x0C, 0x19, 0x12, 0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D, 0x1A, 0x1C, 0x1C, 0x20,
    0x24, 0x2E, 0x27, 0x20, 0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29, 0x2C, 0x30, 0x31,
    0x34, 0x34, 0x34, 0x1F, 0x27, 0x39, 0x3D, 0x38, 0x32, 0x3C, 0x2E, 0x33, 0x34, 0x32, 0xFF,
    0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, 0x00,
    0x1F, 0x00, 0x00, 0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
    0xFF, 0xD9,
];

/// Build a minimal PDF containing a single JPEG image on page 0.
fn build_pdf_with_jpeg(width: u32, height: u32) -> Vec<u8> {
    let mut writer = PdfWriter::with_config(PdfWriterConfig::default());

    let bbox = Rect::new(0.0, 0.0, width as f32, height as f32);
    let image_content =
        ImageContent::new(bbox, ImageFormat::Jpeg, MINIMAL_JPEG.to_vec(), width, height);

    let mut page = writer.add_a4_page();
    page.add_element(&ContentElement::Image(image_content));
    page.finish();

    writer.finish().expect("PDF write failed")
}

#[test]
fn page_image_handles_returns_one_handle_for_single_jpeg() {
    let pdf_bytes = build_pdf_with_jpeg(100, 80);
    let doc = PdfDocument::from_bytes(pdf_bytes).expect("open PDF");

    let handles = doc.page_image_handles(0).expect("page_image_handles");

    assert_eq!(handles.len(), 1, "expected exactly one image handle");
    let h = &handles[0];
    // MINIMAL_JPEG is intrinsically 1×1 (SOF0 marker); display size is separate
    assert_eq!(h.width, 1);
    assert_eq!(h.height, 1);
    assert!(!h.is_inline);
    assert_eq!(h.paint_order, 0);
}

#[test]
fn page_image_handles_jpeg_has_dct_filter() {
    let pdf_bytes = build_pdf_with_jpeg(50, 50);
    let doc = PdfDocument::from_bytes(pdf_bytes).expect("open PDF");

    let handles = doc.page_image_handles(0).expect("page_image_handles");
    assert!(!handles.is_empty());

    let h = &handles[0];
    assert!(
        h.filter_chain.contains(&PdfFilter::DCTDecode),
        "JPEG XObject must report DCTDecode in filter_chain, got {:?}",
        h.filter_chain
    );
}

#[test]
fn page_image_handles_decode_produces_valid_image() {
    let pdf_bytes = build_pdf_with_jpeg(1, 1);
    let doc = PdfDocument::from_bytes(pdf_bytes).expect("open PDF");

    let handles = doc.page_image_handles(0).expect("page_image_handles");
    assert_eq!(handles.len(), 1);

    let handle = handles.into_iter().next().unwrap();
    let image = handle.decode().expect("decode");

    assert_eq!(image.width(), 1);
    assert_eq!(image.height(), 1);
}

#[test]
fn page_image_handles_raw_compressed_bytes_non_empty() {
    let pdf_bytes = build_pdf_with_jpeg(1, 1);
    let doc = PdfDocument::from_bytes(pdf_bytes).expect("open PDF");

    let handles = doc.page_image_handles(0).expect("page_image_handles");
    let handle = handles.into_iter().next().expect("handle");

    let raw = handle.raw_compressed_bytes().expect("raw bytes");
    assert!(!raw.is_empty(), "raw compressed bytes must be non-empty");
}

#[test]
fn page_image_handles_filter_then_decode_skips_small_images() {
    // MINIMAL_JPEG is intrinsically 1×1; filter for >= 100×100 should skip it
    let pdf_bytes = build_pdf_with_jpeg(200, 200);
    let doc = PdfDocument::from_bytes(pdf_bytes).expect("open PDF");

    let handles = doc.page_image_handles(0).expect("page_image_handles");

    let decoded: Vec<_> = handles
        .into_iter()
        .filter(|h| h.width >= 100 && h.height >= 100)
        .map(|h| h.decode())
        .collect::<Result<_, _>>()
        .expect("decode");

    // The 1×1 JPEG is smaller than the 100×100 threshold — zero decoded
    assert_eq!(decoded.len(), 0);
}

#[test]
fn page_image_handles_empty_page_returns_empty_vec() {
    let mut writer = PdfWriter::with_config(PdfWriterConfig::default());
    writer.add_a4_page().finish();
    let pdf_bytes = writer.finish().expect("PDF write");

    let doc = PdfDocument::from_bytes(pdf_bytes).expect("open PDF");
    let handles = doc.page_image_handles(0).expect("page_image_handles");

    assert!(handles.is_empty(), "empty page must yield zero handles");
}

#[test]
fn pdf_filter_from_name_roundtrip() {
    assert_eq!(PdfFilter::from_name("DCTDecode"), PdfFilter::DCTDecode);
    assert_eq!(PdfFilter::from_name("DCT"), PdfFilter::DCTDecode);
    assert_eq!(PdfFilter::from_name("FlateDecode"), PdfFilter::FlateDecode);
    assert_eq!(PdfFilter::from_name("Fl"), PdfFilter::FlateDecode);
    assert_eq!(PdfFilter::from_name("JPXDecode"), PdfFilter::JPXDecode);
    assert_eq!(PdfFilter::from_name("LZWDecode"), PdfFilter::LZWDecode);
    assert_eq!(PdfFilter::from_name("CCITTFaxDecode"), PdfFilter::CCITTFaxDecode);
    assert_eq!(
        PdfFilter::from_name("UnknownFilter"),
        PdfFilter::Other("UnknownFilter".to_string())
    );
}
