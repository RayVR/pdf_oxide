use crate::types::Rect;
use napi_derive::napi;

/// Form field types (AcroForm and XFA)
#[napi]
#[derive(Clone, Debug)]
pub enum FormFieldType {
    /// Single-line text input
    Text,
    /// Multi-line text area
    Paragraph,
    /// Boolean checkbox
    Checkbox,
    /// Radio button selection
    Radio,
    /// Dropdown list
    List,
    /// Combobox (editable dropdown)
    Combo,
    /// Push button
    Button,
    /// Digital signature field
    Signature,
}

/// Form field (AcroForm) - Section 12.7
#[napi]
#[derive(Clone, Debug)]
pub struct FormField {
    /// Unique field identifier
    pub id: String,
    /// Field name (fully qualified)
    pub field_name: String,
    /// Field type
    pub field_type: String, // Text, Checkbox, Radio, Button, List, Combo, Signature
    /// Field label for UI display
    pub label: Option<String>,
    /// Current field value
    pub field_value: Option<String>,
    /// Default value
    pub default_value: Option<String>,
    /// Field position on page
    pub rect: Rect,
    /// Page index where field appears
    pub page_index: i32,
    /// Field is read-only
    pub read_only: bool,
    /// Field is required
    pub required: bool,
    /// Field is hidden
    pub hidden: bool,
    /// Field export value
    pub export_value: Option<String>,
}

/// Text form field with specific properties
#[napi]
#[derive(Clone, Debug)]
pub struct TextFormField {
    /// Base form field
    pub id: String,
    pub field_name: String,
    pub field_value: Option<String>,
    pub rect: Rect,
    /// Font name (Helvetica, Times, Courier, etc.)
    pub font_name: Option<String>,
    /// Font size
    pub font_size: f32,
    /// Maximum characters allowed
    pub max_length: Option<i32>,
    /// Is multiline text
    pub multiline: bool,
    /// Text color (RGB)
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    /// Text alignment: left, center, right
    pub text_alignment: Option<String>,
}

/// Checkbox form field
#[napi]
#[derive(Clone, Debug)]
pub struct CheckboxField {
    /// Base field data
    pub id: String,
    pub field_name: String,
    pub rect: Rect,
    /// Whether checkbox is checked
    pub is_checked: bool,
    /// Export value when checked
    pub checked_value: Option<String>,
    /// Checkbox style (square, circle, diamond, etc.)
    pub style: Option<String>,
}

/// Radio button field
#[napi]
#[derive(Clone, Debug)]
pub struct RadioButtonField {
    /// Base field data
    pub id: String,
    pub field_name: String,
    pub rect: Rect,
    /// Available options
    pub options: Vec<String>,
    /// Currently selected option
    pub selected_option: Option<String>,
    /// Export values for each option
    pub export_values: Option<Vec<String>>,
}

/// Dropdown/List form field
#[napi]
#[derive(Clone, Debug)]
pub struct ListField {
    /// Base field data
    pub id: String,
    pub field_name: String,
    pub rect: Rect,
    /// List options
    pub options: Vec<String>,
    /// Display values (if different from option values)
    pub display_values: Option<Vec<String>>,
    /// Currently selected option(s)
    pub selected_options: Vec<String>,
    /// Allow multiple selection
    pub multi_select: bool,
    /// Is combo box (editable)
    pub is_combo: bool,
}

/// Push button field
#[napi]
#[derive(Clone, Debug)]
pub struct ButtonField {
    /// Base field data
    pub id: String,
    pub field_name: String,
    pub rect: Rect,
    /// Button label/caption
    pub label: String,
    /// Button action on click
    pub action: Option<String>, // Submit, Reset, JavaScript, URI, etc.
    /// Action target (URL for submit, script for JavaScript)
    pub action_target: Option<String>,
    /// Button appearance (normal, pressed, rollover)
    pub appearance: Option<String>,
}

/// Digital signature field
#[napi]
#[derive(Clone, Debug)]
pub struct SignatureField {
    /// Base field data
    pub id: String,
    pub field_name: String,
    pub rect: Rect,
    /// Whether field is signed
    pub is_signed: bool,
    /// Signature type (Approval, Certification)
    pub signature_type: Option<String>,
    /// Signer name
    pub signer_name: Option<String>,
    /// Signature date
    pub signature_date: Option<String>,
    /// Signature reason
    pub reason: Option<String>,
    /// Signature location
    pub location: Option<String>,
    /// Contact information
    pub contact_info: Option<String>,
}

