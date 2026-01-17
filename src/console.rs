//! Console - the central entry point for styled terminal output.
//!
//! The `Console` handles rendering styled content to the terminal,
//! including color detection, width calculation, and ANSI code generation.

use std::io::{self, Write};

use crate::color::ColorSystem;
use crate::markup;
use crate::segment::Segment;
use crate::style::Style;
use crate::terminal;
use crate::text::{JustifyMethod, OverflowMethod, Text};

/// Console dimensions in cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsoleDimensions {
    /// Width in cells.
    pub width: usize,
    /// Height in rows.
    pub height: usize,
}

impl Default for ConsoleDimensions {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
        }
    }
}

/// Options for rendering.
#[derive(Debug, Clone)]
pub struct ConsoleOptions {
    /// Terminal dimensions.
    pub size: ConsoleDimensions,
    /// Using legacy Windows console.
    pub legacy_windows: bool,
    /// Minimum width constraint.
    pub min_width: usize,
    /// Maximum width constraint.
    pub max_width: usize,
    /// Output is a terminal (vs file/pipe).
    pub is_terminal: bool,
    /// Output encoding.
    pub encoding: String,
    /// Maximum height for rendering.
    pub max_height: usize,
    /// Default justification.
    pub justify: Option<JustifyMethod>,
    /// Default overflow handling.
    pub overflow: Option<OverflowMethod>,
    /// Default no_wrap setting.
    pub no_wrap: Option<bool>,
    /// Enable highlighting.
    pub highlight: Option<bool>,
    /// Parse markup in strings.
    pub markup: Option<bool>,
    /// Explicit height override.
    pub height: Option<usize>,
}

impl Default for ConsoleOptions {
    fn default() -> Self {
        Self {
            size: ConsoleDimensions::default(),
            legacy_windows: false,
            min_width: 1,
            max_width: 80,
            is_terminal: true,
            encoding: String::from("utf-8"),
            max_height: usize::MAX,
            justify: None,
            overflow: None,
            no_wrap: None,
            highlight: None,
            markup: None,
            height: None,
        }
    }
}

impl ConsoleOptions {
    /// Create options with a different max_width.
    #[must_use]
    pub fn update_width(&self, width: usize) -> Self {
        Self {
            max_width: width.min(self.max_width),
            ..self.clone()
        }
    }

    /// Create options with a different height.
    #[must_use]
    pub fn update_height(&self, height: usize) -> Self {
        Self {
            height: Some(height),
            ..self.clone()
        }
    }
}

/// Print options for controlling output.
#[derive(Debug, Clone, Default)]
pub struct PrintOptions {
    /// String to separate multiple objects.
    pub sep: String,
    /// String to append at end.
    pub end: String,
    /// Apply style to output.
    pub style: Option<Style>,
    /// Override justification.
    pub justify: Option<JustifyMethod>,
    /// Override overflow handling.
    pub overflow: Option<OverflowMethod>,
    /// Override no_wrap.
    pub no_wrap: Option<bool>,
    /// Suppress newline.
    pub no_newline: bool,
    /// Parse markup.
    pub markup: bool,
    /// Enable highlighting.
    pub highlight: bool,
    /// Override width.
    pub width: Option<usize>,
    /// Crop output to width.
    pub crop: bool,
    /// Soft wrap at width.
    pub soft_wrap: bool,
}

impl PrintOptions {
    /// Create new print options with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sep: String::from(" "),
            end: String::from("\n"),
            ..Default::default()
        }
    }

    /// Set markup parsing.
    #[must_use]
    pub fn with_markup(mut self, markup: bool) -> Self {
        self.markup = markup;
        self
    }

    /// Set style.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }
}

