use crate::error::SigocrError;
use pdf_oxide::object::Object;

pub fn open_doc(data: &[u8]) -> napi::Result<pdf_oxide::PdfDocument> {
    pdf_oxide::PdfDocument::open_from_bytes(data.to_vec())
        .map_err(|e| SigocrError::Pdf(e.to_string()).into())
}

pub fn get_page_dimensions(doc: &mut pdf_oxide::PdfDocument, page_idx: usize) -> (f64, f64) {
    if let Ok(page_obj) = doc.get_page_for_debug(page_idx) {
        if let Some(media_box) = extract_media_box(&page_obj) {
            return media_box;
        }
    }
    (612.0, 792.0)
}

fn extract_media_box(page_obj: &Object) -> Option<(f64, f64)> {
    match page_obj {
        Object::Dictionary(dict) => {
            if let Some(Object::Array(arr)) = dict.get("MediaBox") {
                if arr.len() >= 4 {
                    let x0 = obj_to_f64(&arr[0]).unwrap_or(0.0);
                    let y0 = obj_to_f64(&arr[1]).unwrap_or(0.0);
                    let x1 = obj_to_f64(&arr[2]).unwrap_or(612.0);
                    let y1 = obj_to_f64(&arr[3]).unwrap_or(792.0);
                    return Some(((x1 - x0).abs(), (y1 - y0).abs()));
                }
            }
            None
        }
        _ => None,
    }
}

fn obj_to_f64(obj: &Object) -> Option<f64> {
    match obj {
        Object::Integer(i) => Some(*i as f64),
        Object::Real(f) => Some(*f),
        _ => None,
    }
}
