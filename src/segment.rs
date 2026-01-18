//! Segment - the atomic rendering unit.
//!
//! A `Segment` is a piece of text with a single style applied. The rendering
//! pipeline produces streams of segments that are then written to the terminal.

use std::fmt;
use crate::style::Style;
use crate::cells::cell_len;

/// Control codes for terminal manipulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ControlType {
    Bell = 1,
    CarriageReturn = 2,
    Home = 3,
    Clear = 4,
    ShowCursor = 5,
    HideCursor = 6,
    EnableAltScreen = 7,
    DisableAltScreen = 8,
    CursorUp = 9,
    CursorDown = 10,
    CursorForward = 11,
    CursorBackward = 12,
    CursorMoveToColumn = 13,
    CursorMoveTo = 14,
    EraseInLine = 15,
    SetWindowTitle = 16,
}

/// A control code with optional parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlCode {
    pub control_type: ControlType,
    pub params: Vec<i32>,
}

impl ControlCode {
    /// Create a new control code.
    #[must_use]
    pub fn new(control_type: ControlType) -> Self {
        Self {
            control_type,
            params: Vec::new(),
        }
    }

    /// Create a control code with parameters.
    #[must_use]
    pub fn with_params(control_type: ControlType, params: Vec<i32>) -> Self {
        Self {
            control_type,
            params,
        }
    }
}

/// The atomic unit of rendering.
///
/// A segment represents a piece of text with a single, consistent style.
/// The rendering pipeline breaks down complex renderables into segments
/// for output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    /// The text content.
    pub text: String,
    /// The style to apply (None = no styling).
    pub style: Option<Style>,
    /// Control codes for terminal manipulation.
    pub control: Option<Vec<ControlCode>>,
}

impl Default for Segment {
    fn default() -> Self {
        Self::new("", None)
    }
}

impl Segment {
    /// Create a new segment with text and optional style.
    #[must_use]
    pub fn new(text: impl Into<String>, style: Option<Style>) -> Self {
        Self {
            text: text.into(),
            style,
            control: None,
        }
    }

    /// Create a segment with a style.
    #[must_use]
    pub fn styled(text: impl Into<String>, style: Style) -> Self {
        Self::new(text, Some(style))
    }

    /// Create a plain segment with no style.
    #[must_use]
    pub fn plain(text: impl Into<String>) -> Self {
        Self::new(text, None)
    }

    /// Create a newline segment.
    #[must_use]
    pub fn line() -> Self {
        Self::new("\n", None)
    }

    /// Create a control segment.
    #[must_use]
    pub fn control(control_codes: Vec<ControlCode>) -> Self {
        Self {
            text: String::new(),
            style: None,
            control: Some(control_codes),
        }
    }

    /// Check if this is a control segment.
    #[must_use]
    pub const fn is_control(&self) -> bool {
        self.control.is_some()
    }

    /// Get the cell width of this segment.
    ///
    /// Control segments have zero width.
    #[must_use]
    pub fn cell_length(&self) -> usize {
        if self.is_control() {
            0
        } else {
            cell_len(&self.text)
        }
    }

    /// Check if this segment is empty (no text or control).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty() && self.control.is_none()
    }

    /// Apply a style to this segment.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Split this segment at a cell position.
    ///
    /// Returns (left, right) segments.
    #[must_use]
    pub fn split_at_cell(&self, cell_pos: usize) -> (Self, Self) {
        if self.is_control() {
            return (self.clone(), Self::default());
        }

        let mut width = 0;
        let mut byte_pos = 0;

        for (i, c) in self.text.char_indices() {
            let char_width = crate::cells::get_character_cell_size(c);
            if width + char_width > cell_pos {
                break;
            }
            width += char_width;
            byte_pos = i + c.len_utf8();
        }

        let (left_text, right_text) = self.text.split_at(byte_pos);

        (
            Self::new(left_text, self.style.clone()),
            Self::new(right_text, self.style.clone()),
        )
    }
}

impl From<&str> for Segment {
    fn from(value: &str) -> Self {
        Self::plain(value)
    }
}

impl From<String> for Segment {
    fn from(value: String) -> Self {
        Self::plain(value)
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

// ============================================================================
// Segment Operations
// ============================================================================

/// Apply styles to an iterator of segments.
pub fn apply_style<'a, I>(
    segments: I,
    style: Option<&'a Style>,
    post_style: Option<&'a Style>,
) -> impl Iterator<Item = Segment> + 'a
where
    I: Iterator<Item = Segment> + 'a,
{
    segments.map(move |mut seg| {
        if seg.is_control() {
            return seg;
        }

        if let Some(pre) = style {
            seg.style = Some(match seg.style {
                Some(s) => pre.combine(&s),
                None => pre.clone(),
            });
        }

        if let Some(post) = post_style {
            seg.style = Some(match seg.style {
                Some(s) => s.combine(post),
                None => post.clone(),
            });
        }

        seg
    })
}

/// Split segments into lines at newline characters.
pub fn split_lines(segments: impl Iterator<Item = Segment>) -> Vec<Vec<Segment>> {
    let mut lines: Vec<Vec<Segment>> = vec![Vec::new()];

    for segment in segments {
        if segment.is_control() {
            lines.last_mut().expect("at least one line").push(segment);
            continue;
        }

        let parts: Vec<&str> = segment.text.split('\n').collect();
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                // Start a new line
                lines.push(Vec::new());
            }
            if !part.is_empty() {
                lines
                    .last_mut()
                    .expect("at least one line")
                    .push(Segment::new(*part, segment.style.clone()));
            }
        }
    }

    lines
}

