//! Shared Optional Content (OCG / OCMD) helpers.
//!
//! Both the text-extraction path (`extractors::text`) and the rendering
//! pipeline (`rendering::page_renderer`) need to decide whether a BDC scope
//! tagged `/OC` belongs to an excluded layer. This module owns the logic so
//! the two callers cannot drift apart (a previous duplication caused a real
//! correctness bug where the renderer failed to decode UTF-16LE / PDFDocEncoding
//! layer names that the extractor handled correctly).
//!
//! References:
//!  - ISO 32000-1:2008 §8.11.2 — Optional Content
//!  - ISO 32000-1:2008 §8.11.2.2 — Optional Content Membership Dictionaries
//!  - ISO 32000-1:2008 §7.9.2 — Text string encoding (UTF-16BE/LE/PDFDocEncoding)

use std::collections::{HashMap, HashSet};

use crate::document::PdfDocument;
use crate::object::Object;

/// OCMD visibility policy (`/P` entry). Per ISO 32000-1 §8.11.2.2 Table 102.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcmdPolicy {
    /// Visible if **any** referenced OCG is on. (Default per spec.)
    AnyOn,
    /// Visible only if **all** referenced OCGs are on.
    AllOn,
    /// Visible if **any** referenced OCG is off.
    AnyOff,
    /// Visible only if **all** referenced OCGs are off.
    AllOff,
}

impl OcmdPolicy {
    fn from_name(s: &str) -> Self {
        match s {
            "AllOn" => OcmdPolicy::AllOn,
            "AnyOff" => OcmdPolicy::AnyOff,
            "AllOff" => OcmdPolicy::AllOff,
            // "AnyOn" or anything unknown -> spec default
            _ => OcmdPolicy::AnyOn,
        }
    }
}

/// Decode a PDF text string per ISO 32000-1 §7.9.2.
///
/// Handles:
///  - UTF-16BE with `FE FF` BOM
///  - UTF-16LE with `FF FE` BOM
///  - UTF-8 (lenient — non-spec PDFs sometimes embed raw UTF-8)
///  - PDFDocEncoding fallback (the spec default for non-BOM strings)
pub fn decode_pdf_text_string(bytes: &[u8]) -> String {
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        let utf16_pairs: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect();
        String::from_utf16(&utf16_pairs)
            .unwrap_or_else(|_| String::from_utf8_lossy(bytes).to_string())
    } else if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        let utf16_pairs: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        String::from_utf16(&utf16_pairs)
            .unwrap_or_else(|_| String::from_utf8_lossy(bytes).to_string())
    } else {
        String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| {
            bytes
                .iter()
                .filter_map(|&b| crate::fonts::font_dict::pdfdoc_encoding_lookup(b))
                .collect()
        })
    }
}

/// Check if a `Name` value (which can be either a `/Name` token or a PDF string)
/// matches any entry in `excluded`.
pub fn ocg_name_is_excluded(name_obj: &Object, excluded: &HashSet<String>) -> bool {
    if let Some(name_str) = name_obj.as_name() {
        return excluded.contains(name_str);
    }
    if let Some(name_bytes) = name_obj.as_string() {
        let name_str = decode_pdf_text_string(name_bytes);
        return excluded.contains(&name_str);
    }
    false
}

