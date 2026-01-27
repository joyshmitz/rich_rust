//! Pretty printing / inspection helpers.
//!
//! This is a Rust-idiomatic approximation of Python Rich's `rich.pretty` and
//! `rich.inspect` modules.
//!
//! ## Differences vs Python Rich
//!
//! Rust doesn't support general-purpose runtime reflection of struct fields and
//! attributes (like Python). As a result:
//! - `Pretty` renders values via their `Debug` representation.
//! - `Inspect` can show the Rust type name and a pretty representation; it may
//!   also extract *simple* top-level `Debug` struct fields when available, but
//!   this depends on the `Debug` implementation.

use std::any;
use std::fmt::Debug;

use crate::cells::cell_len;
use crate::console::{Console, ConsoleOptions};
use crate::renderables::Renderable;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;

use super::table::{Column, Table};

/// Configuration for [`Pretty`].
#[derive(Debug, Clone)]
pub struct PrettyOptions {
    /// Override the width used for wrapping (defaults to `ConsoleOptions.max_width`).
    pub max_width: Option<usize>,
    /// If true, use compact `Debug` (`{:?}`) instead of pretty `Debug` (`{:#?}`).
    pub compact: bool,
    /// If true, wrap long lines to `max_width`.
    pub wrap: bool,
}

impl Default for PrettyOptions {
    fn default() -> Self {
        Self {
            max_width: None,
            compact: false,
            wrap: true,
        }
    }
}

/// Render a Rust value using a stable, width-aware `Debug` representation.
///
/// This is a best-effort, Rust-idiomatic "pretty printer" intended for use in
/// terminal UIs.
#[derive(Debug)]
pub struct Pretty<'a, T: Debug + ?Sized> {
    value: &'a T,
    options: PrettyOptions,
    style: Option<Style>,
}

impl<'a, T: Debug + ?Sized> Pretty<'a, T> {
    /// Create a new [`Pretty`] wrapper.
    #[must_use]
    pub fn new(value: &'a T) -> Self {
        Self {
            value,
            options: PrettyOptions::default(),
            style: None,
        }
    }

    /// Override the wrapping width.
    #[must_use]
    pub fn max_width(mut self, width: usize) -> Self {
        self.options.max_width = Some(width);
        self
    }

    /// Render using compact `Debug` output (`{:?}`).
    #[must_use]
    pub fn compact(mut self, compact: bool) -> Self {
        self.options.compact = compact;
        self
    }

    /// Enable/disable wrapping.
    #[must_use]
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.options.wrap = wrap;
        self
    }

    /// Apply a style to the entire pretty output.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }
}

impl<T: Debug + ?Sized> Renderable for Pretty<'_, T> {
    fn render<'a>(&'a self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment<'a>> {
        let width = self.options.max_width.unwrap_or(options.max_width).max(1);

        let repr = if self.options.compact {
            format!("{:?}", self.value)
        } else {
            format!("{:#?}", self.value)
        };

        let lines: Vec<String> = if self.options.wrap {
            wrap_debug_preserving_indent(&repr, width)
        } else {
            repr.lines().map(str::to_string).collect()
        };

        let mut segments: Vec<Segment<'static>> = Vec::new();
        let line_count = lines.len();
        for (idx, line) in lines.into_iter().enumerate() {
            segments.push(Segment::new(line, self.style.clone()));
            if idx + 1 < line_count {
                segments.push(Segment::line());
            }
        }

        segments.into_iter().collect()
    }
}

/// Configuration for [`Inspect`].
#[derive(Debug, Clone)]
pub struct InspectOptions {
    /// Override the width used for rendering (defaults to `ConsoleOptions.max_width`).
    pub max_width: Option<usize>,
    /// Show the Rust type name.
    pub show_type: bool,
    /// Attempt to extract simple top-level fields from `Debug` output.
    pub show_fields: bool,
}

impl Default for InspectOptions {
    fn default() -> Self {
        Self {
            max_width: None,
            show_type: true,
            show_fields: true,
        }
    }
}

/// Inspect a Rust value: show its type and a readable representation.
///
/// This is inspired by Python Rich's `inspect`, but is limited by Rust's lack
/// of runtime reflection. Field extraction is best-effort and relies on the
/// `Debug` output format.
#[derive(Debug)]
pub struct Inspect<'a, T: Debug + ?Sized> {
    value: &'a T,
    options: InspectOptions,
}

impl<'a, T: Debug + ?Sized> Inspect<'a, T> {
    /// Create a new inspector.
    #[must_use]
    pub fn new(value: &'a T) -> Self {
        Self {
            value,
            options: InspectOptions::default(),
        }
    }

    /// Override the rendering width.
    #[must_use]
    pub fn max_width(mut self, width: usize) -> Self {
        self.options.max_width = Some(width);
        self
    }

    /// Show/hide the type line.
    #[must_use]
    pub fn show_type(mut self, show: bool) -> Self {
        self.options.show_type = show;
        self
    }

