//! Python Rich conformance tests - currently disabled pending API alignment
//!
//! Enable with: cargo test --features conformance_test
#![cfg(feature = "conformance_test")]

use std::fs;

use rich_rust::color::ColorSystem;
use rich_rust::console::{Console, PrintOptions};
use rich_rust::prelude::*;
use rich_rust::renderables::{Align, Columns, Padding, Panel, Rule, Table, Tree, TreeNode};
use rich_rust::segment::Segment;
use serde_json::Value;

#[derive(Debug, Clone)]
struct RenderOptions {
    width: Option<usize>,
    color_system: Option<ColorSystem>,
    force_terminal: Option<bool>,
}

fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace("\r", "\n")
}

fn parse_color_system(value: &str) -> Option<ColorSystem> {
    match value {
        "truecolor" => Some(ColorSystem::TrueColor),
        "256" | "eight_bit" => Some(ColorSystem::EightBit),
        "standard" => Some(ColorSystem::Standard),
        "none" | "" => None,
        _ => None,
    }
}

fn parse_render_options(defaults: &Value, overrides: Option<&Value>) -> RenderOptions {
    let default_width = defaults
        .get("width")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);
    let default_color = defaults
        .get("color_system")
        .and_then(|v| v.as_str())
        .and_then(parse_color_system);
    let default_force = defaults.get("force_terminal").and_then(|v| v.as_bool());

    let mut width = default_width;
    let mut color_system = default_color;
    let mut force_terminal = default_force;

    if let Some(overrides) = overrides {
        if let Some(w) = overrides.get("width").and_then(|v| v.as_u64()) {
            width = Some(w as usize);
        }
        if let Some(cs) = overrides.get("color_system").and_then(|v| v.as_str()) {
            color_system = parse_color_system(cs);
        }
        if let Some(force) = overrides.get("force_terminal").and_then(|v| v.as_bool()) {
            force_terminal = Some(force);
        }
    }

    RenderOptions {
        width,
        color_system,
        force_terminal,
    }
}

fn build_console(options: &RenderOptions) -> Console {
    let mut builder = Console::builder();
    if let Some(width) = options.width {
        builder = builder.width(width);
    }
    if let Some(force_terminal) = options.force_terminal {
        builder = builder.force_terminal(force_terminal);
    }
    if let Some(color_system) = options.color_system {
        builder = builder.color_system(color_system);
    } else {
        builder = builder.no_color();
    }
    builder.build()
}

fn render_text(console: &Console, markup: &str, width: Option<usize>) -> (String, String) {
    let mut options = PrintOptions::new().with_markup(true);
    if let Some(width) = width {
        options = options.with_width(width);
    }

    let plain = console.export_text_with_options(markup, &options);
    let mut buf = Vec::new();
    console
        .print_to(&mut buf, markup, &options)
        .expect("print_to failed");

    (
        normalize_line_endings(&plain),
        normalize_line_endings(&String::from_utf8(buf).expect("utf8 output")),
    )
}

