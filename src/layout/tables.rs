use crate::layout::reading_order::TextLine;
use crate::types::{BoundingRegion, Span, Table, TableCell};

/// Threshold for column alignment (in points).
const COLUMN_SNAP_THRESHOLD: f64 = 8.0;

/// Threshold for considering lines to be in the same row.
const ROW_SNAP_THRESHOLD: f64 = 4.0;

/// Minimum number of aligned columns to consider something a table.
const MIN_TABLE_COLUMNS: usize = 2;

/// Minimum number of rows to consider something a table.
const MIN_TABLE_ROWS: usize = 2;

/// A detected column with its X coordinate range.
#[derive(Debug, Clone)]
struct Column {
    min_x: f64,
    max_x: f64,
}

/// Detect tables from lines by finding columnar alignment patterns.
///
/// Strategy:
/// 1. Find lines that start at similar X positions (column candidates)
/// 2. Group consecutive rows that share the same column structure
/// 3. Build table cells from the intersection of rows and columns
pub fn detect_tables(
    lines: &[TextLine],
    page_number: i32,
    content_offset: &mut usize,
    full_content: &mut String,
) -> (Vec<Table>, Vec<usize>) {
    if lines.len() < MIN_TABLE_ROWS {
        return (Vec::new(), Vec::new());
    }

    // Collect X start positions of all lines
    let mut x_starts: Vec<f64> = lines.iter().map(|l| l.min_x).collect();
    x_starts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Cluster X starts into columns
    let columns = cluster_columns(&x_starts);

    if columns.len() < MIN_TABLE_COLUMNS {
        return (Vec::new(), Vec::new());
    }

    // Try to find runs of consecutive lines that align to these columns
    let mut tables: Vec<Table> = Vec::new();
    let mut table_line_indices: Vec<usize> = Vec::new();

    let mut row_start = 0;
    while row_start < lines.len() {
        // Find a run of lines where multiple lines share similar Y values
        // (indicating multi-column rows)
        let run = find_table_run(lines, row_start, &columns);

        if run.rows.len() >= MIN_TABLE_ROWS && run.col_count >= MIN_TABLE_COLUMNS {
            let table = build_table(&run, page_number, content_offset, full_content);
            tables.push(table);
            for idx in &run.line_indices {
                table_line_indices.push(*idx);
            }
            row_start = run.end_index;
        } else {
            row_start += 1;
        }
    }

    (tables, table_line_indices)
}

#[derive(Debug)]
struct TableRun {
    rows: Vec<Vec<(usize, usize)>>, // row -> [(line_idx, col_idx)]
    col_count: usize,
    line_indices: Vec<usize>,
    end_index: usize,
}

fn find_table_run(lines: &[TextLine], start: usize, columns: &[Column]) -> TableRun {
    let mut rows: Vec<Vec<(usize, usize)>> = Vec::new();
    let mut line_indices: Vec<usize> = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let col_idx = find_column(lines[i].min_x, columns);
        if col_idx.is_none() {
            break;
        }

        // Collect all lines at similar Y (same row)
        let mut row: Vec<(usize, usize)> = vec![(i, col_idx.unwrap())];
        line_indices.push(i);

        let mut j = i + 1;
        while j < lines.len() {
            let y_diff = (lines[i].y - lines[j].y).abs();
            if y_diff <= ROW_SNAP_THRESHOLD {
                if let Some(cj) = find_column(lines[j].min_x, columns) {
                    row.push((j, cj));
                    line_indices.push(j);
                }
                j += 1;
            } else {
                break;
            }
        }

        // Only consider it a table row if it has multiple columns
        if row.len() >= MIN_TABLE_COLUMNS {
            rows.push(row);
        } else {
            // Not a table row, stop the run
            break;
        }

        i = j;
    }

    TableRun {
        col_count: columns.len(),
        rows,
        line_indices,
        end_index: i,
    }
}

fn find_column(x: f64, columns: &[Column]) -> Option<usize> {
    columns
        .iter()
        .position(|c| x >= c.min_x - COLUMN_SNAP_THRESHOLD && x <= c.max_x + COLUMN_SNAP_THRESHOLD)
}

fn cluster_columns(sorted_xs: &[f64]) -> Vec<Column> {
    if sorted_xs.is_empty() {
        return Vec::new();
    }

    let mut columns: Vec<Column> = Vec::new();
    let mut cluster_start = sorted_xs[0];
    let mut cluster_end = sorted_xs[0];
    let mut count = 1;

    for &x in sorted_xs.iter().skip(1) {
        if x - cluster_end <= COLUMN_SNAP_THRESHOLD {
            cluster_end = x;
            count += 1;
        } else {
            if count >= MIN_TABLE_ROWS {
                columns.push(Column {
                    min_x: cluster_start,
                    max_x: cluster_end,
                });
            }
            cluster_start = x;
            cluster_end = x;
            count = 1;
        }
    }

    if count >= MIN_TABLE_ROWS {
        columns.push(Column {
            min_x: cluster_start,
            max_x: cluster_end,
        });
    }

    columns
}

fn build_table(
    run: &TableRun,
    page_number: i32,
    content_offset: &mut usize,
    full_content: &mut String,
) -> Table {
    let mut cells: Vec<TableCell> = Vec::new();
    let row_count = run.rows.len() as i32;
    let col_count = run.col_count as i32;

    // Track table content for spans
    let table_content_start = *content_offset;
    let mut table_text = String::new();

    for (row_idx, row) in run.rows.iter().enumerate() {
        for &(line_idx, col_idx) in row {
            let _ = line_idx; // Used for future bounding region refinement
            let content = row
                .iter()
                .filter(|&&(_, ci)| ci == col_idx)
                .map(|&(_li, _)| {
                    // We don't have direct access to lines here, so use empty for now
                    // In practice this would come from the line text
                    format!("cell_{}_{}", row_idx, col_idx)
                })
                .collect::<Vec<_>>()
                .join(" ");

            let cell_offset = *content_offset;
            if !table_text.is_empty() {
                table_text.push(' ');
                *content_offset += 1;
            }
            table_text.push_str(&content);
            *content_offset += content.len();

            let kind = if row_idx == 0 {
                Some("columnHeader".to_string())
            } else {
                None
            };

            cells.push(TableCell {
                content,
                kind,
                row_index: row_idx as i32,
                column_index: col_idx as i32,
                bounding_regions: vec![BoundingRegion {
                    page_number,
                    polygon: vec![0.0; 8], // Placeholder
                }],
                spans: vec![Span {
                    offset: cell_offset as i32,
                    length: 0, // Will be set properly
                }],
            });
        }
    }

    if !full_content.is_empty() && !table_text.is_empty() {
        full_content.push('\n');
    }
    full_content.push_str(&table_text);

    Table {
        row_count,
        column_count: col_count,
        cells,
        bounding_regions: vec![BoundingRegion {
            page_number,
            polygon: vec![0.0; 8], // Placeholder
        }],
        spans: vec![Span {
            offset: table_content_start as i32,
            length: table_text.len() as i32,
        }],
    }
}
