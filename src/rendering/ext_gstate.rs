//! Shared parser for PDF `ExtGState` dictionary entries.
//!
//! Both the page renderer and the separation-plate renderer need to apply
//! transparency / blend-mode overrides from `gs` operators. Keeping the
//! parser in a single module avoids drift between the two renderers and
//! removes the `pub(crate)` leak that previously crossed module boundaries.

use crate::content::graphics_state::{GraphicsState, SoftMaskForm, SoftMaskSubtype};
use crate::document::PdfDocument;
use crate::error::Result;
use crate::object::Object;

/// A parsed `/SMask` value from an ExtGState dict (§11.4.7 / Table 144).
/// `None` corresponds to the spec `/None` value (clear the current mask);
/// `Form` carries the Form XObject reference plus optional backdrop and
/// transfer function (see [`SoftMaskForm`]).
#[derive(Clone, Debug)]
pub(crate) enum SoftMaskValue {
    /// `/SMask /None` — clear the current mask.
    None,
    /// `/SMask <<` … Form-XObject soft mask.
    Form(SoftMaskForm),
}

/// Parsed effects of a PDF `ExtGState` dictionary. Only the fields actually
/// applied during rendering are captured (fill/stroke alpha, blend mode,
/// the overprint parameters from ISO 32000-1 §11.7.4, and §11.4.7
/// Form-XObject soft masks).
#[derive(Clone, Debug, Default)]
pub(crate) struct ParsedExtGState {
    pub(crate) fill_alpha: Option<f32>,
    pub(crate) stroke_alpha: Option<f32>,
    pub(crate) blend_mode: Option<String>,
    /// Overprint for stroking operations (ExtGState `/OP`, §11.7.4).
    pub(crate) stroke_overprint: Option<bool>,
    /// Overprint for non-stroking operations (ExtGState `/op`, §11.7.4).
    pub(crate) fill_overprint: Option<bool>,
    /// Overprint mode (ExtGState `/OPM`, §11.7.4). 0 = standard, 1 = nonzero.
    pub(crate) overprint_mode: Option<u8>,
    /// Soft mask dispatch (§11.4.7). `None` means the entry was absent —
    /// gs.smask is left untouched. `Some(SoftMaskValue::None)` is the
    /// spec `/None` value (clear). `Some(SoftMaskValue::Form(..))` is a
    /// Form-XObject mask.
    pub(crate) smask: Option<SoftMaskValue>,
}

impl ParsedExtGState {
    /// Apply this dictionary's fields to `gs`. Fields that were not present
    /// in the source dictionary are left untouched on `gs`.
    pub(crate) fn apply(&self, gs: &mut GraphicsState) {
        if let Some(a) = self.fill_alpha {
            gs.fill_alpha = a;
        }
        if let Some(a) = self.stroke_alpha {
            gs.stroke_alpha = a;
        }
        if let Some(ref m) = self.blend_mode {
            gs.blend_mode = m.clone();
        }
        if let Some(v) = self.fill_overprint {
            gs.fill_overprint = v;
        }
        if let Some(v) = self.stroke_overprint {
            gs.stroke_overprint = v;
        }
        if let Some(v) = self.overprint_mode {
            gs.overprint_mode = v;
        }
        if let Some(ref sm) = self.smask {
            match sm {
                SoftMaskValue::None => {
                    gs.smask = None;
                },
                SoftMaskValue::Form(f) => {
                    gs.smask = Some(f.clone());
                },
            }
        }
    }
}

