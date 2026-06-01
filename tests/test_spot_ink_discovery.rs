//! Spec-driven tests for deep spot-ink discovery through nested Form
//! XObject resources (ISO 32000-1 §8.6.6.2 Separation, §8.6.6.3 DeviceN,
//! §8.10 Form XObjects).
//!
//! Fixture builders mirror the pattern in `tests/test_separation_overprint.rs`:
//! hand-rolled PDF byte buffers with explicit object numbers and explicit xref.

#![cfg(feature = "rendering")]

use pdf_oxide::document::PdfDocument;

/// Build a single-page PDF whose page-level /Resources/ColorSpace is empty
/// and whose content stream invokes one Form XObject. The Form XObject's
/// /Resources/ColorSpace declares a /Separation /SpotRed space.
fn build_pdf_with_spot_in_nested_form() -> Vec<u8> {
    let page_content = b"/Fm0 Do\n";
    let form_content = b"";

    let mut buf = Vec::new();
    let mut offsets = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n");

    offsets.push(buf.len());
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] \
           /Contents 4 0 R \
           /Resources << /XObject << /Fm0 5 0 R >> >> >>\nendobj\n",
    );
    offsets.push(buf.len());
    let hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", page_content.len());
    buf.extend_from_slice(hdr.as_bytes());
    buf.extend_from_slice(page_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    offsets.push(buf.len());
    let form_hdr = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] \
            /Resources << /ColorSpace << /CS1 6 0 R >> >> \
            /Length {} >>\nstream\n",
        form_content.len()
    );
    buf.extend_from_slice(form_hdr.as_bytes());
    buf.extend_from_slice(form_content);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(b"6 0 obj\n[/Separation /SpotRed /DeviceCMYK 7 0 R]\nendobj\n");
    offsets.push(buf.len());
    buf.extend_from_slice(
        b"7 0 obj\n<< /FunctionType 2 /Domain [0 1] /N 1 /C0 [0 0 0 0] /C1 [0 1 0 0] >>\nendobj\n",
    );

    finalize_pdf(buf, offsets)
}

fn finalize_pdf(mut buf: Vec<u8>, offsets: Vec<usize>) -> Vec<u8> {
    let xref_offset = buf.len();
    buf.extend_from_slice(b"xref\n");
    buf.extend_from_slice(format!("0 {}\n", offsets.len() + 1).as_bytes());
    buf.extend_from_slice(b"0000000000 65535 f \n");
    for off in &offsets {
        buf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    buf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            offsets.len() + 1,
            xref_offset
        )
        .as_bytes(),
    );
    buf
}

#[test]
fn deep_finds_spot_declared_in_nested_form() {
    let doc = PdfDocument::from_bytes(build_pdf_with_spot_in_nested_form()).expect("parse");

    // Shallow API: misses the nested declaration (documented contract).
    let shallow = doc.get_page_inks(0).expect("shallow");
    assert!(
        !shallow.contains(&"SpotRed".to_string()),
        "shallow get_page_inks must NOT find XObject-local inks; got {:?}",
        shallow
    );

    // Deep API: finds it.
    let deep = doc.get_page_inks_deep(0).expect("deep");
    assert!(
        deep.contains(&"SpotRed".to_string()),
        "deep walk must surface SpotRed declared in nested form; got {:?}",
        deep
    );
}
