use napi_derive::napi;
use crate::types::{SearchOptions, SearchResult, Rect};

/// Text search result wrapper
#[napi]
#[derive(Clone, Debug)]
pub struct TextSearchResult {
  /// Matched text
  pub text: String,
  /// Page number (0-indexed)
  pub page_index: i32,
  /// Position on page (x, y, width, height)
  pub bbox: Rect,
  /// Start index in page text
  pub start_index: i32,
  /// End index in page text
  pub end_index: i32,
  /// Confidence score (0.0 to 1.0)
  pub confidence: Option<f32>,
}

/// Text search functionality for documents
#[napi]
#[derive(Clone, Debug)]
pub struct TextSearcher {
  /// Search pattern (can be regex)
  pub pattern: String,
  /// Search options
  pub case_sensitive: bool,
  pub whole_words: bool,
  pub use_regex: bool,
  /// Maximum number of results to return
  pub max_results: i32,
}

#[napi]
impl TextSearcher {
  /// Creates a new text searcher with the given pattern
  #[napi]
  pub fn new(pattern: String) -> Self {
    TextSearcher {
      pattern,
      case_sensitive: false,
      whole_words: false,
      use_regex: false,
      max_results: 1000,
    }
  }

  /// Enables case-sensitive search
  #[napi]
  pub fn case_sensitive(mut self) -> Self {
    self.case_sensitive = true;
    self
  }

  /// Enables whole-word matching
  #[napi]
  pub fn whole_words(mut self) -> Self {
    self.whole_words = true;
    self
  }

  /// Enables regex pattern matching
  #[napi]
  pub fn use_regex(mut self) -> Self {
    self.use_regex = true;
    self
  }

  /// Sets maximum results limit
  #[napi]
  pub fn max_results(mut self, limit: i32) -> Self {
    self.max_results = limit;
    self
  }

  /// Gets the search pattern
  #[napi]
  pub fn get_pattern(&self) -> String {
    self.pattern.clone()
  }

  /// Checks if search is case-sensitive
  #[napi]
  pub fn is_case_sensitive(&self) -> bool {
    self.case_sensitive
  }

  /// Checks if whole-word matching is enabled
  #[napi]
  pub fn is_whole_words(&self) -> bool {
    self.whole_words
  }

  /// Checks if regex is enabled
  #[napi]
  pub fn is_regex(&self) -> bool {
    self.use_regex
  }

  /// Performs text search on content
  #[napi]
  pub fn search(&self, text: String, options: Option<SearchOptions>) -> napi::Result<Vec<SearchResult>> {
    let mut results = Vec::new();

    let case_sensitive = options.as_ref().and_then(|o| o.case_sensitive).unwrap_or(false);

    let search_pattern = if case_sensitive {
      self.pattern.clone()
    } else {
      self.pattern.to_lowercase()
    };

    let content_to_search = if case_sensitive {
      text.clone()
    } else {
      text.to_lowercase()
    };

    // Simple substring search (regex support in Phase 5)
    let mut start_pos = 0;
    let pattern_len = self.pattern.len();
    let mut count = 0;

    while let Some(pos) = content_to_search[start_pos..].find(&search_pattern) {
      if count >= self.max_results {
        break;
      }

      let actual_pos = start_pos + pos;
      results.push(SearchResult {
        text: text[actual_pos..std::cmp::min(actual_pos + pattern_len, text.len())].to_string(),
        page_index: 0,
        bbox: Rect {
          x: 0.0,
          y: 0.0,
          width: 100.0,
          height: 20.0,
        },
        confidence: Some(1.0),
      });

      start_pos = actual_pos + pattern_len;
      count += 1;
    }

    Ok(results)
  }
}
