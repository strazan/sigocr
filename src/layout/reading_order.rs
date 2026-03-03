/// Character with position data extracted from a PDF page.
#[derive(Debug, Clone)]
pub struct PositionedChar {
    pub ch: char,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub font_size: f64,
    pub font_name: String,
    pub page_number: i32,
}

/// A line of text grouped from characters by Y proximity.
#[derive(Debug, Clone)]
pub struct TextLine {
    pub chars: Vec<PositionedChar>,
    pub y: f64,
    pub min_x: f64,
    pub max_x: f64,
    pub font_size: f64,
}

impl TextLine {
    pub fn text(&self) -> String {
        self.chars.iter().map(|c| c.ch).collect()
    }
}

/// Sort characters into reading order (top-to-bottom, left-to-right)
/// and group them into lines based on Y-coordinate proximity.
pub fn group_into_lines(mut chars: Vec<PositionedChar>) -> Vec<TextLine> {
    if chars.is_empty() {
        return Vec::new();
    }

    // Sort by Y descending (PDF coordinates: origin at bottom-left), then X ascending
    chars.sort_by(|a, b| {
        b.y.partial_cmp(&a.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
    });

    let mut lines: Vec<TextLine> = Vec::new();
    let mut current_line_chars: Vec<PositionedChar> = vec![chars[0].clone()];
    let mut current_y = chars[0].y;
    let mut current_font_size = chars[0].font_size;

    for ch in chars.into_iter().skip(1) {
        // Characters on the same line have similar Y values
        // Use half the font size as threshold
        let threshold = current_font_size * 0.5;
        if (ch.y - current_y).abs() <= threshold {
            current_line_chars.push(ch);
        } else {
            // Finish current line
            let line = build_line(current_line_chars);
            lines.push(line);
            current_y = ch.y;
            current_font_size = ch.font_size;
            current_line_chars = vec![ch];
        }
    }

    if !current_line_chars.is_empty() {
        lines.push(build_line(current_line_chars));
    }

    lines
}

fn build_line(mut chars: Vec<PositionedChar>) -> TextLine {
    // Sort by X within the line
    chars.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

    let y = chars.first().map(|c| c.y).unwrap_or(0.0);
    let min_x = chars.first().map(|c| c.x).unwrap_or(0.0);
    let max_x = chars.last().map(|c| c.x + c.width).unwrap_or(0.0);
    let font_size = chars.first().map(|c| c.font_size).unwrap_or(0.0);

    TextLine {
        chars,
        y,
        min_x,
        max_x,
        font_size,
    }
}