/// Adjust line length by padding or truncating.
#[must_use]
pub fn adjust_line_length(
    mut line: Vec<Segment>,
    length: usize,
    style: Option<Style>,
    pad: bool,
) -> Vec<Segment> {
    let current_length: usize = line.iter().map(Segment::cell_length).sum();

    if current_length < length && pad {
        // Pad with spaces
        let padding = length - current_length;
        line.push(Segment::new(" ".repeat(padding), style));
    } else if current_length > length {
        // Truncate
        line = truncate_line(line, length);
    }

    line
}

/// Truncate a line to a maximum cell width.
fn truncate_line(segments: Vec<Segment>, max_width: usize) -> Vec<Segment> {
    let mut result = Vec::new();
    let mut remaining = max_width;

    for segment in segments {
        if segment.is_control() {
            result.push(segment);
            continue;
        }

        let seg_width = segment.cell_length();
        if seg_width <= remaining {
            result.push(segment);
            remaining -= seg_width;
        } else if remaining > 0 {
            let (left, _) = segment.split_at_cell(remaining);
            result.push(left);
            break;
        } else {
            break;
        }
    }

    result
}

/// Simplify segments by merging adjacent segments with identical styles.
#[must_use]
pub fn simplify(segments: impl Iterator<Item = Segment>) -> Vec<Segment> {
    let mut result: Vec<Segment> = Vec::new();

    for segment in segments {
        if segment.is_control() || segment.text.is_empty() {
            if segment.is_control() {
                result.push(segment);
            }
            continue;
        }

        if let Some(last) = result.last_mut() {
            if !last.is_control() && last.style == segment.style {
                last.text.push_str(&segment.text);
                continue;
            }
        }

        result.push(segment);
    }

    result
}

/// Divide segments at specified cell positions.
#[must_use]
pub fn divide(segments: Vec<Segment>, cuts: &[usize]) -> Vec<Vec<Segment>> {
    if cuts.is_empty() {
        return vec![segments];
    }

    let mut result: Vec<Vec<Segment>> = vec![Vec::new(); cuts.len() + 1];
    let mut current_pos = 0;
    let mut cut_idx = 0;

    for segment in segments {
        if segment.is_control() {
            result[cut_idx].push(segment);
            continue;
        }

        let seg_width = segment.cell_length();
        let seg_end = current_pos + seg_width;

        // Find which divisions this segment spans
        while cut_idx < cuts.len() && cuts[cut_idx] <= current_pos {
            cut_idx += 1;
        }

        if cut_idx >= cuts.len() || seg_end <= cuts[cut_idx] {
            // Segment fits entirely in current division
            let target_idx = cut_idx.min(result.len() - 1);
            result[target_idx].push(segment);
        } else {
            // Segment spans multiple divisions - need to split
            let mut remaining = segment;
            let mut pos = current_pos;

            while cut_idx < cuts.len() && pos + remaining.cell_length() > cuts[cut_idx] {
                let split_at = cuts[cut_idx] - pos;
                let (left, right) = remaining.split_at_cell(split_at);

                if !left.text.is_empty() {
                    result[cut_idx].push(left);
                }

                pos = cuts[cut_idx];
                cut_idx += 1;
                remaining = right;
            }

            if !remaining.text.is_empty() {
                let target_idx = cut_idx.min(result.len() - 1);
                result[target_idx].push(remaining);
            }
        }

        current_pos = seg_end;
    }

    result
}

/// Align lines to the top of a given height.
#[must_use]
pub fn align_top(
    lines: Vec<Vec<Segment>>,
    width: usize,
    height: usize,
    style: Style,
) -> Vec<Vec<Segment>> {
    let mut result = lines;

    // Pad existing lines to width
    for line in &mut result {
        let line_width: usize = line.iter().map(Segment::cell_length).sum();
        if line_width < width {
            line.push(Segment::new(" ".repeat(width - line_width), Some(style.clone())));
        }
    }

    // Add blank lines at bottom
    while result.len() < height {
        result.push(vec![Segment::new(" ".repeat(width), Some(style.clone()))]);
    }

    result
}

