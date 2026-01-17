//! Unicode character cell width calculations.
//!
//! This module provides functions to calculate the display width of text
//! in terminal cells, handling wide characters (CJK, emoji) correctly.

use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

/// Get the cell width of a single character.
///
/// Most characters are 1 cell wide, but CJK characters and some emoji
/// are 2 cells wide. Control characters have 0 width.
#[must_use]
pub fn get_character_cell_size(c: char) -> usize {
    c.width().unwrap_or(0)
}

/// Get the total cell width of a string.
///
/// This is the sum of the widths of all characters, accounting for
/// wide characters that take 2 cells.
#[must_use]
pub fn cell_len(text: &str) -> usize {
    text.width()
}

/// Truncate a string to fit within a maximum cell width.
///
/// Returns the truncated string, padded with spaces if a wide character
/// would exceed the limit.
#[must_use]
pub fn set_cell_size(text: &str, total: usize) -> String {
    let current = cell_len(text);

    if current == total {
        return text.to_string();
    }

    if current < total {
        // Pad with spaces
        let padding = total - current;
        return format!("{text}{}", " ".repeat(padding));
    }

    // Need to truncate
    let (truncated, width) = truncate_to_width(text, total);

    // Pad if needed (when a wide character didn't fit)
    if width < total {
        format!("{truncated}{}", " ".repeat(total - width))
    } else {
        truncated
    }
}

/// Truncate a string to a maximum cell width.
///
/// Returns the truncated string and its actual width.
fn truncate_to_width(text: &str, max_width: usize) -> (String, usize) {
    let mut width = 0;
    let mut result = String::new();

    for c in text.chars() {
        let char_width = get_character_cell_size(c);
        if width + char_width > max_width {
            break;
        }
        width += char_width;
        result.push(c);
    }

    (result, width)
}

/// Split a string at a cell position.
///
/// Returns (left, right) where left has the specified width (or less if
/// a wide character would exceed it).
#[must_use]
pub fn chop_cells(text: &str, max_size: usize) -> (&str, &str) {
    let mut width = 0;
    let mut byte_pos = 0;

    for (i, c) in text.char_indices() {
        let char_width = get_character_cell_size(c);
        if width + char_width > max_size {
            break;
        }
        width += char_width;
        byte_pos = i + c.len_utf8();
    }

    (&text[..byte_pos], &text[byte_pos..])
}

/// Get the cell position for each character in a string.
///
/// Returns a vector of (byte_index, cell_position) pairs.
#[must_use]
pub fn cell_positions(text: &str) -> Vec<(usize, usize)> {
    let mut positions = Vec::new();
    let mut cell_pos = 0;

    for (byte_idx, c) in text.char_indices() {
        positions.push((byte_idx, cell_pos));
        cell_pos += get_character_cell_size(c);
    }

    positions
}

/// Find the byte index for a given cell position.
///
/// Returns None if the cell position is beyond the string's width.
#[must_use]
pub fn cell_to_byte_index(text: &str, cell_pos: usize) -> Option<usize> {
    let mut current_cell = 0;

    for (byte_idx, c) in text.char_indices() {
        if current_cell >= cell_pos {
            return Some(byte_idx);
        }
        current_cell += get_character_cell_size(c);
    }

    if current_cell >= cell_pos {
        Some(text.len())
    } else {
        None
    }
}

/// Check if a string contains any wide (2-cell) characters.
#[must_use]
pub fn has_wide_chars(text: &str) -> bool {
    text.chars().any(|c| get_character_cell_size(c) > 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_width() {
        assert_eq!(cell_len("hello"), 5);
        assert_eq!(cell_len("Hello, World!"), 13);
    }

    #[test]
    fn test_character_width() {
        assert_eq!(get_character_cell_size('a'), 1);
        assert_eq!(get_character_cell_size(' '), 1);
    }

    #[test]
    fn test_cjk_width() {
        // CJK characters are 2 cells wide
        assert_eq!(cell_len("日本語"), 6); // 3 characters * 2 cells
        assert_eq!(cell_len("中文"), 4);   // 2 characters * 2 cells
    }

    #[test]
    fn test_mixed_width() {
        // Mix of ASCII and CJK
        assert_eq!(cell_len("Hello日本"), 9); // 5 + 2*2
    }

    #[test]
    fn test_set_cell_size_pad() {
        let result = set_cell_size("hi", 5);
        assert_eq!(result, "hi   ");
        assert_eq!(cell_len(&result), 5);
    }

    #[test]
    fn test_set_cell_size_truncate() {
        let result = set_cell_size("hello world", 5);
        assert_eq!(result, "hello");
        assert_eq!(cell_len(&result), 5);
    }

    #[test]
    fn test_set_cell_size_exact() {
        let result = set_cell_size("hello", 5);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_chop_cells() {
        let (left, right) = chop_cells("hello world", 5);
        assert_eq!(left, "hello");
        assert_eq!(right, " world");
    }

    #[test]
    fn test_chop_cells_cjk() {
        // Chopping in the middle of wide characters
        let (left, right) = chop_cells("日本語", 3);
        // Should stop at 2 (one character) since next would be 4
        assert_eq!(cell_len(left), 2);
        assert_eq!(left, "日");
        assert_eq!(right, "本語");
    }

    #[test]
    fn test_cell_positions() {
        let positions = cell_positions("aあb");
        assert_eq!(positions[0], (0, 0)); // 'a' at byte 0, cell 0
        assert_eq!(positions[1], (1, 1)); // 'あ' at byte 1, cell 1
        assert_eq!(positions[2], (4, 3)); // 'b' at byte 4, cell 3 (あ is 3 bytes, 2 cells)
    }

    #[test]
    fn test_has_wide_chars() {
        assert!(!has_wide_chars("hello"));
        assert!(has_wide_chars("hello日本"));
        assert!(has_wide_chars("日本語"));
    }

    #[test]
    fn test_control_characters() {
        // Control characters should have 0 width
        assert_eq!(get_character_cell_size('\0'), 0);
        assert_eq!(get_character_cell_size('\x1b'), 0); // ESC
    }
}
