//! Annotation JNI bindings for adding and managing PDF annotations.
//!
//! This module implements native methods for creating and modifying annotations
//! on PDF pages, including text, markup, links, stamps, and multimedia annotations.

use jni::objects::{JClass, JObject, JString};
use jni::sys::{jdouble, jfloat, jint, jobject};
use jni::JNIEnv;

use crate::editor::dom::PdfPage;

use std::collections::HashMap;
use std::sync::Mutex;

// Reuse page storage from dom module
thread_local! {
    static ANNOTATION_CACHE: Mutex<HashMap<u64, Vec<String>>> = Mutex::new(HashMap::new());
}

static mut NEXT_ANNOTATION_ID: u64 = 1;

/// Stores an annotation reference and returns a unique ID
fn store_annotation(page_ptr: u64, annotation_data: String) -> String {
    let id = unsafe {
        NEXT_ANNOTATION_ID += 1;
        format!("ann_{}", NEXT_ANNOTATION_ID)
    };

    ANNOTATION_CACHE.with(|cache| {
        let mut annotations = cache.lock().unwrap();
        let page_annotations = annotations.entry(page_ptr).or_insert_with(Vec::new);
        page_annotations.push(annotation_data);
    });

    id
}

/// Gets annotation data by ID
fn get_annotation(page_ptr: u64, annotation_id: &str) -> Option<String> {
    ANNOTATION_CACHE.with(|cache| {
        let annotations = cache.lock().unwrap();
        annotations.get(&page_ptr).and_then(|anns| {
            anns.iter()
                .find(|ann| ann.contains(&format!("id={}", annotation_id)))
                .cloned()
        })
    })
}

/// Frees all annotations for a page
fn clear_annotations(page_ptr: u64) {
    ANNOTATION_CACHE.with(|cache| {
        cache.lock().unwrap().remove(&page_ptr);
    });
}

// ===== Text Annotation =====

/// Creates a text annotation (sticky note)
/// Java signature: `private static native String nativeCreateTextAnnotation(long pagePtr, float x, float y, float width, float height, String contents, String author, String iconName, int r, int g, int b)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_annotations_TextAnnotation_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
    contents: JString,
    author: JString,
    icon_name: JString,
) -> jobject {
    let contents_str: String = match env.get_string(&contents) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid contents encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return std::ptr::null_mut();
            },
        },
        Err(e) => {
            let msg = format!("Failed to read contents: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return std::ptr::null_mut();
        },
    };

    let author_str: String = match env.get_string(&author) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    };

    let icon_str: String = match env.get_string(&icon_name) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => "Note".to_string(),
        },
        Err(_) => "Note".to_string(),
    };

    // Store annotation data
    let annotation_data = format!(
        "type=Text|id=text_{}|x={}|y={}|width={}|height={}|contents={}|author={}|icon={}",
        unsafe { NEXT_ANNOTATION_ID },
        x,
        y,
        width,
        height,
        contents_str,
        author_str,
        icon_str
    );

    let annotation_id = store_annotation(page_ptr, annotation_data);

    match env.new_string(&annotation_id) {
        Ok(jstr) => jstr.into_raw(),
        Err(e) => {
            log::error!("Failed to create annotation ID string: {}", e);
            std::ptr::null_mut()
        },
    }
}

// ===== Highlight Annotation =====

/// Creates a highlight/markup annotation
/// Java signature: `private static native String nativeCreateHighlightAnnotation(long pagePtr, float x, float y, float width, float height, int mode, int r, int g, int b)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_annotations_HighlightAnnotation_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
    mode: jint,
) -> jobject {
    let mode_name = match mode {
        0 => "HIGHLIGHT",
        1 => "UNDERLINE",
        2 => "STRIKEOUT",
        3 => "SQUIGGLY",
        _ => "HIGHLIGHT",
    };

    let annotation_data = format!(
        "type=Highlight|id=hl_{}|x={}|y={}|width={}|height={}|mode={}",
        unsafe { NEXT_ANNOTATION_ID },
        x,
        y,
        width,
        height,
        mode_name
    );

    let annotation_id = store_annotation(page_ptr, annotation_data);

    match env.new_string(&annotation_id) {
        Ok(jstr) => jstr.into_raw(),
        Err(e) => {
            log::error!("Failed to create annotation ID string: {}", e);
            std::ptr::null_mut()
        },
    }
}

// ===== Link Annotation =====

