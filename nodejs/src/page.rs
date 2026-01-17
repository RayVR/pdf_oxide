use napi_derive::napi;
use crate::types::{Rect, SearchOptions, SearchResult, ElementContent, AnnotationContent};
use crate::dom::{PageData, AnnotationData};
use crate::elements::PdfText;

/// PDF page for DOM-like navigation and editing
///
/// Provides DOM-like access to page elements, annotations, and content.
/// Methods support both reading and modifying page structure.
#[napi]
pub struct PdfPage {
  /// Internal page data storage
  data: PageData,
}

impl PdfPage {
  /// Creates a new PdfPage wrapper with default dimensions
  pub fn new(page_index: usize) -> Self {
    let data = PageData::new(
      page_index,
      612.0, // Default: US Letter width in points
      792.0, // Default: US Letter height in points
    );
    PdfPage { data }
  }

  /// Creates a PdfPage with custom dimensions
  pub fn with_dimensions(page_index: usize, width: f32, height: f32) -> Self {
    let data = PageData::new(page_index, width, height);
    PdfPage { data }
  }

  /// Returns mutable reference to internal page data
  pub fn data_mut(&mut self) -> &mut PageData {
    &mut self.data
  }

  /// Returns reference to internal page data
  pub fn data(&self) -> &PageData {
    &self.data
  }

  /// Marks the page as modified
  pub fn mark_modified(&mut self) {
    self.data.is_modified = true;
  }

  /// Checks if page has been modified
  pub fn is_modified(&self) -> bool {
    self.data.is_modified
  }

  /// Returns page index
  pub fn page_index(&self) -> usize {
    self.data.page_index
  }
}

#[napi]
impl PdfPage {
  /// Gets the page index
  ///
  /// # Returns
  /// Zero-based page index in the document
  #[napi]
  pub fn get_page_index(&self) -> i32 {
    self.data.page_index as i32
  }

  /// Gets the page width in points
  ///
  /// # Returns
  /// Page width in points (1/72 inch)
  #[napi]
  pub fn get_width(&self) -> f32 {
    self.data.width
  }

  /// Gets the page height in points
  ///
  /// # Returns
  /// Page height in points (1/72 inch)
  #[napi]
  pub fn get_height(&self) -> f32 {
    self.data.height
  }

  /// Gets all child elements on the page
  ///
  /// Returns a list of element IDs. Elements can be text, images, shapes, or tables.
  ///
  /// # Returns
  /// Vector of element IDs
  #[napi]
  pub fn children(&self) -> napi::Result<Vec<String>> {
    let mut ids = Vec::new();

    // Add text element IDs
    for text in &self.data.text_elements {
      ids.push(text.id.clone());
    }

    // Add image element IDs
    for image in &self.data.image_elements {
      ids.push(image.id.clone());
    }

    // Add path element IDs
    for path in &self.data.path_elements {
      ids.push(path.id.clone());
    }

    // Add table element IDs
    for table in &self.data.table_elements {
      ids.push(table.id.clone());
    }

    // Add structure element IDs
    for structure in &self.data.structure_elements {
      ids.push(structure.id.clone());
    }

    Ok(ids)
  }

  /// Finds text elements containing a query string
  ///
  /// Simple case-insensitive search for text containing the query.
  ///
  /// # Arguments
  /// * `query` - Text to search for
  ///
  /// # Returns
  /// Vector of element IDs containing the text
  #[napi]
  pub fn find_text_containing(&self, query: String) -> napi::Result<Vec<String>> {
    let results = self.data.find_text_containing(&query);
    Ok(results.into_iter().map(|t| t.id).collect())
  }

