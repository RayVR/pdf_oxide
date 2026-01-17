use crate::pdf::Pdf;
use crate::types::PageSize;
use napi_derive::napi;

/// PdfBuilder for advanced PDF creation with fluent API
///
/// Enables method chaining for PDF configuration.
///
/// # Examples
/// ```javascript
/// const doc = PdfBuilder.create()
///   .title('My Document')
///   .author('John Doe')
///   .subject('2024 Report')
///   .pageSize(PageSize.A4)
///   .margins(72, 72, 72, 72)
///   .fromMarkdown('# Title\n\nContent here');
///
/// doc.save('output.pdf');
/// ```
#[napi]
pub struct PdfBuilder {
    title: Option<String>,
    author: Option<String>,
    subject: Option<String>,
    page_size: Option<String>,
    margin_top: Option<f32>,
    margin_right: Option<f32>,
    margin_bottom: Option<f32>,
    margin_left: Option<f32>,
}

#[napi]
impl PdfBuilder {
    /// Creates a new PdfBuilder with default configuration
    ///
    /// # Returns
    /// A new PdfBuilder instance
    #[napi]
    pub fn create() -> PdfBuilder {
        PdfBuilder {
            title: None,
            author: None,
            subject: None,
            page_size: None,
            margin_top: None,
            margin_right: None,
            margin_bottom: None,
            margin_left: None,
        }
    }

    /// Sets the PDF title (metadata)
    ///
    /// # Arguments
    /// * `title` - Title string
    ///
    /// # Returns
    /// Self for method chaining
    #[napi]
    pub fn title(&mut self, title: String) -> &mut PdfBuilder {
        self.title = Some(title);
        self
    }

    /// Sets the PDF author (metadata)
    ///
    /// # Arguments
    /// * `author` - Author name
    ///
    /// # Returns
    /// Self for method chaining
    #[napi]
    pub fn author(&mut self, author: String) -> &mut PdfBuilder {
        self.author = Some(author);
        self
    }

    /// Sets the PDF subject (metadata)
    ///
    /// # Arguments
    /// * `subject` - Subject string
    ///
    /// # Returns
    /// Self for method chaining
    #[napi]
    pub fn subject(&mut self, subject: String) -> &mut PdfBuilder {
        self.subject = Some(subject);
        self
    }

    /// Sets the page size
    ///
    /// # Arguments
    /// * `size` - Page size string (e.g., "A4", "Letter", "A3")
    ///
    /// # Returns
    /// Self for method chaining
    #[napi]
    pub fn page_size(&mut self, size: String) -> &mut PdfBuilder {
        self.page_size = Some(size);
        self
    }

    /// Sets the page margins in points (1/72 inch)
    ///
    /// # Arguments
    /// * `top` - Top margin
    /// * `right` - Right margin
    /// * `bottom` - Bottom margin
    /// * `left` - Left margin
    ///
    /// # Returns
    /// Self for method chaining
    #[napi]
    pub fn margins(&mut self, top: f32, right: f32, bottom: f32, left: f32) -> &mut PdfBuilder {
        self.margin_top = Some(top);
        self.margin_right = Some(right);
        self.margin_bottom = Some(bottom);
        self.margin_left = Some(left);
        self
    }

    /// Creates PDF from Markdown content with configured settings
    ///
    /// # Arguments
    /// * `markdown` - Markdown content string
    ///
    /// # Returns
    /// A Pdf document object
    #[napi]
    pub fn from_markdown(&self, markdown: String) -> napi::Result<Pdf> {
        let mut doc = Pdf::from_markdown(markdown)?;

        // Apply configuration
        if let Some(ref title) = self.title {
            doc.set_metadata_title(title.clone())?;
        }
        if let Some(ref author) = self.author {
            doc.set_metadata_author(author.clone())?;
        }
        if let Some(ref subject) = self.subject {
            doc.set_metadata_subject(subject.clone())?;
        }

        Ok(doc)
    }

    /// Creates PDF from HTML content with configured settings
    ///
    /// # Arguments
    /// * `html` - HTML content string
    ///
    /// # Returns
    /// A Pdf document object
    #[napi]
    pub fn from_html(&self, html: String) -> napi::Result<Pdf> {
        let mut doc = Pdf::from_html(html)?;

        // Apply configuration
        if let Some(ref title) = self.title {
            doc.set_metadata_title(title.clone())?;
        }
        if let Some(ref author) = self.author {
            doc.set_metadata_author(author.clone())?;
        }
        if let Some(ref subject) = self.subject {
            doc.set_metadata_subject(subject.clone())?;
        }

        Ok(doc)
    }

    /// Creates PDF from plain text content with configured settings
    ///
    /// # Arguments
    /// * `text` - Plain text content string
    ///
    /// # Returns
    /// A Pdf document object
    #[napi]
    pub fn from_text(&self, text: String) -> napi::Result<Pdf> {
        let mut doc = Pdf::from_text(text)?;

        // Apply configuration
        if let Some(ref title) = self.title {
            doc.set_metadata_title(title.clone())?;
        }
        if let Some(ref author) = self.author {
            doc.set_metadata_author(author.clone())?;
        }
        if let Some(ref subject) = self.subject {
            doc.set_metadata_subject(subject.clone())?;
        }

        Ok(doc)
    }
}
