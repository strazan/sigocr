export interface Span {
  offset: number;
  length: number;
}

export interface BoundingRegion {
  pageNumber: number;
  /** [x1,y1, x2,y2, x3,y3, x4,y4] in points */
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
  /** Width in points (72 points = 1 inch) */
  width: number;
  /** Height in points (72 points = 1 inch) */
  height: number;
}

export interface Document {
  content: string;
  pages: Page[];
  paragraphs: Paragraph[];
  tables: Table[];
}
