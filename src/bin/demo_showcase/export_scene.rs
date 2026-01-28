//! Export scene for demo_showcase.
//!
//! Demonstrates the export functionality and shows viewing instructions.
//! When running in export mode, displays the export summary.

use std::sync::Arc;

use rich_rust::r#box::{DOUBLE, ROUNDED};
use rich_rust::console::Console;
use rich_rust::renderables::panel::Panel;
use rich_rust::style::Style;

use crate::Config;
use crate::scenes::{Scene, SceneError};

/// Export showcase scene: demonstrates export capabilities.
pub struct ExportScene;

impl ExportScene {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Scene for ExportScene {
    fn name(&self) -> &'static str {
        "export"
    }

    fn summary(&self) -> &'static str {
        "Export HTML/SVG bundle with viewing instructions."
    }

    fn run(&self, console: &Arc<Console>, cfg: &Config) -> Result<(), SceneError> {
        console.print("[section.title]Export: Sharing Terminal Output[/]");
        console.print("");
        console.print("[dim]rich_rust can export terminal output to HTML and SVG for sharing.[/]");
        console.print("");

        // Show export formats
        render_export_formats(console, cfg);

        console.print("");

        // Show usage instructions
        render_usage_instructions(console);

        console.print("");

        // If we're in export mode, show what will be exported
        if cfg.is_export() {
            render_export_summary(console, cfg);
            console.print("");
        }

        Ok(())
    }
}

/// Render export format descriptions.
fn render_export_formats(console: &Console, cfg: &Config) {
    console.print("[brand.accent]Available Export Formats[/]");
    console.print("");

    // HTML format panel
    let html_content = r#"[bold]HTML Export[/]

Generates a standalone HTML file with inline or external CSS.
- Colors and styles preserved
- Works in any modern browser
- Easy to share via email or hosting

[dim]Use `--export` or `--export-dir <path>`[/]"#;

    let html_panel = Panel::from_text(html_content)
        .title("[cyan]demo_showcase.html[/]")
        .box_style(&ROUNDED)
        .border_style(Style::parse("cyan").unwrap_or_default())
        .safe_box(cfg.is_safe_box());

    console.print_renderable(&html_panel);

    console.print("");

    // SVG format panel
    let svg_content = r#"[bold]SVG Export[/]

Generates a scalable vector graphic with embedded fonts.
- Perfect for documentation
- Scales to any size without pixelation
- Uses `<foreignObject>` for text rendering

[dim]Note: Best viewed in modern browsers (Chrome, Firefox, Safari)[/]"#;

    let svg_panel = Panel::from_text(svg_content)
        .title("[magenta]demo_showcase.svg[/]")
        .box_style(&ROUNDED)
        .border_style(Style::parse("magenta").unwrap_or_default())
        .safe_box(cfg.is_safe_box());

    console.print_renderable(&svg_panel);
}

/// Render usage instructions.
fn render_usage_instructions(console: &Console) {
    console.print("[brand.accent]How to Export[/]");
    console.print("");

    let instructions = r#"[bold]Quick Export (temp directory):[/]
  demo_showcase --export

[bold]Export to specific directory:[/]
  demo_showcase --export-dir ./output

[bold]Export single scene:[/]
  demo_showcase --scene hero --export-dir ./output

[bold]Recommended flags for clean export:[/]
  demo_showcase --export-dir ./output \
    --no-interactive \
    --color-system truecolor \
    --width 100 \
    --quick"#;

    console.print(instructions);
}

/// Render export summary when in export mode.
fn render_export_summary(console: &Console, cfg: &Config) {
    console.print("[brand.accent]Export Summary[/]");
    console.print("");

    if let Some(export_dir) = cfg.export_dir() {
        let html_path = export_dir.join("demo_showcase.html");
        let svg_path = export_dir.join("demo_showcase.svg");

        let summary = format!(
            r#"[bold green]Files will be written to:[/]

  [cyan]HTML:[/] {}
  [magenta]SVG:[/]  {}

[dim]Open the HTML file in your browser to view the output.
The SVG can be embedded in documentation or presentations.[/]"#,
            html_path.display(),
            svg_path.display()
        );

        let summary_panel = Panel::from_text(&summary)
            .title("[bold]Export Complete[/]")
            .box_style(&DOUBLE)
            .border_style(Style::parse("green").unwrap_or_default())
            .safe_box(cfg.is_safe_box());

        console.print_renderable(&summary_panel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_scene_has_correct_name() {
        let scene = ExportScene::new();
        assert_eq!(scene.name(), "export");
    }

    #[test]
    fn export_scene_runs_without_error() {
        let scene = ExportScene::new();
        let console = Console::builder()
            .force_terminal(false)
            .markup(true)
            .build()
            .shared();
        let cfg = Config::with_defaults();

        let result = scene.run(&console, &cfg);
        assert!(result.is_ok());
    }
}