/// Resolve a BDC `properties` operand into a property dictionary.
///
/// `properties` is either an inline dictionary (e.g. `BDC /OC << /Name /Foo >>`)
/// or a name (e.g. `BDC /OC /MC0`) which references an entry in the page's
/// `/Resources /Properties` dictionary.
///
/// Handles indirect references at every level (the resources dict, the
/// Properties sub-dict, and the property entry itself can all be `Reference`s).
/// `doc` may be `None` — in that case only the inline-dict fast path resolves;
/// the name-reference path requires indirect-object resolution.
pub fn resolve_bdc_properties(
    properties: &Object,
    resources: Option<&Object>,
    doc: Option<&PdfDocument>,
) -> Option<HashMap<String, Object>> {
    if let Some(dict) = properties.as_dict() {
        return Some(dict.clone());
    }

    let prop_name = properties.as_name()?;
    let resources = resources?;
    let doc = doc?;
    let res_dict = if let Some(res_ref) = resources.as_reference() {
        doc.load_object(res_ref).ok()?
    } else {
        resources.clone()
    };
    let res_dict = res_dict.as_dict()?;
    let properties_dict_obj = res_dict.get("Properties")?;
    let properties_dict = if let Some(r) = properties_dict_obj.as_reference() {
        doc.load_object(r).ok()?
    } else {
        properties_dict_obj.clone()
    };
    let properties_dict = properties_dict.as_dict()?;
    let prop_obj = properties_dict.get(prop_name)?;
    let resolved = if let Some(r) = prop_obj.as_reference() {
        doc.load_object(r).ok()?
    } else {
        prop_obj.clone()
    };
    resolved.as_dict().cloned()
}

/// Collect the `/Name` strings of every OCG referenced by an OCMD `/OCGs` entry.
///
/// `/OCGs` may be either a single OCG dictionary (or reference) or an array of
/// them. Each entry that resolves to a dictionary with a `/Name` contributes
/// one name. References that fail to resolve are silently skipped.
fn collect_ocmd_ocg_names(ocgs_obj: &Object, doc: &PdfDocument) -> Vec<Object> {
    let refs: Vec<&Object> = if let Some(arr) = ocgs_obj.as_array() {
        arr.iter().collect()
    } else {
        vec![ocgs_obj]
    };

    let mut names = Vec::with_capacity(refs.len());
    for obj in refs {
        let resolved = if let Some(r) = obj.as_reference() {
            match doc.load_object(r) {
                Ok(o) => o,
                Err(_) => continue,
            }
        } else {
            obj.clone()
        };
        if let Some(d) = resolved.as_dict() {
            if let Some(name_obj) = d.get("Name") {
                names.push(name_obj.clone());
            }
        }
    }
    names
}

/// Decide whether a resolved BDC properties dict represents an excluded
/// optional-content scope (OCG or OCMD).
///
/// Semantics:
///  - **OCG** (dict has `/Name`): excluded iff the name is in `excluded`.
///  - **OCMD** (dict has `/Type /OCMD` and `/OCGs`): excluded iff the OCMD
///    visibility policy applied to the membership states (`on = !excluded`)
///    evaluates to "hidden". `/P` defaults to `AnyOn` per spec. `/VE` is
///    not yet supported — see [`OcmdPolicy`].
pub fn check_ocg_excluded(
    props_dict: &HashMap<String, Object>,
    doc: &PdfDocument,
    excluded: &HashSet<String>,
) -> bool {
    if let Some(ocg_name) = props_dict.get("Name") {
        return ocg_name_is_excluded(ocg_name, excluded);
    }

    if let Some(Object::Name(t)) = props_dict.get("Type") {
        if t == "OCMD" {
            // /VE — visibility expression. Not implemented; if present, fall
            // through to /P / /OCGs evaluation. A future implementation would
            // walk the operator tree and short-circuit return here.
            //
            // /P — visibility policy. Defaults to /AnyOn.
            let policy = props_dict
                .get("P")
                .and_then(|o| o.as_name())
                .map(OcmdPolicy::from_name)
                .unwrap_or(OcmdPolicy::AnyOn);

            let names = match props_dict.get("OCGs") {
                Some(o) => collect_ocmd_ocg_names(o, doc),
                None => return false,
            };

            return ocmd_is_hidden(&names, policy, excluded);
        }
    }

    false
}

