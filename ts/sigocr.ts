import {
  extractPdf,
  extractPdfBuffer,
  extractPdfs,
  hasEmbeddedText as nativeHasText,
  hasEmbeddedTextBuffer as nativeHasTextBuffer,
} from "../native.cjs";
import { toSigocrError } from "./errors.js";
import type { Document } from "./types.js";

async function withErrors<T>(fn: () => Promise<T>): Promise<T> {
  try {
    return await fn();
  } catch (err) {
    throw toSigocrError(err);
  }
}

const pdf = (path: string): Promise<Document | null> => withErrors(() => extractPdf(path));

const buffer = (data: Buffer | Uint8Array): Promise<Document | null> =>
  withErrors(() => {
    const buf = Buffer.isBuffer(data) ? data : Buffer.from(data);
    return extractPdfBuffer(buf);
  });

const files = (paths: string[]): Promise<(Document | null)[]> =>
  withErrors(() => extractPdfs(paths));

const hasText = (path: string): Promise<boolean> => withErrors(() => nativeHasText(path));

const hasTextBuffer = (data: Buffer | Uint8Array): Promise<boolean> =>
  withErrors(() => {
    const buf = Buffer.isBuffer(data) ? data : Buffer.from(data);
    return nativeHasTextBuffer(buf);
  });

export const sigocr = {
  pdf,
  buffer,
  files,
  hasText,
  hasTextBuffer,
};