/// Parse the fields we need from an `ExtGState` *entry* (the inner dict, not
/// the resource dict that holds it). Resolves `state_obj` once if it is a
/// reference.
pub(crate) fn parse_ext_g_state_inner(
    state_obj: &Object,
    doc: &PdfDocument,
) -> Result<ParsedExtGState> {
    let mut out = ParsedExtGState::default();
    let state_resolved = doc.resolve_object(state_obj)?;
    let state_dict = match state_resolved.as_dict() {
        Some(d) => d,
        None => return Ok(out),
    };

    if let Some(ca) = state_dict.get("ca") {
        out.fill_alpha = ca
            .as_real()
            .map(|v| v as f32)
            .or_else(|| ca.as_integer().map(|v| v as f32));
    }
    if let Some(ca_upper) = state_dict.get("CA") {
        out.stroke_alpha = ca_upper
            .as_real()
            .map(|v| v as f32)
            .or_else(|| ca_upper.as_integer().map(|v| v as f32));
    }
    if let Some(bm) = state_dict.get("BM") {
        let mode = match bm {
            Object::Name(n) => n.clone(),
            Object::Array(arr) => arr
                .first()
                .and_then(|o| o.as_name())
                .unwrap_or("Normal")
                .to_string(),
            _ => "Normal".to_string(),
        };
        out.blend_mode = Some(mode);
    }

    // ISO 32000-1 §11.7.4 / Table 128. `/OP` is the stroking overprint;
    // `/op` (lowercase) is the non-stroking overprint. When `/OP` is
    // present without `/op`, the spec says it sets both.
    let op_stroke = state_dict.get("OP").and_then(Object::as_bool);
    let op_fill = state_dict.get("op").and_then(Object::as_bool);
    out.stroke_overprint = op_stroke;
    out.fill_overprint = op_fill.or(op_stroke);

    if let Some(opm) = state_dict.get("OPM").and_then(Object::as_integer) {
        // Spec defines only 0 (standard) and 1 (nonzero). Any other
        // value is undefined; clamp to 0 so a malformed PDF doesn't
        // accidentally enable nonzero-overprint mode.
        out.overprint_mode = Some(if opm == 1 { 1 } else { 0 });
    }

    // ISO 32000-1:2008 §11.4.7 / Table 144. `/SMask` is either the
    // name `/None` (clear the current soft mask) or a soft-mask
    // dictionary referencing a Form XObject. Image-attached soft
    // masks (via an image XObject's own /SMask entry) are handled
    // at the image-blit site; this parser covers the ExtGState
    // path.
    if let Some(smask_obj) = state_dict.get("SMask") {
        // Resolve through references before classifying.
        let resolved = doc.resolve_object(smask_obj).unwrap_or(smask_obj.clone());
        match &resolved {
            Object::Name(n) if n == "None" => {
                out.smask = Some(SoftMaskValue::None);
            },
            Object::Dictionary(mask_dict) => {
                // Subtype: /S /Alpha or /S /Luminosity (default Alpha
                // per spec). Anything else falls through to None — a
                // malformed mask must not silently mis-render.
                let subtype = match mask_dict.get("S").and_then(Object::as_name) {
                    Some("Alpha") => SoftMaskSubtype::Alpha,
                    Some("Luminosity") => SoftMaskSubtype::Luminosity,
                    _ => SoftMaskSubtype::Alpha,
                };

                // /G — required Form XObject reference.
                let form_ref = mask_dict
                    .get("G")
                    .and_then(|o| match o {
                        Object::Reference(r) => Some(*r),
                        _ => None,
                    });

                if let Some(form_ref) = form_ref {
                    // /BC backdrop colour — array of N reals. Only
                    // honoured for /S /Luminosity per §11.4.7; for
                    // /S /Alpha the spec ignores /BC.
                    let backdrop = if subtype == SoftMaskSubtype::Luminosity {
                        mask_dict.get("BC").and_then(|o| o.as_array()).map(|arr| {
                            arr.iter()
                                .filter_map(|v| {
                                    v.as_real()
                                        .map(|r| r as f32)
                                        .or_else(|| v.as_integer().map(|i| i as f32))
                                })
                                .collect::<Vec<f32>>()
                        })
                    } else {
                        None
                    };

                    // /TR transfer function — stored raw; the
                    // renderer evaluates per-pixel via the Function
                    // evaluator already used for tint transforms.
                    let transfer = mask_dict.get("TR").cloned();

                    out.smask = Some(SoftMaskValue::Form(SoftMaskForm {
                        form_ref,
                        subtype,
                        backdrop,
                        transfer,
                    }));
                }
            },
            _ => {},
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Minimal PDF document used purely as a `&PdfDocument` argument for
    /// `parse_ext_g_state_inner`. The parser only calls `resolve_object`
    /// on the input; when the input is already an inline dict (not a
    /// `Reference`), that call short-circuits to a clone and never touches
    /// the document's xref. So any successfully-parsed PDF is sufficient.
    fn fixture_doc() -> PdfDocument {
        // Construct the smallest valid PDF that `from_bytes` will accept.
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

    fn dict(entries: &[(&str, Object)]) -> Object {
        let mut m = HashMap::new();
        for (k, v) in entries {
            m.insert((*k).to_string(), v.clone());
        }
        Object::Dictionary(m)
    }

    #[test]
    fn parses_op_op_opm_from_extgstate_dict() {
        let obj = dict(&[
            ("OP", Object::Boolean(true)),
            ("op", Object::Boolean(false)),
            ("OPM", Object::Integer(1)),
        ]);
        let doc = fixture_doc();
        let parsed = parse_ext_g_state_inner(&obj, &doc).expect("parses");
        assert_eq!(parsed.stroke_overprint, Some(true));
        assert_eq!(parsed.fill_overprint, Some(false));
        assert_eq!(parsed.overprint_mode, Some(1));
    }

    #[test]
    fn op_without_op_sets_both_overprints() {
        // §11.7.4 / Table 128: "Specifying an OP entry sets both
        // parameters unless there is also an op entry in the same
        // graphics state parameter dictionary".
        let obj = dict(&[("OP", Object::Boolean(true))]);
        let doc = fixture_doc();
        let parsed = parse_ext_g_state_inner(&obj, &doc).expect("parses");
        assert_eq!(parsed.stroke_overprint, Some(true));
        assert_eq!(parsed.fill_overprint, Some(true));
    }

    #[test]
    fn op_without_op_uppercase_only_does_not_affect_stroke() {
        // /op is the non-stroking parameter only; /OP is absent so the
        // stroking overprint stays unset (caller falls back to gs default).
        let obj = dict(&[("op", Object::Boolean(true))]);
        let doc = fixture_doc();
        let parsed = parse_ext_g_state_inner(&obj, &doc).expect("parses");
        assert_eq!(parsed.stroke_overprint, None);
        assert_eq!(parsed.fill_overprint, Some(true));
    }

    #[test]
    fn opm_clamps_unknown_values_to_zero() {
        // §11.7.4: OPM is 0 or 1; any other value is undefined. We clamp
        // to 0 (standard mode) to preserve the spec-default behavior on
        // malformed PDFs.
        let obj = dict(&[("OPM", Object::Integer(42))]);
        let doc = fixture_doc();
        let parsed = parse_ext_g_state_inner(&obj, &doc).expect("parses");
        assert_eq!(parsed.overprint_mode, Some(0));
    }

    #[test]
    fn missing_overprint_keys_leave_options_none() {
        // Empty dict → no fields touched. Apply() is a no-op on the gs.
        let obj = dict(&[]);
        let doc = fixture_doc();
        let parsed = parse_ext_g_state_inner(&obj, &doc).expect("parses");
        assert_eq!(parsed.stroke_overprint, None);
        assert_eq!(parsed.fill_overprint, None);
        assert_eq!(parsed.overprint_mode, None);
    }
}
