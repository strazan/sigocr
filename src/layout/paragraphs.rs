use crate::layout::reading_order::TextLine;
use crate::types::{BoundingRegion, Paragraph, Span};

/// Detected paragraph role based on font size and position heuristics.
fn detect_role(line: &TextLine, page_height: f64, median_font_size: f64) -> Option<String> {
    let fs = line.font_size;

    // Page number: small text near bottom of page
    if line.y < page_height * 0.08
        && line.text().trim().len() <= 6
        && line.text().trim().parse::<u32>().is_ok()
    {
        return Some("pageNumber".to_string());
    }

    // Page header: near top of page
    if line.y > page_height * 0.92 {
        return Some("pageHeader".to_string());
    }

    // Page footer: near bottom of page
    if line.y < page_height * 0.05 {
        return Some("pageFooter".to_string());
    }

    // Title: significantly larger than median
    if fs > median_font_size * 1.6 {
        return Some("title".to_string());
    }

    // Section heading: moderately larger than median
    if fs > median_font_size * 1.2 {
        return Some("sectionHeading".to_string());
    }

    None
}

/// Group lines into paragraphs based on vertical spacing.
/// Lines that are close together (within ~1.5x line height) form a paragraph.
/// Lines with different font sizes or large gaps start new paragraphs.
pub fn group_into_paragraphs(
    lines: Vec<TextLine>,
    page_number: i32,
    page_height: f64,
    content_offset: &mut usize,
    full_content: &mut String,
) -> Vec<Paragraph> {
    if lines.is_empty() {
        return Vec::new();
    }

    // Calculate median font size for role detection
    let median_font_size = {
        let mut sizes: Vec<f64> = lines.iter().map(|l| l.font_size).collect();
        sizes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        sizes[sizes.len() / 2]
    };

    let mut paragraphs: Vec<Paragraph> = Vec::new();
    let mut current_lines: Vec<&TextLine> = vec![&lines[0]];

    for i in 1..lines.len() {
        let prev = &lines[i - 1];
        let curr = &lines[i];

        // Gap between lines (in PDF Y-coords, larger Y = higher on page)
        let gap = prev.y - curr.y;
        let expected_line_gap = prev.font_size * 1.5;

        // Start new paragraph if:
        // - Large vertical gap (> 1.5x expected line spacing)
        // - Significant font size change
        let font_size_changed = (curr.font_size - prev.font_size).abs() > prev.font_size * 0.15;
        let large_gap = gap > expected_line_gap * 1.5;

        if large_gap || font_size_changed {
            let para = build_paragraph(
                &current_lines,
                page_number,
                page_height,
                median_font_size,
                content_offset,
                full_content,
            );
            paragraphs.push(para);
            current_lines = vec![curr];
        } else {
            current_lines.push(curr);
        }
    }

    if !current_lines.is_empty() {
        let para = build_paragraph(
            &current_lines,
            page_number,
            page_height,
            median_font_size,
            content_offset,
            full_content,
        );
        paragraphs.push(para);
    }

    paragraphs
}

fn build_paragraph(
    lines: &[&TextLine],
    page_number: i32,
    page_height: f64,
    median_font_size: f64,
    content_offset: &mut usize,
    full_content: &mut String,
) -> Paragraph {
    // Build paragraph text by joining lines with spaces
    let mut text_parts: Vec<String> = Vec::new();
    for line in lines {
        let line_text = line_to_text(line);
        let trimmed = line_text.trim();
        if !trimmed.is_empty() {
            text_parts.push(trimmed.to_string());
        }
    }
    let content = text_parts.join("\n");

    // Compute bounding box from all lines
    let min_x = lines.iter().map(|l| l.min_x).fold(f64::INFINITY, f64::min);
    let max_x = lines
        .iter()
        .map(|l| l.max_x)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = lines
        .iter()
        .map(|l| l.y + l.font_size)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = lines.iter().map(|l| l.y).fold(f64::INFINITY, f64::min);

    let polygon = vec![min_x, min_y, max_x, min_y, max_x, max_y, min_x, max_y];

    // Detect role from first line
    let role = lines
        .first()
        .and_then(|l| detect_role(l, page_height, median_font_size));

    // Add to full content with paragraph separator
    // Use UTF-16 code unit counts for JS compatibility (String.slice uses UTF-16 indices)
    if !full_content.is_empty() && !content.is_empty() {
        full_content.push('\n');
        *content_offset += 1; // '\n' is 1 UTF-16 code unit
    }

    let span_offset = *content_offset;
    let content_len_utf16 = content.encode_utf16().count();
    full_content.push_str(&content);
    *content_offset += content_len_utf16;

    Paragraph {
        content,
        role,
        spans: vec![Span {
            offset: span_offset as i32,
            length: content_len_utf16 as i32,
        }],
        bounding_regions: vec![BoundingRegion {
            page_number,
            polygon,
        }],
    }
}

/// Convert a TextLine to a string, inserting spaces between characters
/// that have gaps larger than expected character width.
fn line_to_text(line: &TextLine) -> String {
    if line.chars.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    result.push(line.chars[0].ch);

    for i in 1..line.chars.len() {
        let prev = &line.chars[i - 1];
        let curr = &line.chars[i];

        // Insert space if gap between characters is larger than expected
        let expected_next_x = prev.x + prev.width;
        let gap = curr.x - expected_next_x;
        let space_threshold = prev.width * 0.3;

        if gap > space_threshold && !prev.ch.is_whitespace() && !curr.ch.is_whitespace() {
            result.push(' ');
        }

        result.push(curr.ch);
    }

    result
}
