//! Rule - horizontal line with optional title.
//!
//! A Rule renders as a horizontal line that spans the console width,
//! optionally with a centered (or aligned) title.

use crate::cells;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::{JustifyMethod, Text};

/// A horizontal rule with optional title.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Optional title text.
    title: Option<Text>,
    /// Character to use for the rule line.
    character: String,
    /// Style for the rule line.
    style: Style,
    /// Title alignment.
    align: JustifyMethod,
}

impl Default for Rule {
    fn default() -> Self {
        Self {
            title: None,
            character: String::from("\u{2500}"), // ─
            style: Style::new(),
            align: JustifyMethod::Center,
        }
    }
}

impl Rule {
    /// Create a new rule without a title.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a rule with a title.
    #[must_use]
    pub fn with_title(title: impl Into<Text>) -> Self {
        Self {
            title: Some(title.into()),
            ..Self::default()
        }
    }

    /// Set the rule character.
    #[must_use]
    pub fn character(mut self, ch: impl Into<String>) -> Self {
        self.character = ch.into();
        self
    }

    /// Set the rule style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set title alignment.
    #[must_use]
    pub fn align(mut self, align: JustifyMethod) -> Self {
        self.align = align;
        self
    }

    /// Left-align the title.
    #[must_use]
    pub fn align_left(self) -> Self {
        self.align(JustifyMethod::Left)
    }

    /// Center the title.
    #[must_use]
    pub fn align_center(self) -> Self {
        self.align(JustifyMethod::Center)
    }

    /// Right-align the title.
    #[must_use]
    pub fn align_right(self) -> Self {
        self.align(JustifyMethod::Right)
    }

    /// Render the rule to segments for a given width.
    #[must_use]
    pub fn render(&self, width: usize) -> Vec<Segment> {
        let char_width = cells::cell_len(&self.character);
        if char_width == 0 || width == 0 {
            return vec![Segment::line()];
        }

        let mut segments = Vec::new();

        match &self.title {
            Some(title) => {
                // Render title with surrounding spaces
                let title_text = format!(" {} ", title.plain());
                let title_width = cells::cell_len(&title_text);

                // Calculate available space for rule characters
                let available = width.saturating_sub(title_width);
                let rule_chars = available / char_width;

                if rule_chars < 2 {
                    // Not enough space for rule, just show title
                    segments.push(Segment::new(&title_text, Some(title.style().clone())));
                } else {
                    let (left_count, right_count) = match self.align {
                        JustifyMethod::Left | JustifyMethod::Default => (1, rule_chars - 1),
                        JustifyMethod::Right => (rule_chars - 1, 1),
                        JustifyMethod::Center | JustifyMethod::Full => {
                            let left = rule_chars / 2;
                            let right = rule_chars - left;
                            (left, right)
                        }
                    };

                    // Left rule section
                    let left_rule = self.character.repeat(left_count);
                    segments.push(Segment::new(&left_rule, Some(self.style.clone())));

                    // Title
                    segments.push(Segment::new(&title_text, Some(title.style().clone())));

                    // Right rule section
                    let right_rule = self.character.repeat(right_count);
                    segments.push(Segment::new(&right_rule, Some(self.style.clone())));
                }
            }
            None => {
                // No title, just a full-width rule
                let count = width / char_width;
                let rule_text = self.character.repeat(count);
                segments.push(Segment::new(&rule_text, Some(self.style.clone())));
            }
        }

        segments.push(Segment::line());
        segments
    }

    /// Render the rule as a string (for simple output).
    #[must_use]
    pub fn render_plain(&self, width: usize) -> String {
        self.render(width)
            .into_iter()
            .map(|seg| seg.text)
            .collect()
    }
}

/// Create an ASCII-safe rule.
#[must_use]
pub fn ascii_rule() -> Rule {
    Rule::new().character("-")
}

/// Create a double-line rule.
#[must_use]
pub fn double_rule() -> Rule {
    Rule::new().character("\u{2550}") // ═
}

/// Create a heavy (thick) rule.
#[must_use]
pub fn heavy_rule() -> Rule {
    Rule::new().character("\u{2501}") // ━
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_no_title() {
        let rule = Rule::new();
        let segments = rule.render(10);
        assert!(!segments.is_empty());
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains('\u{2500}')); // ─
    }

    #[test]
    fn test_rule_with_title() {
        let rule = Rule::with_title("Test");
        let segments = rule.render(20);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("Test"));
        assert!(text.contains('\u{2500}')); // ─
    }

    #[test]
    fn test_rule_custom_char() {
        let rule = Rule::new().character("=");
        let segments = rule.render(10);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains('='));
    }

    #[test]
    fn test_rule_alignment() {
        let rule = Rule::with_title("X").align_left();
        let plain = rule.render_plain(20);
        // Title should be near the left, so more rule chars on right
        let parts: Vec<&str> = plain.trim().split(" X ").collect();
        assert_eq!(parts.len(), 2);
        assert!(parts[0].len() < parts[1].len());
    }

    #[test]
    fn test_ascii_rule() {
        let rule = ascii_rule();
        let segments = rule.render(10);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains('-'));
    }

    #[test]
    fn test_heavy_rule() {
        let rule = heavy_rule();
        let segments = rule.render(10);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains('\u{2501}')); // ━
    }
}