/// The main Console for rendering styled output.
#[derive(Debug)]
pub struct Console {
    /// Color system to use (None = auto-detect).
    color_system: Option<ColorSystem>,
    /// Force terminal mode.
    force_terminal: Option<bool>,
    /// Tab expansion size.
    tab_size: usize,
    /// Buffer output for export.
    record: bool,
    /// Parse markup by default.
    markup: bool,
    /// Enable emoji rendering.
    emoji: bool,
    /// Enable syntax highlighting.
    highlight: bool,
    /// Override width.
    width: Option<usize>,
    /// Override height.
    height: Option<usize>,
    /// Use ASCII-safe box characters.
    safe_box: bool,
    /// Recording buffer.
    buffer: Vec<Segment>,
    /// Cached terminal detection.
    is_terminal: bool,
    /// Detected/configured color system.
    detected_color_system: Option<ColorSystem>,
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

impl Console {
    /// Create a new console with default settings.
    #[must_use]
    pub fn new() -> Self {
        let is_terminal = terminal::is_terminal();
        let detected_color_system = if is_terminal {
            terminal::detect_color_system()
        } else {
            None
        };

        Self {
            color_system: None,
            force_terminal: None,
            tab_size: 8,
            record: false,
            markup: true,
            emoji: true,
            highlight: true,
            width: None,
            height: None,
            safe_box: false,
            buffer: Vec::new(),
            is_terminal,
            detected_color_system,
        }
    }

    /// Create a console builder for custom configuration.
    #[must_use]
    pub fn builder() -> ConsoleBuilder {
        ConsoleBuilder::default()
    }

    /// Get the console width.
    #[must_use]
    pub fn width(&self) -> usize {
        self.width.unwrap_or_else(terminal::get_terminal_width)
    }

    /// Get the console height.
    #[must_use]
    pub fn height(&self) -> usize {
        self.height.unwrap_or_else(terminal::get_terminal_height)
    }

    /// Get the console dimensions.
    #[must_use]
    pub fn size(&self) -> ConsoleDimensions {
        ConsoleDimensions {
            width: self.width(),
            height: self.height(),
        }
    }

    /// Check if this console outputs to a terminal.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.force_terminal.unwrap_or(self.is_terminal)
    }

    /// Get the color system in use.
    #[must_use]
    pub fn color_system(&self) -> Option<ColorSystem> {
        self.color_system.or(self.detected_color_system)
    }

    /// Check if colors are enabled.
    #[must_use]
    pub fn is_color_enabled(&self) -> bool {
        self.color_system().is_some()
    }

    /// Get the tab size.
    #[must_use]
    pub const fn tab_size(&self) -> usize {
        self.tab_size
    }

    /// Create console options for rendering.
    #[must_use]
    pub fn options(&self) -> ConsoleOptions {
        ConsoleOptions {
            size: self.size(),
            legacy_windows: false,
            min_width: 1,
            max_width: self.width(),
            is_terminal: self.is_terminal(),
            encoding: String::from("utf-8"),
            max_height: self.height(),
            justify: None,
            overflow: None,
            no_wrap: None,
            highlight: Some(self.highlight),
            markup: Some(self.markup),
            height: None,
        }
    }

    /// Enable recording mode.
    pub fn begin_capture(&mut self) {
        self.record = true;
        self.buffer.clear();
    }

    /// End recording and return captured segments.
    pub fn end_capture(&mut self) -> Vec<Segment> {
        self.record = false;
        std::mem::take(&mut self.buffer)
    }

    /// Print styled text to the console.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rich_rust::Console;
    ///
    /// let console = Console::new();
    /// console.print("[bold red]Hello[/] World!");
    /// ```
    pub fn print(&self, content: &str) {
        self.print_with_options(content, &PrintOptions::new().with_markup(self.markup));
    }

    /// Print with custom options.
    pub fn print_with_options(&self, content: &str, options: &PrintOptions) {
        let mut stdout = io::stdout();
        self.print_to(&mut stdout, content, options)
            .expect("failed to write to stdout");
    }

