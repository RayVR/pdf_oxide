use napi_derive::napi;

/// XMP Metadata - Extensible Metadata Platform (Section 14.3)
#[napi]
#[derive(Clone, Debug)]
pub struct XMPMetadata {
  /// Document title
  pub title: Option<String>,
  /// Document author
  pub author: Option<String>,
  /// Document subject
  pub subject: Option<String>,
  /// Keywords associated with document
  pub keywords: Option<String>,
  /// Document creator (application name)
  pub creator: Option<String>,
  /// Document creation date (ISO 8601 format)
  pub created: Option<String>,
  /// Document modification date (ISO 8601 format)
  pub modified: Option<String>,
  /// Copyright notice
  pub copyright: Option<String>,
  /// Producer (PDF creation tool)
  pub producer: Option<String>,
  /// Language code (e.g., "en", "fr", "de")
  pub language: Option<String>,
  /// Custom description
  pub description: Option<String>,
  /// Rights management information
  pub rights: Option<String>,
  /// Contributor names
  pub contributors: Option<Vec<String>>,
  /// Format (e.g., "application/pdf")
  pub format: Option<String>,
  /// Unique document identifier
  pub identifier: Option<String>,
  /// Custom source reference
  pub source: Option<String>,
  /// Relation to other documents
  pub relation: Option<String>,
  /// Rating or review information
  pub coverage: Option<String>,
  /// XMP raw XML (for advanced metadata)
  pub raw_xml: Option<String>,
}

/// Page Labels - Section 12.4.2
#[napi]
#[derive(Clone, Debug)]
pub struct PageLabel {
  /// Page label index (0-based)
  pub page_index: i32,
  /// Label style: decimal, roman, letters, etc.
  pub style: Option<String>,
  /// Prefix string (e.g., "Chapter")
  pub prefix: Option<String>,
  /// Starting value for numbering
  pub start_value: Option<i32>,
}

/// Embedded File - Referenced file within PDF
#[napi]
#[derive(Clone, Debug)]
pub struct EmbeddedFile {
  /// Unique file identifier
  pub id: String,
  /// File name
  pub filename: String,
  /// File description
  pub description: Option<String>,
  /// MIME type
  pub mime_type: String,
  /// File size in bytes
  pub size: i32,
  /// Creation date (ISO 8601)
  pub creation_date: Option<String>,
  /// Modification date (ISO 8601)
  pub modification_date: Option<String>,
  /// Access date (ISO 8601)
  pub access_date: Option<String>,
  /// Optional file data (base64 encoded for binary files)
  pub data: Option<String>,
}

/// Document Information Dictionary - Basic metadata
#[napi]
#[derive(Clone, Debug)]
pub struct DocumentInfo {
  /// PDF version (e.g., "1.7")
  pub version: String,
  /// Title from document information dictionary
  pub title: Option<String>,
  /// Author from document information dictionary
  pub author: Option<String>,
  /// Subject from document information dictionary
  pub subject: Option<String>,
  /// Keywords from document information dictionary
  pub keywords: Option<String>,
  /// Creator application
  pub creator: Option<String>,
  /// Producer application
  pub producer: Option<String>,
  /// Creation date (ISO 8601)
  pub created: Option<String>,
  /// Modification date (ISO 8601)
  pub modified: Option<String>,
  /// Whether document is encrypted
  pub is_encrypted: bool,
  /// Encryption algorithm
  pub encryption_algorithm: Option<String>,
}

#[napi]
impl XMPMetadata {
  /// Creates new empty XMP metadata
  #[napi]
  pub fn new() -> Self {
    XMPMetadata {
      title: None,
      author: None,
      subject: None,
      keywords: None,
      creator: None,
      created: None,
      modified: None,
      copyright: None,
      producer: None,
      language: None,
      description: None,
      rights: None,
      contributors: None,
      format: None,
      identifier: None,
      source: None,
      relation: None,
      coverage: None,
      raw_xml: None,
    }
  }

  /// Sets title
  #[napi]
  pub fn set_title(&mut self, title: String) {
    self.title = Some(title);
  }

  /// Gets title
  #[napi]
  pub fn get_title(&self) -> Option<String> {
    self.title.clone()
  }

  /// Sets author
  #[napi]
  pub fn set_author(&mut self, author: String) {
    self.author = Some(author);
  }