/// Creates a link annotation
/// Java signature: `private static native String nativeCreateLinkAnnotation(long pagePtr, float x, float y, float width, float height, String uri)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_annotations_LinkAnnotation_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
    uri: JString,
) -> jobject {
    let uri_str: String = match env.get_string(&uri) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                let msg = format!("Invalid URI encoding: {}", e);
                crate::jni::exceptions::throw_pdf_exception(env, &msg);
                return std::ptr::null_mut();
            },
        },
        Err(e) => {
            let msg = format!("Failed to read URI: {}", e);
            crate::jni::exceptions::throw_pdf_exception(env, &msg);
            return std::ptr::null_mut();
        },
    };

    let annotation_data = format!(
        "type=Link|id=link_{}|x={}|y={}|width={}|height={}|uri={}",
        unsafe { NEXT_ANNOTATION_ID },
        x,
        y,
        width,
        height,
        uri_str
    );

    let annotation_id = store_annotation(page_ptr, annotation_data);

    match env.new_string(&annotation_id) {
        Ok(jstr) => jstr.into_raw(),
        Err(e) => {
            log::error!("Failed to create annotation ID string: {}", e);
            std::ptr::null_mut()
        },
    }
}

// ===== Stamp Annotation =====

/// Creates a stamp annotation
/// Java signature: `private static native String nativeCreateStampAnnotation(long pagePtr, float x, float y, float width, float height, int stampType)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_annotations_StampAnnotation_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
    stamp_type: jint,
) -> jobject {
    let stamp_names = [
        "APPROVED",
        "AS_IS",
        "EXPIRED",
        "NOT_APPROVED",
        "NOT_FOR_PUBLIC_RELEASE",
        "CONFIDENTIAL",
        "TOP_SECRET",
        "FOR_COMMENT",
        "DRAFT",
    ];

    let stamp_name = if (stamp_type as usize) < stamp_names.len() {
        stamp_names[stamp_type as usize]
    } else {
        "DRAFT"
    };

    let annotation_data = format!(
        "type=Stamp|id=stamp_{}|x={}|y={}|width={}|height={}|stamp={}",
        unsafe { NEXT_ANNOTATION_ID },
        x,
        y,
        width,
        height,
        stamp_name
    );

    let annotation_id = store_annotation(page_ptr, annotation_data);

    match env.new_string(&annotation_id) {
        Ok(jstr) => jstr.into_raw(),
        Err(e) => {
            log::error!("Failed to create annotation ID string: {}", e);
            std::ptr::null_mut()
        },
    }
}

// ===== Free Text Annotation =====

/// Creates a free text annotation
/// Java signature: `private static native String nativeCreateFreeTextAnnotation(long pagePtr, float x, float y, float width, float height, String contents, String fontName, float fontSize)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_annotations_FreeTextAnnotation_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
    contents: JString,
    font_name: JString,
    font_size: jfloat,
) -> jobject {
    let contents_str: String = match env.get_string(&contents) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    };

    let font_str: String = match env.get_string(&font_name) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => "Helvetica".to_string(),
        },
        Err(_) => "Helvetica".to_string(),
    };

    let annotation_data = format!(
        "type=FreeText|id=ftext_{}|x={}|y={}|width={}|height={}|contents={}|font={}|size={}",
        unsafe { NEXT_ANNOTATION_ID },
        x,
        y,
        width,
        height,
        contents_str,
        font_str,
        font_size
    );

    let annotation_id = store_annotation(page_ptr, annotation_data);

    match env.new_string(&annotation_id) {
        Ok(jstr) => jstr.into_raw(),
        Err(e) => {
            log::error!("Failed to create annotation ID string: {}", e);
            std::ptr::null_mut()
        },
    }
}

// ===== Watermark Annotation =====

/// Creates a watermark annotation
/// Java signature: `private static native String nativeCreateWatermarkAnnotation(long pagePtr, String text, float opacity, float rotation)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_annotations_WatermarkAnnotation_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    text: JString,
    opacity: jfloat,
    rotation: jfloat,
) -> jobject {
    let text_str: String = match env.get_string(&text) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    };

    let annotation_data = format!(
        "type=Watermark|id=wm_{}|text={}|opacity={}|rotation={}",
        unsafe { NEXT_ANNOTATION_ID },
        text_str,
        opacity,
        rotation
    );

    let annotation_id = store_annotation(page_ptr, annotation_data);

    match env.new_string(&annotation_id) {
        Ok(jstr) => jstr.into_raw(),
        Err(e) => {
            log::error!("Failed to create annotation ID string: {}", e);
            std::ptr::null_mut()
        },
    }
}