/// Resolve BDC properties and decide if the resulting scope is excluded.
///
/// Convenience wrapper combining [`resolve_bdc_properties`] and
/// [`check_ocg_excluded`] — the typical call site (BDC operator handler) just
/// needs the boolean.
pub fn resolve_and_check_ocg_excluded(
    properties: &Object,
    resources: Option<&Object>,
    doc: Option<&PdfDocument>,
    excluded: &HashSet<String>,
) -> bool {
    let props_dict = match resolve_bdc_properties(properties, resources, doc) {
        Some(d) => d,
        None => return false,
    };
    // OCMD evaluation needs the document to resolve referenced OCG /Name fields,
    // but the inline-OCG case (Name in the props dict) does not. If we have no
    // doc, only the OCG-Name short-circuit inside check_ocg_excluded can fire.
    match doc {
        Some(d) => check_ocg_excluded(&props_dict, d, excluded),
        None => {
            // Without a doc, only direct OCG checks (Name in props dict) are
            // possible — the OCMD path needs to resolve /OCGs refs.
            if let Some(ocg_name) = props_dict.get("Name") {
                return ocg_name_is_excluded(ocg_name, excluded);
            }
            false
        },
    }
}

/// Resolve an annotation `/OC` entry (an OCG or OCMD dict, possibly indirect)
/// and decide whether the annotation belongs to an excluded layer.
///
/// Per ISO 32000-1 §12.5.2, annotation dictionaries can carry an `/OC` entry
/// that references the OCG / OCMD the annotation belongs to. If that scope is
/// excluded, the annotation should not be rendered.
pub fn annotation_is_excluded(
    oc_obj: &Object,
    doc: &PdfDocument,
    excluded: &HashSet<String>,
) -> bool {
    if excluded.is_empty() {
        return false;
    }
    let resolved = if let Some(r) = oc_obj.as_reference() {
        match doc.load_object(r) {
            Ok(o) => o,
            Err(_) => return false,
        }
    } else {
        oc_obj.clone()
    };
    let dict = match resolved.as_dict() {
        Some(d) => d,
        None => return false,
    };
    check_ocg_excluded(dict, doc, excluded)
}

