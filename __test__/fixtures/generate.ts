/**
 * Generate test PDF fixtures for sigocr tests and benchmarks.
 *
 * Run: npx tsx __test__/fixtures/generate.ts
 */
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { PDFDocument, rgb, StandardFonts } from "pdf-lib";

const FIXTURES_DIR = dirname(new URL(import.meta.url).pathname);

async function generateDigitalPdf(pages: number, filename: string) {
  const doc = await PDFDocument.create();
  const font = await doc.embedFont(StandardFonts.Helvetica);
  const boldFont = await doc.embedFont(StandardFonts.HelveticaBold);

  for (let i = 0; i < pages; i++) {
    const page = doc.addPage([612, 792]); // US Letter
    const { height } = page.getSize();
    const pageNum = i + 1;

    // Title on first page
    if (i === 0) {
      page.drawText("Sample Document for Testing", {
        x: 72,
        y: height - 72,
        size: 24,
        font: boldFont,
        color: rgb(0, 0, 0),
      });
      page.drawText("This document was generated for sigocr test fixtures.", {
        x: 72,
        y: height - 110,
        size: 12,
        font,
        color: rgb(0, 0, 0),
      });
    }

    // Section heading
    page.drawText(`Section ${pageNum}: Content Block`, {
      x: 72,
      y: height - (i === 0 ? 160 : 72),
      size: 16,
      font: boldFont,
      color: rgb(0, 0, 0),
    });

    // Body paragraphs
    const paragraphs = [
      `This is paragraph one on page ${pageNum}. It contains sample text that demonstrates how sigocr extracts structured content from PDF documents with embedded text.`,
      `The second paragraph continues with more content. PDF documents store text as positioned glyphs with font references, and sigocr groups these into logical paragraphs with bounding regions.`,
      `A third paragraph provides additional density. Each paragraph is detected by analyzing vertical spacing between lines and font size changes across the document.`,
    ];

    let y = height - (i === 0 ? 190 : 100);
    for (const para of paragraphs) {
      // Word-wrap at ~80 chars per line
      const words = para.split(" ");
      let line = "";
      for (const word of words) {
        const test = line ? `${line} ${word}` : word;
        if (test.length > 80) {
          page.drawText(line, { x: 72, y, size: 11, font, color: rgb(0, 0, 0) });
          y -= 16;
          line = word;
        } else {
          line = test;
        }
      }
      if (line) {
        page.drawText(line, { x: 72, y, size: 11, font, color: rgb(0, 0, 0) });
        y -= 16;
      }
      y -= 10; // paragraph gap
    }

    // Page footer
    page.drawText(`Page ${pageNum}`, {
      x: 280,
      y: 30,
      size: 9,
      font,
      color: rgb(0.5, 0.5, 0.5),
    });
  }

  const bytes = await doc.save();
  const path = join(FIXTURES_DIR, filename);
  writeFileSync(path, bytes);
  console.log(`  ${filename} (${pages} pages, ${(bytes.length / 1024).toFixed(0)} KB)`);
}

async function generateEmptyPdf(filename: string) {
  const doc = await PDFDocument.create();
  doc.addPage([612, 792]); // blank page, no text
  const bytes = await doc.save();
  writeFileSync(join(FIXTURES_DIR, filename), bytes);
  console.log(`  ${filename} (1 blank page, ${(bytes.length / 1024).toFixed(0)} KB)`);
}

async function main() {
  if (!existsSync(FIXTURES_DIR)) {
    mkdirSync(FIXTURES_DIR, { recursive: true });
  }

  console.log("Generating test fixtures:");
  await generateDigitalPdf(3, "digital-3p.pdf");
  await generateDigitalPdf(50, "digital-50p.pdf");
  await generateDigitalPdf(200, "digital-200p.pdf");
  await generateEmptyPdf("empty.pdf");

  // Generate 100 small PDFs for batch benchmarks
  const batchDir = join(FIXTURES_DIR, "batch");
  if (!existsSync(batchDir)) {
    mkdirSync(batchDir, { recursive: true });
  }
  // Reuse the same 3-page template for all 100 files
  const templateBytes = readFileSync(join(FIXTURES_DIR, "digital-3p.pdf"));
  for (let i = 0; i < 100; i++) {
    writeFileSync(join(batchDir, `file-${String(i).padStart(3, "0")}.pdf`), templateBytes);
  }
  console.log("  batch/ (100 x 3-page PDFs)");

  console.log("Done.");
}

main();
