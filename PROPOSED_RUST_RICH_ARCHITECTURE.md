# Proposed Architecture for `rich_rust`

> **Author:** Gemini
> **Date:** 2026-01-16
> **Reference:** `EXISTING_RICH_STRUCTURE_AND_ARCHITECTURE.md`

## Executive Summary

This document defines the Rust architecture for `rich_rust`. It translates the dynamic, protocol-based architecture of Python's Rich into a static, trait-based, and zero-cost architecture in Rust.

## 1. Core Traits (The Protocols)

In Python, Rich relies on `__rich_console__` and `__rich__` dunder methods. In Rust, we will define traits.

### 1.1 `ConsoleRender` (The Primary Trait)

This is the equivalent of `__rich_console__`. It produces an iterator of Segments.

```rust
pub trait ConsoleRender {
    fn render(&self, console: &Console, options: &ConsoleOptions) -> RenderResult;
}

// RenderResult is likely an Iterator or a custom struct that implements Iterator
pub type RenderResult = Box<dyn Iterator<Item = Segment> + Send>; 
// OR: simplified to return a Vec<Segment> for Phase 1 simplicity
```

### 1.2 `RichDisplay` (The Conversion Trait)

Equivalent to `__rich__`. It converts a high-level object into something that implements `ConsoleRender` (usually `Text`).

```rust
pub trait RichDisplay {
    fn to_rich(&self) -> impl ConsoleRender;
}
```

### 1.3 `Measure` (The Layout Trait)

Equivalent to `__rich_measure__`.

```rust
pub trait Measure {
    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement;
}

pub struct Measurement {
    pub min: usize,
    pub max: usize,
}
```

## 2. Core Data Structures

### 2.1 `Console`

The coordinator.

```rust
pub struct Console {
    pub options: ConsoleOptions,
    writer: Box<dyn Write + Send + Sync>,
    // thread-local buffer?
}

impl Console {
    pub fn print(&self, renderable: &impl ConsoleRender) {
        // 1. Get iterator from renderable
        // 2. Iterate segments
        // 3. Diff styles
        // 4. Write ANSI codes + Text to stream
    }
}
```

### 2.2 `Style`

Optimized for size and copying.

```rust
use bitflags::bitflags;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub attributes: Attributes,
}

bitflags! {
    #[derive(Default)]
    pub struct Attributes: u16 {
        const BOLD      = 1 << 0;
        const DIM       = 1 << 1;
        const ITALIC    = 1 << 2;
        const UNDERLINE = 1 << 3;
        const BLINK     = 1 << 4;
        const REVERSE   = 1 << 5;
        const HIDDEN    = 1 << 6;
        const STRIKE    = 1 << 7;
    }
}
```

### 2.3 `Text` and `Segment`

```rust
pub struct Segment {
    pub text: String, // Or Cow<'a, str> for optimization
    pub style: Style,
}

pub struct Text {
    pub spans: Vec<Span>,
    pub plain: String,
}

pub struct Span {
    pub start: usize,
    pub end: usize,
    pub style: Style,
}
```

## 3. Rendering Pipeline Strategy

### 3.1 Immediate Mode vs Buffering

Rich (Python) is largely immediate mode but buffers lines for layout (tables). `rich_rust` will strictly follow the **Iterator** pattern. Renderables will return Iterators that yield Segments lazily where possible.

### 3.2 ANSI Generation

We will use a dedicated module `ansi.rs` to handle the diffing of styles.

```rust
// Logic:
// current_style = Style::default();
// for segment in segments {
//     let diff_codes = current_style.diff(segment.style);
//     writer.write(diff_codes);
//     writer.write(segment.text);
//     current_style = segment.style;
// }
// writer.write(RESET);
```

## 4. Layout Engine

The `Table` implementation is the hardest part.

1.  **Measure Pass:** Call `measure()` on all cells to determine min/max widths.
2.  **Calculate Column Widths:** Use the same ratio/distribute algorithm as Python (ported to Rust).
3.  **Render Pass:** Call `render()` with the calculated column widths injected into `ConsoleOptions`.

## 5. Ecosystem Dependencies

| Component | Recommended Crate |
|-----------|-------------------|
| CLI Args | `clap` |
| Regex | `regex` (for markup parsing) |
| Colors | `palette` or custom struct |
| Terminal | `crossterm` (for detection/size) |
| Syntax | `syntect` |
| Markdown | `pulldown-cmark` |

## 6. Directory Structure

```
src/
├── main.rs (CLI entry point for testing)
├── lib.rs
├── console.rs
├── style.rs
├── text.rs
├── segment.rs
├── measure.rs
├── terminal.rs
├── renderables/
│   ├── mod.rs
│   ├── table.rs
│   ├── panel.rs
│   └── ...
├── markup/
│   ├── mod.rs
│   └── parser.rs
└── macros.rs (e.g., console_print!)
```