fn render_segments_to_ansi(console: &Console, segments: &[Segment<'_>]) -> String {
    let mut buf = Vec::new();
    console
        .print_segments_to(&mut buf, segments)
        .expect("print_segments_to failed");
    normalize_line_endings(&String::from_utf8(buf).expect("utf8 output"))
}

fn render_renderable(
    console: &Console,
    renderable: &dyn rich_rust::renderables::Renderable,
) -> (String, String) {
    let plain = console.export_renderable_text(renderable);
    let options = console.options();
    let segments = renderable.render(console, &options);
    let ansi = render_segments_to_ansi(console, &segments);
    (normalize_line_endings(&plain), ansi)
}

fn value_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn value_bool(value: &Value, key: &str, default: bool) -> bool {
    value.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

fn value_usize(value: &Value, key: &str) -> Option<usize> {
    value.get(key).and_then(|v| v.as_u64()).map(|v| v as usize)
}

fn build_table(input: &Value) -> Table {
    let show_header = value_bool(input, "show_header", true);
    let show_lines = value_bool(input, "show_lines", false);
    let mut table = Table::new().show_header(show_header).show_lines(show_lines);

    if let Some(title) = value_string(input, "title") {
        table = table.title(title);
    }
    if let Some(caption) = value_string(input, "caption") {
        table = table.caption(caption);
    }

    let columns = input
        .get("columns")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let justifies = input
        .get("column_justifies")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    for (idx, col) in columns.iter().enumerate() {
        let name = col.as_str().unwrap_or("");
        let mut column = Column::new(name);
        if let Some(justify) = justifies
            .get(idx)
            .and_then(|v| v.as_str())
            .map(|s| s.to_lowercase())
        {
            column = match justify.as_str() {
                "right" => column.justify(JustifyMethod::Right),
                "center" => column.justify(JustifyMethod::Center),
                _ => column.justify(JustifyMethod::Left),
            };
        }
        table.add_column(column);
    }

    if let Some(rows) = input.get("rows").and_then(|v| v.as_array()) {
        for row in rows {
            if let Some(cells) = row.as_array() {
                let row_cells: Vec<Cell> = cells
                    .iter()
                    .map(|cell| Cell::new(cell.as_str().unwrap_or("")))
                    .collect();
                table.add_row(Row::new(row_cells));
            }
        }
    }

    table
}

fn build_tree_node(node: &Value) -> TreeNode {
    let label = node.get("label").and_then(|v| v.as_str()).unwrap_or("");
    let mut tree = TreeNode::new(label);
    if let Some(children) = node.get("children").and_then(|v| v.as_array()) {
        for child in children {
            tree = tree.child(build_tree_node(child));
        }
    }
    tree
}

fn build_renderable<'a>(
    kind: &'a str,
    input: &'a Value,
    options: &RenderOptions,
) -> Box<dyn rich_rust::renderables::Renderable + 'a> {
    match kind {
        "rule" => {
            let title = value_string(input, "title");
            let align = value_string(input, "align").unwrap_or_else(|| "center".to_string());
            let character = value_string(input, "character").unwrap_or_else(|| "─".to_string());
            let mut rule = if let Some(title) = title {
                Rule::with_title(title)
            } else {
                Rule::new()
            };
            rule = rule.character(character);
            rule = match align.as_str() {
                "left" => rule.align_left(),
                "right" => rule.align_right(),
                _ => rule.align_center(),
            };
            Box::new(rule)
        }
        "panel" => {
            let text = value_string(input, "text").unwrap_or_default();
            let mut panel = Panel::from_text(&text);
            if let Some(title) = value_string(input, "title") {
                panel = panel.title(title);
            }
            if let Some(subtitle) = value_string(input, "subtitle") {
                panel = panel.subtitle(subtitle);
            }
            if let Some(width) = value_usize(input, "width") {
                panel = panel.width(width);
            }
            if let Some(box_style) = value_string(input, "box") {
                panel = match box_style.as_str() {
                    "ASCII" => panel.ascii(),
                    "SQUARE" | "DOUBLE" => panel.square(), // DOUBLE uses square as fallback
                    _ => panel.rounded(),
                };
            }
            Box::new(panel)
        }
        "table" => Box::new(build_table(input)),
        "tree" => {
            let node = build_tree_node(input);
            Box::new(Tree::new(node))
        }
        "progress" => {
            let total = input.get("total").and_then(|v| v.as_u64()).unwrap_or(100);
            let completed = input.get("completed").and_then(|v| v.as_u64()).unwrap_or(0);
            let width = value_usize(input, "width");
            let mut bar = ProgressBar::with_total(total);
            if let Some(width) = width {
                bar = bar.width(width);
            }
            bar.update(completed);
            Box::new(bar)
        }
        "columns" => {
            let items = input
                .get("items")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|v| v.as_str().unwrap_or("").to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let strings: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
            Box::new(Columns::from_strings(&strings))
        }
        "padding" => {
            let text = value_string(input, "text").unwrap_or_default();
            let pad = input
                .get("pad")
                .and_then(|v| v.as_array())
                .map(|values| {
                    let mut nums = [0usize; 4];
                    for (idx, value) in values.iter().enumerate().take(4) {
                        nums[idx] = value.as_u64().unwrap_or(0) as usize;
                    }
                    nums
                })
                .unwrap_or([0, 0, 0, 0]);
            let width = value_usize(input, "width").or(options.width).unwrap_or(0);
            let text = Text::new(text);
            let content = vec![text.render("")];
            Box::new(Padding::new(content, pad, width))
        }
        "align" => {
            let text = value_string(input, "text").unwrap_or_default();
            let width = value_usize(input, "width").unwrap_or(0);
            let align = value_string(input, "align").unwrap_or_else(|| "left".to_string());
            let align = match align.as_str() {
                "center" => Align::from_str(&text, width).center(),
                "right" => Align::from_str(&text, width).right(),
                _ => Align::from_str(&text, width).left(),
            };
            Box::new(align)
        }
        other => panic!("unsupported kind: {other}"),
    }
}

#[test]
fn python_rich_fixtures() {
    let fixture_path = "tests/conformance/fixtures/python_rich.json";
    let raw = fs::read_to_string(fixture_path)
        .unwrap_or_else(|_| panic!("missing fixtures at {fixture_path}"));
    let data: Value = serde_json::from_str(&raw).expect("invalid fixture JSON");

    let defaults = data.get("defaults").expect("defaults missing");
    let cases = data
        .get("cases")
        .and_then(|v| v.as_array())
        .expect("cases missing");

    for case in cases {
        let id = case
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>");
        let kind = case
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>");
        let input = case.get("input").expect("input missing");
        let expected = case.get("expected").expect("expected missing");
        let expected_plain = expected.get("plain").and_then(|v| v.as_str()).unwrap_or("");
        let expected_ansi = expected.get("ansi").and_then(|v| v.as_str()).unwrap_or("");

        let options = parse_render_options(defaults, case.get("render_options"));
        let console = build_console(&options);

        let (actual_plain, actual_ansi) = if kind == "text" {
            let markup = input.get("markup").and_then(|v| v.as_str()).unwrap_or("");
            render_text(&console, markup, options.width)
        } else {
            let renderable = build_renderable(kind, input, &options);
            render_renderable(&console, &*renderable)
        };

        assert_eq!(
            actual_plain, expected_plain,
            "plain mismatch for case {id} ({kind})"
        );
        assert_eq!(
            actual_ansi, expected_ansi,
            "ansi mismatch for case {id} ({kind})"
        );
    }
}
