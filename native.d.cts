export interface Span {
  offset: number;
  length: number;
}

export interface BoundingRegion {
  pageNumber: number;
  polygon: number[];
}

export interface Paragraph {
  content: string;
  role?: string;
  spans: Span[];
  boundingRegions: BoundingRegion[];
}

export interface TableCell {
  content: string;
  kind?: string;
  rowIndex: number;
  columnIndex: number;
  boundingRegions: BoundingRegion[];
  spans: Span[];
}

export interface Table {
  rowCount: number;
  columnCount: number;
  cells: TableCell[];
  boundingRegions: BoundingRegion[];
  spans: Span[];
}

export interface Page {
  pageNumber: number;
  width: number;
  height: number;
}

export interface Document {
  content: string;
  pages: Page[];
  paragraphs: Paragraph[];
  tables: Table[];
}

export declare function extractPdf(path: string): Promise<Document | null>;

export declare function extractPdfBuffer(data: Buffer): Promise<Document | null>;

export declare function extractPdfs(paths: string[]): Promise<(Document | null)[]>;

export declare function hasEmbeddedText(path: string): Promise<boolean>;

export declare function hasEmbeddedTextBuffer(data: Buffer): Promise<boolean>;
