//! Traceback rendering.
//!
//! This is a Rust-idiomatic approximation of Python Rich's `rich.traceback`.
//! For deterministic testing and conformance fixtures, this implementation
//! supports rendering from **synthetic frames** (function name + line number).
//!
//! A higher-fidelity Rust backtrace capture layer can be built on top by
//! converting a backtrace to [`TracebackFrame`]s.

use crate::console::{Console, ConsoleOptions};
use crate::renderables::Renderable;
use crate::segment::Segment;
use crate::text::Text;

use super::panel::Panel;

/// A single traceback frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TracebackFrame {
    pub filename: Option<String>,
    pub name: String,
    pub line: usize,
    /// Optional source code snippet for this frame.
    ///
    /// When provided, this is used instead of reading from the filesystem.
    /// This enables deterministic testing and rendering without file access.
    /// The snippet should contain lines around the error, with the error line
    /// being at position `line` (1-indexed relative to the start of the snippet's
    /// first line number, specified by `source_first_line`).
    pub source_context: Option<String>,
    /// The line number of the first line in `source_context`.
    /// Defaults to 1 if not specified.
    pub source_first_line: usize,
}

impl TracebackFrame {
    #[must_use]
    pub fn new(name: impl Into<String>, line: usize) -> Self {
        Self {
            filename: None,
            name: name.into(),
            line,
            source_context: None,
            source_first_line: 1,
        }
    }

    #[must_use]
    pub fn filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// Provide source context lines directly instead of reading from filesystem.
    ///
    /// This is useful for:
    /// - Deterministic testing without filesystem dependencies
    /// - Rendering tracebacks when source files are not available
    /// - Embedding source snippets from memory
    ///
    /// # Arguments
    /// * `source` - The source code snippet (may contain multiple lines)
    /// * `first_line` - The line number of the first line in the snippet
    ///
    /// # Example
    /// ```
    /// use rich_rust::renderables::TracebackFrame;
    ///
    /// let frame = TracebackFrame::new("my_function", 5)
    ///     .source_context("fn my_function() {\n    let x = 1;\n    panic!(\"oops\");\n}", 3);
    /// ```
    #[must_use]
    pub fn source_context(mut self, source: impl Into<String>, first_line: usize) -> Self {
        self.source_context = Some(source.into());
        self.source_first_line = first_line.max(1);
        self
    }
}

/// A rendered traceback, inspired by Python Rich.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Traceback {
    frames: Vec<TracebackFrame>,
    exception_type: String,
    exception_message: String,
    title: Text,
    extra_lines: usize,
}

impl Traceback {
    /// Create a new traceback from frames and exception info.
    #[must_use]
    pub fn new(
        frames: impl Into<Vec<TracebackFrame>>,
        exception_type: impl Into<String>,
        exception_message: impl Into<String>,
    ) -> Self {
        Self {
            frames: frames.into(),
            exception_type: exception_type.into(),
            exception_message: exception_message.into(),
            title: Text::new("Traceback (most recent call last)"),
            extra_lines: 0,
        }
    }

    /// Override the title shown in the traceback panel.
    #[must_use]
    pub fn title(mut self, title: impl Into<Text>) -> Self {
        self.title = title.into();
        self
    }

    #[must_use]
    pub fn extra_lines(mut self, extra_lines: usize) -> Self {
        self.extra_lines = extra_lines;
        self
    }

    /// Push a frame (builder-style).
    pub fn push_frame(&mut self, frame: TracebackFrame) {
        self.frames.push(frame);
    }

    /// Get source for a frame, preferring provided context over filesystem.
    ///
    /// Returns `Some((source, first_line))` if source is available,
    /// `None` if no source can be obtained.
    fn get_frame_source(&self, frame: &TracebackFrame) -> Option<(String, usize)> {
        // Priority 1: Use provided source context
        if let Some(ref source) = frame.source_context {
            return Some((source.clone(), frame.source_first_line));
        }

        // Priority 2: Read from filesystem if filename is provided
        if let Some(ref filename) = frame.filename
            && let Ok(source) = std::fs::read_to_string(filename)
        {
            return Some((source, 1));
        }

        None
    }
}

impl Renderable for Traceback {
    fn render<'a>(&'a self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment<'a>> {
        let width = options.max_width.max(1);

        let mut content_lines: Vec<Vec<Segment<'static>>> = Vec::new();
        for frame in &self.frames {
            // Try to get source: first from provided context, then from filesystem
            let source_result = self.get_frame_source(frame);

            if let Some((source, first_line)) = source_result {
                // Render frame header with location info
                let location = if let Some(filename) = frame.filename.as_deref() {
                    format!("{filename}:{} in {}", frame.line, frame.name)
                } else {
                    format!("in {}:{}", frame.name, frame.line)
                };
                content_lines.push(vec![Segment::new(location, None)]);
                content_lines.push(vec![Segment::new(String::new(), None)]);

                // Render source context with line numbers
                let source_lines: Vec<&str> = source.lines().collect();
                let last_line = first_line + source_lines.len().saturating_sub(1);

                // Calculate which lines to show based on extra_lines
                let start = frame.line.saturating_sub(self.extra_lines).max(first_line);
                let end = (frame.line + self.extra_lines).min(last_line);

                if start <= end && frame.line >= first_line && frame.line <= last_line {
                    let line_number_width = end.to_string().len() + 5;

                    for line_no in start..=end {
                        let source_idx = line_no.saturating_sub(first_line);
                        if source_idx < source_lines.len() {
                            let code = source_lines[source_idx].trim_start();
                            let indicator = if line_no == frame.line { "❱" } else { " " };
                            let line = format!("{indicator} {line_no:<line_number_width$}{code}");
                            content_lines.push(vec![Segment::new(line, None)]);
                        }
                    }
                }

                continue;
            }

            // Fallback: no source available, just show frame info
            content_lines.push(vec![Segment::new(
                format!("in {}:{}", frame.name, frame.line),
                None,
            )]);
        }

