//! Typography and spacing patterns for demo_showcase.
//!
//! This module provides consistent visual patterns across all scenes:
//! - Section headers (Rule + title combinations)
//! - Standard padding and spacing
//! - Hero/title alignment utilities
//! - Dashboard layout builder (wide mode)
//!
//! These helpers ensure the demo output feels designed and cohesive.

// Many helpers are provided for future scene implementations
#![allow(dead_code)]

use rich_rust::console::{Console, ConsoleOptions, PrintOptions};
use rich_rust::r#box::ROUNDED;
use rich_rust::renderables::layout::Layout;
use rich_rust::renderables::panel::Panel;
use rich_rust::renderables::rule::Rule;
use rich_rust::renderables::table::{Column, Table};
use rich_rust::renderables::Renderable;
use rich_rust::segment::Segment;
use rich_rust::style::Style;
use rich_rust::text::Text;

use super::log_pane::LogPane;
use super::state::{DemoStateSnapshot, PipelineStage, ServiceHealth, ServiceInfo, StageStatus};

/// Standard vertical spacing between major sections.
pub const SECTION_SPACING: usize = 1;

/// Standard padding inside panels (top, right, bottom, left).
pub const PANEL_PADDING: (usize, usize, usize, usize) = (1, 2, 1, 2);

/// Standard margin around major blocks (top, right, bottom, left).
pub const BLOCK_MARGIN: (usize, usize, usize, usize) = (0, 1, 0, 1);

/// Print a styled section header with a rule and title.
///
/// Creates a visually distinct section break with:
/// - A horizontal rule styled with section.rule
/// - A styled title
/// - A blank line after for spacing
///
/// # Example
/// ```ignore
/// section_header(&console, "Table Showcase", false);
/// ```
pub fn section_header(console: &Console, title: &str, centered: bool) {
    let rule_style = console.get_style("section.rule");
    console.print_renderable(&Rule::new().style(rule_style));

    let styled_title = format!("[section.title]{}[/]", title);
    if centered {
        // Use justify for centering
        console.print_with_options(
            &styled_title,
            &PrintOptions::new()
                .with_markup(true)
                .with_justify(rich_rust::text::JustifyMethod::Center),
        );
    } else {
        console.print_with_options(&styled_title, &PrintOptions::new().with_markup(true));
    }

    console.print_plain(""); // Blank line after title
}

/// Print a styled scene header with prominent title and subtitle.
///
/// Used at the start of major scenes for hero-style presentation.
pub fn scene_header(console: &Console, title: &str, subtitle: Option<&str>) {
    let title_markup = format!("[brand.title]{}[/]", title);
    console.print_with_options(
        &title_markup,
        &PrintOptions::new()
            .with_markup(true)
            .with_justify(rich_rust::text::JustifyMethod::Center),
    );

    if let Some(sub) = subtitle {
        let sub_markup = format!("[brand.subtitle]{}[/]", sub);
        console.print_with_options(
            &sub_markup,
            &PrintOptions::new()
                .with_markup(true)
                .with_justify(rich_rust::text::JustifyMethod::Center),
        );
    }

    console.print_plain("");
}

/// Print a muted hint/instruction line.
pub fn hint(console: &Console, text: &str) {
    console.print_with_options(
        &format!("[hint]{}[/]", text),
        &PrintOptions::new().with_markup(true),
    );
}

/// Print a blank line for vertical spacing.
pub fn spacer(console: &Console) {
    console.print_plain("");
}

/// Print multiple blank lines for larger vertical gaps.
pub fn spacer_n(console: &Console, n: usize) {
    for _ in 0..n {
        console.print_plain("");
    }
}

/// Create a thin divider rule (less prominent than section header).
pub fn divider() -> Rule {
    let dim_style = Style::parse("dim").unwrap_or_default();
    Rule::new().style(dim_style)
}

/// Print a divider directly to the console.
pub fn print_divider(console: &Console) {
    console.print_renderable(&divider());
}

/// Create a styled status badge text.
///
/// Returns markup that can be used inline:
/// - `status_badge("OK", "ok")` → `"[status.ok.badge] OK [/]"`
/// - `status_badge("FAIL", "err")` → `"[status.err.badge] FAIL [/]"`
#[must_use]
pub fn status_badge(text: &str, status: &str) -> String {
    format!("[status.{}.badge] {} [/]", status, text)
}