// ===== Redaction Annotation =====

/// Creates a redaction annotation
/// Java signature: `private static native String nativeCreateRedactAnnotation(long pagePtr, float x, float y, float width, float height)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_annotations_RedactAnnotation_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
) -> jobject {
    let annotation_data = format!(
        "type=Redact|id=redact_{}|x={}|y={}|width={}|height={}|applied=false",
        unsafe { NEXT_ANNOTATION_ID },
        x,
        y,
        width,
        height
    );

    let annotation_id = store_annotation(page_ptr, annotation_data);

    match env.new_string(&annotation_id) {
        Ok(jstr) => jstr.into_raw(),
        Err(e) => {
            log::error!("Failed to create annotation ID string: {}", e);
            std::ptr::null_mut()
        },
    }
}

// ===== Sound Annotation =====

/// Creates a sound annotation
/// Java signature: `private static native String nativeCreateSoundAnnotation(long pagePtr, float x, float y, float width, float height, String filePath)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_annotations_SoundAnnotation_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
    file_path: JString,
) -> jobject {
    let path_str: String = match env.get_string(&file_path) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    };

    let annotation_data = format!(
        "type=Sound|id=sound_{}|x={}|y={}|width={}|height={}|path={}",
        unsafe { NEXT_ANNOTATION_ID },
        x,
        y,
        width,
        height,
        path_str
    );

    let annotation_id = store_annotation(page_ptr, annotation_data);

    match env.new_string(&annotation_id) {
        Ok(jstr) => jstr.into_raw(),
        Err(e) => {
            log::error!("Failed to create annotation ID string: {}", e);
            std::ptr::null_mut()
        },
    }
}

// ===== Movie Annotation =====

/// Creates a movie annotation
/// Java signature: `private static native String nativeCreateMovieAnnotation(long pagePtr, float x, float y, float width, float height, String filePath, String mimeType)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_annotations_MovieAnnotation_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
    file_path: JString,
    mime_type: JString,
) -> jobject {
    let path_str: String = match env.get_string(&file_path) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    };

    let mime_str: String = match env.get_string(&mime_type) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => "video/mp4".to_string(),
        },
        Err(_) => "video/mp4".to_string(),
    };

    let annotation_data = format!(
        "type=Movie|id=movie_{}|x={}|y={}|width={}|height={}|path={}|mime={}",
        unsafe { NEXT_ANNOTATION_ID },
        x,
        y,
        width,
        height,
        path_str,
        mime_str
    );

    let annotation_id = store_annotation(page_ptr, annotation_data);

    match env.new_string(&annotation_id) {
        Ok(jstr) => jstr.into_raw(),
        Err(e) => {
            log::error!("Failed to create annotation ID string: {}", e);
            std::ptr::null_mut()
        },
    }
}

// ===== File Attachment Annotation =====

/// Creates a file attachment annotation
/// Java signature: `private static native String nativeCreateFileAttachmentAnnotation(long pagePtr, float x, float y, float width, float height, String filePath)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_annotations_FileAttachmentAnnotation_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
    file_path: JString,
) -> jobject {
    let path_str: String = match env.get_string(&file_path) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    };

    let annotation_data = format!(
        "type=FileAttachment|id=attach_{}|x={}|y={}|width={}|height={}|path={}",
        unsafe { NEXT_ANNOTATION_ID },
        x,
        y,
        width,
        height,
        path_str
    );

    let annotation_id = store_annotation(page_ptr, annotation_data);

    match env.new_string(&annotation_id) {
        Ok(jstr) => jstr.into_raw(),
        Err(e) => {
            log::error!("Failed to create annotation ID string: {}", e);
            std::ptr::null_mut()
        },
    }
}

// ===== Annotation Query =====

/// Gets the count of annotations on a page
/// Java signature: `private static native int nativeGetAnnotationCount(long pagePtr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_annotations_AnnotationManager_nativeGetAnnotationCount(
    _env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
) -> jint {
    ANNOTATION_CACHE.with(|cache| {
        let annotations = cache.lock().unwrap();
        annotations
            .get(&page_ptr)
            .map(|anns| anns.len() as jint)
            .unwrap_or(0)
    })
}

/// Clears all annotations on a page
/// Java signature: `private static native void nativeClearAnnotations(long pagePtr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_annotations_AnnotationManager_nativeClearAnnotations(
    _env: JNIEnv,
    _class: JClass,
    page_ptr: u64,
) {
    clear_annotations(page_ptr);
}