        let panel = Panel::new(content_lines)
            .title(self.title.clone())
            .width(width);
        let mut segments: Vec<Segment<'static>> = panel.render(width);

        segments.push(Segment::new(
            format!("{}: {}", self.exception_type, self.exception_message),
            None,
        ));
        segments.push(Segment::line());

        segments.into_iter().collect()
    }
}

/// Convenience helper mirroring Python Rich's `Console.print_exception`.
///
/// Rust doesn't have a standardized structured backtrace API across error
/// types; for now, this prints a provided [`Traceback`] renderable.
pub fn print_exception(console: &Console, traceback: &Traceback) {
    console.print_exception(traceback);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_to_text(traceback: &Traceback, width: usize) -> String {
        let console = Console::new();
        let options = ConsoleOptions {
            max_width: width,
            ..Default::default()
        };
        let segments = traceback.render(&console, &options);
        segments.iter().map(|s| s.text.as_ref()).collect()
    }

    #[test]
    fn frame_without_source_shows_minimal_info() {
        let frame = TracebackFrame::new("my_func", 42);
        let traceback = Traceback::new(vec![frame], "Error", "something went wrong");

        let output = render_to_text(&traceback, 80);
        assert!(output.contains("my_func"));
        assert!(output.contains("42"));
        assert!(output.contains("Error: something went wrong"));
    }

    #[test]
    fn frame_with_source_context_renders_code() {
        let source = "fn main() {\n    let x = 1;\n    panic!(\"oops\");\n    let y = 2;\n}";
        let frame = TracebackFrame::new("main", 3).source_context(source, 1);
        let traceback = Traceback::new(vec![frame], "PanicError", "oops").extra_lines(1);

        let output = render_to_text(&traceback, 80);

        // Should show the error line with indicator
        assert!(output.contains("❱"));
        assert!(output.contains("panic!"));

        // Should show context lines (extra_lines=1)
        assert!(output.contains("let x = 1"));
        assert!(output.contains("let y = 2"));

        // Should show exception info
        assert!(output.contains("PanicError: oops"));
    }

    #[test]
    fn source_context_with_offset_first_line() {
        // Simulating a snippet from lines 10-14 of a larger file
        let source = "    fn helper() {\n        do_thing();\n        crash_here();\n    }\n";
        let frame = TracebackFrame::new("helper", 12).source_context(source, 10);
        let traceback = Traceback::new(vec![frame], "Error", "crashed").extra_lines(1);

        let output = render_to_text(&traceback, 80);

        // Should show line 12 with indicator
        assert!(output.contains("❱"));
        assert!(output.contains("12"));
        assert!(output.contains("crash_here"));

        // Should show context (lines 11 and 13)
        assert!(output.contains("11"));
        assert!(output.contains("do_thing"));
    }

    #[test]
    fn source_context_takes_priority_over_filename() {
        // Even if filename is set, source_context should be used
        let source = "custom source line";
        let frame = TracebackFrame::new("func", 1)
            .filename("/nonexistent/file.rs")
            .source_context(source, 1);
        let traceback = Traceback::new(vec![frame], "Error", "test");

        let output = render_to_text(&traceback, 80);

        // Should render the provided source, not try to read file
        assert!(output.contains("custom source line"));
        // Should still show filename in header
        assert!(output.contains("/nonexistent/file.rs"));
    }

    #[test]
    fn extra_lines_zero_shows_only_error_line() {
        let source = "line1\nline2\nline3\nline4\nline5";
        let frame = TracebackFrame::new("func", 3).source_context(source, 1);
        let traceback = Traceback::new(vec![frame], "Error", "test").extra_lines(0);

        let output = render_to_text(&traceback, 80);

        // Should show only line 3
        assert!(output.contains("line3"));
        assert!(output.contains("❱"));
        // Should not show other lines
        assert!(!output.contains("line1"));
        assert!(!output.contains("line5"));
    }

    #[test]
    fn multiple_frames_with_source_context() {
        let frame1 =
            TracebackFrame::new("outer", 2).source_context("fn outer() {\n    inner();\n}", 1);
        let frame2 =
            TracebackFrame::new("inner", 2).source_context("fn inner() {\n    panic!();\n}", 1);

        let traceback = Traceback::new(vec![frame1, frame2], "PanicError", "boom");

        let output = render_to_text(&traceback, 80);

        // Both frames should be rendered
        assert!(output.contains("outer"));
        assert!(output.contains("inner"));
        assert!(output.contains("PanicError: boom"));
    }

    #[test]
    fn frame_builder_methods() {
        let frame = TracebackFrame::new("test", 10)
            .filename("src/lib.rs")
            .source_context("test code", 5);

        assert_eq!(frame.name, "test");
        assert_eq!(frame.line, 10);
        assert_eq!(frame.filename, Some("src/lib.rs".to_string()));
        assert_eq!(frame.source_context, Some("test code".to_string()));
        assert_eq!(frame.source_first_line, 5);
    }

    #[test]
    fn source_first_line_minimum_is_one() {
        let frame = TracebackFrame::new("test", 1).source_context("code", 0);
        assert_eq!(frame.source_first_line, 1);
    }
}