  /// Gets author
  #[napi]
  pub fn get_author(&self) -> Option<String> {
    self.author.clone()
  }

  /// Sets subject
  #[napi]
  pub fn set_subject(&mut self, subject: String) {
    self.subject = Some(subject);
  }

  /// Gets subject
  #[napi]
  pub fn get_subject(&self) -> Option<String> {
    self.subject.clone()
  }

  /// Sets keywords
  #[napi]
  pub fn set_keywords(&mut self, keywords: String) {
    self.keywords = Some(keywords);
  }

  /// Gets keywords
  #[napi]
  pub fn get_keywords(&self) -> Option<String> {
    self.keywords.clone()
  }

  /// Sets creator application name
  #[napi]
  pub fn set_creator(&mut self, creator: String) {
    self.creator = Some(creator);
  }

  /// Gets creator
  #[napi]
  pub fn get_creator(&self) -> Option<String> {
    self.creator.clone()
  }

  /// Sets copyright notice
  #[napi]
  pub fn set_copyright(&mut self, copyright: String) {
    self.copyright = Some(copyright);
  }

  /// Gets copyright
  #[napi]
  pub fn get_copyright(&self) -> Option<String> {
    self.copyright.clone()
  }

  /// Sets language code
  #[napi]
  pub fn set_language(&mut self, language: String) {
    self.language = Some(language);
  }

  /// Gets language
  #[napi]
  pub fn get_language(&self) -> Option<String> {
    self.language.clone()
  }

  /// Gets all non-None metadata as key-value pairs
  #[napi]
  pub fn to_map(&self) -> napi::Result<Vec<(String, String)>> {
    let mut map = Vec::new();

    if let Some(ref title) = self.title {
      map.push(("title".to_string(), title.clone()));
    }
    if let Some(ref author) = self.author {
      map.push(("author".to_string(), author.clone()));
    }
    if let Some(ref subject) = self.subject {
      map.push(("subject".to_string(), subject.clone()));
    }
    if let Some(ref keywords) = self.keywords {
      map.push(("keywords".to_string(), keywords.clone()));
    }
    if let Some(ref creator) = self.creator {
      map.push(("creator".to_string(), creator.clone()));
    }
    if let Some(ref created) = self.created {
      map.push(("created".to_string(), created.clone()));
    }
    if let Some(ref modified) = self.modified {
      map.push(("modified".to_string(), modified.clone()));
    }
    if let Some(ref copyright) = self.copyright {
      map.push(("copyright".to_string(), copyright.clone()));
    }
    if let Some(ref producer) = self.producer {
      map.push(("producer".to_string(), producer.clone()));
    }
    if let Some(ref language) = self.language {
      map.push(("language".to_string(), language.clone()));
    }
    if let Some(ref description) = self.description {
      map.push(("description".to_string(), description.clone()));
    }

    Ok(map)
  }

  /// Checks if metadata is empty (all fields None)
  #[napi]
  pub fn is_empty(&self) -> bool {
    self.title.is_none()
      && self.author.is_none()
      && self.subject.is_none()
      && self.keywords.is_none()
      && self.creator.is_none()
      && self.created.is_none()
      && self.modified.is_none()
      && self.copyright.is_none()
      && self.producer.is_none()
      && self.language.is_none()
      && self.description.is_none()
      && self.rights.is_none()
      && self.contributors.is_none()
      && self.format.is_none()
      && self.identifier.is_none()
      && self.source.is_none()
      && self.relation.is_none()
      && self.coverage.is_none()
      && self.raw_xml.is_none()
  }
}

#[napi]
impl PageLabel {
  /// Creates new page label
  #[napi]
  pub fn new(page_index: i32) -> Self {
    PageLabel {
      page_index,
      style: None,
      prefix: None,
      start_value: None,
    }
  }

  /// Sets label style (decimal, roman, letters, uppercase, lowercase)
  #[napi]
  pub fn set_style(&mut self, style: String) {
    self.style = Some(style);
  }

  /// Sets label prefix
  #[napi]
  pub fn set_prefix(&mut self, prefix: String) {
    self.prefix = Some(prefix);
  }

  /// Sets starting value for numbering
  #[napi]
  pub fn set_start_value(&mut self, value: i32) {
    self.start_value = Some(value);
  }

