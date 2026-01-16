//! Form Field JNI bindings for creating and managing PDF form fields.
//!
//! This module implements native methods for creating and configuring form fields
//! on PDF pages, including text fields, checkboxes, radio buttons, combo boxes,
//! list boxes, push buttons, and signature fields.

use jni::objects::{JClass, JObjectArray, JString};
use jni::sys::{jboolean, jfloat, jint, jobject};
use jni::JNIEnv;

use std::collections::HashMap;
use std::sync::Mutex;

// Thread-local storage for form fields
thread_local! {
    static FORM_FIELD_CACHE: Mutex<HashMap<u64, Vec<FormFieldData>>> = Mutex::new(HashMap::new());
}

static mut NEXT_FIELD_ID: u64 = 1;

/// Internal representation of form field data
#[derive(Clone, Debug)]
struct FormFieldData {
    id: String,
    name: String,
    field_type: String, // "TEXT", "CHECKBOX", "COMBO", "LIST", "RADIO", "BUTTON", "SIGNATURE"
    metadata: String,   // JSON-encoded field-specific metadata
}

/// Stores a form field reference and returns a unique ID
fn store_form_field(page_ptr: u64, field_data: FormFieldData) -> String {
    let id = unsafe {
        NEXT_FIELD_ID += 1;
        format!("field_{}", NEXT_FIELD_ID)
    };

    FORM_FIELD_CACHE.with(|cache| {
        let mut fields = cache.lock().unwrap();
        let page_fields = fields.entry(page_ptr).or_insert_with(Vec::new);
        page_fields.push(FormFieldData {
            id: id.clone(),
            ..field_data
        });
    });

    id
}

/// Gets form field data by ID
fn get_form_field(page_ptr: u64, field_id: &str) -> Option<FormFieldData> {
    FORM_FIELD_CACHE.with(|cache| {
        let fields = cache.lock().unwrap();
        fields.get(&page_ptr).and_then(|page_fields| {
            page_fields
                .iter()
                .find(|field| field.id == field_id)
                .cloned()
        })
    })
}

/// Gets all form fields for a page
fn get_all_form_fields(page_ptr: u64) -> Vec<FormFieldData> {
    FORM_FIELD_CACHE.with(|cache| {
        let fields = cache.lock().unwrap();
        fields.get(&page_ptr).cloned().unwrap_or_default()
    })
}

/// Frees all form fields for a page
fn clear_form_fields(page_ptr: u64) {
    FORM_FIELD_CACHE.with(|cache| {
        cache.lock().unwrap().remove(&page_ptr);
    });
}

// ===== Text Field =====

/// Creates a text form field
/// Java signature: `private static native String nativeCreate(long pagePtr, String name, float x, float y, float width, float height)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_forms_TextField_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    name: JString,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
) -> jobject {
    let name_str: String = match env.get_string(&name) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                crate::jni::exceptions::throw_pdf_exception(env, "Invalid field name encoding");
                return std::ptr::null_mut();
            },
        },
        Err(_) => {
            crate::jni::exceptions::throw_pdf_exception(env, "Failed to read field name");
            return std::ptr::null_mut();
        },
    };

    let field_data = FormFieldData {
        id: String::new(), // Will be set by store_form_field
        name: name_str.clone(),
        field_type: "TEXT".to_string(),
        metadata: format!(
            r#"{{"x":{:.2},"y":{:.2},"width":{:.2},"height":{:.2}}}"#,
            x, y, width, height
        ),
    };

    let field_id = store_form_field(page_ptr, field_data);

    match env.new_string(&field_id) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// ===== Checkbox Field =====

/// Creates a checkbox form field
/// Java signature: `private static native String nativeCreate(long pagePtr, String name, float x, float y, float width, float height)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_forms_CheckboxField_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    name: JString,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
) -> jobject {
    let name_str: String = match env.get_string(&name) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                crate::jni::exceptions::throw_pdf_exception(env, "Invalid field name encoding");
                return std::ptr::null_mut();
            },
        },
        Err(_) => {
            crate::jni::exceptions::throw_pdf_exception(env, "Failed to read field name");
            return std::ptr::null_mut();
        },
    };

    let field_data = FormFieldData {
        id: String::new(),
        name: name_str.clone(),
        field_type: "CHECKBOX".to_string(),
        metadata: format!(
            r#"{{"x":{:.2},"y":{:.2},"width":{:.2},"height":{:.2},"checked":false}}"#,
            x, y, width, height
        ),
    };

    let field_id = store_form_field(page_ptr, field_data);

    match env.new_string(&field_id) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// ===== ComboBox Field =====

