use napi::bindgen_prelude::{Buffer, Result as NapiResult};

/// Convert Rust String to napi string
pub fn string_to_napi(s: String) -> String {
  s
}

/// Convert napi buffer to Vec<u8>
pub fn buffer_to_vec(buffer: Buffer) -> Vec<u8> {
  buffer.to_vec()
}

/// Convert Vec<u8> to napi buffer
pub fn vec_to_buffer(vec: Vec<u8>) -> Buffer {
  Buffer::from(vec)
}

/// Helper for Result mapping
pub fn map_result<T>(result: std::result::Result<T, pdf_oxide::error::Error>) -> NapiResult<T> {
  result.map_err(|e| crate::errors::map_error(e))
}
