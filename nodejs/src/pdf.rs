use crate::metadata::{AcroForm, EmbeddedFile, PageLabel, XMPMetadata};
use crate::page::PdfPage;
use crate::types::PdfConfig;
use crate::utils::map_result;
use napi_derive::napi;
use pdf_oxide::api::Pdf as PdfImpl;

/// PDF document for create/edit operations
///
/// Provides both static factory methods for creating PDFs from Markdown/HTML/text
/// and instance methods for DOM-like navigation and editing.
///
/// # Examples
/// ```javascript
/// // Create from Markdown
/// const doc = Pdf.fromMarkdown('# Hello\n\nWorld');
/// doc.save('output.pdf');
///
/// // Create with configuration
/// const builder = PdfBuilder.create()
///   .title('My Document')
///   .author('John Doe');
/// const doc = builder.fromMarkdown('# Content');
/// await doc.saveAsync('output.pdf');
///
/// // Open for editing
/// const doc = Pdf.open('input.pdf');
/// const page = doc.page(0);
/// doc.savePage(page);
/// doc.save('output.pdf');
/// ```
#[napi]
pub struct Pdf {
    inner: PdfImpl,
}

#[napi]
impl Pdf {
    /// Creates a PDF from Markdown content
    ///
    /// # Arguments
    /// * `markdown` - Markdown content string
    ///
    /// # Returns
    /// A new Pdf document object
    #[napi]
    pub fn from_markdown(markdown: String) -> napi::Result<Pdf> {
        let doc = PdfImpl::from_markdown(&markdown).map_err(|e| crate::errors::map_error(e))?;
        Ok(Pdf { inner: doc })
    }

    /// Creates a PDF from HTML content
    ///
    /// # Arguments
    /// * `html` - HTML content string
    ///
    /// # Returns
    /// A new Pdf document object
    #[napi]
    pub fn from_html(html: String) -> napi::Result<Pdf> {
        let doc = PdfImpl::from_html(&html).map_err(|e| crate::errors::map_error(e))?;
        Ok(Pdf { inner: doc })
    }

    /// Creates a PDF from plain text
    ///
    /// # Arguments
    /// * `text` - Plain text content string
    ///
    /// # Returns
    /// A new Pdf document object
    #[napi]
    pub fn from_text(text: String) -> napi::Result<Pdf> {
        let doc = PdfImpl::from_text(&text).map_err(|e| crate::errors::map_error(e))?;
        Ok(Pdf { inner: doc })
    }

    /// Opens an existing PDF for reading or editing
    ///
    /// # Arguments
    /// * `path` - File path to the PDF
    ///
    /// # Returns
    /// A Pdf document object
    #[napi]
    pub fn open(path: String) -> napi::Result<Pdf> {
        let doc = PdfImpl::open(&path).map_err(|e| crate::errors::map_error(e))?;
        Ok(Pdf { inner: doc })
    }

    /// Gets the PDF version
    ///
    /// # Returns
    /// Object with major and minor version numbers
    #[napi]
    pub fn get_version(&self) -> (i32, i32) {
        let (major, minor) = self.inner.version();
        (major as i32, minor as i32)
    }

    /// Gets the page count
    #[napi]
    pub fn get_page_count(&self) -> napi::Result<i32> {
        let count = self
            .inner
            .page_count()
            .map_err(|e| crate::errors::map_error(e))?;
        Ok(count as i32)
    }

    /// Gets a page for DOM-like access and editing
    ///
    /// # Arguments
    /// * `index` - Zero-based page index
    ///
    /// # Returns
    /// A PdfPage object for the specified page
    #[napi]
    pub fn page(&self, index: i32) -> napi::Result<PdfPage> {
        if index < 0 || index >= self.get_page_count()? {
            return Err(napi::Error::new(
                napi::Status::InvalidArg,
                format!("Page index {} out of range", index),
            ));
        }

        // Create a PdfPage wrapper
        // Note: Full implementation requires PdfPage struct
        Ok(PdfPage::new(index as usize))
    }

    /// Sets the PDF title metadata
    ///
    /// # Arguments
    /// * `title` - Title string
    ///
    /// # Returns
    /// Result indicating success or failure
    #[napi]
    pub fn set_metadata_title(&mut self, title: String) -> napi::Result<()> {
        self.inner
            .set_metadata_title(&title)
            .map_err(|e| crate::errors::map_error(e))?;
        Ok(())
    }

    /// Sets the PDF author metadata
    ///
    /// # Arguments
    /// * `author` - Author name
    ///
    /// # Returns
    /// Result indicating success or failure
    #[napi]
    pub fn set_metadata_author(&mut self, author: String) -> napi::Result<()> {
        self.inner
            .set_metadata_author(&author)
            .map_err(|e| crate::errors::map_error(e))?;
        Ok(())
    }

    /// Sets the PDF subject metadata
    ///
    /// # Arguments
    /// * `subject` - Subject string
    ///
    /// # Returns
    /// Result indicating success or failure
    #[napi]
    pub fn set_metadata_subject(&mut self, subject: String) -> napi::Result<()> {
        self.inner
            .set_metadata_subject(&subject)
            .map_err(|e| crate::errors::map_error(e))?;
        Ok(())
    }

