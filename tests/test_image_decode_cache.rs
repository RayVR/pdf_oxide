//! Decoded-image reuse across pages and paints.
//!
//! Inflating + colour-converting an image XObject is the dominant per-page
//! cost in many-page jobs that reuse artwork (a RIP imposing the same label
//! across thousands of pages). The document caches the decoded base image
//! keyed by object reference (+ resolved colour-space fingerprint), so a
//! reference to an already-decoded image is a cache hit. These probes pin that
//! contract via the `image_decode_count` test-support counter: N references to
//! one image must decode it once.

#![cfg(all(feature = "rendering", feature = "test-support"))]

use pdf_oxide::document::PdfDocument;
use pdf_oxide::rendering::{render_page, RenderOptions};

/// Build a PDF whose `n_pages` pages each invoke the SAME image XObject
/// (object 7) `do_per_page` times. The image is a 2×2 DeviceRGB image, so each
/// decode runs the full extract + colour-convert path.
fn build_shared_image_pdf(n_pages: usize, do_per_page: usize) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    let mut offsets: Vec<(u32, usize)> = Vec::new(); // (obj num, byte offset)
    let push = |buf: &mut Vec<u8>, offsets: &mut Vec<(u32, usize)>, num: u32, body: &str| {
        offsets.push((num, buf.len()));
        buf.extend_from_slice(format!("{num} 0 obj\n{body}\nendobj\n").as_bytes());
    };

    buf.extend_from_slice(b"%PDF-1.6\n");

    // Page object numbers 3..3+n_pages; content streams 3+n_pages..; image is 7
    // only when n_pages small — keep numbering explicit and simple instead.
    // Layout: 1=Catalog, 2=Pages, 3..=pages, then contents, then image (last).
    let page_obj = |i: usize| 3 + i as u32;
    let content_obj = |i: usize| 3 + n_pages as u32 + i as u32;
    let image_obj = 3 + 2 * n_pages as u32;

    push(&mut buf, &mut offsets, 1, "<< /Type /Catalog /Pages 2 0 R >>");
    let kids: Vec<String> = (0..n_pages)
        .map(|i| format!("{} 0 R", page_obj(i)))
        .collect();
    push(
        &mut buf,
        &mut offsets,
        2,
        &format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), n_pages),
    );

    for i in 0..n_pages {
        let body = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Contents {} 0 R \
             /Resources << /XObject << /Im0 {} 0 R >> >> >>",
            content_obj(i),
            image_obj
        );
        push(&mut buf, &mut offsets, page_obj(i), &body);
    }

    // One Do per requested invocation, each in its own q/Q with a 50×50 placement.
    let mut content = String::new();
    for _ in 0..do_per_page {
        content.push_str("q 50 0 0 50 25 25 cm /Im0 Do Q\n");
    }
    for i in 0..n_pages {
        let body = format!("<< /Length {} >>\nstream\n{}\nendstream", content.len(), content);
        push(&mut buf, &mut offsets, content_obj(i), &body);
    }

    // Shared image: 2×2 DeviceRGB, 8 bpc (12 raw bytes).
    let pixels: [u8; 12] = [255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0];
    offsets.push((image_obj, buf.len()));
    buf.extend_from_slice(
        format!(
            "{image_obj} 0 obj\n<< /Type /XObject /Subtype /Image /Width 2 /Height 2 \
             /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length 12 >>\nstream\n"
        )
        .as_bytes(),
    );
    buf.extend_from_slice(&pixels);
    buf.extend_from_slice(b"\nendstream\nendobj\n");

    // xref + trailer. Objects are numbered 1..=image_obj contiguously.
    offsets.sort_by_key(|(num, _)| *num);
    let max_obj = image_obj;
    let xref_off = buf.len();
    buf.extend_from_slice(format!("xref\n0 {}\n", max_obj + 1).as_bytes());
    buf.extend_from_slice(b"0000000000 65535 f \n");
    for num in 1..=max_obj {
        let off = offsets
            .iter()
            .find(|(n, _)| *n == num)
            .map(|(_, o)| *o)
            .unwrap();
        buf.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    buf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            max_obj + 1,
            xref_off
        )
        .as_bytes(),
    );
    buf
}

/// Two pages that invoke the same image XObject must decode it once: page 2's
/// reference hits the document decode cache populated by page 1.
#[test]
fn same_image_across_pages_decodes_once() {
    let doc = PdfDocument::from_bytes(build_shared_image_pdf(2, 1)).expect("parse");
    let opts = RenderOptions::with_dpi(72);
    render_page(&doc, 0, &opts).expect("render page 0");
    render_page(&doc, 1, &opts).expect("render page 1");
    assert_eq!(
        doc.image_decode_count(),
        1,
        "two pages sharing one image XObject must decode it once (got {})",
        doc.image_decode_count()
    );
}

/// The same image invoked multiple times on a single page also decodes once.
#[test]
fn same_image_twice_on_one_page_decodes_once() {
    let doc = PdfDocument::from_bytes(build_shared_image_pdf(1, 3)).expect("parse");
    let opts = RenderOptions::with_dpi(72);
    render_page(&doc, 0, &opts).expect("render");
    assert_eq!(
        doc.image_decode_count(),
        1,
        "one image invoked three times on a page must decode once (got {})",
        doc.image_decode_count()
    );
}