/// Align lines to the bottom of a given height.
#[must_use]
pub fn align_bottom(
    lines: Vec<Vec<Segment>>,
    width: usize,
    height: usize,
    style: Style,
) -> Vec<Vec<Segment>> {
    let mut result = Vec::new();
    let blank_line = vec![Segment::new(" ".repeat(width), Some(style.clone()))];

    // Add blank lines at top
    let padding = height.saturating_sub(lines.len());
    for _ in 0..padding {
        result.push(blank_line.clone());
    }

    // Add content lines
    for mut line in lines {
        let line_width: usize = line.iter().map(Segment::cell_length).sum();
        if line_width < width {
            line.push(Segment::new(" ".repeat(width - line_width), Some(style.clone())));
        }
        result.push(line);
    }

    result
}

/// Align lines to the middle of a given height.
#[must_use]
pub fn align_middle(
    lines: Vec<Vec<Segment>>,
    width: usize,
    height: usize,
    style: Style,
) -> Vec<Vec<Segment>> {
    let content_height = lines.len();
    if content_height >= height {
        return align_top(lines, width, height, style);
    }

    let mut result = Vec::new();
    let blank_line = vec![Segment::new(" ".repeat(width), Some(style.clone()))];

    let total_padding = height - content_height;
    let top_padding = total_padding / 2;
    let bottom_padding = total_padding - top_padding;

    // Top padding
    for _ in 0..top_padding {
        result.push(blank_line.clone());
    }

    // Content
    for mut line in lines {
        let line_width: usize = line.iter().map(Segment::cell_length).sum();
        if line_width < width {
            line.push(Segment::new(" ".repeat(width - line_width), Some(style.clone())));
        }
        result.push(line);
    }

    // Bottom padding
    for _ in 0..bottom_padding {
        result.push(blank_line.clone());
    }

    result
}

/// Get the total cell length of a line of segments.
#[must_use]
pub fn line_length(line: &[Segment]) -> usize {
    line.iter().map(Segment::cell_length).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_new() {
        let seg = Segment::new("hello", None);
        assert_eq!(seg.text, "hello");
        assert!(seg.style.is_none());
    }

    #[test]
    fn test_segment_styled() {
        let style = Style::new().bold();
        let seg = Segment::styled("hello", style.clone());
        assert_eq!(seg.style, Some(style));
    }

    #[test]
    fn test_segment_line() {
        let seg = Segment::line();
        assert_eq!(seg.text, "\n");
    }

    #[test]
    fn test_segment_cell_length() {
        let seg = Segment::new("hello", None);
        assert_eq!(seg.cell_length(), 5);
    }

    #[test]
    fn test_segment_control_zero_length() {
        let seg = Segment::control(vec![ControlCode::new(ControlType::Bell)]);
        assert_eq!(seg.cell_length(), 0);
        assert!(seg.is_control());
    }

    #[test]
    fn test_segment_split_at_cell() {
        let seg = Segment::new("hello world", None);
        let (left, right) = seg.split_at_cell(5);
        assert_eq!(left.text, "hello");
        assert_eq!(right.text, " world");
    }

    #[test]
    fn test_split_lines() {
        let segments = vec![
            Segment::new("line1\nline2", None),
            Segment::new("\nline3", None),
        ];
        let lines = split_lines(segments.into_iter());
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_simplify() {
        let style = Style::new().bold();
        let segments = vec![
            Segment::styled("hello", style.clone()),
            Segment::styled(" ", style.clone()),
            Segment::styled("world", style.clone()),
        ];
        let simplified = simplify(segments.into_iter());
        assert_eq!(simplified.len(), 1);
        assert_eq!(simplified[0].text, "hello world");
    }

    #[test]
    fn test_adjust_line_length_pad() {
        let line = vec![Segment::new("hi", None)];
        let adjusted = adjust_line_length(line, 5, None, true);
        assert_eq!(line_length(&adjusted), 5);
    }

    #[test]
    fn test_adjust_line_length_truncate() {
        let line = vec![Segment::new("hello world", None)];
        let adjusted = adjust_line_length(line, 5, None, false);
        assert_eq!(line_length(&adjusted), 5);
    }

    #[test]
    fn test_divide() {
        let segments = vec![Segment::new("hello world", None)];
        let divided = divide(segments, &[5]);
        assert_eq!(divided.len(), 2);
        assert_eq!(divided[0][0].text, "hello");
        assert_eq!(divided[1][0].text, " world");
    }

    #[test]
    fn test_align_top() {
        let lines = vec![vec![Segment::new("hi", None)]];
        let aligned = align_top(lines, 5, 3, Style::null());
        assert_eq!(aligned.len(), 3);
    }

    #[test]
    fn test_align_bottom() {
        let lines = vec![vec![Segment::new("hi", None)]];
        let aligned = align_bottom(lines, 5, 3, Style::null());
        assert_eq!(aligned.len(), 3);
        // Content should be at bottom
        assert!(aligned[2][0].text.starts_with("hi"));
    }

    #[test]
    fn test_align_middle() {
        let lines = vec![vec![Segment::new("hi", None)]];
        let aligned = align_middle(lines, 5, 3, Style::null());
        assert_eq!(aligned.len(), 3);
        // Content should be in middle
        assert!(aligned[1][0].text.starts_with("hi"));
    }
}
