//! Regression test for issue #395 — `doc.RenderPage(0, 0)` from C# threw
//! `SignatureException [8500]` on a real-world PDF. Reported by
//! @gevorgter on 2026-04-21.
//!
//! Two distinct bugs combined to produce that error:
//!
//! 1. The C# `ExceptionMapper` was off-by-one — FFI code 8 (Unsupported)
//!    was labelled `SignatureException`. Fixed in commit 327251c
//!    (shipped in v0.3.38). Tests for the mapping live at
//!    `csharp/PdfOxide.Tests/ExceptionMapperTests.cs`.
//!
//! 2. The underlying render call on the user's PDF *also* needed to
//!    succeed — no point fixing the error mapping if the render still
//!    failed. This test pins that part: rendering the user's exact
//!    fixture must complete without error.
//!
//! Fixture lives in the external `pdf_oxide_tests` corpus. Skip when
//! not present, matching `tests/test_multiline_obj_and_xref.rs`.

#[cfg(feature = "rendering")]
#[test]
fn issue_395_user_pdf_renders_without_error() {
    use pdf_oxide::document::PdfDocument;
    use pdf_oxide::rendering::{render_page, RenderOptions};

    let Ok(home) = std::env::var("HOME") else {
        return;
    };
    let path = std::path::PathBuf::from(home)
        .join("projects/pdf_oxide_tests/pdfs_issue_regression/issue_395_csharp_render.pdf");
    if !path.exists() {
        eprintln!("Skipping: {} not found", path.display());
        return;
    }

    let mut doc = PdfDocument::open(&path).expect("open #395 fixture");
    let n_pages = doc.page_count().expect("page count");
    assert!(n_pages > 0, "fixture should have at least one page");

    let opts = RenderOptions::with_dpi(150);
    let img = render_page(&mut doc, 0, &opts).expect(
        "rendering page 0 of #395 fixture must succeed — was emitting an FFI error code that \
         the C# binding mismapped to SignatureException [8500]",
    );
    assert!(!img.data.is_empty(), "rendered image must have bytes");
    assert!(img.width > 0 && img.height > 0);
}