/// Create styled status text (without badge background).
#[must_use]
pub fn status_text(text: &str, status: &str) -> String {
    format!("[status.{}]{}[/]", status, text)
}

/// Create a brand accent markup string.
#[must_use]
pub fn brand_accent(text: &str) -> String {
    format!("[brand.accent]{}[/]", text)
}

/// Create a muted text markup string.
#[must_use]
pub fn muted(text: &str) -> String {
    format!("[brand.muted]{}[/]", text)
}

/// Create a key-value row suitable for panels or lists.
///
/// Returns a formatted string with the key styled as a label and value as-is:
/// `"[dim]key:[/] value"`
///
/// # Example
/// ```ignore
/// let row = kv_row("Version", "1.2.3");
/// // Returns: "[dim]Version:[/] 1.2.3"
/// ```
#[must_use]
pub fn kv_row(key: &str, value: &str) -> String {
    format!("[dim]{}:[/] {}", key, value)
}

/// Create a key-value row with custom key and value styles.
///
/// # Example
/// ```ignore
/// let row = kv_row_styled("Status", "status.ok", "Running", "status.ok");
/// ```
#[must_use]
pub fn kv_row_styled(key: &str, key_style: &str, value: &str, value_style: &str) -> String {
    format!("[{}]{}:[/] [{}]{}[/]", key_style, key, value_style, value)
}

/// Generic badge helper that wraps text with a style.
///
/// # Example
/// ```ignore
/// let badge = badge("NEW", "brand.accent");
/// // Returns: "[brand.accent] NEW [/]"
/// ```
#[must_use]
pub fn badge(text: &str, style: &str) -> String {
    format!("[{}] {} [/]", style, text)
}

// ============================================================================
// Dashboard Layout Builder (Wide Mode)
// ============================================================================
//
// The wide dashboard layout follows this structure:
//
// ┌─────────────────────────────────────────────────────────────────┐
// │                         header                                   │
// ├─────────────────────────────────────┬───────────────────────────┤
// │             left (ratio 2)          │      right (ratio 1)      │
// │  ┌───────────────────────────────┐  │  ┌─────────────────────┐  │
// │  │         pipeline              │  │  │     services        │  │
// │  └───────────────────────────────┘  │  └─────────────────────┘  │
// │  ┌───────────────────────────────┐  │  ┌─────────────────────┐  │
// │  │         step_info             │  │  │     quick_facts     │  │
// │  └───────────────────────────────┘  │  └─────────────────────┘  │
// ├─────────────────────────────────────┴───────────────────────────┤
// │                         logs                                     │
// └─────────────────────────────────────────────────────────────────┘
//
// Named nodes allow targeted updates via `layout.get_mut("name")`.

/// Minimum terminal width for wide layout mode.
pub const DASHBOARD_MIN_WIDTH_WIDE: usize = 80;

/// Default height for the log pane.
pub const DASHBOARD_LOG_HEIGHT: usize = 8;

/// Default height for the header bar.
pub const DASHBOARD_HEADER_HEIGHT: usize = 3;

// ----------------------------------------------------------------------------
// Wrapper types for Layout-compatible renderables
// ----------------------------------------------------------------------------

/// A simple text block renderable that owns its markup content.
///
/// Implements Renderable so it can be used with Layout nodes.
#[derive(Debug, Clone)]
pub struct TextBlock {
    markup: String,
}

impl TextBlock {
    #[must_use]
    pub fn new(markup: impl Into<String>) -> Self {
        Self {
            markup: markup.into(),
        }
    }
}

impl Renderable for TextBlock {
    fn render<'a>(&'a self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment<'a>> {
        let text = rich_rust::markup::render_or_plain(&self.markup);
        text.render("")
            .into_iter()
            .map(Segment::into_owned)
            .collect()
    }
}

/// A bordered panel renderable that owns its content.
///
/// Wraps content with a ROUNDED box border and title.
#[derive(Debug, Clone)]
pub struct BorderedBlock {
    title: String,
    content_markup: String,
}

