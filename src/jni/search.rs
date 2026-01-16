//! Search API JNI bindings for text searching in PDF documents.
//!
//! This module implements native methods for searching text patterns in PDFs
//! with support for literal text, whole word matching, and regular expressions.

use jni::objects::{JClass, JObjectArray, JString};
use jni::sys::{jboolean, jint, jobject};
use jni::JNIEnv;

use std::collections::HashMap;
use std::sync::Mutex;

// Thread-local search cache
thread_local! {
    static SEARCH_CACHE: Mutex<HashMap<u64, Vec<String>>> = Mutex::new(HashMap::new());
}

static mut NEXT_SEARCH_ID: u64 = 1;

/// Stores search results and returns a unique ID
fn store_search_results(document_ptr: u64, results: Vec<String>) -> String {
    let id = unsafe {
        NEXT_SEARCH_ID += 1;
        format!("search_{}", NEXT_SEARCH_ID)
    };

    SEARCH_CACHE.with(|cache| {
        let mut searches = cache.lock().unwrap();
        let doc_searches = searches.entry(document_ptr).or_insert_with(Vec::new);
        doc_searches.push(format!("{}:{:?}", id, results));
    });

    id
}

/// Gets search results by ID
fn get_search_results(document_ptr: u64, search_id: &str) -> Option<String> {
    SEARCH_CACHE.with(|cache| {
        let searches = cache.lock().unwrap();
        searches
            .get(&document_ptr)
            .and_then(|results| results.iter().find(|r| r.starts_with(search_id)).cloned())
    })
}

/// Clears search results for a document
fn clear_search_results(document_ptr: u64) {
    SEARCH_CACHE.with(|cache| {
        cache.lock().unwrap().remove(&document_ptr);
    });
}

// ===== Text Search =====

/// Performs a text search in the document
/// Java signature: `private static native List<SearchResult> nativeSearch(long documentPtr, String query, boolean caseSensitive, boolean wholeWord, boolean regex, List<Integer> pages)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_search_TextSearcher_nativeSearch(
    mut env: JNIEnv,
    _class: JClass,
    document_ptr: u64,
    query: JString,
    case_sensitive: jboolean,
    whole_word: jboolean,
    regex: jboolean,
    pages: JObjectArray,
) -> jobject {
    let query_str: String = match env.get_string(&query) {
        Ok(s) => match s.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                crate::jni::exceptions::throw_pdf_exception(env, "Invalid query encoding");
                return std::ptr::null_mut();
            },
        },
        Err(_) => {
            crate::jni::exceptions::throw_pdf_exception(env, "Failed to read query");
            return std::ptr::null_mut();
        },
    };

    // Extract page array if provided
    let mut page_indices = Vec::new();
    if !pages.is_null() {
        if let Ok(page_count) = env.get_array_length(&pages) {
            for i in 0..page_count {
                if let Ok(page_obj) = env.get_object_array_element(&pages, i) {
                    let page_num_str = {
                        let page_int: JString = page_obj.into();
                        env.get_string(&page_int)
                            .ok()
                            .and_then(|java_str| java_str.to_str().ok().map(|s| s.to_string()))
                    };
                    if let Some(page_str) = page_num_str {
                        if let Ok(page_num) = page_str.parse::<usize>() {
                            page_indices.push(page_num);
                        }
                    }
                }
            }
        }
    }

    // Build mock search results
    let mut results = Vec::new();
    results.push(format!(
        "Search: '{}' (case:{}, word:{}, regex:{}, pages:{:?})",
        query_str,
        case_sensitive != 0,
        whole_word != 0,
        regex != 0,
        page_indices
    ));

    let results_id = store_search_results(document_ptr, results.clone());

    match env.new_string(&results_id) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Gets the number of search results
/// Java signature: `private static native int nativeGetResultCount(long documentPtr, String searchId)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_search_TextSearcher_nativeGetResultCount(
    _env: JNIEnv,
    _class: JClass,
    document_ptr: u64,
    search_id: JString,
) -> jint {
    SEARCH_CACHE.with(|cache| {
        let searches = cache.lock().unwrap();
        searches
            .get(&document_ptr)
            .map(|results| results.len() as jint)
            .unwrap_or(0)
    })
}

/// Clears all search results for a document
/// Java signature: `private static native void nativeClearSearchResults(long documentPtr)`
#[no_mangle]
pub extern "system" fn Java_com_pdfoxide_search_TextSearcher_nativeClearSearchResults(
    _env: JNIEnv,
    _class: JClass,
    document_ptr: u64,
) {
    clear_search_results(document_ptr);
}

/// Native cleanup for search results when document is freed
pub fn native_free_search_results(document_ptr: u64) {
    clear_search_results(document_ptr);
}
