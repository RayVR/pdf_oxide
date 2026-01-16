//! C# P/Invoke FFI layer for pdf_oxide
//!
//! This module provides C-compatible FFI functions for .NET P/Invoke interoperability.
//! Built as a cdylib and consumed by C# via P/Invoke declarations.
//!
//! # Architecture
//!
//! The FFI layer provides:
//! - **Error handling**: Standardized error codes for C# exception mapping
//! - **Memory management**: Explicit allocation/deallocation functions
//! - **String marshaling**: UTF-8 string conversion with explicit lifetime management
//! - **Type mapping**: Rust types converted to C-compatible representations
//!
//! # Design Principles
//!
//! 1. **Explicit Memory Management**: C# code explicitly frees allocated memory
//! 2. **Error Codes**: Return error codes, exceptions mapped on C# side
//! 3. **UTF-8 Strings**: All strings marshaled as UTF-8 pointers
//! 4. **No Panics**: All Rust errors converted to error codes before FFI boundary
//!

pub mod annotations;
pub mod conversion;
pub mod document_editor;
pub mod dom;
pub mod dom_elements;
pub mod exceptions;
pub mod forms;
pub mod geometry;
pub mod pdf;
pub mod pdf_document;
pub mod search;
pub mod utils;
