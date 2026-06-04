//! Read-only context the pipeline borrows for the duration of a single
//! resolution call.
//!
//! All the cross-cutting state the resolver stages need lives here:
//! - The document handle, for object resolution (tint-transform streams,
//!   ICC profiles, function dictionaries).
//! - The page's resolved colour-space dictionary, so `Spaced` logical
//!   colours can be evaluated against the spaces the resource map declared.
//! - The catalog's default-CMYK output intent (ISO 32000-1 §14.11.5), which
//!   `ColorResolver` consults as the fallback profile for plain DeviceCMYK
//!   when no source-specific ICC profile is available.
//! - The active rendering intent ([`crate::color::RenderingIntent`]), which
//!   the colour resolver uses when materialising ICC transforms.
//!
//! The context is a struct of borrows so that the operator walker can build
//! it once per page (or once per Form XObject scope) and hand it to every
//! `resolve` call without per-intent allocation.

use std::collections::HashMap;
use std::sync::Arc;

use crate::color::{IccProfile, RenderingIntent};
use crate::document::PdfDocument;
use crate::object::Object;

/// Per-page (or per-Form XObject) context for the resolution pipeline.
///
/// Lifetime `'a` ties the context to the operator walker's owned state.
pub(crate) struct ResolutionContext<'a> {
    pub(crate) doc: &'a PdfDocument,
    pub(crate) color_spaces: &'a HashMap<String, Object>,
    pub(crate) output_intent_cmyk: Option<&'a Arc<IccProfile>>,
    pub(crate) rendering_intent: RenderingIntent,
}

impl<'a> ResolutionContext<'a> {
    /// Build a context from the page-resource snapshot the operator walker
    /// already maintains. The walker computes `color_spaces` from
    /// `resources["ColorSpace"]` once per page; we just borrow it.
    pub(crate) fn new(
        doc: &'a PdfDocument,
        color_spaces: &'a HashMap<String, Object>,
        output_intent_cmyk: Option<&'a Arc<IccProfile>>,
        rendering_intent: RenderingIntent,
    ) -> Self {
        Self {
            doc,
            color_spaces,
            output_intent_cmyk,
            rendering_intent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal `PdfDocument` for context-construction tests. The pipeline
    /// doesn't dereference `doc` for any of the stages exercised in this
    /// module — only `ColorResolver` does, and only when it encounters a
    /// stream-backed function or ICC profile (which the tests in
    /// `color.rs` cover directly).
    fn fixture_doc() -> PdfDocument {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(b"%PDF-1.4\n");
        let cat_off = buf.len();
        buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
        let pages_off = buf.len();
        buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [] /Count 0 >>\nendobj\n");
        let xref_off = buf.len();
        buf.extend_from_slice(b"xref\n0 3\n0000000000 65535 f \n");
        buf.extend_from_slice(format!("{:010} 00000 n \n", cat_off).as_bytes());
        buf.extend_from_slice(format!("{:010} 00000 n \n", pages_off).as_bytes());
        buf.extend_from_slice(
            format!("trailer\n<< /Size 3 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref_off)
                .as_bytes(),
        );
        PdfDocument::from_bytes(buf).expect("fixture PDF parses")
    }

    #[test]
    fn context_carries_intent_default() {
        let doc = fixture_doc();
        let color_spaces = HashMap::new();
        let ctx = ResolutionContext::new(
            &doc,
            &color_spaces,
            None,
            RenderingIntent::RelativeColorimetric,
        );
        assert_eq!(ctx.rendering_intent, RenderingIntent::RelativeColorimetric);
        assert!(ctx.output_intent_cmyk.is_none());
        assert!(ctx.color_spaces.is_empty());
    }

    #[test]
    fn context_borrows_color_space_map() {
        // The point of taking `&HashMap` is that the walker's page-scope
        // map is reused across intents; building a fresh context per
        // intent must be cheap (no clone).
        let doc = fixture_doc();
        let mut color_spaces = HashMap::new();
        color_spaces.insert("CS1".to_string(), Object::Name("DeviceCMYK".to_string()));

        let ctx = ResolutionContext::new(&doc, &color_spaces, None, RenderingIntent::Perceptual);
        assert!(ctx.color_spaces.contains_key("CS1"));
        // Re-build context — must still see the same entries through the
        // same borrow without any heap traffic.
        let ctx2 = ResolutionContext::new(&doc, &color_spaces, None, RenderingIntent::Saturation);
        assert_eq!(ctx2.color_spaces.len(), 1);
        assert_eq!(ctx2.rendering_intent, RenderingIntent::Saturation);
    }
}