  /// Gets full label text (prefix + numbered value if applicable)
  #[napi]
  pub fn get_label_text(&self) -> String {
    let mut text = String::new();

    if let Some(ref prefix) = self.prefix {
      text.push_str(prefix);
    }

    if let Some(ref style) = self.style {
      let number = self.start_value.unwrap_or(self.page_index + 1);
      match style.as_str() {
        "decimal" => text.push_str(&number.to_string()),
        "roman" => text.push_str(&Self::to_roman(number)),
        "uppercase_roman" => text.push_str(&Self::to_roman(number).to_uppercase()),
        "letters" => text.push_str(&Self::to_letters(number, false)),
        "uppercase_letters" => text.push_str(&Self::to_letters(number, true)),
        "lowercase" => text.push_str(&Self::to_letters(number, false)),
        "uppercase" => text.push_str(&Self::to_letters(number, true)),
        _ => text.push_str(&number.to_string()),
      }
    }

    if text.is_empty() {
      text.push_str(&(self.page_index + 1).to_string());
    }

    text
  }

  /// Converts number to Roman numerals (I, II, III, IV, V, ...)
  fn to_roman(mut num: i32) -> String {
    let values = vec![
      (1000, "m"),
      (900, "cm"),
      (500, "d"),
      (400, "cd"),
      (100, "c"),
      (90, "xc"),
      (50, "l"),
      (40, "xl"),
      (10, "x"),
      (9, "ix"),
      (5, "v"),
      (4, "iv"),
      (1, "i"),
    ];

    let mut result = String::new();
    for (value, numeral) in values {
      while num >= value {
        result.push_str(numeral);
        num -= value;
      }
    }
    result
  }

  /// Converts number to letters (a, b, c, ..., z, aa, ab, ...)
  fn to_letters(mut num: i32, uppercase: bool) -> String {
    let mut result = String::new();
    let base = if uppercase { 'A' } else { 'a' };

    while num > 0 {
      let remainder = ((num - 1) % 26) as u8;
      result.insert(0, (base as u8 + remainder) as char);
      num = (num - 1) / 26;
    }

    result
  }
}

#[napi]
impl EmbeddedFile {
  /// Creates new embedded file reference
  #[napi]
  pub fn new(id: String, filename: String, mime_type: String, size: i32) -> Self {
    EmbeddedFile {
      id,
      filename,
      description: None,
      mime_type,
      size,
      creation_date: None,
      modification_date: None,
      access_date: None,
      data: None,
    }
  }

  /// Sets file description
  #[napi]
  pub fn set_description(&mut self, description: String) {
    self.description = Some(description);
  }

  /// Gets file description
  #[napi]
  pub fn get_description(&self) -> Option<String> {
    self.description.clone()
  }

  /// Sets creation date (ISO 8601 format)
  #[napi]
  pub fn set_creation_date(&mut self, date: String) {
    self.creation_date = Some(date);
  }

  /// Gets creation date
  #[napi]
  pub fn get_creation_date(&self) -> Option<String> {
    self.creation_date.clone()
  }

  /// Checks if file data is available
  #[napi]
  pub fn has_data(&self) -> bool {
    self.data.is_some()
  }

  /// Gets file data (base64 encoded)
  #[napi]
  pub fn get_data(&self) -> Option<String> {
    self.data.clone()
  }

  /// Sets file data (should be base64 encoded for binary files)
  #[napi]
  pub fn set_data(&mut self, data: String) {
    self.data = Some(data);
  }
}

#[napi]
impl DocumentInfo {
  /// Creates new document info
  #[napi]
  pub fn new(version: String) -> Self {
    DocumentInfo {
      version,
      title: None,
      author: None,
      subject: None,
      keywords: None,
      creator: None,
      producer: None,
      created: None,
      modified: None,
      is_encrypted: false,
      encryption_algorithm: None,
    }
  }

  /// Sets title
  #[napi]
  pub fn set_title(&mut self, title: String) {
    self.title = Some(title);
  }

  /// Gets all metadata as JSON-serializable object
  #[napi]
  pub fn to_summary(&self) -> String {
    format!(
      "{{\"version\": \"{}\", \"title\": {:?}, \"author\": {:?}, \"subject\": {:?}, \"encrypted\": {}}}",
      self.version, self.title, self.author, self.subject, self.is_encrypted
    )
  }
}
