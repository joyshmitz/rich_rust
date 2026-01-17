//! # rich_rust
//!
//! A Rust port of Python's Rich library for beautiful terminal output.
//!
//! This library provides an abstraction over ANSI escape codes to render styled text,
//! tables, panels, and more in the terminal.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use rich_rust::prelude::*;
//!
//! let console = Console::new();
//! console.print("[bold red]Hello[/] [green]World[/]!");
//! ```
//!
//! ## Core Concepts
//!
//! - **Console**: The central entry point for printing styled output
//! - **Style**: Visual attributes (color, bold, italic, etc.)
//! - **Text**: Rich text with overlapping style spans
//! - **Segment**: The atomic rendering unit (text + style)
//! - **Renderable**: Trait for anything that can be printed

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod color;
pub mod style;
pub mod segment;
pub mod text;
pub mod markup;
pub mod measure;
pub mod console;
pub mod terminal;
pub mod cells;
pub mod r#box;
pub mod renderables;

/// Re-exports for convenient usage
pub mod prelude {
    pub use crate::color::{Color, ColorSystem, ColorType, ColorTriplet};
    pub use crate::style::{Style, Attributes};
    pub use crate::segment::Segment;
    pub use crate::text::{Text, Span, JustifyMethod, OverflowMethod};
    pub use crate::console::{Console, ConsoleOptions};
    pub use crate::measure::Measurement;
    pub use crate::r#box::BoxChars;
    pub use crate::renderables::{
        Align, AlignLines, AlignMethod, VerticalAlignMethod, align_text,
        Columns, Rule, Panel, Table, Column, Row, Cell, PaddingDimensions, VerticalAlign,
        ProgressBar, BarStyle, Spinner, Tree, TreeNode, TreeGuides,
    };

    #[cfg(feature = "syntax")]
    pub use crate::renderables::{Syntax, SyntaxError};

    #[cfg(feature = "markdown")]
    pub use crate::renderables::Markdown;

    #[cfg(feature = "json")]
    pub use crate::renderables::{Json, JsonError, JsonTheme};
}

// Re-export key types at crate root
pub use color::{Color, ColorSystem, ColorType, ColorTriplet};
pub use style::{Style, Attributes};
pub use segment::Segment;
pub use text::{Text, Span};
pub use console::Console;