impl BorderedBlock {
    #[must_use]
    pub fn new(title: impl Into<String>, content_markup: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content_markup: content_markup.into(),
        }
    }
}

impl Renderable for BorderedBlock {
    fn render<'a>(&'a self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment<'a>> {
        let text = rich_rust::markup::render_or_plain(&self.content_markup);
        let width = options.max_width.saturating_sub(2); // Account for borders

        let panel = Panel::from_rich_text(&text, width)
            .title(self.title.as_str())
            .rounded();

        panel
            .render(options.max_width)
            .into_iter()
            .map(Segment::into_owned)
            .collect()
    }
}

/// Build the wide dashboard layout with named nodes.
///
/// This creates a Layout structure suitable for Live display updates.
/// Each region has a name so it can be updated independently.
///
/// # Layout Structure
/// - `root`: The outermost container (column split)
/// - `header`: Top bar with headline/title
/// - `main`: Middle content area (row split)
/// - `left`: Left column (pipeline + step info)
/// - `pipeline`: Pipeline progress panel
/// - `step_info`: Current step detail panel
/// - `right`: Right column (services + facts)
/// - `services`: Services health table
/// - `quick_facts`: Quick stats panel
/// - `logs`: Bottom log tail pane
#[must_use]
pub fn build_dashboard_layout_wide(snapshot: &DemoStateSnapshot, log_limit: usize) -> Layout {
    // Build individual components
    let header_block = build_header_block(snapshot);
    let services_table = build_services_table(&snapshot.services);
    let pipeline_block = build_pipeline_block(&snapshot.pipeline);
    let step_block = build_step_info_block(&snapshot.pipeline);
    let facts_block = build_quick_facts_block(snapshot);
    let log_block = build_log_block(&snapshot.logs, log_limit);

    // Assemble the layout tree
    let mut root = Layout::new().name("root");

    // Header row (fixed height)
    let header = Layout::new()
        .name("header")
        .size(DASHBOARD_HEADER_HEIGHT)
        .renderable(header_block);

    // Main content area
    let mut main = Layout::new().name("main").ratio(1);

    // Left column: pipeline + step info
    let mut left = Layout::new().name("left").ratio(2);
    left.split_column(vec![
        Layout::new()
            .name("pipeline")
            .ratio(2)
            .renderable(pipeline_block),
        Layout::new()
            .name("step_info")
            .ratio(1)
            .renderable(step_block),
    ]);

    // Right column: services + quick facts
    let mut right = Layout::new().name("right").ratio(1);
    right.split_column(vec![
        Layout::new()
            .name("services")
            .ratio(2)
            .renderable(services_table),
        Layout::new()
            .name("quick_facts")
            .ratio(1)
            .renderable(facts_block),
    ]);

    main.split_row(vec![left, right]);

    // Log pane (fixed height at bottom)
    let logs = Layout::new()
        .name("logs")
        .size(DASHBOARD_LOG_HEIGHT)
        .renderable(log_block);

    // Assemble root: header, main, logs
    root.split_column(vec![header, main, logs]);

    root
}

/// Build the header bar text with headline and run info.
#[must_use]
pub fn build_header_bar(snapshot: &DemoStateSnapshot) -> Text {
    let elapsed_secs = snapshot.elapsed.as_secs();
    let elapsed_ms = snapshot.elapsed.subsec_millis();

    let markup = format!(
        "[brand.title]{}[/]  [dim]Run #{}  |  Seed {}  |  Elapsed {}.{:03}s[/]",
        snapshot.headline, snapshot.run_id, snapshot.seed, elapsed_secs, elapsed_ms
    );

    rich_rust::markup::render_or_plain(&markup)
}

/// Build a services health table.
///
/// # Panics
/// Currently unimplemented (bd-1xoj) - will panic if called.
#[must_use]
pub fn build_services_table(_services: &[ServiceInfo]) -> Table {
    // TODO(bd-1xoj): Implement with proper cell conversion
    todo!("bd-1xoj: services table")
}