/// AcroForm (traditional form) - Section 12.7.2
#[napi]
#[derive(Clone, Debug)]
pub struct AcroForm {
    /// Form name
    pub name: Option<String>,
    /// Default appearance string
    pub default_appearance: Option<String>,
    /// All form fields
    pub fields: Vec<FormField>,
    /// Whether form needs signature
    pub needs_signatures: bool,
    /// Signature flags
    pub signature_flags: i32,
    /// Calculate order of fields
    pub calculate_order: Option<Vec<String>>,
}

/// XFA Form (XML Forms Architecture) - Adobe extension
#[napi]
#[derive(Clone, Debug)]
pub struct XFAForm {
    /// Form template XML
    pub template_xml: String,
    /// Form data XML
    pub data_xml: Option<String>,
    /// Form configuration
    pub config_xml: Option<String>,
    /// Localization data
    pub locales_xml: Option<String>,
    /// Whether form is dynamic
    pub is_dynamic: bool,
    /// Form version
    pub version: Option<String>,
}

/// Form submission information
#[napi]
#[derive(Clone, Debug)]
pub struct FormSubmission {
    /// Submit button ID
    pub button_id: String,
    /// Submit URL/action
    pub action: String,
    /// Submit method (POST, GET, PDF, HTML, etc.)
    pub method: String,
    /// Fields to submit
    pub fields: Vec<String>,
    /// Include annotations in submission
    pub include_annotations: bool,
    /// Canonical format for submission
    pub canonical_format: Option<String>,
}

/// Form reset information
#[napi]
#[derive(Clone, Debug)]
pub struct FormReset {
    /// Fields to reset (None means all)
    pub fields: Option<Vec<String>>,
    /// Reset to default values
    pub reset_to_defaults: bool,
}

#[napi]
impl AcroForm {
    /// Creates new AcroForm
    #[napi]
    pub fn new(name: Option<String>) -> Self {
        AcroForm {
            name,
            default_appearance: None,
            fields: Vec::new(),
            needs_signatures: false,
            signature_flags: 0,
            calculate_order: None,
        }
    }

    /// Adds a form field
    #[napi]
    pub fn add_field(&mut self, field: FormField) {
        self.fields.push(field);
    }

    /// Gets field by name
    #[napi]
    pub fn get_field(&self, field_name: String) -> Option<FormField> {
        self.fields
            .iter()
            .find(|f| f.field_name == field_name)
            .cloned()
    }

    /// Gets all field names
    #[napi]
    pub fn get_field_names(&self) -> Vec<String> {
        self.fields.iter().map(|f| f.field_name.clone()).collect()
    }

    /// Sets field value by name
    #[napi]
    pub fn set_field_value(&mut self, field_name: String, value: String) -> bool {
        if let Some(field) = self.fields.iter_mut().find(|f| f.field_name == field_name) {
            field.field_value = Some(value);
            true
        } else {
            false
        }
    }

    /// Gets field count
    #[napi]
    pub fn field_count(&self) -> i32 {
        self.fields.len() as i32
    }

    /// Checks if form has signature fields
    #[napi]
    pub fn has_signature_fields(&self) -> bool {
        self.fields.iter().any(|f| f.field_type == "Signature")
    }

    /// Gets required fields
    #[napi]
    pub fn get_required_fields(&self) -> Vec<String> {
        self.fields
            .iter()
            .filter(|f| f.required)
            .map(|f| f.field_name.clone())
            .collect()
    }
}

#[napi]
impl XFAForm {
    /// Creates new XFA form from template XML
    #[napi]
    pub fn new(template_xml: String) -> Self {
        XFAForm {
            template_xml,
            data_xml: None,
            config_xml: None,
            locales_xml: None,
            is_dynamic: false,
            version: None,
        }
    }

    /// Sets form data
    #[napi]
    pub fn set_data(&mut self, data_xml: String) {
        self.data_xml = Some(data_xml);
    }

    /// Gets template as string
    #[napi]
    pub fn get_template(&self) -> String {
        self.template_xml.clone()
    }

    /// Gets form data
    #[napi]
    pub fn get_data(&self) -> Option<String> {
        self.data_xml.clone()
    }
}