    /// Enable/disable field extraction.
    #[must_use]
    pub fn show_fields(mut self, show: bool) -> Self {
        self.options.show_fields = show;
        self
    }
}

impl<T: Debug + ?Sized> Renderable for Inspect<'_, T> {
    fn render<'a>(&'a self, console: &Console, options: &ConsoleOptions) -> Vec<Segment<'a>> {
        let width = self.options.max_width.unwrap_or(options.max_width).max(1);

        let mut output: Vec<Segment<'static>> = Vec::new();

        if self.options.show_type {
            let type_name = any::type_name_of_val(self.value);
            let header =
                Text::assemble(&[("Type: ", Some(Style::new().bold())), (type_name, None)]);
            output.extend(header.render("").into_iter().map(Segment::into_owned));
            output.push(Segment::line());
        }

        if self.options.show_fields {
            let repr = format!("{:#?}", self.value);
            if let Some(fields) = extract_simple_struct_fields(&repr) {
                let mut table = Table::new()
                    .with_column(Column::new("Field").style(Style::new().bold()))
                    .with_column(Column::new("Value"));
                for (name, value) in fields {
                    table.add_row_cells([name, value]);
                }
                let mut rendered: Vec<Segment<'static>> = table.render(width);
                output.append(&mut rendered);
                return output.into_iter().collect();
            }
        }

        let pretty = Pretty::new(self.value).max_width(width);
        output.extend(
            pretty
                .render(console, options)
                .into_iter()
                .map(Segment::into_owned),
        );
        output.into_iter().collect()
    }
}

/// Convenience helper to print an [`Inspect`] view to a [`Console`].
pub fn inspect<T: Debug + ?Sized>(console: &Console, value: &T) {
    let renderable = Inspect::new(value);
    console.print_renderable(&renderable);
}

fn wrap_debug_preserving_indent(text: &str, width: usize) -> Vec<String> {
    text.lines()
        .flat_map(|line| wrap_line_preserving_indent(line, width))
        .collect()
}

fn wrap_line_preserving_indent(line: &str, width: usize) -> Vec<String> {
    let indent_len = line.chars().take_while(|c| c.is_whitespace()).count();
    let indent: String = line.chars().take(indent_len).collect();
    let rest: String = line.chars().skip(indent_len).collect();

    let indent_width = cell_len(&indent);
    if rest.is_empty() || width <= indent_width + 1 {
        return vec![line.to_string()];
    }

    let available = width.saturating_sub(indent_width).max(1);
    let wrapped = Text::new(rest).wrap(available);
    wrapped
        .into_iter()
        .map(|t| format!("{indent}{}", t.plain()))
        .collect()
}

fn extract_simple_struct_fields(repr: &str) -> Option<Vec<(String, String)>> {
    let mut lines = repr.lines();
    let first = lines.next()?.trim_end();
    if !first.ends_with('{') {
        return None;
    }

    let mut fields = Vec::new();
    for line in lines {
        let trimmed = line.trim_end();
        if trimmed == "}" {
            break;
        }
        // Only consider simple `Debug` fields which are single-line.
        let Some(stripped) = trimmed.strip_prefix("    ") else {
            continue;
        };
        let Some((name, value)) = stripped.split_once(':') else {
            continue;
        };
        let name = name.trim().to_string();
        if name.is_empty() {
            continue;
        }
        let mut value = value.trim().to_string();
        if value.ends_with(',') {
            value.pop();
            value = value.trim_end().to_string();
        }
        if value.is_empty() {
            continue;
        }
        fields.push((name, value));
    }

    if fields.is_empty() {
        None
    } else {
        Some(fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::Console;

    #[derive(Debug)]
    #[allow(dead_code)]
    struct Inner {
        name: String,
        values: Vec<i32>,
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    struct Outer {
        id: u32,
        inner: Inner,
    }

    fn test_console(width: usize) -> Console {
        Console::builder()
            .no_color()
            .force_terminal(false)
            .emoji(false)
            .markup(false)
            .highlight(false)
            .width(width)
            .build()
    }

    #[test]
    fn pretty_wraps_to_width_and_is_stable() {
        let value = Outer {
            id: 42,
            inner: Inner {
                name: "a-very-long-name-to-wrap".to_string(),
                values: vec![1, 2, 3, 4, 5],
            },
        };
        let console = test_console(22);
        let pretty = Pretty::new(&value);
        let plain = console.export_renderable_text(&pretty);
        insta::assert_snapshot!(plain);
    }

    #[test]
    fn inspect_shows_type_and_fields_when_available() {
        let value = Inner {
            name: "Zed".to_string(),
            values: vec![1, 2, 3],
        };
        let console = test_console(60);
        let inspect = Inspect::new(&value);
        let plain = console.export_renderable_text(&inspect);
        insta::assert_snapshot!(plain);
    }
}