/// Build the pipeline progress panel.
///
/// # Panics
/// Currently unimplemented (bd-1xoj) - will panic if called.
#[must_use]
pub fn build_pipeline_panel(_stages: &[PipelineStage]) -> Panel<'static> {
    // TODO(bd-1xoj): Implement with proper lifetime handling
    todo!("bd-1xoj: pipeline panel")
}

/// Build the current step info panel.
///
/// # Panics
/// Currently unimplemented (bd-1xoj) - will panic if called.
#[must_use]
pub fn build_step_info_panel(_stages: &[PipelineStage]) -> Panel<'static> {
    // TODO(bd-1xoj): Implement with proper lifetime handling
    todo!("bd-1xoj: step info panel")
}

/// Build the quick facts panel with summary stats.
///
/// # Panics
/// Currently unimplemented (bd-1xoj) - will panic if called.
#[must_use]
pub fn build_quick_facts_panel(_snapshot: &DemoStateSnapshot) -> Panel<'static> {
    // TODO(bd-1xoj): Implement with proper lifetime handling
    todo!("bd-1xoj: quick facts panel")
}

/// Update an existing dashboard layout with new snapshot data.
///
/// This updates only the leaf renderables, preserving the layout structure.
/// Useful for Live display updates without rebuilding the entire tree.
///
/// # Panics
/// Currently unimplemented (bd-1xoj) - will panic if called.
pub fn update_dashboard_layout(
    _layout: &mut Layout,
    _snapshot: &DemoStateSnapshot,
    _log_limit: usize,
) {
    // TODO(bd-1xoj): Implement dashboard update
    todo!("bd-1xoj: dashboard layout update")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_badge_formatting() {
        assert_eq!(status_badge("OK", "ok"), "[status.ok.badge] OK [/]");
        assert_eq!(status_badge("WARN", "warn"), "[status.warn.badge] WARN [/]");
        assert_eq!(status_badge("FAIL", "err"), "[status.err.badge] FAIL [/]");
    }

    #[test]
    fn test_status_text_formatting() {
        assert_eq!(status_text("Passed", "ok"), "[status.ok]Passed[/]");
        assert_eq!(status_text("Warning", "warn"), "[status.warn]Warning[/]");
    }

    #[test]
    fn test_brand_accent_formatting() {
        assert_eq!(brand_accent("highlight"), "[brand.accent]highlight[/]");
    }

    #[test]
    fn test_muted_formatting() {
        assert_eq!(muted("subtle"), "[brand.muted]subtle[/]");
    }

    #[test]
    fn test_padding_constants_are_reasonable() {
        let (top, right, bottom, left) = PANEL_PADDING;
        assert!(top <= 3, "panel padding top should be modest");
        assert!(right <= 4, "panel padding right should be modest");
        assert!(bottom <= 3, "panel padding bottom should be modest");
        assert!(left <= 4, "panel padding left should be modest");
    }

    #[test]
    fn test_section_spacing_is_reasonable() {
        const { assert!(SECTION_SPACING >= 1) };
        const { assert!(SECTION_SPACING <= 3) };
    }

    #[test]
    fn test_divider_creates_rule() {
        let rule = divider();
        // Just verify it compiles and creates a Rule
        let _ = rule;
    }

    #[test]
    fn test_kv_row_formatting() {
        assert_eq!(kv_row("Name", "Alice"), "[dim]Name:[/] Alice");
        assert_eq!(kv_row("Version", "1.0.0"), "[dim]Version:[/] 1.0.0");
    }

    #[test]
    fn test_kv_row_styled_formatting() {
        assert_eq!(
            kv_row_styled("Status", "bold", "Running", "status.ok"),
            "[bold]Status:[/] [status.ok]Running[/]"
        );
    }

    #[test]
    fn test_badge_formatting() {
        assert_eq!(badge("NEW", "brand.accent"), "[brand.accent] NEW [/]");
        assert_eq!(badge("INFO", "status.info"), "[status.info] INFO [/]");
    }

    // ========== Dashboard Layout Builder Tests ==========

    use super::super::state::{DemoState, LogLevel};
    use std::time::Duration;

    fn make_test_snapshot() -> DemoStateSnapshot {
        let mut state = DemoState::demo_seeded(1, 42);
        state.headline = "Test Deploy".to_string();
        state.push_log(LogLevel::Info, "test log line");
        DemoStateSnapshot::from(&state)
    }

    #[test]
    #[ignore = "bd-1xoj: dashboard components not yet implemented"]
    fn test_build_services_table_creates_table() {
        let snapshot = make_test_snapshot();
        let table = build_services_table(&snapshot.services);
        // Table should have been created with columns
        // (We can't easily inspect Table internals, but we can verify it doesn't panic)
        let _ = table;
    }

    #[test]
    #[ignore = "bd-1xoj: dashboard components not yet implemented"]
    fn test_build_pipeline_panel_creates_panel() {
        let snapshot = make_test_snapshot();
        let panel = build_pipeline_panel(&snapshot.pipeline);
        let _ = panel;
    }

    #[test]
    #[ignore = "bd-1xoj: dashboard components not yet implemented"]
    fn test_build_step_info_panel_creates_panel() {
        let snapshot = make_test_snapshot();
        let panel = build_step_info_panel(&snapshot.pipeline);
        let _ = panel;
    }

    #[test]
    #[ignore = "bd-1xoj: dashboard components not yet implemented"]
    fn test_build_quick_facts_panel_creates_panel() {
        let snapshot = make_test_snapshot();
        let panel = build_quick_facts_panel(&snapshot);
        let _ = panel;
    }

    #[test]
    fn test_build_header_bar_creates_text() {
        let snapshot = make_test_snapshot();
        let text = build_header_bar(&snapshot);
        let _ = text;
    }

    #[test]
    fn test_build_dashboard_layout_wide_creates_layout() {
        let snapshot = make_test_snapshot();
        let layout = build_dashboard_layout_wide(&snapshot, 10);

        // Verify all named nodes exist
        assert!(layout.get("root").is_some(), "root node should exist");
        assert!(layout.get("header").is_some(), "header node should exist");
        assert!(layout.get("main").is_some(), "main node should exist");
        assert!(layout.get("left").is_some(), "left node should exist");
        assert!(layout.get("right").is_some(), "right node should exist");
        assert!(
            layout.get("pipeline").is_some(),
            "pipeline node should exist"
        );
        assert!(
            layout.get("step_info").is_some(),
            "step_info node should exist"
        );
        assert!(
            layout.get("services").is_some(),
            "services node should exist"
        );
        assert!(
            layout.get("quick_facts").is_some(),
            "quick_facts node should exist"
        );
        assert!(layout.get("logs").is_some(), "logs node should exist");
    }

    #[test]
    fn test_update_dashboard_layout_updates_nodes() {
        let snapshot = make_test_snapshot();
        let mut layout = build_dashboard_layout_wide(&snapshot, 10);

        // Create a modified snapshot
        let mut state = DemoState::demo_seeded(2, 99);
        state.headline = "Updated Deploy".to_string();
        let updated_snapshot = DemoStateSnapshot::from(&state);

        // Update should not panic
        update_dashboard_layout(&mut layout, &updated_snapshot, 10);

        // Layout structure should still be intact
        assert!(layout.get("header").is_some());
        assert!(layout.get("services").is_some());
        assert!(layout.get("pipeline").is_some());
    }

    #[test]
    fn test_build_services_table_handles_empty_services() {
        let services: Vec<ServiceInfo> = vec![];
        let table = build_services_table(&services);
        let _ = table;
    }

    #[test]
    fn test_build_pipeline_panel_handles_empty_stages() {
        let stages: Vec<PipelineStage> = vec![];
        let panel = build_pipeline_panel(&stages);
        let _ = panel;
    }

    #[test]
    fn test_build_step_info_panel_handles_empty_stages() {
        let stages: Vec<PipelineStage> = vec![];
        let panel = build_step_info_panel(&stages);
        let _ = panel;
    }

    #[test]
    fn test_dashboard_constants_are_reasonable() {
        assert!(
            DASHBOARD_MIN_WIDTH_WIDE >= 60,
            "wide mode needs reasonable min width"
        );
        assert!(
            DASHBOARD_LOG_HEIGHT >= 4,
            "log pane needs reasonable height"
        );
        assert!(DASHBOARD_HEADER_HEIGHT >= 1, "header needs at least 1 line");
    }
}