/// Creates a combo box form field
/// Java signature: `private static native String nativeCreate(long pagePtr, String name, float x, float y, float width, float height, String[] options)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_forms_ComboBoxField_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    name: JString,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
    options: JObjectArray,
) -> jobject {
    let name_str: String = match env.get_string(&name) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                crate::jni::exceptions::throw_pdf_exception(env, "Invalid field name encoding");
                return std::ptr::null_mut();
            },
        },
        Err(_) => {
            crate::jni::exceptions::throw_pdf_exception(env, "Failed to read field name");
            return std::ptr::null_mut();
        },
    };

    // Extract options array
    let opt_count = match env.get_array_length(&options) {
        Ok(len) => len,
        Err(_) => {
            crate::jni::exceptions::throw_pdf_exception(env, "Failed to read options array length");
            return std::ptr::null_mut();
        },
    };

    let mut options_vec = Vec::new();
    for i in 0..opt_count {
        if let Ok(opt_obj) = env.get_object_array_element(&options, i) {
            let opt_str: JString = opt_obj.into();
            {
                let res = env.get_string(&opt_str);
                if let Ok(java_str) = res {
                    if let Ok(s) = java_str.to_str() {
                        options_vec.push(s.to_string());
                    }
                }
            }
        }
    }

    let options_json = serde_json::to_string(&options_vec).unwrap_or_default();

    let field_data = FormFieldData {
        id: String::new(),
        name: name_str.clone(),
        field_type: "COMBO".to_string(),
        metadata: format!(
            r#"{{"x":{:.2},"y":{:.2},"width":{:.2},"height":{:.2},"options":{}}}"#,
            x, y, width, height, options_json
        ),
    };

    let field_id = store_form_field(page_ptr, field_data);

    match env.new_string(&field_id) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// ===== ListBox Field =====

/// Creates a list box form field
/// Java signature: `private static native String nativeCreate(long pagePtr, String name, float x, float y, float width, float height, String[] options, boolean multiSelect)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_forms_ListBoxField_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    name: JString,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
    options: JObjectArray,
    multi_select: jboolean,
) -> jobject {
    let name_str: String = match env.get_string(&name) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                crate::jni::exceptions::throw_pdf_exception(env, "Invalid field name encoding");
                return std::ptr::null_mut();
            },
        },
        Err(_) => {
            crate::jni::exceptions::throw_pdf_exception(env, "Failed to read field name");
            return std::ptr::null_mut();
        },
    };

    // Extract options array
    let opt_count = match env.get_array_length(&options) {
        Ok(len) => len,
        Err(_) => {
            crate::jni::exceptions::throw_pdf_exception(env, "Failed to read options array length");
            return std::ptr::null_mut();
        },
    };

    let mut options_vec = Vec::new();
    for i in 0..opt_count {
        if let Ok(opt_obj) = env.get_object_array_element(&options, i) {
            let opt_str: JString = opt_obj.into();
            {
                let res = env.get_string(&opt_str);
                if let Ok(java_str) = res {
                    if let Ok(s) = java_str.to_str() {
                        options_vec.push(s.to_string());
                    }
                }
            }
        }
    }

    let options_json = serde_json::to_string(&options_vec).unwrap_or_default();

    let field_data = FormFieldData {
        id: String::new(),
        name: name_str.clone(),
        field_type: "LIST".to_string(),
        metadata: format!(
            r#"{{"x":{:.2},"y":{:.2},"width":{:.2},"height":{:.2},"options":{},"multiSelect":{}}}"#,
            x,
            y,
            width,
            height,
            options_json,
            multi_select != 0
        ),
    };

    let field_id = store_form_field(page_ptr, field_data);

    match env.new_string(&field_id) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// ===== RadioButton Field =====

