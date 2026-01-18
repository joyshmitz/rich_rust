//! Unicode character cell width calculations.
//!
//! This module provides functions to calculate the display width of text
//! in terminal cells, handling wide characters (CJK, emoji) correctly.

use std::num::NonZeroUsize;
use std::sync::{LazyLock, Mutex};

use lru::LruCache;
use unicode_width::UnicodeWidthChar;

/// Minimum string length to cache (shorter strings have minimal overhead).
const CACHE_MIN_LEN: usize = 8;

/// LRU cache for `cell_len` calculations.
/// Per `RICH_SPEC.md` Section 12.4, string widths should be cached.
static CELL_LEN_CACHE: LazyLock<Mutex<LruCache<String, usize>>> =
    LazyLock::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(1024).expect("non-zero"))));

/// Get the cell width of a single character.
///
/// Most characters are 1 cell wide, but CJK characters and some emoji
/// are 2 cells wide. Control characters have 0 width.
#[must_use]
pub fn get_character_cell_size(c: char) -> usize {
    c.width().unwrap_or(0)
}

/// Compute cell width by summing character widths.
///
/// This ensures consistent handling of control characters (width 0)
/// using the same logic as `get_character_cell_size`.
#[inline]
fn compute_cell_width(text: &str) -> usize {
    text.chars().map(get_character_cell_size).sum()
}

/// Get the total cell width of a string (cached for longer strings).
///
/// This is the sum of the widths of all characters, accounting for
/// wide characters that take 2 cells. Control characters have 0 width.
///
/// Per `RICH_SPEC.md` Section 12.4, results are cached using an LRU cache
/// for strings of 8+ characters to avoid repeated calculations.
#[must_use]
pub fn cell_len(text: &str) -> usize {
    // Short strings: compute directly (cache overhead not worth it)
    if text.len() < CACHE_MIN_LEN {
        return compute_cell_width(text);
    }

    // Check cache first
    if let Ok(mut cache) = CELL_LEN_CACHE.lock()
        && let Some(&cached) = cache.get(text)
    {
        return cached;
    }

    // Compute width using character-level function for consistency
    let width = compute_cell_width(text);

    // Store in cache
    if let Ok(mut cache) = CELL_LEN_CACHE.lock() {
        cache.put(text.to_string(), width);
    }

    width
}