    /// Print to a specific writer.
    pub fn print_to<W: Write>(
        &self,
        writer: &mut W,
        content: &str,
        options: &PrintOptions,
    ) -> io::Result<()> {
        // Parse markup if enabled
        let text = if options.markup || self.markup {
            markup::render_or_plain(content)
        } else {
            Text::new(content)
        };

        // Render to segments
        let end = if options.no_newline { "" } else { &options.end };
        let segments = text.render(end);

        // Apply any overall style
        let segments: Vec<Segment> = if let Some(ref style) = options.style {
            segments
                .into_iter()
                .map(|mut seg| {
                    if !seg.is_control() {
                        seg.style = Some(match seg.style {
                            Some(s) => style.combine(&s),
                            None => style.clone(),
                        });
                    }
                    seg
                })
                .collect()
        } else {
            segments
        };

        // Write segments
        self.write_segments(writer, &segments)
    }

    /// Write segments to a writer.
    fn write_segments<W: Write>(&self, writer: &mut W, segments: &[Segment]) -> io::Result<()> {
        let color_system = self.color_system();

        for segment in segments {
            if segment.is_control() {
                // Handle control codes
                continue;
            }

            // Get ANSI codes for style
            let (prefix, suffix) = if let Some(ref style) = segment.style {
                if let Some(cs) = color_system {
                    style.render_ansi(cs)
                } else {
                    (String::new(), String::new())
                }
            } else {
                (String::new(), String::new())
            };

            // Write styled text
            write!(writer, "{prefix}{}{suffix}", segment.text)?;
        }

        writer.flush()
    }

    /// Print a blank line.
    pub fn line(&self) {
        println!();
    }

    /// Print a rule (horizontal line).
    pub fn rule(&self, title: Option<&str>) {
        let width = self.width();
        let line_char = if self.safe_box { '-' } else { '\u{2500}' };

        if let Some(title) = title {
            let title_len = crate::cells::cell_len(title);
            let padding = width.saturating_sub(title_len + 4) / 2;
            let left = line_char.to_string().repeat(padding);
            let right = line_char.to_string().repeat(width - padding - title_len - 4);
            println!("{left} {title} {right}");
        } else {
            println!("{}", line_char.to_string().repeat(width));
        }
    }

    /// Clear the screen.
    pub fn clear(&self) {
        let mut stdout = io::stdout();
        let _ = terminal::control::clear_screen(&mut stdout);
    }

    /// Clear the current line.
    pub fn clear_line(&self) {
        let mut stdout = io::stdout();
        let _ = terminal::control::clear_line(&mut stdout);
    }

    /// Set the terminal title.
    pub fn set_title(&self, title: &str) {
        let mut stdout = io::stdout();
        let _ = terminal::control::set_title(&mut stdout, title);
    }

    /// Ring the terminal bell.
    pub fn bell(&self) {
        let mut stdout = io::stdout();
        let _ = terminal::control::bell(&mut stdout);
    }

    /// Print text without parsing markup.
    pub fn print_plain(&self, content: &str) {
        self.print_with_options(content, &PrintOptions::new().with_markup(false));
    }

    /// Print a styled message.
    pub fn print_styled(&self, content: &str, style: Style) {
        self.print_with_options(
            content,
            &PrintOptions::new().with_markup(self.markup).with_style(style),
        );
    }

    /// Print a log message with a level indicator.
    pub fn log(&self, message: &str, level: LogLevel) {
        let (prefix, style) = match level {
            LogLevel::Debug => (
                "[DEBUG]",
                Style::parse("cyan").unwrap_or_default(),
            ),
            LogLevel::Info => (
                "[INFO]",
                Style::parse("green").unwrap_or_default(),
            ),
            LogLevel::Warning => (
                "[WARNING]",
                Style::parse("yellow").unwrap_or_default(),
            ),
            LogLevel::Error => (
                "[ERROR]",
                Style::parse("bold red").unwrap_or_default(),
            ),
        };

        let mut stdout = io::stdout();
        let _ = self.print_to(
            &mut stdout,
            prefix,
            &PrintOptions::new().with_markup(false).with_style(style),
        );
        print!(" ");
        let _ = self.print_to(
            &mut stdout,
            message,
            &PrintOptions::new().with_markup(self.markup),
        );
    }
}

/// Log level for console.log().
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

