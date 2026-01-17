use napi_derive::napi;
use crate::types::{Rect, SearchOptions, SearchResult};
use crate::elements::{PdfElement, PdfText, PdfImage, PdfPath, PdfTable, PdfStructure};

/// Internal page data holder for DOM access
///
/// Stores a snapshot of page elements and annotations for in-memory manipulation.
/// This allows JavaScript to work with page data without holding references to the Rust document.
#[derive(Clone, Debug)]
pub struct PageData {
  /// Page index in the document
  pub page_index: usize,
  /// Page width in points
  pub width: f32,
  /// Page height in points
  pub height: f32,
  /// Elements on the page
  pub elements: Vec<PdfElement>,
  /// Text elements (extracted for faster search)
  pub text_elements: Vec<PdfText>,
  /// Image elements
  pub image_elements: Vec<PdfImage>,
  /// Path/graphics elements
  pub path_elements: Vec<PdfPath>,
  /// Table elements
  pub table_elements: Vec<PdfTable>,
  /// Structure elements
  pub structure_elements: Vec<PdfStructure>,
  /// Annotation data
  pub annotations: Vec<AnnotationData>,
  /// Whether the page has been modified
  pub is_modified: bool,
}

/// Annotation data for storage and manipulation
#[napi]
#[derive(Clone, Debug)]
pub struct AnnotationData {
  /// Unique annotation identifier
  pub id: String,
  /// Annotation type: "text", "highlight", "link", "freetext", "ink", "stamp", "popup"
  pub annotation_type: String,
  /// Position of annotation
  pub x: f32,
  pub y: f32,
  pub width: f32,
  pub height: f32,
  /// Annotation content
  pub contents: Option<String>,
  /// Color (RGB, 0-255 each)
  pub color_r: u8,
  pub color_g: u8,
  pub color_b: u8,
  /// Author of annotation
  pub author: Option<String>,
}

impl PageData {
  /// Creates a new empty page data structure
  pub fn new(page_index: usize, width: f32, height: f32) -> Self {
    PageData {
      page_index,
      width,
      height,
      elements: Vec::new(),
      text_elements: Vec::new(),
      image_elements: Vec::new(),
      path_elements: Vec::new(),
      table_elements: Vec::new(),
      structure_elements: Vec::new(),
      annotations: Vec::new(),
      is_modified: false,
    }
  }

  /// Adds a text element to the page
  pub fn add_text(&mut self, text: PdfText) {
    self.text_elements.push(text.clone());
    self.elements.push(PdfElement::Text);
    self.is_modified = true;
  }

  /// Adds an image element to the page
  pub fn add_image(&mut self, image: PdfImage) {
    self.image_elements.push(image.clone());
    self.elements.push(PdfElement::Image);
    self.is_modified = true;
  }

  /// Finds text elements containing a query string (case-insensitive)
  pub fn find_text_containing(&self, query: &str) -> Vec<PdfText> {
    let query_lower = query.to_lowercase();
    self.text_elements
      .iter()
      .filter(|t| t.text.to_lowercase().contains(&query_lower))
      .cloned()
      .collect()
  }

  /// Finds text elements matching a predicate
  pub fn find_text_by_predicate<F>(&self, predicate: F) -> Vec<PdfText>
  where
    F: Fn(&PdfText) -> bool,
  {
    self.text_elements
      .iter()
      .filter(|t| predicate(t))
      .cloned()
      .collect()
  }

  /// Finds elements in a specific region
  pub fn find_in_region(&self, region: Rect) -> Vec<PdfElement> {
    self.elements
      .iter()
      .enumerate()
      .filter_map(|(_, _elem)| {
        // TODO: Filter by bbox when element bbox is available
        // For now, return empty
        None::<PdfElement>
      })
      .collect()
  }

  /// Finds images on the page
  pub fn find_images(&self) -> Vec<PdfImage> {
    self.image_elements.clone()
  }

  /// Finds paths/graphics on the page
  pub fn find_paths(&self) -> Vec<PdfPath> {
    self.path_elements.clone()
  }

  /// Finds tables on the page
  pub fn find_tables(&self) -> Vec<PdfTable> {
    self.table_elements.clone()
  }

  /// Finds structure elements on the page
  pub fn find_structures(&self) -> Vec<PdfStructure> {
    self.structure_elements.clone()
  }

  /// Modifies text with given ID
  pub fn set_text(&mut self, element_id: &str, new_text: &str) -> bool {
    for text in &mut self.text_elements {
      if text.id == element_id {
        text.text = new_text.to_string();
        self.is_modified = true;
        return true;
      }
    }
    false
  }

  /// Removes element by ID
  pub fn remove_element(&mut self, element_id: &str) -> bool {
    let initial_text_len = self.text_elements.len();
    self.text_elements.retain(|t| t.id != element_id);
    if self.text_elements.len() < initial_text_len {
      self.is_modified = true;
      return true;
    }

    let initial_image_len = self.image_elements.len();
    self.image_elements.retain(|i| i.id != element_id);
    if self.image_elements.len() < initial_image_len {
      self.is_modified = true;
      return true;
    }

    let initial_path_len = self.path_elements.len();
    self.path_elements.retain(|p| p.id != element_id);
    if self.path_elements.len() < initial_path_len {
      self.is_modified = true;
      return true;
    }

    let initial_table_len = self.table_elements.len();
    self.table_elements.retain(|t| t.id != element_id);
    if self.table_elements.len() < initial_table_len {
      self.is_modified = true;
      return true;
    }

    let initial_struct_len = self.structure_elements.len();
    self.structure_elements.retain(|s| s.id != element_id);
    if self.structure_elements.len() < initial_struct_len {
      self.is_modified = true;
      return true;
    }

    false
  }

  /// Adds an annotation to the page
  pub fn add_annotation(&mut self, annotation: AnnotationData) -> String {
    let id = annotation.id.clone();
    self.annotations.push(annotation);
    self.is_modified = true;
    id
  }

  /// Removes annotation by ID
  pub fn remove_annotation(&mut self, annotation_id: &str) -> bool {
    let initial_len = self.annotations.len();
    self.annotations.retain(|a| a.id != annotation_id);
    if self.annotations.len() < initial_len {
      self.is_modified = true;
      true
    } else {
      false
    }
  }

  /// Gets all children elements
  pub fn children(&self) -> Vec<PdfElement> {
    self.elements.clone()
  }

  /// Returns all elements as a flat list
  pub fn all_elements(&self) -> Vec<PdfElement> {
    let mut all = Vec::new();
    all.extend(std::iter::repeat(PdfElement::Text).take(self.text_elements.len()));
    all.extend(std::iter::repeat(PdfElement::Image).take(self.image_elements.len()));
    all.extend(std::iter::repeat(PdfElement::Path).take(self.path_elements.len()));
    all.extend(std::iter::repeat(PdfElement::Table).take(self.table_elements.len()));
    all.extend(std::iter::repeat(PdfElement::Structure).take(self.structure_elements.len()));
    all
  }
}