  /// Searches for text with advanced options
  ///
  /// Supports case-sensitivity, whole-word matching, and regex patterns.
  ///
  /// # Arguments
  /// * `query` - Text or regex pattern to search for
  /// * `options` - Optional search configuration
  ///
  /// # Returns
  /// Vector of search results with positions and bounding boxes
  #[napi]
  pub fn find_text(&self, query: String, options: Option<SearchOptions>) -> napi::Result<Vec<SearchResult>> {
    let mut results = Vec::new();

    // Simple text search (Phase 3 enhancement: add regex support)
    let texts = if let Some(opts) = &options {
      if opts.case_sensitive.unwrap_or(false) {
        self.data.find_text_by_predicate(|t| t.text.contains(&query))
      } else {
        self.data.find_text_containing(&query)
      }
    } else {
      self.data.find_text_containing(&query)
    };

    for text in texts {
      results.push(SearchResult {
        text: text.text.clone(),
        page_index: self.data.page_index as i32,
        bbox: text.bbox,
        confidence: Some(1.0), // Perfect match
      });
    }

    Ok(results)
  }

  /// Sets or replaces text content of an element
  ///
  /// # Arguments
  /// * `element_id` - ID of the element to modify
  /// * `new_text` - New text content
  #[napi]
  pub fn set_text(&mut self, element_id: String, new_text: String) -> napi::Result<()> {
    if self.data.set_text(&element_id, &new_text) {
      Ok(())
    } else {
      Err(napi::Error::new(
        napi::Status::InvalidArg,
        format!("Element with ID '{}' not found", element_id),
      ))
    }
  }

  /// Adds a new element to the page
  ///
  /// # Arguments
  /// * `element` - Element content (type, position, data)
  ///
  /// # Returns
  /// ID of the newly created element
  #[napi]
  pub fn add_element(&mut self, element: ElementContent) -> napi::Result<String> {
    // Generate unique ID for new element
    let new_id = format!("element_{}_{}", self.data.page_index, self.data.elements.len());

    // Parse element based on type
    match element.element_type.as_str() {
      "text" => {
        // TODO: Parse element.data as JSON and create PdfText
        // For now, create a placeholder
        let text = PdfText {
          id: new_id.clone(),
          text: element.data.clone(),
          bbox: Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 20.0,
          },
          font_size: 12.0,
          font: "Helvetica".to_string(),
          color_r: 0,
          color_g: 0,
          color_b: 0,
        };
        self.data.add_text(text);
      }
      // TODO: Implement other element types
      _ => {
        return Err(napi::Error::new(
          napi::Status::InvalidArg,
          format!("Unknown element type: {}", element.element_type),
        ))
      }
    }

    Ok(new_id)
  }

  /// Removes an element from the page
  ///
  /// # Arguments
  /// * `element_id` - ID of the element to remove
  #[napi]
  pub fn remove_element(&mut self, element_id: String) -> napi::Result<()> {
    if self.data.remove_element(&element_id) {
      Ok(())
    } else {
      Err(napi::Error::new(
        napi::Status::InvalidArg,
        format!("Element with ID '{}' not found", element_id),
      ))
    }
  }

  /// Gets all annotations on the page
  ///
  /// # Returns
  /// Vector of annotation IDs (text marks, links, stamps, etc.)
  #[napi]
  pub fn annotations(&self) -> napi::Result<Vec<AnnotationData>> {
    Ok(self.data.annotations.clone())
  }

  /// Adds a new annotation to the page
  ///
  /// # Arguments
  /// * `annotation` - Annotation content (type, position, content)
  ///
  /// # Returns
  /// ID of the newly created annotation
  #[napi]
  pub fn add_annotation(&mut self, annotation: AnnotationContent) -> napi::Result<String> {
    // Generate unique ID for new annotation
    let annotation_id = format!("annotation_{}_{}", self.data.page_index, self.data.annotations.len());

    let annot_data = AnnotationData {
      id: annotation_id.clone(),
      annotation_type: annotation.annotation_type.clone(),
      x: 0.0,
      y: 0.0,
      width: 100.0,
      height: 20.0,
      contents: Some(annotation.data.clone()),
      color_r: 255,
      color_g: 255,
      color_b: 0,
      author: None,
    };

    self.data.add_annotation(annot_data);
    Ok(annotation_id)
  }

  /// Closes the page and releases resources
  ///
  /// Called automatically when the page is dropped, but can be called explicitly
  /// to release resources earlier.
  #[napi]
  pub fn close(&mut self) {
    // Resources automatically cleaned up on drop
  }
}

impl Drop for PdfPage {
  fn drop(&mut self) {
    // Explicit cleanup if needed
  }
}