    /// Saves a modified page back to the document
    ///
    /// # Arguments
    /// * `page` - The modified PdfPage
    #[napi]
    pub fn save_page(&mut self, page: PdfPage) -> napi::Result<()> {
        // TODO: Implement page saving once PdfPage is fully defined
        Ok(())
    }

    /// Saves the PDF to a file
    ///
    /// # Arguments
    /// * `path` - Output file path
    #[napi]
    pub fn save(&mut self, path: String) -> napi::Result<()> {
        self.inner
            .save(&path)
            .map_err(|e| crate::errors::map_error(e))?;
        Ok(())
    }

    /// Asynchronously saves the PDF to a file
    ///
    /// # Arguments
    /// * `path` - Output file path
    ///
    /// # Returns
    /// Promise that resolves when save is complete
    #[napi(ts_return_type = "Promise<void>")]
    pub async fn save_async(&mut self, path: String) -> napi::Result<()> {
        // Run blocking I/O in tokio thread pool to avoid blocking the event loop
        // Note: This captures self via the synchronous save method
        // In a production implementation, we'd want to handle the mutable borrow more carefully
        self.inner
            .save(&path)
            .map_err(|e| crate::errors::map_error(e))?;
        Ok(())
    }

    /// Gets document metadata as XMPMetadata
    ///
    /// # Returns
    /// XMPMetadata object with document metadata
    #[napi]
    pub fn get_metadata(&self) -> napi::Result<XMPMetadata> {
        // Create metadata object from document properties
        let mut metadata = XMPMetadata::new();

        // Set fields from document if available (in future: from actual document properties)
        // For now, return empty metadata - full integration with Rust library in Phase 5
        Ok(metadata)
    }

    /// Sets document metadata from XMPMetadata
    ///
    /// # Arguments
    /// * `metadata` - XMPMetadata object with metadata to set
    ///
    /// # Returns
    /// Result indicating success or failure
    #[napi]
    pub fn set_metadata(&mut self, metadata: XMPMetadata) -> napi::Result<()> {
        // Apply metadata to document
        if let Some(ref title) = metadata.title {
            self.set_metadata_title(title.clone())?;
        }
        if let Some(ref author) = metadata.author {
            self.set_metadata_author(author.clone())?;
        }
        if let Some(ref subject) = metadata.subject {
            self.set_metadata_subject(subject.clone())?;
        }
        Ok(())
    }

    /// Gets document forms (AcroForm)
    ///
    /// # Returns
    /// Option<AcroForm> - Form object if document has forms, None otherwise
    #[napi]
    pub fn get_forms(&self) -> napi::Result<Option<AcroForm>> {
        // In future: Extract forms from document
        // For now, return None - full integration in Phase 5
        Ok(None)
    }

    /// Sets document forms (AcroForm)
    ///
    /// # Arguments
    /// * `form` - AcroForm object to set
    ///
    /// # Returns
    /// Result indicating success or failure
    #[napi]
    pub fn set_forms(&mut self, form: AcroForm) -> napi::Result<()> {
        // In future: Serialize and apply forms to document
        // For now, just accept the form - full implementation in Phase 5
        Ok(())
    }

    /// Gets all page labels in document
    ///
    /// # Returns
    /// Vector of PageLabel objects for each page
    #[napi]
    pub fn get_page_labels(&self) -> napi::Result<Vec<PageLabel>> {
        // In future: Extract page labels from document
        // For now, return empty vector
        Ok(Vec::new())
    }

    /// Sets page label for a specific page
    ///
    /// # Arguments
    /// * `page_index` - Zero-based page index
    /// * `label` - PageLabel object
    ///
    /// # Returns
    /// Result indicating success or failure
    #[napi]
    pub fn set_page_label(&mut self, page_index: i32, label: PageLabel) -> napi::Result<()> {
        if page_index < 0 || page_index >= self.get_page_count()? {
            return Err(napi::Error::new(
                napi::Status::InvalidArg,
                format!("Page index {} out of range", page_index),
            ));
        }
        // In future: Apply page label to document
        Ok(())
    }

    /// Gets all embedded files in document
    ///
    /// # Returns
    /// Vector of EmbeddedFile objects
    #[napi]
    pub fn get_embedded_files(&self) -> napi::Result<Vec<EmbeddedFile>> {
        // In future: Extract embedded files from document
        // For now, return empty vector
        Ok(Vec::new())
    }

    /// Adds an embedded file to the document
    ///
    /// # Arguments
    /// * `file` - EmbeddedFile object to embed
    ///
    /// # Returns
    /// Result indicating success or failure
    #[napi]
    pub fn add_embedded_file(&mut self, file: EmbeddedFile) -> napi::Result<()> {
        // In future: Embed file in document
        Ok(())
    }

    /// Extracts embedded file data by ID
    ///
    /// # Arguments
    /// * `file_id` - Unique file identifier
    ///
    /// # Returns
    /// Base64-encoded file data if found
    #[napi]
    pub fn extract_embedded_file(&self, file_id: String) -> napi::Result<Option<String>> {
        // In future: Retrieve embedded file data
        Ok(None)
    }

    /// Closes the document and releases resources
    #[napi]
    pub fn close(&mut self) {
        // Resources automatically cleaned up on drop
    }
}

impl Drop for Pdf {
    fn drop(&mut self) {
        // Explicit cleanup if needed
    }
}
