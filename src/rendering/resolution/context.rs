//! Read-only context the pipeline borrows for the duration of a single
//! resolution call.
//!
//! All the cross-cutting state the resolver stages need lives here:
//! - The document handle, for object resolution (tint-transform streams,
//!   ICC profiles, function dictionaries).
//! - The page's resolved colour-space dictionary, so `Spaced` logical
//!   colours can be evaluated against the spaces the resource map declared.
//! - The document `/OutputIntents` CMYK profile, when present, so the
//!   colour stage can convert `/DeviceCMYK` paint (and `/Separation` /
//!   `/DeviceN` alternates that land in `/DeviceCMYK`) through the
//!   press-target ICC profile instead of the §10.3.5 additive-clamp
//!   fallback. Precedence between embedded ICC, page-level `/DefaultCMYK`,
//!   the document `/OutputIntents` profile, and the additive-clamp
//!   fallback (ISO 32000-1:2008 §14.11.5 / §10) is enforced inside the
//!   resolver — this struct just carries the inputs.
//! - The active graphics-state rendering intent (§10.7.3 `/RI`) so every
//!   ICC conversion is dispatched to the matching qcms intent.
//! - Page-level `/DefaultGray` / `/DefaultRGB` / `/DefaultCMYK` colour-
//!   space overrides (§8.6.5.6) so paint operators using the bare
//!   device families are routed through the page's declared default
//!   before any document-level OutputIntent lookup.
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
    /// Document `/OutputIntents` CMYK profile, when present. Consumed by
    /// `ColorResolver` for `/DeviceCMYK` paint and for `/Separation` /
    /// `/DeviceN` resolved alternates that land in `/DeviceCMYK`.
    pub(crate) output_intent_cmyk: Option<&'a Arc<IccProfile>>,
    /// Active graphics-state rendering intent (§10.7.3). Defaults to
    /// `/RelativeColorimetric` when the page graphics state hasn't set
    /// `/RI` explicitly.
    pub(crate) rendering_intent: RenderingIntent,
    /// Page-level `/DefaultGray` override (§8.6.5.6), when present.
    pub(crate) default_gray: Option<&'a Object>,
    /// Page-level `/DefaultRGB` override (§8.6.5.6), when present.
    pub(crate) default_rgb: Option<&'a Object>,
    /// Page-level `/DefaultCMYK` override (§8.6.5.6), when present.
    pub(crate) default_cmyk: Option<&'a Object>,
}

impl<'a> ResolutionContext<'a> {
    /// Build a context from the page-resource snapshot the operator walker
    /// already maintains. The walker computes `color_spaces` from
    /// `resources["ColorSpace"]` once per page; we just borrow it.
    ///
    /// Callers chain `with_output_intent` / `with_rendering_intent` /
    /// `with_defaults` to populate the colour-policy fields. The bare
    /// constructor leaves them unset so unit tests that only probe the
    /// `Device*` paths don't need to thread fixture profiles through.
    pub(crate) fn new(doc: &'a PdfDocument, color_spaces: &'a HashMap<String, Object>) -> Self {
        Self {
            doc,
            color_spaces,
            output_intent_cmyk: None,
            rendering_intent: RenderingIntent::default(),
            default_gray: None,
            default_rgb: None,
            default_cmyk: None,
        }
    }

    /// Attach the document's `/OutputIntents` CMYK profile, when one is
    /// available. `None` is a no-op and leaves the additive-clamp
    /// fallback in place — the colour stage only consults the profile
    /// when it's `Some`.
    pub(crate) fn with_output_intent(
        mut self,
        profile: Option<&'a Arc<IccProfile>>,
    ) -> Self {
        self.output_intent_cmyk = profile;
        self
    }

    /// Set the active rendering intent (§10.7.3) the colour stage
    /// dispatches to qcms with. Defaults to `RelativeColorimetric` per
    /// the spec's "unrecognised → RelativeColorimetric" rule when the
    /// graphics state hasn't otherwise set it.
    pub(crate) fn with_rendering_intent(mut self, intent: RenderingIntent) -> Self {
        self.rendering_intent = intent;
        self
    }

