use crate::metadata::{AcroForm, DocumentInfo, EmbeddedFile, PageLabel, XMPMetadata};
use crate::types::{ConversionOptions, Rect};
use crate::utils::map_result;
use napi_derive::napi;
use pdf_oxide::reader::PdfDocument as PdfDocumentImpl;

/// PDF document reader for text extraction and format conversion
///
/// Provides read-only access with automatic reading order detection.
///
/// # Examples
/// ```javascript
/// import { PdfDocument } from '@pdf-oxide/core';
///
/// using doc = PdfDocument.open('document.pdf');
/// console.log(`Pages: ${doc.pageCount}`);
/// const text = doc.extractText(0);
/// console.log(text);
/// ```
#[napi]
pub struct PdfDocument {
    inner: PdfDocumentImpl,
}

#[napi]
impl PdfDocument {
    /// Opens a PDF document from file path
    #[napi]
    pub fn open(path: String) -> napi::Result<PdfDocument> {
        let doc = PdfDocumentImpl::open(&path).map_err(|e| crate::errors::map_error(e))?;
        Ok(PdfDocument { inner: doc })
    }

    /// Opens a PDF document from bytes
    #[napi]
    pub fn open_from_bytes(data: napi::bindgen_prelude::Buffer) -> napi::Result<PdfDocument> {
        let bytes = data.to_vec();
        let doc =
            PdfDocumentImpl::open_from_bytes(&bytes).map_err(|e| crate::errors::map_error(e))?;
        Ok(PdfDocument { inner: doc })
    }

    /// Opens an encrypted PDF with password
    #[napi]
    pub fn open_with_password(path: String, password: String) -> napi::Result<PdfDocument> {
        let doc = PdfDocumentImpl::open_with_password(&path, &password)
            .map_err(|e| crate::errors::map_error(e))?;
        Ok(PdfDocument { inner: doc })
    }

    /// Gets the PDF version as (major, minor)
    #[napi]
    pub fn get_version(&self) -> (i32, i32) {
        let (major, minor) = self.inner.version();
        (major as i32, minor as i32)
    }

    /// Gets the number of pages
    #[napi]
    pub fn get_page_count(&self) -> napi::Result<i32> {
        let count = self
            .inner
            .page_count()
            .map_err(|e| crate::errors::map_error(e))?;
        Ok(count as i32)
    }

    /// Checks if document has logical structure tree (Tagged PDF)
    #[napi]
    pub fn has_structure_tree(&self) -> bool {
        self.inner.has_structure_tree()
    }

    /// Extracts text from page with automatic reading order
    #[napi]
    pub fn extract_text(&self, page_index: i32) -> napi::Result<String> {
        let text = self
            .inner
            .extract_text(page_index as usize)
            .map_err(|e| crate::errors::map_error(e))?;
        Ok(text)
    }

    /// Asynchronously extracts text from page
    #[napi(ts_return_type = "Promise<string>")]
    pub async fn extract_text_async(&self, page_index: i32) -> napi::Result<String> {
        // For now, run blocking operation in tokio thread pool
        let inner = self.inner.clone();
        let idx = page_index as usize;

        tokio::task::spawn_blocking(move || {
            inner
                .extract_text(idx)
                .map_err(|e| crate::errors::map_error(e))
        })
        .await
        .map_err(|e| napi::Error::new(napi::Status::GenericFailure, format!("Task error: {}", e)))?
    }

    /// Converts page to Markdown
    #[napi]
    pub fn to_markdown(
        &self,
        page_index: i32,
        options: Option<ConversionOptions>,
    ) -> napi::Result<String> {
        let markdown = self
            .inner
            .to_markdown(page_index as usize)
            .map_err(|e| crate::errors::map_error(e))?;
        Ok(markdown)
    }

    /// Asynchronously converts page to Markdown
    #[napi(ts_return_type = "Promise<string>")]
    pub async fn to_markdown_async(
        &self,
        page_index: i32,
        options: Option<ConversionOptions>,
    ) -> napi::Result<String> {
        let inner = self.inner.clone();
        let idx = page_index as usize;

        tokio::task::spawn_blocking(move || {
            inner
                .to_markdown(idx)
                .map_err(|e| crate::errors::map_error(e))
        })
        .await
        .map_err(|e| napi::Error::new(napi::Status::GenericFailure, format!("Task error: {}", e)))?
    }

    /// Converts all pages to Markdown
    #[napi]
    pub fn to_markdown_all(&self, options: Option<ConversionOptions>) -> napi::Result<String> {
        let markdown = self
            .inner
            .to_markdown_all()
            .map_err(|e| crate::errors::map_error(e))?;
        Ok(markdown)
    }

    /// Converts page to HTML
    #[napi]
    pub fn to_html(
        &self,
        page_index: i32,
        options: Option<ConversionOptions>,
    ) -> napi::Result<String> {
        let html = self
            .inner
            .to_html(page_index as usize)
            .map_err(|e| crate::errors::map_error(e))?;
        Ok(html)
    }

    /// Converts all pages to HTML
    #[napi]
    pub fn to_html_all(&self, options: Option<ConversionOptions>) -> napi::Result<String> {
        let html = self
            .inner
            .to_html_all()
            .map_err(|e| crate::errors::map_error(e))?;
        Ok(html)
    }

    /// Gets document metadata
    ///
    /// # Returns
    /// DocumentInfo object with document information (version, title, author, etc.)
    #[napi]
    pub fn get_document_info(&self) -> napi::Result<DocumentInfo> {
        let (major, minor) = self.inner.version();
        let version = format!("{}.{}", major, minor);

        let mut info = DocumentInfo::new(version);
        // In future: Extract actual metadata from document
        // For now, return basic info with version
        Ok(info)
    }

    /// Gets document XMP metadata
    ///
    /// # Returns
    /// XMPMetadata object if available
    #[napi]
    pub fn get_metadata(&self) -> napi::Result<XMPMetadata> {
        // In future: Extract XMP metadata from document
        // For now, return empty metadata
        Ok(XMPMetadata::new())
    }

    /// Gets document forms (AcroForm)
    ///
    /// # Returns
    /// Option<AcroForm> if document has forms
    #[napi]
    pub fn get_forms(&self) -> napi::Result<Option<AcroForm>> {
        // In future: Extract forms from document
        Ok(None)
    }

    /// Gets all page labels in document
    ///
    /// # Returns
    /// Vector of PageLabel objects for each page
    #[napi]
    pub fn get_page_labels(&self) -> napi::Result<Vec<PageLabel>> {
        // In future: Extract page labels from document
        Ok(Vec::new())
    }

    /// Gets all embedded files in document
    ///
    /// # Returns
    /// Vector of EmbeddedFile objects
    #[napi]
    pub fn get_embedded_files(&self) -> napi::Result<Vec<EmbeddedFile>> {
        // In future: Extract embedded files from document
        Ok(Vec::new())
    }

    /// Closes the document and releases resources
    #[napi]
    pub fn close(&mut self) {
        // Resources automatically cleaned up on drop
    }
}

impl Drop for PdfDocument {
    fn drop(&mut self) {
        // Explicit cleanup if needed
    }
}