/// Apply an OCMD policy to a list of referenced OCG names.
///
/// Returns `true` if the content should be **hidden** (i.e. the scope evaluates
/// to "not visible"). With no referenced OCGs the result is `false` (spec says
/// such an OCMD is always visible, which mirrors the AnyOn-with-empty case).
///
/// Semantics (membership state: `on = !excluded`):
///  - `AnyOn`  — visible iff any referenced OCG is on → hide iff all are off.
///  - `AllOn`  — visible iff all referenced OCGs are on → hide iff any is off.
///  - `AnyOff` — visible iff any referenced OCG is off → hide iff all are on.
///  - `AllOff` — visible iff all referenced OCGs are off → hide iff any is on.
fn ocmd_is_hidden(ocg_names: &[Object], policy: OcmdPolicy, excluded: &HashSet<String>) -> bool {
    if ocg_names.is_empty() {
        return false;
    }

    let mut any_on = false;
    let mut any_off = false;
    let mut all_on = true;
    let mut all_off = true;

    for name in ocg_names {
        let is_off = ocg_name_is_excluded(name, excluded);
        if is_off {
            any_off = true;
            all_on = false;
        } else {
            any_on = true;
            all_off = false;
        }
    }

    match policy {
        OcmdPolicy::AnyOn => !any_on,   // hide when none are on
        OcmdPolicy::AllOn => !all_on,   // hide when any is off
        OcmdPolicy::AnyOff => !any_off, // hide when none are off
        OcmdPolicy::AllOff => !all_off, // hide when any is on
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_utf16be_bom() {
        // FE FF "Layer"
        let bytes = b"\xFE\xFF\x00L\x00a\x00y\x00e\x00r";
        assert_eq!(decode_pdf_text_string(bytes), "Layer");
    }

    #[test]
    fn decode_utf16le_bom() {
        // FF FE "Layer"
        let bytes = b"\xFF\xFEL\x00a\x00y\x00e\x00r\x00";
        assert_eq!(decode_pdf_text_string(bytes), "Layer");
    }

    #[test]
    fn decode_utf8_ascii() {
        assert_eq!(decode_pdf_text_string(b"Hello"), "Hello");
    }

    #[test]
    fn decode_pdfdoc_fallback() {
        // 0x85 in PDFDocEncoding = U+2013 (endash)
        assert_eq!(decode_pdf_text_string(&[0x85]), "\u{2013}");
    }

    #[test]
    fn ocg_name_is_excluded_matches_name_token() {
        let mut excluded = HashSet::new();
        excluded.insert("Watermark".to_string());
        assert!(ocg_name_is_excluded(&Object::Name("Watermark".into()), &excluded));
        assert!(!ocg_name_is_excluded(&Object::Name("Other".into()), &excluded));
    }

    #[test]
    fn ocg_name_is_excluded_matches_utf16le_string() {
        let mut excluded = HashSet::new();
        excluded.insert("Layer".to_string());
        let bytes: Vec<u8> = b"\xFF\xFEL\x00a\x00y\x00e\x00r\x00".to_vec();
        assert!(ocg_name_is_excluded(&Object::String(bytes), &excluded));
    }

    #[test]
    fn policy_default_is_any_on() {
        assert_eq!(OcmdPolicy::from_name("nonsense"), OcmdPolicy::AnyOn);
        assert_eq!(OcmdPolicy::from_name("AnyOn"), OcmdPolicy::AnyOn);
        assert_eq!(OcmdPolicy::from_name("AllOn"), OcmdPolicy::AllOn);
        assert_eq!(OcmdPolicy::from_name("AnyOff"), OcmdPolicy::AnyOff);
        assert_eq!(OcmdPolicy::from_name("AllOff"), OcmdPolicy::AllOff);
    }

    fn names(slice: &[&str]) -> Vec<Object> {
        slice.iter().map(|s| Object::Name((*s).into())).collect()
    }

    #[test]
    fn policy_any_on_hides_when_all_excluded() {
        let mut excluded = HashSet::new();
        excluded.insert("A".to_string());
        excluded.insert("B".to_string());

        // both off -> hidden
        assert!(ocmd_is_hidden(&names(&["A", "B"]), OcmdPolicy::AnyOn, &excluded));
        // one on -> visible
        assert!(!ocmd_is_hidden(&names(&["A", "C"]), OcmdPolicy::AnyOn, &excluded));
    }

    #[test]
    fn policy_all_on_hides_when_any_excluded() {
        let mut excluded = HashSet::new();
        excluded.insert("A".to_string());

        // any off -> hidden
        assert!(ocmd_is_hidden(&names(&["A", "B"]), OcmdPolicy::AllOn, &excluded));
        // all on -> visible
        assert!(!ocmd_is_hidden(&names(&["C", "B"]), OcmdPolicy::AllOn, &excluded));
    }

    #[test]
    fn policy_any_off_hides_when_all_on() {
        let mut excluded = HashSet::new();
        excluded.insert("A".to_string());

        // some off -> visible
        assert!(!ocmd_is_hidden(&names(&["A", "B"]), OcmdPolicy::AnyOff, &excluded));
        // all on -> hidden
        assert!(ocmd_is_hidden(&names(&["B", "C"]), OcmdPolicy::AnyOff, &excluded));
    }

    #[test]
    fn policy_all_off_hides_when_any_on() {
        let mut excluded = HashSet::new();
        excluded.insert("A".to_string());
        excluded.insert("B".to_string());

        // any on -> hidden
        assert!(ocmd_is_hidden(&names(&["A", "C"]), OcmdPolicy::AllOff, &excluded));
        // all off -> visible
        assert!(!ocmd_is_hidden(&names(&["A", "B"]), OcmdPolicy::AllOff, &excluded));
    }

    #[test]
    fn empty_ocgs_is_not_hidden() {
        let excluded = HashSet::new();
        assert!(!ocmd_is_hidden(&[], OcmdPolicy::AnyOn, &excluded));
        assert!(!ocmd_is_hidden(&[], OcmdPolicy::AllOn, &excluded));
        assert!(!ocmd_is_hidden(&[], OcmdPolicy::AnyOff, &excluded));
        assert!(!ocmd_is_hidden(&[], OcmdPolicy::AllOff, &excluded));
    }
}
