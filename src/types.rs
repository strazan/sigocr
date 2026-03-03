use napi_derive::napi;

/// A span referencing a range in the full document content string.
#[napi(object)]
#[derive(Clone)]
pub struct Span {
    pub offset: i32,
    pub length: i32,
}

/// A bounding region on a specific page.
#[napi(object)]
#[derive(Clone)]
pub struct BoundingRegion {
    pub page_number: i32,
    /// [x1,y1, x2,y2, x3,y3, x4,y4] in points
    pub polygon: Vec<f64>,
}

/// A paragraph of text with optional role annotation.
#[napi(object)]
pub struct Paragraph {
    pub content: String,
    pub role: Option<String>,
    pub spans: Vec<Span>,
    pub bounding_regions: Vec<BoundingRegion>,
}

/// A single cell in a table.
#[napi(object)]
pub struct TableCell {
    pub content: String,
    pub kind: Option<String>,
    pub row_index: i32,
    pub column_index: i32,
    pub bounding_regions: Vec<BoundingRegion>,
    pub spans: Vec<Span>,
}

/// A table detected in the document.
#[napi(object)]
pub struct Table {
    pub row_count: i32,
    pub column_count: i32,
    pub cells: Vec<TableCell>,
    pub bounding_regions: Vec<BoundingRegion>,
    pub spans: Vec<Span>,
}

/// A single page's metadata.
#[napi(object)]
pub struct Page {
    pub page_number: i32,
    pub width: f64,
    pub height: f64,
}

/// The full extracted document.
#[napi(object)]
pub struct Document {
    pub content: String,
    pub pages: Vec<Page>,
    pub paragraphs: Vec<Paragraph>,
    pub tables: Vec<Table>,
}
