use crate::error::{spawn_blocking_napi, SigocrError};
use crate::extract::buffer::{extract_from_bytes, extract_from_bytes_sequential};
use crate::types::Document;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rayon::prelude::*;
use std::path::Path;

/// Extract structured text from a PDF file.
/// Returns null if the PDF has no embedded text (scanned/image PDF).
#[napi]
pub async fn extract_pdf(path: String) -> Result<Option<Document>> {
    spawn_blocking_napi(move || {
        let data = read_pdf(&path)?;
        extract_from_bytes(&data)
    })
    .await
}

/// Batch extract from multiple PDF files in parallel.
/// Rayon distributes files across cores. Each file is extracted sequentially
/// internally to avoid nested parallelism contention.
#[napi]
pub async fn extract_pdfs(paths: Vec<String>) -> Result<Vec<Option<Document>>> {
    spawn_blocking_napi(move || {
        Ok(paths
            .par_iter()
            .map(|p| {
                let data = match read_pdf(p) {
                    Ok(d) => d,
                    Err(_) => return None,
                };
                extract_from_bytes_sequential(&data).unwrap_or(None)
            })
            .collect())
    })
    .await
}

fn read_pdf(path: &str) -> Result<Vec<u8>> {
    let file_path = Path::new(path);
    if !file_path.exists() {
        return Err(SigocrError::NotFound(path.to_owned()).into());
    }
    std::fs::read(file_path).map_err(|e| SigocrError::Io(e).into())
}
