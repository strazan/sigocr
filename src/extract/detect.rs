use crate::error::{spawn_blocking_napi, SigocrError};
use crate::extract::pdf::open_doc;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::path::Path;

/// Fast check: does this PDF have embedded text?
/// Returns true if at least one page has extractable text characters.
#[napi]
pub async fn has_embedded_text(path: String) -> Result<bool> {
    spawn_blocking_napi(move || has_embedded_text_sync(&path)).await
}

#[napi]
pub async fn has_embedded_text_buffer(data: Buffer) -> Result<bool> {
    spawn_blocking_napi(move || has_embedded_text_bytes(&data)).await
}

pub fn has_embedded_text_sync(path: &str) -> Result<bool> {
    let file_path = Path::new(path);
    if !file_path.exists() {
        return Err(SigocrError::NotFound(path.to_owned()).into());
    }

    let data = std::fs::read(file_path).map_err(SigocrError::Io)?;

    has_embedded_text_bytes(&data)
}

fn has_embedded_text_bytes(data: &[u8]) -> Result<bool> {
    let mut doc = open_doc(data)?;

    let page_count = doc
        .page_count()
        .map_err(|e| SigocrError::Pdf(e.to_string()))?;

    // Check first few pages for text (don't need to check all for detection)
    let pages_to_check = page_count.min(3);
    for i in 0..pages_to_check {
        let text = doc
            .extract_text(i)
            .map_err(|e| SigocrError::Pdf(e.to_string()))?;
        if !text.trim().is_empty() {
            return Ok(true);
        }
    }

    Ok(false)
}
