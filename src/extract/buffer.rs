use crate::error::{spawn_blocking_napi, SigocrError};
use crate::extract::pdf::{get_page_dimensions, open_doc};
use crate::layout::paragraphs::group_into_paragraphs;
use crate::layout::reading_order::{group_into_lines, PositionedChar};
use crate::types::{Document, Page, Paragraph};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rayon::prelude::*;

/// Minimum number of non-whitespace characters to consider a page as having text.
const MIN_TEXT_CHARS: usize = 10;

/// Minimum pages before we use parallel extraction.
const PARALLEL_THRESHOLD: usize = 20;

/// Extract structured text from a PDF buffer.
/// Returns null if the PDF has no embedded text (scanned/image PDF).
#[napi]
pub async fn extract_pdf_buffer(data: Buffer) -> Result<Option<Document>> {
    spawn_blocking_napi(move || extract_from_bytes(&data)).await
}

/// Result from extracting a chunk of pages.
struct ChunkResult {
    pages: Vec<Page>,
    paragraphs: Vec<Paragraph>,
    content: String,
    has_text: bool,
}

fn page_count(data: &[u8]) -> Result<usize> {
    let mut doc = open_doc(data)?;
    doc.page_count()
        .map_err(|e| SigocrError::Pdf(e.to_string()).into())
}

fn chunks_to_document(chunk_results: Vec<ChunkResult>) -> Option<Document> {
    let mut full_content = String::new();
    let mut content_offset: usize = 0;
    let mut all_pages: Vec<Page> = Vec::new();
    let mut all_paragraphs: Vec<Paragraph> = Vec::new();
    let mut has_any_text = false;

    for chunk in chunk_results {
        all_pages.extend(chunk.pages);

        if !chunk.has_text {
            continue;
        }
        has_any_text = true;

        if !full_content.is_empty() && !chunk.content.is_empty() {
            full_content.push('\n');
            content_offset += 1;
        }

        let base_offset = content_offset;
        content_offset += chunk.content.encode_utf16().count();
        full_content.push_str(&chunk.content);

        for mut para in chunk.paragraphs {
            for span in &mut para.spans {
                span.offset += base_offset as i32;
            }
            all_paragraphs.push(para);
        }
    }

    if !has_any_text {
        return None;
    }

    Some(Document {
        content: full_content,
        pages: all_pages,
        paragraphs: all_paragraphs,
        tables: Vec::new(),
    })
}

/// Always-sequential extraction, used by batch `extract_pdfs` to avoid nested rayon.
pub fn extract_from_bytes_sequential(data: &[u8]) -> Result<Option<Document>> {
    let count = page_count(data)?;
    if count == 0 {
        return Ok(None);
    }
    Ok(chunks_to_document(vec![extract_chunk(data, 0, count)]))
}

pub fn extract_from_bytes(data: &[u8]) -> Result<Option<Document>> {
    let count = page_count(data)?;
    if count == 0 {
        return Ok(None);
    }

    // Small PDFs: sequential (no re-parse overhead)
    if count <= PARALLEL_THRESHOLD {
        return Ok(chunks_to_document(vec![extract_chunk(data, 0, count)]));
    }

    // Large PDFs: split into chunks, each thread re-opens the PDF
    let num_threads = rayon::current_num_threads();
    let chunk_size = (count / num_threads).max(10);
    let chunks: Vec<(usize, usize)> = (0..count)
        .step_by(chunk_size)
        .map(|start| (start, (start + chunk_size).min(count)))
        .collect();

    let data_vec = data.to_vec();
    let chunk_results: Vec<ChunkResult> = chunks
        .par_iter()
        .map(|&(start, end)| extract_chunk(&data_vec, start, end))
        .collect();

    Ok(chunks_to_document(chunk_results))
}

/// Extract a range of pages [start, end) from the PDF bytes.
fn extract_chunk(data: &[u8], start: usize, end: usize) -> ChunkResult {
    let mut doc = match open_doc(data) {
        Ok(d) => d,
        Err(_) => {
            return ChunkResult {
                pages: Vec::new(),
                paragraphs: Vec::new(),
                content: String::new(),
                has_text: false,
            };
        }
    };

    let mut chunk_content = String::new();
    let mut chunk_offset: usize = 0;
    let mut pages: Vec<Page> = Vec::new();
    let mut paragraphs: Vec<Paragraph> = Vec::new();
    let mut has_text = false;

    for page_idx in start..end {
        let page_number = (page_idx + 1) as i32;
        let (width, height) = get_page_dimensions(&mut doc, page_idx);

        pages.push(Page {
            page_number,
            width,
            height,
        });

        let chars = match doc.extract_chars(page_idx) {
            Ok(chars) => chars,
            Err(_) => continue,
        };

        let positioned: Vec<PositionedChar> = chars
            .into_iter()
            .filter_map(|c| {
                if c.char.is_control() && c.char != '\n' && c.char != '\t' {
                    return None;
                }
                Some(PositionedChar {
                    ch: c.char,
                    x: c.bbox.x as f64,
                    y: c.bbox.y as f64,
                    width: if c.bbox.width > 0.0 {
                        c.bbox.width as f64
                    } else {
                        c.font_size as f64 * 0.6
                    },
                    height: if c.bbox.height > 0.0 {
                        c.bbox.height as f64
                    } else {
                        c.font_size as f64
                    },
                    font_size: c.font_size as f64,
                    font_name: c.font_name.clone(),
                    page_number,
                })
            })
            .collect();

        if positioned.len() < MIN_TEXT_CHARS {
            continue;
        }

        has_text = true;
        let lines = group_into_lines(positioned);
        let page_paragraphs = group_into_paragraphs(
            lines,
            page_number,
            height,
            &mut chunk_offset,
            &mut chunk_content,
        );
        paragraphs.extend(page_paragraphs);
    }

    ChunkResult {
        pages,
        paragraphs,
        content: chunk_content,
        has_text,
    }
}
