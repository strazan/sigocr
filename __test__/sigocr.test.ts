import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { FileNotFoundError, SigocrError, sigocr } from "../ts/index.js";

const FIXTURES = join(__dirname, "fixtures");
const DIGITAL_3P = join(FIXTURES, "digital-3p.pdf");
const DIGITAL_50P = join(FIXTURES, "digital-50p.pdf");
const EMPTY_PDF = join(FIXTURES, "empty.pdf");

describe("sigocr", () => {
  describe("buffer", () => {
    it("extracts structured text from a digital PDF", async () => {
      const pdf = readFileSync(DIGITAL_3P);
      const doc = await sigocr.buffer(pdf);

      expect(doc).not.toBeNull();
      expect(doc?.content).toContain("Sample Document for Testing");
      expect(doc?.pages).toHaveLength(3);
      expect(doc?.pages[0]?.pageNumber).toBe(1);
      expect(doc?.pages[0]?.width).toBe(612);
      expect(doc?.pages[0]?.height).toBe(792);
      expect(doc?.paragraphs.length).toBeGreaterThan(0);
    });

    it("returns null for an empty PDF (no text)", async () => {
      const pdf = readFileSync(EMPTY_PDF);
      const doc = await sigocr.buffer(pdf);
      expect(doc).toBeNull();
    });
  });

  describe("hasText", () => {
    it("throws FileNotFoundError for non-existent file", async () => {
      await expect(sigocr.hasText("/nonexistent/path.pdf")).rejects.toThrow(FileNotFoundError);
    });

    it("returns true for a digital PDF", async () => {
      const has = await sigocr.hasText(DIGITAL_3P);
      expect(has).toBe(true);
    });

    it("returns false for an empty PDF", async () => {
      const has = await sigocr.hasTextBuffer(readFileSync(EMPTY_PDF));
      expect(has).toBe(false);
    });
  });

  describe("pdf (file path)", () => {
    it("throws FileNotFoundError for non-existent file", async () => {
      await expect(sigocr.pdf("/nonexistent/path.pdf")).rejects.toThrow(FileNotFoundError);
    });

    it("throws SigocrError base class for non-existent file", async () => {
      await expect(sigocr.pdf("/nonexistent/path.pdf")).rejects.toThrow(SigocrError);
    });

    it("extracts from file path", async () => {
      const doc = await sigocr.pdf(DIGITAL_3P);
      expect(doc).not.toBeNull();
      expect(doc?.content).toContain("Sample Document");
    });
  });

  describe("structured output", () => {
    it("produces valid paragraphs with bounding regions", async () => {
      const doc = await sigocr.buffer(readFileSync(DIGITAL_3P));
      expect(doc).not.toBeNull();
      if (!doc) return;

      for (const para of doc.paragraphs) {
        expect(para.content.length).toBeGreaterThan(0);
        expect(para.spans).toHaveLength(1);
        expect(para.spans[0]?.offset).toBeGreaterThanOrEqual(0);
        expect(para.spans[0]?.length).toBeGreaterThan(0);
        expect(para.boundingRegions).toHaveLength(1);
        expect(para.boundingRegions[0]?.pageNumber).toBeGreaterThan(0);

        // Polygon should have 8 values (4 corners x 2 coords)
        expect(para.boundingRegions[0]?.polygon).toHaveLength(8);
      }
    });

    it("span offsets correctly index into content string", async () => {
      const doc = await sigocr.buffer(readFileSync(DIGITAL_3P));
      expect(doc).not.toBeNull();
      if (!doc) return;

      for (const para of doc.paragraphs) {
        const span = para.spans[0];
        if (span) {
          const extracted = doc.content.slice(span.offset, span.offset + span.length);
          expect(extracted).toBe(para.content);
        }
      }
    });

    it("detects paragraph roles", async () => {
      const doc = await sigocr.buffer(readFileSync(DIGITAL_3P));
      expect(doc).not.toBeNull();
      if (!doc) return;

      const roles = doc.paragraphs.map((p) => p.role).filter(Boolean);
      // Should detect at least the title and some headings
      expect(roles.length).toBeGreaterThan(0);
    });
  });

  describe("file vs buffer consistency", () => {
    it("file extraction matches buffer extraction", async () => {
      const fromFile = await sigocr.pdf(DIGITAL_50P);
      const fromBuffer = await sigocr.buffer(readFileSync(DIGITAL_50P));

      if (fromFile === null) {
        expect(fromBuffer).toBeNull();
      } else {
        expect(fromBuffer).not.toBeNull();
        expect(fromFile.content).toBe(fromBuffer?.content);
        expect(fromFile.pages).toEqual(fromBuffer?.pages);
        expect(fromFile.paragraphs.length).toBe(fromBuffer?.paragraphs.length);
      }
    });
  });

  describe("files (batch)", () => {
    it("batch extraction matches individual calls", async () => {
      const batchResult = await sigocr.files([DIGITAL_3P, DIGITAL_3P]);
      const singleResult = await sigocr.pdf(DIGITAL_3P);

      expect(batchResult).toHaveLength(2);

      if (singleResult === null) {
        expect(batchResult[0]).toBeNull();
        expect(batchResult[1]).toBeNull();
      } else {
        expect(batchResult[0]).not.toBeNull();
        expect(batchResult[1]).not.toBeNull();
        expect(batchResult[0]?.content).toBe(singleResult.content);
        expect(batchResult[1]?.content).toBe(singleResult.content);
      }
    });
  });
});