/// Creates a radio button group form field
/// Java signature: `private static native String nativeCreate(long pagePtr, String name, String[] exportValues)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_forms_RadioButtonField_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    name: JString,
    export_values: JObjectArray,
) -> jobject {
    let name_str: String = match env.get_string(&name) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                crate::jni::exceptions::throw_pdf_exception(env, "Invalid field name encoding");
                return std::ptr::null_mut();
            },
        },
        Err(_) => {
            crate::jni::exceptions::throw_pdf_exception(env, "Failed to read field name");
            return std::ptr::null_mut();
        },
    };

    // Extract export values array
    let val_count = match env.get_array_length(&export_values) {
        Ok(len) => len,
        Err(_) => {
            crate::jni::exceptions::throw_pdf_exception(
                env,
                "Failed to read export values array length",
            );
            return std::ptr::null_mut();
        },
    };

    let mut values_vec = Vec::new();
    for i in 0..val_count {
        if let Ok(val_obj) = env.get_object_array_element(&export_values, i) {
            let val_str: JString = val_obj.into();
            {
                let res = env.get_string(&val_str);
                if let Ok(java_str) = res {
                    if let Ok(s) = java_str.to_str() {
                        values_vec.push(s.to_string());
                    }
                }
            }
        }
    }

    let values_json = serde_json::to_string(&values_vec).unwrap_or_default();

    let field_data = FormFieldData {
        id: String::new(),
        name: name_str.clone(),
        field_type: "RADIO".to_string(),
        metadata: format!(r#"{{"exportValues":{}}}"#, values_json),
    };

    let field_id = store_form_field(page_ptr, field_data);

    match env.new_string(&field_id) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// ===== PushButton Field =====

/// Creates a push button form field
/// Java signature: `private static native String nativeCreate(long pagePtr, String name, float x, float y, float width, float height, String caption, String action)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_forms_PushButtonField_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    name: JString,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
    caption: JString,
    action: JString,
) -> jobject {
    let name_str: String = match env.get_string(&name) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                crate::jni::exceptions::throw_pdf_exception(env, "Invalid field name encoding");
                return std::ptr::null_mut();
            },
        },
        Err(_) => {
            crate::jni::exceptions::throw_pdf_exception(env, "Failed to read field name");
            return std::ptr::null_mut();
        },
    };

    let caption_str: String = match env.get_string(&caption) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    };

    let action_str: String = match env.get_string(&action) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => "NONE".to_string(),
        },
        Err(_) => "NONE".to_string(),
    };

    let field_data = FormFieldData {
        id: String::new(),
        name: name_str.clone(),
        field_type: "BUTTON".to_string(),
        metadata: format!(
            r#"{{"x":{:.2},"y":{:.2},"width":{:.2},"height":{:.2},"caption":"{}","action":"{}"}}"#,
            x, y, width, height, caption_str, action_str
        ),
    };

    let field_id = store_form_field(page_ptr, field_data);

    match env.new_string(&field_id) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// ===== Signature Field =====

/// Creates a signature form field
/// Java signature: `private static native String nativeCreate(long pagePtr, String name, float x, float y, float width, float height)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_forms_SignatureField_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    name: JString,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
) -> jobject {
    let name_str: String = match env.get_string(&name) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                crate::jni::exceptions::throw_pdf_exception(env, "Invalid field name encoding");
                return std::ptr::null_mut();
            },
        },
        Err(_) => {
            crate::jni::exceptions::throw_pdf_exception(env, "Failed to read field name");
            return std::ptr::null_mut();
        },
    };

    let field_data = FormFieldData {
        id: String::new(),
        name: name_str.clone(),
        field_type: "SIGNATURE".to_string(),
        metadata: format!(
            r#"{{"x":{:.2},"y":{:.2},"width":{:.2},"height":{:.2},"signed":false}}"#,
            x, y, width, height
        ),
    };

    let field_id = store_form_field(page_ptr, field_data);

    match env.new_string(&field_id) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// ===== Form Field Query Methods =====

/// Gets the count of form fields on a page
/// Java signature: `public static native int nativeGetFormFieldCount(long pagePtr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_forms_FormFieldManager_nativeGetFormFieldCount(
    _env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
) -> jint {
    let fields = get_all_form_fields(page_ptr);
    fields.len() as jint
}

/// Gets the name of a form field by index
/// Java signature: `public static native String nativeGetFormFieldName(long pagePtr, int index)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_forms_FormFieldManager_nativeGetFormFieldName(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    index: jint,
) -> jobject {
    let fields = get_all_form_fields(page_ptr);

    if index < 0 || index as usize >= fields.len() {
        return std::ptr::null_mut();
    }

    match env.new_string(&fields[index as usize].name) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Gets the type of a form field by index
/// Java signature: `public static native String nativeGetFormFieldType(long pagePtr, int index)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_forms_FormFieldManager_nativeGetFormFieldType(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    index: jint,
) -> jobject {
    let fields = get_all_form_fields(page_ptr);

    if index < 0 || index as usize >= fields.len() {
        return std::ptr::null_mut();
    }

    match env.new_string(&fields[index as usize].field_type) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Clears all form fields for a page
/// Java signature: `public static native void nativeClearFormFields(long pagePtr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_forms_FormFieldManager_nativeClearFormFields(
    _env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
) {
    clear_form_fields(page_ptr);
}

/// Native cleanup for form fields when page is freed
pub fn native_free_form_fields(page_ptr: u64) {
    clear_form_fields(page_ptr);
}