    /// Set the page-level `/DefaultGray` / `/DefaultRGB` / `/DefaultCMYK`
    /// colour-space overrides (§8.6.5.6). Each `None` means the page
    /// didn't declare that override; the colour stage then resolves the
    /// bare device family normally.
    pub(crate) fn with_defaults(
        mut self,
        gray: Option<&'a Object>,
        rgb: Option<&'a Object>,
        cmyk: Option<&'a Object>,
    ) -> Self {
        self.default_gray = gray;
        self.default_rgb = rgb;
        self.default_cmyk = cmyk;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::fixture_doc;
    use super::*;

    #[test]
    fn context_carries_empty_color_spaces() {
        let doc = fixture_doc();
        let color_spaces = HashMap::new();
        let ctx = ResolutionContext::new(&doc, &color_spaces);
        assert!(ctx.color_spaces.is_empty());
        assert!(ctx.output_intent_cmyk.is_none());
        assert_eq!(ctx.rendering_intent, RenderingIntent::RelativeColorimetric);
        assert!(ctx.default_gray.is_none());
        assert!(ctx.default_rgb.is_none());
        assert!(ctx.default_cmyk.is_none());
    }

    #[test]
    fn context_borrows_color_space_map() {
        // The point of taking `&HashMap` is that the walker's page-scope
        // map is reused across intents; building a fresh context per
        // intent must be cheap (no clone).
        let doc = fixture_doc();
        let mut color_spaces = HashMap::new();
        color_spaces.insert("CS1".to_string(), Object::Name("DeviceCMYK".to_string()));

        let ctx = ResolutionContext::new(&doc, &color_spaces);
        assert!(ctx.color_spaces.contains_key("CS1"));
        // Re-build context — must still see the same entries through the
        // same borrow without any heap traffic.
        let ctx2 = ResolutionContext::new(&doc, &color_spaces);
        assert_eq!(ctx2.color_spaces.len(), 1);
    }

    #[test]
    fn context_carries_output_intent_when_set() {
        // Pin that the OutputIntent builder method actually attaches the
        // profile borrow to the context — the colour stage relies on
        // `ctx.output_intent_cmyk.is_some()` to decide whether to consult
        // the ICC path, so a no-op `with_output_intent` would silently
        // fall back to additive-clamp without anyone noticing.
        let doc = fixture_doc();
        let color_spaces = HashMap::new();
        let profile = Arc::new(
            IccProfile::parse(super::tests::header_only_cmyk_profile_bytes(), 4)
                .expect("header-only stub profile parses"),
        );
        let ctx = ResolutionContext::new(&doc, &color_spaces).with_output_intent(Some(&profile));
        assert!(ctx.output_intent_cmyk.is_some());
    }

    #[test]
    fn with_rendering_intent_overrides_default() {
        let doc = fixture_doc();
        let color_spaces = HashMap::new();
        let ctx = ResolutionContext::new(&doc, &color_spaces)
            .with_rendering_intent(RenderingIntent::AbsoluteColorimetric);
        assert_eq!(ctx.rendering_intent, RenderingIntent::AbsoluteColorimetric);
    }

    #[test]
    fn with_defaults_attaches_each_override_independently() {
        let doc = fixture_doc();
        let color_spaces = HashMap::new();
        let gray = Object::Name("DeviceGray".to_string());
        let cmyk = Object::Name("DeviceCMYK".to_string());
        let ctx = ResolutionContext::new(&doc, &color_spaces)
            .with_defaults(Some(&gray), None, Some(&cmyk));
        assert!(ctx.default_gray.is_some());
        assert!(ctx.default_rgb.is_none());
        assert!(ctx.default_cmyk.is_some());
    }

    /// Header-only CMYK stub — same shape as the existing
    /// `tests/test_icc_cmyk_conversion.rs` helper. qcms will reject it
    /// at transform-build time (no tag table), so it's only useful as a
    /// "profile-shaped" Arc for tests probing whether the context
    /// carries the borrow at all.
    pub(crate) fn header_only_cmyk_profile_bytes() -> Vec<u8> {
        let mut v = vec![0u8; 128];
        v[8..12].copy_from_slice(&0x04000000u32.to_be_bytes());
        v[12..16].copy_from_slice(b"prtr");
        v[16..20].copy_from_slice(b"CMYK");
        v[20..24].copy_from_slice(b"Lab ");
        v[36..40].copy_from_slice(b"acsp");
        v
    }
}
