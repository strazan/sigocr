/**
 * Benchmark: sigocr vs competitors on PDF text extraction.
 *
 * Structured extractors (with positions/bounding boxes):
 *   - sigocr:         native Rust, paragraphs + bounding regions + spans
 *   - pdf.js-extract: JS, per-character x/y/width/height
 *   - pdf2json:       JS, per-text-run x/y with font info
 *
 * Plain text extractors (no positions):
 *   - pdf-parse
 *   - unpdf
 *
 * Uses generated fixtures.
 */
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { bench, describe } from "vitest";
import { sigocr } from "../../ts/index.js";

const FIXTURES = join(__dirname, "../fixtures");
const PDF_200P = join(FIXTURES, "digital-200p.pdf");
const BATCH_DIR = join(FIXTURES, "batch");

const pdfBuffer = readFileSync(PDF_200P);
const batchPaths = readdirSync(BATCH_DIR)
  .filter((f) => f.endsWith(".pdf"))
  .sort()
  .map((f) => join(BATCH_DIR, f));

const ITERS = 10;

describe(`Structured extraction (with positions) - 200 pages`, () => {
  bench(
    "sigocr (paragraphs + bounding regions + spans)",
    async () => {
      await sigocr.buffer(pdfBuffer);
    },
    { iterations: ITERS, warmupIterations: 2, time: 0 },
  );

  bench(
    "pdf.js-extract (per-item x/y/w/h)",
    async () => {
      const { PDFExtract } = await import("pdf.js-extract");
      const pdfExtract = new PDFExtract();
      await pdfExtract.extractBuffer(pdfBuffer, {});
    },
    { iterations: ITERS, warmupIterations: 2, time: 0 },
  );

  bench(
    "pdf2json (per-text-run x/y + font)",
    async () => {
      const PDFParser = (await import("pdf2json")).default;
      await new Promise<void>((resolve, reject) => {
        const parser = new PDFParser();
        parser.on("pdfParser_dataReady", () => resolve());
        parser.on("pdfParser_dataError", () => reject());
        parser.parseBuffer(pdfBuffer);
      });
    },
    { iterations: ITERS, warmupIterations: 2, time: 0 },
  );
});

describe(`Plain text extraction - 200 pages`, () => {
  bench(
    "pdf-parse (text only)",
    async () => {
      const { PDFParse } = await import("pdf-parse");
      const parser = new PDFParse({ data: new Uint8Array(pdfBuffer) });
      await parser.getText();
      await parser.destroy();
    },
    { iterations: ITERS, warmupIterations: 2, time: 0 },
  );

  bench(
    "unpdf (text only)",
    async () => {
      const { extractText } = await import("unpdf");
      await extractText(new Uint8Array(pdfBuffer));
    },
    { iterations: ITERS, warmupIterations: 2, time: 0 },
  );
});

describe(`Batch extraction - 100 x 3-page PDFs (${batchPaths.length} files)`, () => {
  bench(
    "sigocr.files (native batch)",
    async () => {
      await sigocr.files(batchPaths);
    },
    { iterations: ITERS, warmupIterations: 2, time: 0 },
  );

  bench(
    "sigocr.pdf (sequential loop)",
    async () => {
      for (const p of batchPaths) {
        await sigocr.pdf(p);
      }
    },
    { iterations: ITERS, warmupIterations: 2, time: 0 },
  );

  bench(
    "pdf.js-extract (sequential loop)",
    async () => {
      const { PDFExtract } = await import("pdf.js-extract");
      const pdfExtract = new PDFExtract();
      for (const p of batchPaths) {
        await pdfExtract.extract(p, {});
      }
    },
    { iterations: ITERS, warmupIterations: 2, time: 0 },
  );
});

describe("hasText detection (first 3 pages only)", () => {
  bench(
    "sigocr hasTextBuffer",
    async () => {
      await sigocr.hasTextBuffer(pdfBuffer);
    },
    { iterations: ITERS * 2, warmupIterations: 2, time: 0 },
  );
});