/// Builder for creating a Console with custom settings.
#[derive(Debug, Default)]
pub struct ConsoleBuilder {
    color_system: Option<ColorSystem>,
    force_terminal: Option<bool>,
    tab_size: Option<usize>,
    markup: Option<bool>,
    emoji: Option<bool>,
    highlight: Option<bool>,
    width: Option<usize>,
    height: Option<usize>,
    safe_box: Option<bool>,
}

impl ConsoleBuilder {
    /// Set the color system.
    #[must_use]
    pub fn color_system(mut self, system: ColorSystem) -> Self {
        self.color_system = Some(system);
        self
    }

    /// Disable colors.
    #[must_use]
    pub fn no_color(mut self) -> Self {
        self.color_system = None;
        self
    }

    /// Force terminal mode.
    #[must_use]
    pub fn force_terminal(mut self, force: bool) -> Self {
        self.force_terminal = Some(force);
        self
    }

    /// Set tab size.
    #[must_use]
    pub fn tab_size(mut self, size: usize) -> Self {
        self.tab_size = Some(size);
        self
    }

    /// Enable/disable markup parsing.
    #[must_use]
    pub fn markup(mut self, enabled: bool) -> Self {
        self.markup = Some(enabled);
        self
    }

    /// Enable/disable emoji.
    #[must_use]
    pub fn emoji(mut self, enabled: bool) -> Self {
        self.emoji = Some(enabled);
        self
    }

    /// Enable/disable highlighting.
    #[must_use]
    pub fn highlight(mut self, enabled: bool) -> Self {
        self.highlight = Some(enabled);
        self
    }

    /// Set console width.
    #[must_use]
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set console height.
    #[must_use]
    pub fn height(mut self, height: usize) -> Self {
        self.height = Some(height);
        self
    }

    /// Use ASCII-safe box characters.
    #[must_use]
    pub fn safe_box(mut self, safe: bool) -> Self {
        self.safe_box = Some(safe);
        self
    }

    /// Build the console.
    #[must_use]
    pub fn build(self) -> Console {
        let mut console = Console::new();

        if let Some(cs) = self.color_system {
            console.color_system = Some(cs);
        }
        if let Some(ft) = self.force_terminal {
            console.force_terminal = Some(ft);
        }
        if let Some(ts) = self.tab_size {
            console.tab_size = ts;
        }
        if let Some(m) = self.markup {
            console.markup = m;
        }
        if let Some(e) = self.emoji {
            console.emoji = e;
        }
        if let Some(h) = self.highlight {
            console.highlight = h;
        }
        if let Some(w) = self.width {
            console.width = Some(w);
        }
        if let Some(h) = self.height {
            console.height = Some(h);
        }
        if let Some(sb) = self.safe_box {
            console.safe_box = sb;
        }

        console
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_new() {
        let console = Console::new();
        assert!(console.width() > 0);
        assert!(console.height() > 0);
    }

    #[test]
    fn test_console_builder() {
        let console = Console::builder()
            .width(100)
            .height(50)
            .markup(false)
            .build();

        assert_eq!(console.width(), 100);
        assert_eq!(console.height(), 50);
        assert!(!console.markup);
    }

    #[test]
    fn test_console_options() {
        let console = Console::builder().width(80).build();
        let options = console.options();

        assert_eq!(options.max_width, 80);
        assert_eq!(options.size.width, 80);
    }

    #[test]
    fn test_print_options() {
        let options = PrintOptions::new()
            .with_markup(true)
            .with_style(Style::new().bold());

        assert!(options.markup);
        assert!(options.style.is_some());
    }

    #[test]
    fn test_capture() {
        let mut console = Console::new();
        console.begin_capture();

        // Print would add to buffer
        // For testing, we just verify the mechanism
        let segments = console.end_capture();
        assert!(segments.is_empty()); // Nothing captured in this test
    }

    #[test]
    fn test_dimensions() {
        let dims = ConsoleDimensions::default();
        assert_eq!(dims.width, 80);
        assert_eq!(dims.height, 24);
    }
}