/// Get the total cell width of a string without caching.
///
/// Use this when you know the string is unique or when you want to
/// avoid cache overhead for single-use calculations.
#[must_use]
pub fn cell_len_uncached(text: &str) -> usize {
    compute_cell_width(text)
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
/// Returns a vector of (`byte_index`, `cell_position`) pairs.
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
        assert_eq!(cell_len("中文"), 4); // 2 characters * 2 cells
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

    // ============================================================================
    // SPEC VALIDATION TESTS - RICH_SPEC.md Section 12 (Unicode Cell Width)
    // ============================================================================

    // 12.1 Cell Width Concept - Most characters = 1 cell, CJK/emoji = 2 cells
    #[test]
    fn test_spec_basic_width_concept() {
        // ASCII printable = 1 cell
        for c in ' '..='~' {
            assert_eq!(
                get_character_cell_size(c),
                1,
                "ASCII '{c}' should be 1 cell"
            );
        }

        // CJK = 2 cells per character
        let cjk_chars = ['日', '本', '語', '中', '文', '한', '국', '어'];
        for c in cjk_chars {
            assert_eq!(get_character_cell_size(c), 2, "CJK '{c}' should be 2 cells");
        }

        // Control characters = 0 cells
        assert_eq!(get_character_cell_size('\x00'), 0); // NULL
        assert_eq!(get_character_cell_size('\x01'), 0); // SOH
        assert_eq!(get_character_cell_size('\x1f'), 0); // US
    }

    // 12.2 Cell Width Table - Verify specific Unicode ranges
    #[test]
    fn test_spec_cell_width_ranges() {
        // Combining diacritical marks (768-879) = 0 width
        assert_eq!(get_character_cell_size('\u{0300}'), 0); // Combining grave accent
        assert_eq!(get_character_cell_size('\u{0301}'), 0); // Combining acute accent

        // Hangul Jamo (4352-4447) = 2 width
        assert_eq!(get_character_cell_size('\u{1100}'), 2); // Hangul Choseong Kiyeok

        // Ideographic space (12288) = 2 width
        assert_eq!(get_character_cell_size('\u{3000}'), 2); // Ideographic space

        // CJK Unified Ideographs (19968-40956) = 2 width
        assert_eq!(get_character_cell_size('\u{4E00}'), 2); // CJK character "one"
        assert_eq!(get_character_cell_size('\u{9FCC}'), 2); // Another CJK character
    }

    // 12.3 Fast-Path Detection - ASCII should be efficient
    #[test]
    fn test_spec_ascii_fast_path() {
        // Printable ASCII (0x20-0x7E) = 1 cell
        assert_eq!(get_character_cell_size(' '), 1); // 0x20
        assert_eq!(get_character_cell_size('~'), 1); // 0x7E
        assert_eq!(get_character_cell_size('A'), 1);
        assert_eq!(get_character_cell_size('z'), 1);
        assert_eq!(get_character_cell_size('0'), 1);
        assert_eq!(get_character_cell_size('!'), 1);

        // Latin Extended (0xA0-0x02FF) = 1 cell
        assert_eq!(get_character_cell_size('\u{00A0}'), 1); // Non-breaking space
        assert_eq!(get_character_cell_size('é'), 1); // e with acute
        assert_eq!(get_character_cell_size('ñ'), 1); // n with tilde
    }

    // 12.4 Cell Width Algorithm - Total string width
    #[test]
    fn test_spec_cell_len_algorithm() {
        // Pure ASCII
        assert_eq!(cell_len("hello"), 5);
        assert_eq!(cell_len(""), 0);

        // Pure CJK (each char = 2 cells)
        assert_eq!(cell_len("日本語"), 6); // 3 chars * 2 cells
        assert_eq!(cell_len("中文测试"), 8); // 4 chars * 2 cells

        // Mixed ASCII and CJK
        assert_eq!(cell_len("Hello日本"), 9); // 5 + 2*2
        assert_eq!(cell_len("a中b"), 4); // 1 + 2 + 1

        // Note: Control character handling is tested in test_control_characters
        // The behavior can vary between char.width() and str.width() in unicode_width
    }

    // 12.5 Cell-Based String Operations - set_cell_size
    #[test]
    fn test_spec_set_cell_size_operations() {
        // Exact fit
        assert_eq!(set_cell_size("hello", 5), "hello");

        // Padding needed
        let padded = set_cell_size("hi", 5);
        assert_eq!(padded, "hi   ");
        assert_eq!(cell_len(&padded), 5);

        // Truncation needed
        let truncated = set_cell_size("hello world", 5);
        assert_eq!(truncated, "hello");
        assert_eq!(cell_len(&truncated), 5);

        // CJK truncation - must handle partial wide characters
        let cjk_trunc = set_cell_size("日本語", 5);
        // Can only fit 2 full characters (4 cells), need 1 space to reach 5
        assert_eq!(cell_len(&cjk_trunc), 5);
        assert!(cjk_trunc.starts_with("日本"));

        // Mixed truncation
        let mixed = set_cell_size("Hello日本", 7);
        assert_eq!(cell_len(&mixed), 7);
    }

    // 12.5 Cell-Based String Operations - chop_cells
    #[test]
    fn test_spec_chop_cells_operations() {
        // ASCII chopping
        let (left, right) = chop_cells("hello world", 5);
        assert_eq!(left, "hello");
        assert_eq!(right, " world");

        // CJK chopping - stops before exceeding width
        let (left, right) = chop_cells("日本語", 3);
        assert_eq!(left, "日"); // 2 cells, next would be 4
        assert_eq!(right, "本語");
        assert_eq!(cell_len(left), 2);

        // Exact width boundary
        let (left, right) = chop_cells("日本語", 4);
        assert_eq!(left, "日本"); // Exactly 4 cells
        assert_eq!(right, "語");

        // Zero width
        let (left, right) = chop_cells("hello", 0);
        assert_eq!(left, "");
        assert_eq!(right, "hello");
    }

    // Additional: cell_positions mapping
    #[test]
    fn test_spec_cell_positions_mapping() {
        // Pure ASCII - byte pos = cell pos
        let pos = cell_positions("abc");
        assert_eq!(pos, vec![(0, 0), (1, 1), (2, 2)]);

        // Mixed content - cell positions account for wide chars
        let pos = cell_positions("a日b");
        assert_eq!(pos[0], (0, 0)); // 'a' at byte 0, cell 0
        assert_eq!(pos[1], (1, 1)); // '日' at byte 1, cell 1
        assert_eq!(pos[2], (4, 3)); // 'b' at byte 4 (日 is 3 bytes), cell 3 (日 is 2 cells)
    }

    // Additional: cell_to_byte_index conversion
    #[test]
    fn test_spec_cell_to_byte_index() {
        // ASCII
        assert_eq!(cell_to_byte_index("hello", 0), Some(0));
        assert_eq!(cell_to_byte_index("hello", 3), Some(3));
        assert_eq!(cell_to_byte_index("hello", 5), Some(5));
        assert_eq!(cell_to_byte_index("hello", 10), None);

        // With wide characters
        let s = "a日b";
        assert_eq!(cell_to_byte_index(s, 0), Some(0)); // 'a'
        assert_eq!(cell_to_byte_index(s, 1), Some(1)); // '日' starts
        assert_eq!(cell_to_byte_index(s, 3), Some(4)); // 'b'
    }

    // Additional: has_wide_chars detection
    #[test]
    fn test_spec_has_wide_chars() {
        // ASCII only
        assert!(!has_wide_chars("hello world"));
        assert!(!has_wide_chars("Hello, World! 123"));
        assert!(!has_wide_chars(""));

        // Contains wide chars
        assert!(has_wide_chars("日"));
        assert!(has_wide_chars("Hello日本"));
        assert!(has_wide_chars("a中b文c"));
    }

    // Edge case: Empty strings
    #[test]
    fn test_spec_empty_string_handling() {
        assert_eq!(cell_len(""), 0);
        assert_eq!(set_cell_size("", 5), "     ");
        let (left, right) = chop_cells("", 5);
        assert_eq!(left, "");
        assert_eq!(right, "");
        assert!(cell_positions("").is_empty());
    }

    // Edge case: Full-width punctuation
    #[test]
    fn test_spec_fullwidth_punctuation() {
        // Full-width forms (U+FF00-U+FF5E) should be 2 cells
        assert_eq!(get_character_cell_size('！'), 2); // Full-width exclamation
        assert_eq!(get_character_cell_size('Ａ'), 2); // Full-width A
        assert_eq!(cell_len("！Ａ"), 4);
    }

    // LRU cache behavior (per RICH_SPEC.md Section 12.4)
    #[test]
    fn test_cell_len_caching() {
        // Short strings (< 8 chars) bypass cache
        let short = "hello";
        assert_eq!(cell_len(short), 5);
        assert_eq!(cell_len(short), 5); // Same result

        // Long strings use cache
        let long = "Hello, this is a longer string for testing";
        let width1 = cell_len(long);
        let width2 = cell_len(long); // Should hit cache
        assert_eq!(width1, width2);
        assert_eq!(width1, 42);

        // Verify uncached version gives same result
        assert_eq!(cell_len_uncached(long), 42);

        // CJK strings
        let cjk_long = "日本語テスト文字列";
        let cjk_width = cell_len(cjk_long);
        assert_eq!(cjk_width, 18); // 9 chars * 2 cells
        assert_eq!(cell_len(cjk_long), cjk_width); // Cache hit
    }
}
