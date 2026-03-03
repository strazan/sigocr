# sigocr

Fast PDF text extraction with paragraph layout and bounding regions.

## Install

```bash
pnpm add sigocr
```

## Usage

```ts
import { sigocr } from "sigocr";

// Extract structured text from a PDF (returns null if scanned/no embedded text)
const doc = await sigocr.pdf("/path/to/file.pdf");
// -> Document | null

// Extract from a buffer
const doc = await sigocr.buffer(pdfBuffer);

// Check if a PDF has embedded text (fast, doesn't extract)
const has = await sigocr.hasText("/path/to/file.pdf");
// -> boolean
```

### Batch extraction

Extract many files in a single native call. Each file uses internal page-chunk parallelism via Rayon.

```ts
const docs = await sigocr.files([
  "/uploads/contract.pdf",
  "/uploads/invoice.pdf",
  "/uploads/report.pdf",
]);
// -> (Document | null)[]
```

### Output shape

The output shape:

```ts
interface Document {
  content: string;                    // full text in reading order
  pages: Page[];
  paragraphs: Paragraph[];
  tables: Table[];
}

interface Paragraph {
  content: string;
  role?: string;                      // "title" | "sectionHeading" | "pageHeader" | "pageFooter" | "pageNumber"
  spans: Span[];                      // character offset + length into Document.content
  boundingRegions: BoundingRegion[];  // page number + polygon in points
}
```

## Benchmarks

Measured on Apple M3 Pro, Node.js v24.

### Structured extraction (with positions) - 200-page PDF

| Library | Mean | ms/page | vs sigocr |
|---------|------|---------|-----------|
| **sigocr** (native) | 16.5ms | 0.08 | 1x |
| pdf.js-extract (JS) | 71.0ms | 0.36 | **4.3x slower** |
| pdf2json (JS) | 280ms | 1.40 | **17x slower** |

### Plain text extraction (no positions) - 200-page PDF

| Library | Mean | ms/page |
|---------|------|---------|
| pdf-parse (JS) | 51.6ms | 0.26 |
| unpdf (JS) | 63.9ms | 0.32 |

### Batch extraction - 100 x 3-page PDFs

| Library | Mean | vs sigocr |
|---------|------|-----------|
| **sigocr.files** (native batch) | 17.2ms | 1x |
| sigocr.pdf (sequential loop) | 97.3ms | **5.7x slower** |
| pdf.js-extract (sequential loop) | 134.0ms | **7.8x slower** |

### hasText detection

| | Mean |
|---|------|
| sigocr hasTextBuffer | 1.7ms |

sigocr is **4.3x faster** than pdf.js-extract and **17x faster** than pdf2json while producing richer output: grouped paragraphs with roles, bounding regions, and document-level spans. pdf.js-extract gives raw text items that still need assembly. sigocr is also **3.1x faster** than plain-text-only extractors despite doing strictly more work. Batch extraction distributes files across cores via Rayon - 100 PDFs in 17ms.

Run benchmarks yourself:

```bash
pnpm bench
```

## How it works

- **`null` for scanned PDFs**: No OCR engine - this library only handles embedded text
- **Pure Rust, no C deps**: Uses `pdf_oxide` for character-level extraction. No PDFium/MuPDF binaries to ship. ~1 MB package
- **Parallel page chunks**: Large PDFs are split into chunks processed in parallel via Rayon, each opening its own PDF instance
- **Paragraph roles via heuristics**: Font size for heading detection, page position for header/footer/page number
- **`hasText()` fast path**: Check if a PDF has embedded text without full extraction. Checks first 3 pages only
- **UTF-16 span offsets**: Span offsets and lengths are in JavaScript string units for direct use with `String.slice()`
