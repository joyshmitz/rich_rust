//! Renderable components for rich output.
//!
//! This module contains various renderables like Rule, Panel, Table, etc.
//! that can be used to create rich terminal output.

pub mod align;
pub mod columns;
pub mod rule;
pub mod padding;
pub mod panel;
pub mod table;
pub mod progress;
pub mod tree;

// Re-export commonly used types
pub use align::{Align, AlignLines, AlignMethod, VerticalAlignMethod, align_text};
pub use columns::Columns;
pub use rule::Rule;
pub use padding::{Padding, PaddingDimensions};
pub use panel::Panel;
pub use table::{Table, Column, Row, Cell, VerticalAlign};
pub use progress::{ProgressBar, BarStyle, Spinner};
pub use tree::{Tree, TreeNode, TreeGuides};

// Phase 3+: Syntax highlighting (requires "syntax" feature)
#[cfg(feature = "syntax")]
pub mod syntax;

#[cfg(feature = "syntax")]
pub use syntax::{Syntax, SyntaxError};

// Phase 3+: Markdown rendering (requires "markdown" feature)
#[cfg(feature = "markdown")]
pub mod markdown;

#[cfg(feature = "markdown")]
pub use markdown::Markdown;

// Phase 4: JSON rendering (requires "json" feature)
#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "json")]
pub use json::{Json, JsonError, JsonTheme};
