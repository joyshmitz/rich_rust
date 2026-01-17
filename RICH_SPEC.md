# Rich Specification for Rust Port

> **Purpose:** Complete behavioral specification extracted from Python Rich source.
> **Rule:** After reading this spec, you should NOT need the legacy Python code.
> **Status:** COMPLETE - All sections extracted

---

## Table of Contents

1. [Color System](#1-color-system)
2. [Style System](#2-style-system)
3. [Segment (Rendering Atom)](#3-segment-rendering-atom)
4. [Markup Parser](#4-markup-parser)
5. [Measurement Protocol](#5-measurement-protocol)
6. [Text Object](#6-text-object)
7. [Console](#7-console)
8. [Box Characters](#8-box-characters)
9. [Table Layout](#9-table-layout)
10. [Panel & Padding](#10-panel--padding)
11. [Alignment System](#11-alignment-system)
12. [Unicode Cell Width](#12-unicode-cell-width)
13. [Text Wrapping](#13-text-wrapping)
14. [Ratio Distribution](#14-ratio-distribution)
15. [Exclusions](#15-exclusions)

---

## 1. Color System

> Source: `rich/color.py` (621 lines), `rich/color_triplet.py`, `rich/_palettes.py`

### 1.1 Data Structures

#### ColorTriplet

NamedTuple representing RGB color components.

```rust
struct ColorTriplet {
    red: u8,        // Red component in 0-255 range
    green: u8,      // Green component in 0-255 range
    blue: u8,       // Blue component in 0-255 range
}
```

**Properties:**
- `hex()` -> String: Returns CSS-style hex format `#rrggbb` (e.g., `#FF0000`)
- `rgb()` -> String: Returns CSS-style rgb format `rgb(r,g,b)` (e.g., `rgb(255,0,0)`)
- `normalized()` -> (f64, f64, f64): Returns (red, green, blue) as floats in range 0.0-1.0

#### ColorSystem (IntEnum)

Represents the color system capability of terminals.

```rust
enum ColorSystem {
    STANDARD = 1,    // 4-bit ANSI colors (16 colors)
    EIGHT_BIT = 2,   // 8-bit colors (256 colors)
    TRUECOLOR = 3,   // 24-bit RGB colors (16 million colors)
    WINDOWS = 4,     // Windows 10+ console palette (16 colors)
}
```

#### ColorType (IntEnum)

Type of color stored in Color structure.

```rust
enum ColorType {
    DEFAULT = 0,     // Default terminal color (no RGB/number)
    STANDARD = 1,    // 4-bit ANSI standard color (0-15)
    EIGHT_BIT = 2,   // 8-bit color (0-255)
    TRUECOLOR = 3,   // 24-bit RGB color
    WINDOWS = 4,     // Windows console color (0-15)
}
```

#### Color Structure

```rust
struct Color {
    name: String,                    // Name of the color (input that was parsed)
    color_type: ColorType,           // Type of color
    number: Option<u8>,             // Color number (for STANDARD, EIGHT_BIT, WINDOWS)
    triplet: Option<ColorTriplet>,  // RGB components (for TRUECOLOR)
}
```

**Methods:**
- `system()` -> ColorSystem: Returns the native color system for this color
- `is_system_defined()` -> bool: Returns true if system is STANDARD or WINDOWS
- `is_default()` -> bool: Returns true if color_type == DEFAULT
- `get_truecolor(theme, foreground)` -> ColorTriplet: Converts color to RGB triplet
- `from_ansi(number: u8)` -> Color: Create from 8-bit ANSI number
- `from_triplet(triplet)` -> Color: Create from RGB triplet as TRUECOLOR
- `from_rgb(red, green, blue)` -> Color: Create from RGB components
- `default()` -> Color: Create default color
- `parse(color: &str)` -> Result<Color, ColorParseError>: Parse color string (cached, LRU 1024)
- `get_ansi_codes(foreground: bool)` -> Vec<String>: Get ANSI escape codes
- `downgrade(system: ColorSystem)` -> Color: Convert to lower-capability color system

### 1.2 Color Parsing

The `Color::parse()` method accepts these formats (case-insensitive):

| Format | Example | Result |
|--------|---------|--------|
| Named colors | `red`, `bright_blue` | STANDARD (0-15) or EIGHT_BIT (16-255) |
| Hex format | `#FF0000` | TRUECOLOR with RGB triplet |
| Color number | `color(196)` | STANDARD if 0-15, EIGHT_BIT if 16-255 |
| RGB format | `rgb(255,0,0)` | TRUECOLOR with RGB triplet |
| Default | `default` | ColorType::DEFAULT |

**Regex Pattern:**
```
^#([0-9a-f]{6})$|color\(([0-9]{1,3})\)$|rgb\(([\d\s,]+)\)$
```

**Parsing Rules:**
- Input is lowercased and trimmed
- Whitespace allowed in rgb() format
- Color numbers must be <= 255
- RGB components must be <= 255
- Results cached with LRU cache (max 1024 entries)

### 1.3 Color Palettes

#### STANDARD_PALETTE (16 colors)

```
Index  RGB
0      (0,     0,     0)      # Black
1      (170,   0,     0)      # Red
2      (0,     170,   0)      # Green
3      (170,   85,    0)      # Yellow
4      (0,     0,     170)    # Blue
5      (170,   0,     170)    # Magenta
6      (0,     170,   170)    # Cyan
7      (170,   170,   170)    # White
8      (85,    85,    85)     # Bright Black (Gray)
9      (255,   85,    85)     # Bright Red
10     (85,    255,   85)     # Bright Green
11     (255,   255,   85)     # Bright Yellow
12     (85,    85,    255)    # Bright Blue
13     (255,   85,    255)    # Bright Magenta
14     (85,    255,   255)    # Bright Cyan
15     (255,   255,   255)    # Bright White
```

#### EIGHT_BIT_PALETTE (256 colors)

- Indices 0-15: Same as STANDARD_PALETTE
- Indices 16-231: 6x6x6 RGB color cube (216 colors)
  - Grid: 6 levels per component (0, 95, 135, 175, 215, 255)
  - Index formula: `16 + 36 * red_index + 6 * green_index + blue_index`
- Indices 232-255: Grayscale ramp (24 shades)
  - Index 232: (8, 8, 8) ... Index 255: (238, 238, 238)

#### WINDOWS_PALETTE (16 colors)

```
Index  RGB
0      (12,    12,    12)     # Black
1      (197,   15,    31)     # Red
2      (19,    161,   14)     # Green
3      (193,   156,   0)      # Yellow
4      (0,     55,    218)    # Blue
5      (136,   23,    152)    # Magenta
6      (58,    150,   221)    # Cyan
7      (204,   204,   204)    # White
8      (118,   118,   118)    # Bright Black
9      (231,   72,    86)     # Bright Red
10     (22,    198,   12)     # Bright Green
11     (249,   241,   165)    # Bright Yellow
12     (59,    120,   255)    # Bright Blue
13     (180,   0,     158)    # Bright Magenta
14     (97,    214,   214)    # Bright Cyan
15     (242,   242,   242)    # Bright White
```

### 1.4 Color Conversion Algorithms

#### RGB to 8-bit (TRUECOLOR -> EIGHT_BIT)

**Grayscale Detection:** Convert RGB to HLS, check if saturation < 0.15:
- If grayscale, use luminance-based mapping to indices 232-255

**Color Cube Mapping:** For non-grayscale:
```
for each component in [red, green, blue]:
    if component < 95:
        quantized = component / 95
    else:
        quantized = 1 + (component - 95) / 40
    quantized_index = round(quantized)  // 0-5

color_number = 16 + 36 * red_idx + 6 * green_idx + blue_idx
```

#### RGB to Standard (-> STANDARD)

Use weighted CIE76 color distance formula:
```
red_mean = (r1 + r2) / 2
distance = sqrt(
    (((512 + red_mean) * red_diff^2) >> 8)
    + 4 * green_diff^2
    + (((767 - red_mean) * blue_diff^2) >> 8)
)
```

### 1.5 ANSI Code Generation

| ColorType | Foreground | Background |
|-----------|-----------|-----------|
| DEFAULT | ["39"] | ["49"] |
| STANDARD (0-7) | ["30"+n] | ["40"+n] |
| STANDARD (8-15) | ["82"+n] | ["92"+n] |
| EIGHT_BIT | ["38", "5", "N"] | ["48", "5", "N"] |
| TRUECOLOR | ["38", "2", "R", "G", "B"] | ["48", "2", "R", "G", "B"] |

---

## 2. Style System

> Source: `rich/style.py` (792 lines)

### 2.1 Style Data Structure

```rust
struct Style {
    color: Option<Color>,           // Foreground color
    bgcolor: Option<Color>,         // Background color
    attributes: u16,                // Bit flags for enabled attributes (13 bits)
    set_attributes: u16,            // Bit flags for which attributes are explicitly set
    link: Option<String>,           // URL for hyperlinks
    link_id: String,                // Random ID for hyperlink tracking
    meta: Option<Vec<u8>>,          // Serialized metadata
    null: bool,                     // True if this is an empty/null style
}
```

### 2.2 Style Attributes (Bitflags)

| Bit | Attribute    | SGR Code | Meaning |
|-----|--------------|----------|---------|
| 0   | bold         | 1        | Bold/bright text |
| 1   | dim          | 2        | Dim/faint text |
| 2   | italic       | 3        | Italic text |
| 3   | underline    | 4        | Single underline |
| 4   | blink        | 5        | Blinking text (slow) |
| 5   | blink2       | 6        | Fast blinking text |
| 6   | reverse      | 7        | Reverse video |
| 7   | conceal      | 8        | Concealed/hidden text |
| 8   | strike       | 9        | Strikethrough text |
| 9   | underline2   | 21       | Double underline |
| 10  | frame        | 51       | Framed text |
| 11  | encircle     | 52       | Encircled text |
| 12  | overline     | 53       | Overlined text |

**Attribute Aliases for Parsing:**
```
bold -> "bold", "b"
dim -> "dim", "d"
italic -> "italic", "i"
underline -> "underline", "u"
reverse -> "reverse", "r"
conceal -> "conceal", "c"
strike -> "strike", "s"
underline2 -> "underline2", "uu"
overline -> "overline", "o"
```

### 2.3 Style Parsing

Supported style string formats:

| Format | Example | Result |
|--------|---------|--------|
| Empty/Null | `""` or `"none"` | NULL_STYLE |
| Attribute | `"bold"`, `"italic"` | Enable attribute |
| Negative | `"not bold"` | Disable attribute |
| Color | `"red"`, `"#ff0000"` | Set foreground |
| Background | `"on red"`, `"on #ff0000"` | Set background |
| Link | `"link https://..."` | Set hyperlink |
| Combined | `"bold red on white"` | Multiple properties |

### 2.4 Style Combination Logic (`style1 + style2`)

```rust
fn combine(self, other: Style) -> Style {
    if other.is_null() { return self; }
    if self.is_null() { return other; }

    Style {
        color: other.color.or(self.color),
        bgcolor: other.bgcolor.or(self.bgcolor),
        attributes: (self.attributes & !other.set_attributes)
                  | (other.attributes & other.set_attributes),
        set_attributes: self.set_attributes | other.set_attributes,
        link: other.link.or(self.link),
        meta: merge(self.meta, other.meta),  // other overwrites
    }
}
```

**Rules:**
1. `style2.color` overrides if set, else keep `style1.color`
2. `style2.bgcolor` overrides if set, else keep `style1.bgcolor`
3. For attributes: if `style2.set_attributes[bit] == 1`, use `style2.attributes[bit]`
4. `style2.link` overrides if set

### 2.5 ANSI Code Generation

```rust
fn make_ansi_codes(&self, color_system: ColorSystem) -> String {
    let mut codes = Vec::new();

    // Enabled attributes
    for (bit, sgr) in STYLE_MAP {
        if self.attributes & self.set_attributes & (1 << bit) != 0 {
            codes.push(sgr);
        }
    }

    // Foreground color
    if let Some(color) = &self.color {
        codes.extend(color.downgrade(color_system).get_ansi_codes(true));
    }

    // Background color
    if let Some(bgcolor) = &self.bgcolor {
        codes.extend(bgcolor.downgrade(color_system).get_ansi_codes(false));
    }

    codes.join(";")
}
```

Final ANSI sequence: `"\x1b[" + codes + "m" + text + "\x1b[0m"`

### 2.6 Hyperlink Support

OSC 8 hyperlink protocol:
```
"\x1b]8;id={link_id};{url}\x1b\\{text}\x1b]8;;\x1b\\"
```

### 2.7 StyleStack

```rust
struct StyleStack {
    stack: Vec<Style>,
}

impl StyleStack {
    fn new(default: Style) -> Self { Self { stack: vec![default] } }
    fn current(&self) -> &Style { self.stack.last().unwrap() }
    fn push(&mut self, style: Style) {
        self.stack.push(self.current().clone() + style);
    }
    fn pop(&mut self) -> &Style {
        self.stack.pop();
        self.current()
    }
}
```

---

## 3. Segment (Rendering Atom)

> Source: `rich/segment.py` (752 lines)

### 3.1 ControlType Enum

```rust
enum ControlType {
    BELL = 1,
    CARRIAGE_RETURN = 2,
    HOME = 3,
    CLEAR = 4,
    SHOW_CURSOR = 5,
    HIDE_CURSOR = 6,
    ENABLE_ALT_SCREEN = 7,
    DISABLE_ALT_SCREEN = 8,
    CURSOR_UP = 9,
    CURSOR_DOWN = 10,
    CURSOR_FORWARD = 11,
    CURSOR_BACKWARD = 12,
    CURSOR_MOVE_TO_COLUMN = 13,
    CURSOR_MOVE_TO = 14,
    ERASE_IN_LINE = 15,
    SET_WINDOW_TITLE = 16,
}
```

### 3.2 Segment Structure

```rust
struct Segment {
    text: String,
    style: Option<Style>,
    control: Option<Vec<ControlCode>>,
}

impl Segment {
    fn cell_length(&self) -> usize {
        if self.control.is_some() { 0 } else { cell_len(&self.text) }
    }

    fn is_control(&self) -> bool {
        self.control.is_some()
    }
}
```

### 3.3 Segment Operations

#### Line Creation
```rust
fn line() -> Segment { Segment { text: "\n".into(), style: None, control: None } }
```

#### Style Application
```rust
fn apply_style(segments: impl Iterator<Item=Segment>, style: Option<Style>, post_style: Option<Style>) -> impl Iterator<Item=Segment>
```
- If style provided: applies `style + segment.style`
- If post_style provided: applies `segment.style + post_style`

#### Line Splitting
```rust
fn split_lines(segments: impl Iterator<Item=Segment>) -> impl Iterator<Item=Vec<Segment>>
```
Splits at newline characters. Each yielded Vec is one line (excluding newline).

#### Line Length Adjustment
```rust
fn adjust_line_length(line: Vec<Segment>, length: usize, style: Option<Style>, pad: bool) -> Vec<Segment>
```
- If line shorter than length and pad=true: appends padding
- If line longer: truncates (may split segments)
- Control segments never truncated

#### Simplification
```rust
fn simplify(segments: impl Iterator<Item=Segment>) -> impl Iterator<Item=Segment>
```
Merges contiguous segments with identical styles.

#### Division
```rust
fn divide(segments: impl Iterator<Item=Segment>, cuts: impl Iterator<Item=usize>) -> impl Iterator<Item=Vec<Segment>>
```
Divides segments at specified cell positions.

#### Alignment Methods
```rust
fn align_top(lines: Vec<Vec<Segment>>, width: usize, height: usize, style: Style) -> Vec<Vec<Segment>>
fn align_bottom(lines: Vec<Vec<Segment>>, width: usize, height: usize, style: Style) -> Vec<Vec<Segment>>
fn align_middle(lines: Vec<Vec<Segment>>, width: usize, height: usize, style: Style) -> Vec<Vec<Segment>>
```

---

## 4. Markup Parser

> Source: `rich/markup.py` (251 lines)

### 4.1 Markup Syntax

```
[tag_name]text[/tag_name]     # Basic tag
[/]                            # Close most recent tag
[tag=parameter]text[/tag]      # Tag with parameter
[bold red]text[/]              # Multiple styles
[@handler(args)]text[/@handler] # Metadata tag
```

**Tag Name Rules:**
- Must start with: `a-z`, `#`, `@`, or `/`
- Cannot contain `[` or `]`

### 4.2 Regex Patterns

**Main tag pattern:**
```regex
((\\*)\[([a-z#/@][^[]*?)])
```
- Group 1: Full match including escapes
- Group 2: Leading backslashes
- Group 3: Tag content

**Handler pattern:**
```regex
^([\w.]*?)(\(.*?\))?$
```

### 4.3 Parsing Algorithm

```rust
fn render(markup: &str) -> Text {
    // Optimization: if no '[', return plain text
    if !markup.contains('[') {
        return Text::new(markup);
    }

    let mut text = Text::new();
    let mut style_stack: Vec<(usize, Tag)> = Vec::new();

    for (position, plain_text, tag) in parse(markup) {
        if let Some(plain) = plain_text {
            // Replace escaped brackets
            let unescaped = plain.replace("\\[", "[");
            text.append(&unescaped);
        }

        if let Some(tag) = tag {
            if !tag.name.starts_with('/') {
                // Opening tag
                style_stack.push((text.len(), tag));
            } else {
                // Closing tag
                let style_name = &tag.name[1..].trim();
                let (start, open_tag) = if style_name.is_empty() {
                    // Implicit close [/]
                    style_stack.pop().ok_or(MarkupError)?
                } else {
                    // Explicit close [/name]
                    pop_matching(&mut style_stack, style_name)?
                };
                text.add_span(start, text.len(), &open_tag);
            }
        }
    }

    // Auto-close unclosed tags
    while let Some((start, tag)) = style_stack.pop() {
        text.add_span(start, text.len(), &tag);
    }

    text
}
```

### 4.4 Escape Sequences

| Input | Output |
|-------|--------|
| `\[` | Literal `[` |
| `\\[tag]` | Literal `\` + tag applied |
| `\\\[tag]` | Literal `\[tag]` (escaped) |

### 4.5 Tag Nesting

- Tags can nest arbitrarily deep
- `[/]` closes most recent tag (LIFO)
- `[/name]` closes specific tag (searches stack)
- Unclosed tags auto-close at end

### 4.6 Error Conditions

| Error | Message |
|-------|---------|
| `[/]` with empty stack | "closing tag '[/]' has nothing to close" |
| `[/name]` not found | "closing tag '[/name]' doesn't match any open tag" |

---

## 5. Measurement Protocol

> Source: `rich/measure.py` (151 lines)

### 5.1 Measurement Structure

```rust
struct Measurement {
    minimum: usize,  // Minimum cells required
    maximum: usize,  // Maximum cells required
}

impl Measurement {
    fn span(&self) -> usize { self.maximum - self.minimum }

    fn normalize(&self) -> Self {
        let min = self.minimum.min(self.maximum).max(0);
        let max = self.maximum.max(self.minimum).max(0);
        Measurement { minimum: min, maximum: max }
    }

    fn with_maximum(&self, width: usize) -> Self {
        Measurement {
            minimum: self.minimum.min(width),
            maximum: self.maximum.min(width),
        }
    }

    fn with_minimum(&self, width: usize) -> Self {
        let width = width.max(0);
        Measurement {
            minimum: self.minimum.max(width),
            maximum: self.maximum.max(width),
        }
    }

    fn clamp(&self, min_width: Option<usize>, max_width: Option<usize>) -> Self {
        let mut m = *self;
        if let Some(min) = min_width { m = m.with_minimum(min); }
        if let Some(max) = max_width { m = m.with_maximum(max); }
        m
    }
}
```

### 5.2 Measurement.get()

```rust
fn get(console: &Console, options: &ConsoleOptions, renderable: &dyn Renderable) -> Measurement {
    let max_width = options.max_width;
    if max_width < 1 { return Measurement { minimum: 0, maximum: 0 }; }

    if let Some(measure_fn) = renderable.rich_measure() {
        measure_fn(console, options)
            .normalize()
            .with_maximum(max_width)
            .normalize()
    } else {
        Measurement { minimum: 0, maximum: max_width }
    }
}
```

### 5.3 measure_renderables()

```rust
fn measure_renderables(console: &Console, options: &ConsoleOptions, renderables: &[&dyn Renderable]) -> Measurement {
    if renderables.is_empty() {
        return Measurement { minimum: 0, maximum: 0 };
    }

    let measurements: Vec<_> = renderables.iter()
        .map(|r| Measurement::get(console, options, *r))
        .collect();

    Measurement {
        minimum: measurements.iter().map(|m| m.minimum).max().unwrap(),
        maximum: measurements.iter().map(|m| m.maximum).max().unwrap(),
    }
}
```

**Aggregation Rules:**
- Combined minimum = max of all minimums (tightest constraint)
- Combined maximum = max of all maximums (most flexible)

---

## 6. Text Object

> Source: `rich/text.py` (1361 lines)

### 6.1 Text Data Structure

```rust
/// Justify method for text alignment
enum JustifyMethod {
    Default,  // Use console default
    Left,
    Center,
    Right,
    Full,     // Justify to fill width
}

/// Overflow handling method
enum OverflowMethod {
    Fold,     // Fold onto next line (default)
    Crop,     // Crop at boundary
    Ellipsis, // Show "..." at truncation
    Ignore,   // No overflow handling
}

/// A span of styled text (indices are CHARACTER offsets, not byte offsets)
struct Span {
    start: usize,   // Start character index (inclusive)
    end: usize,     // End character index (exclusive)
    style: Style,   // Style to apply
}

/// Rich text with spans
struct Text {
    plain: String,           // Plain text content (String of text pieces joined)
    spans: Vec<Span>,        // List of style spans
    length: usize,           // Cached character length
    style: Style,            // Base style for entire text
    justify: JustifyMethod,
    overflow: OverflowMethod,
    no_wrap: bool,           // Disable wrapping
    end: String,             // String to append after text (default "\n")
    tab_size: usize,         // Tab expansion size (default 8)
}
```

### 6.2 Span Management

**Span Invariants:**
- `start <= end`
- Spans can overlap (later spans take precedence in rendering)
- Indices are character positions, NOT byte positions

**Key Methods:**

```rust
impl Span {
    /// Right-adjust span by offset
    fn move_right(&self, offset: usize, max: usize) -> Span {
        Span {
            start: (self.start + offset).min(max),
            end: (self.end + offset).min(max),
            style: self.style.clone(),
        }
    }

    /// Split span at position
    fn split(&self, offset: usize) -> (Span, Span) {
        (
            Span { start: self.start, end: self.start + offset, style: self.style.clone() },
            Span { start: self.start + offset, end: self.end, style: self.style.clone() },
        )
    }
}
```

### 6.3 Text Manipulation Methods

```rust
impl Text {
    /// Create from plain string
    fn new(text: &str) -> Self;

    /// Create from markup string (parses [tags])
    fn from_markup(markup: &str) -> Self;

    /// Append plain text
    fn append(&mut self, text: &str);

    /// Append another Text object (merges spans)
    fn append_text(&mut self, text: &Text);

    /// Apply style to range
    fn stylize(&mut self, start: usize, end: usize, style: Style);

    /// Highlight text matching regex with style
    fn highlight_regex(&mut self, pattern: &str, style: Style);

    /// Highlight text matching string with style
    fn highlight_words(&mut self, words: &[&str], style: Style, case_sensitive: bool);

    /// Truncate to max width, adding suffix if needed
    fn truncate(&mut self, max_width: usize, overflow: OverflowMethod, pad: bool);

    /// Pad text to width
    fn pad(&mut self, width: usize, align: JustifyMethod);

    /// Split into lines at newlines
    fn split_lines(&self, split_on_space: bool) -> Vec<Text>;

    /// Get substring as new Text (preserves styles)
    fn slice(&self, start: usize, end: usize) -> Text;
}
```

### 6.4 Text Division Algorithm (CRITICAL)

The `divide()` method splits Text at specified cut points while preserving spans.

```rust
/// Divide text into parts at specified character offsets
fn divide(&self, offsets: &[usize]) -> Vec<Text> {
    if offsets.is_empty() {
        return vec![self.clone()];
    }

    let text_length = self.length;
    let mut result = Vec::with_capacity(offsets.len());

    // For each span, distribute to appropriate output divisions
    for span in &self.spans {
        // Use binary search to find which divisions this span overlaps
        let lower = match offsets.binary_search(&span.start) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let upper = match offsets.binary_search(&span.end) {
            Ok(i) => i,
            Err(i) => i,
        };

        // Span may appear in multiple divisions
        for div_idx in lower..=upper {
            let div_start = if div_idx == 0 { 0 } else { offsets[div_idx - 1] };
            let div_end = offsets.get(div_idx).copied().unwrap_or(text_length);

            // Calculate span position relative to division
            let rel_start = span.start.saturating_sub(div_start);
            let rel_end = span.end.min(div_end).saturating_sub(div_start);

            if rel_start < rel_end {
                // Add adjusted span to this division
                result[div_idx].spans.push(Span {
                    start: rel_start,
                    end: rel_end,
                    style: span.style.clone(),
                });
            }
        }
    }

    result
}
```

### 6.5 Text Rendering to Segments

```rust
/// Render Text to iterator of Segments
fn render(&self, console: &Console, end: &str) -> Vec<Segment> {
    // Style combination cache for performance
    let mut style_cache: HashMap<usize, Style> = HashMap::new();

    let null_style = Style::null();
    let enumerated_spans: Vec<(usize, &Span)> = self.spans.iter().enumerate().collect();

    let mut result = Vec::new();

    // Build a map: character position -> list of (span_index, is_start)
    let mut events: BTreeMap<usize, Vec<(usize, bool)>> = BTreeMap::new();
    for (idx, span) in &enumerated_spans {
        events.entry(span.start).or_default().push((*idx, true));  // start
        events.entry(span.end).or_default().push((*idx, false));   // end
    }

    // Walk through text, tracking active spans via stack
    let mut active_spans: Vec<usize> = Vec::new();  // Stack of span indices
    let mut pos = 0;
    let chars: Vec<char> = self.plain.chars().collect();

    for (event_pos, span_events) in events {
        // Emit text before this event
        if event_pos > pos {
            let text: String = chars[pos..event_pos].iter().collect();
            let style = compute_combined_style(&active_spans, &enumerated_spans, &self.style, &mut style_cache);
            result.push(Segment { text, style: Some(style), control: None });
            pos = event_pos;
        }

        // Process events (ends before starts for correct nesting)
        for (span_idx, is_start) in span_events {
            if is_start {
                active_spans.push(span_idx);
            } else {
                active_spans.retain(|&x| x != span_idx);
            }
        }
    }

    // Emit remaining text
    if pos < chars.len() {
        let text: String = chars[pos..].iter().collect();
        let style = compute_combined_style(&active_spans, &enumerated_spans, &self.style, &mut style_cache);
        result.push(Segment { text, style: Some(style), control: None });
    }

    // Append end string
    if !end.is_empty() {
        result.push(Segment { text: end.to_string(), style: None, control: None });
    }

    result
}

/// Combine styles from active spans (stack-based, later spans override)
fn compute_combined_style(
    active_spans: &[usize],
    spans: &[(usize, &Span)],
    base_style: &Style,
    cache: &mut HashMap<usize, Style>,
) -> Style {
    // Create cache key from active span indices
    let cache_key = hash(active_spans);
    if let Some(cached) = cache.get(&cache_key) {
        return cached.clone();
    }

    let mut combined = base_style.clone();
    for &span_idx in active_spans {
        combined = combined + spans[span_idx].1.style.clone();
    }

    cache.insert(cache_key, combined.clone());
    combined
}
```

### 6.6 Text Wrapping

```rust
/// Wrap text to fit within width
fn wrap(
    &self,
    console: &Console,
    width: usize,
    justify: JustifyMethod,
    overflow: OverflowMethod,
    tab_size: usize,
    no_wrap: bool,
) -> Vec<Text> {
    // Expand tabs first
    let expanded = self.expand_tabs(tab_size);

    // If no_wrap or width is huge, return as single line
    if no_wrap || width >= expanded.cell_len() {
        return vec![expanded];
    }

    let mut lines = Vec::new();

    for line in expanded.split_lines(false) {
        if line.cell_len() <= width {
            lines.push(line);
        } else {
            // Need to wrap this line
            match overflow {
                OverflowMethod::Fold => {
                    lines.extend(wrap_fold(&line, width));
                }
                OverflowMethod::Crop => {
                    lines.push(line.slice(0, width));
                }
                OverflowMethod::Ellipsis => {
                    let mut truncated = line.slice(0, width.saturating_sub(1));
                    truncated.append("...");
                    lines.push(truncated);
                }
                OverflowMethod::Ignore => {
                    lines.push(line);
                }
            }
        }
    }

    // Apply justification
    for line in &mut lines {
        line.apply_justify(justify, width);
    }

    lines
}
```

---

## 7. Console

> Source: `rich/console.py` (2680 lines)

### 7.1 ConsoleDimensions

```rust
struct ConsoleDimensions {
    width: usize,   // Console width in cells
    height: usize,  // Console height in rows
}
```

### 7.2 ConsoleOptions Data Structure

```rust
/// Options passed to renderables during rendering
struct ConsoleOptions {
    size: ConsoleDimensions,          // Terminal dimensions
    legacy_windows: bool,             // Using legacy Windows console
    min_width: usize,                 // Minimum width constraint
    max_width: usize,                 // Maximum width constraint
    is_terminal: bool,                // Output is a terminal (vs file/pipe)
    encoding: String,                 // Output encoding (e.g., "utf-8")
    max_height: usize,                // Maximum height for rendering
    justify: Option<JustifyMethod>,   // Default justification
    overflow: Option<OverflowMethod>, // Default overflow handling
    no_wrap: Option<bool>,            // Default no_wrap setting
    highlight: Option<bool>,          // Enable highlighting
    markup: Option<bool>,             // Parse markup in strings
    height: Option<usize>,            // Explicit height override
}

impl ConsoleOptions {
    /// Create new options with different max_width
    fn update_width(&self, width: usize) -> Self {
        ConsoleOptions {
            max_width: width.min(self.max_width),
            ..self.clone()
        }
    }

    /// Create options for rendering within container (reduces width)
    fn update_dimensions(&self, width: usize, height: usize) -> Self {
        ConsoleOptions {
            max_width: width.min(self.max_width),
            height: Some(height),
            ..self.clone()
        }
    }
}
```

### 7.3 Console Structure

```rust
struct Console {
    // Configuration
    color_system: Option<ColorSystem>,  // None = auto-detect
    force_terminal: Option<bool>,       // Force terminal mode
    tab_size: usize,                    // Tab expansion (default 8)
    record: bool,                       // Buffer output for export
    markup: bool,                       // Parse markup by default
    emoji: bool,                        // Enable emoji rendering
    highlight: bool,                    // Enable syntax highlighting
    width: Option<usize>,               // Override width
    height: Option<usize>,              // Override height
    safe_box: bool,                     // Use ASCII-safe box chars

    // State
    file: Box<dyn Write>,               // Output stream
    buffer: Vec<Segment>,               // Recording buffer
    is_terminal: bool,                  // Cached terminal detection
    encoding: String,                   // Output encoding
}
```

### 7.4 Color System Detection

```rust
fn detect_color_system() -> Option<ColorSystem> {
    // Check NO_COLOR env var (https://no-color.org/)
    if std::env::var("NO_COLOR").is_ok() {
        return None;
    }

    // Check COLORTERM for truecolor
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        if colorterm == "truecolor" || colorterm == "24bit" {
            return Some(ColorSystem::TRUECOLOR);
        }
    }

    // Check TERM for 256 color support
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("256color") || term.contains("256") {
            return Some(ColorSystem::EIGHT_BIT);
        }
        if term == "dumb" {
            return None;
        }
    }

    // Default to standard colors if terminal
    Some(ColorSystem::STANDARD)
}
```

### 7.5 Rendering Pipeline

```rust
impl Console {
    /// Main print method
    fn print(&mut self, renderable: impl Renderable, options: PrintOptions) {
        // 1. Collect all renderables
        let renderables = self.collect_renderables(renderable);

        // 2. Create console options
        let console_options = self.make_options();

        // 3. Render to segments
        let segments = self.render(renderables, &console_options);

        // 4. Write or buffer
        if self.record {
            self.buffer.extend(segments);
        } else {
            self.write_segments(segments);
        }
    }

    /// Collect renderables, handling strings and conversions
    fn collect_renderables(&self, obj: impl Renderable) -> Vec<Box<dyn Renderable>> {
        // If object implements __rich_console__, use it
        // If object implements __rich__, convert to Text
        // If string, convert to Text (with optional markup parsing)
    }

    /// Render all objects to flat segment list
    fn render(&self, renderables: Vec<Box<dyn Renderable>>, options: &ConsoleOptions) -> Vec<Segment> {
        let mut result = Vec::new();

        for renderable in renderables {
            // Call rich_console to get segments/nested renderables
            for item in renderable.rich_console(self, options) {
                match item {
                    RenderItem::Segment(seg) => result.push(seg),
                    RenderItem::Renderable(nested) => {
                        // Recursive render
                        result.extend(self.render(vec![nested], options));
                    }
                }
            }
        }

        result
    }

    /// Write segments to output with ANSI codes
    fn write_segments(&mut self, segments: Vec<Segment>) {
        let mut current_style = Style::null();
        let color_system = self.color_system.unwrap_or(ColorSystem::STANDARD);

        for segment in segments {
            if segment.is_control() {
                // Handle control codes
                self.write_control(&segment);
                continue;
            }

            let style = segment.style.unwrap_or_default();

            // Generate style transition
            if style != current_style {
                // Reset then apply new style
                if !current_style.is_null() {
                    write!(self.file, "\x1b[0m").ok();
                }
                if !style.is_null() {
                    let codes = style.make_ansi_codes(color_system);
                    write!(self.file, "\x1b[{}m", codes).ok();
                }
                current_style = style;
            }

            // Write text
            write!(self.file, "{}", segment.text).ok();
        }

        // Reset at end
        if !current_style.is_null() {
            write!(self.file, "\x1b[0m").ok();
        }
    }
}
```

### 7.6 render_lines Helper

```rust
/// Render to list of lines, each line being a list of segments
fn render_lines(
    &self,
    renderable: &dyn Renderable,
    options: &ConsoleOptions,
    style: Option<&Style>,
    pad: bool,
    new_lines: bool,
) -> Vec<Vec<Segment>> {
    let segments = self.render(vec![renderable], options);

    // Split into lines
    let mut lines = Segment::split_lines(segments.into_iter());

    // Adjust each line to width
    if pad || options.max_width > 0 {
        for line in &mut lines {
            *line = Segment::adjust_line_length(
                std::mem::take(line),
                options.max_width,
                style.cloned(),
                pad,
            );
        }
    }

    // Add newlines if requested
    if new_lines {
        for line in &mut lines {
            line.push(Segment::line());
        }
    }

    lines
}
```

---

## 8. Box Characters

> Source: `rich/box.py` (474 lines)

### 8.1 Box Data Structure

Box characters are defined as an 8-line string, one character per position:

```rust
/// Box drawing definition
/// Format: 8 lines of 4 characters each
///   Line 0: top (left, middle, divider, right)
///   Line 1: head (left, center, vertical, right)
///   Line 2: head_row (left, middle, cross, right)
///   Line 3: mid (left, middle, cross, right)
///   Line 4: row (left, middle, cross, right)
///   Line 5: foot_row (left, middle, cross, right)
///   Line 6: foot (left, center, vertical, right)
///   Line 7: bottom (left, middle, divider, right)
struct Box {
    top: [char; 4],
    head: [char; 4],
    head_row: [char; 4],
    mid: [char; 4],
    row: [char; 4],
    foot_row: [char; 4],
    foot: [char; 4],
    bottom: [char; 4],
    ascii: bool,  // Whether this is ASCII-safe
}

impl Box {
    /// Parse from 8-line string format
    fn from_str(s: &str) -> Self;

    /// Get top row string for given widths
    fn get_top(&self, widths: &[usize]) -> String;

    /// Get bottom row string for given widths
    fn get_bottom(&self, widths: &[usize]) -> String;

    /// Get separator row string for given widths
    fn get_row(
        &self,
        widths: &[usize],
        level: RowLevel,  // Head, Mid, Foot, Row
        edge: bool,       // Include edge characters
    ) -> String;
}
```

### 8.2 Built-in Box Styles

```
ASCII:
+--+
| ||
|--+
|--+
|-+|
|--+
| ||
+--+

ASCII2:
+-++
| ||
+-++
+-++
+-++
+-++
| ||
+-++

ASCII_DOUBLE_HEAD:
+-++
| ||
+=++
|-+|
|-+|
|-+|
| ||
+-++

SQUARE:
+--+
| ||
+--+
+--+
+-++
+--+
| ||
+--+

SQUARE_DOUBLE_HEAD:
+--+
| ||
+==+
+--+
+-++
+--+
| ||
+--+

MINIMAL:
    (spaces)
| ||
+--+



| ||


MINIMAL_HEAVY_HEAD:

| ||
+==+



| ||


MINIMAL_DOUBLE_HEAD:

| ||
+==+



| ||


SIMPLE:


+--+


+--+



SIMPLE_HEAD:


+--+






SIMPLE_HEAVY:


+==+


+==+



HORIZONTALS:
+--+

+--+
+--+
+--+
+--+

+--+

ROUNDED:
(Uses Unicode rounded corners: ., ', etc.)
.--,
| ||
|--+
|--+
|-+|
|--+
| ||
`--'

HEAVY:
+==+
# ##
+=++
+=++
+=++
+=++
# ##
+==+

HEAVY_EDGE:
+==+
| ||
+--+
+--+
+-++
+--+
| ||
+==+

HEAVY_HEAD:
+--+
| ||
+==+
+--+
+-++
+--+
| ||
+--+

DOUBLE:
+==+
| ||
+=++
+=++
+=++
+=++
| ||
+==+

DOUBLE_EDGE:
+==+
| ||
+--+
+--+
+-++
+--+
| ||
+==+

MARKDOWN:

| ||
|-||



| ||

```

**Note:** The above uses ASCII placeholders. Actual Unicode characters:
- `+` variants: `+`, `+`, `+`, `+` (corners)
- `-` variants: `-`, `=`, `_` (horizontal)
- `|` variants: `|`, `||`, `#` (vertical)
- Rounded: `.-,/` corner variants

### 8.3 Box Substitution Maps

**LEGACY_WINDOWS_SUBSTITUTIONS:**
Maps Unicode box characters to ASCII equivalents for legacy Windows console:

```rust
const LEGACY_WINDOWS_SUBSTITUTIONS: &[(&str, &str)] = &[
    ("-", "-"),    // Heavy horizontal to light
    ("|", "|"),    // Heavy vertical to light
    // ... more mappings for double-line and rounded characters
];
```

**PLAIN_HEADED_SUBSTITUTIONS:**
Maps SQUARE boxes to SQUARE_DOUBLE_HEAD when header style is needed.

### 8.4 Row Generation Methods

```rust
impl Box {
    /// Generate a row with given column widths
    fn get_row(&self, widths: &[usize], level: RowLevel, edge: bool) -> String {
        let (left, mid, cross, right) = match level {
            RowLevel::Top => self.top,
            RowLevel::Head => self.head_row,
            RowLevel::Mid => self.mid,
            RowLevel::Row => self.row,
            RowLevel::Foot => self.foot_row,
            RowLevel::Bottom => self.bottom,
        };

        let mut result = String::new();

        if edge {
            result.push(left);
        }

        for (i, &width) in widths.iter().enumerate() {
            // Add horizontal chars to fill width
            for _ in 0..width {
                result.push(mid);
            }
            // Add cross or right edge
            if i < widths.len() - 1 {
                result.push(cross);
            }
        }

        if edge {
            result.push(right);
        }

        result
    }
}
```

---

## 9. Table Layout

> Source: `rich/table.py` (1006 lines)

### 9.1 Table Data Structures

```rust
/// Single table column definition
struct Column {
    header: Text,                   // Column header text
    footer: Text,                   // Column footer text
    header_style: Style,            // Style for header
    footer_style: Style,            // Style for footer
    style: Style,                   // Style for cell content
    justify: JustifyMethod,         // Cell content justification
    vertical: VerticalAlignMethod,  // Vertical alignment
    overflow: OverflowMethod,       // Overflow handling
    width: Option<usize>,           // Fixed width (cells)
    min_width: Option<usize>,       // Minimum width
    max_width: Option<usize>,       // Maximum width
    ratio: Option<usize>,           // Ratio for flexible sizing
    no_wrap: bool,                  // Disable text wrapping
    // Internal state
    _index: usize,                  // Column index
    _cells: Vec<Box<dyn Renderable>>, // Cells in this column
}

/// Single table row
struct Row {
    style: Style,     // Row-level style
    end_section: bool, // Draw separator after this row
}

/// Single table cell (internal)
struct Cell {
    style: Style,               // Cell-specific style
    renderable: Box<dyn Renderable>,
    vertical: VerticalAlignMethod,
}

/// Table configuration
struct Table {
    columns: Vec<Column>,
    rows: Vec<Row>,
    cells: Vec<Vec<Cell>>,     // cells[row_idx][col_idx]

    // Configuration
    title: Option<Text>,
    caption: Option<Text>,
    width: Option<usize>,       // Fixed table width
    min_width: Option<usize>,
    box_style: Box,             // Box drawing style
    safe_box: Option<bool>,     // Force ASCII boxes
    padding: (usize, usize),    // (horizontal, vertical) cell padding
    collapse_padding: bool,     // Remove padding between cells
    pad_edge: bool,             // Pad outer edges
    expand: bool,               // Expand to fill console width
    show_header: bool,
    show_footer: bool,
    show_edge: bool,            // Show left/right edges
    show_lines: bool,           // Show lines between rows
    leading: usize,             // Extra lines between rows
    style: Style,               // Table-level style
    row_styles: Vec<Style>,     // Alternating row styles
    header_style: Style,
    footer_style: Style,
    border_style: Style,
    title_style: Style,
    caption_style: Style,
    title_justify: JustifyMethod,
    caption_justify: JustifyMethod,
    highlight: bool,
}
```

### 9.2 Column Width Calculation (CRITICAL ALGORITHM)

This is the most complex algorithm in Rich. It determines how to distribute available width among columns.

```rust
fn calculate_column_widths(&self, console: &Console, max_width: usize) -> Vec<usize> {
    // Step 1: Get measurement for each column
    let measurements: Vec<Measurement> = self.columns.iter()
        .map(|col| self.measure_column(console, col, max_width))
        .collect();

    // Step 2: Calculate space needed for borders and padding
    let border_width = if self.show_edge { 2 } else { 0 };
    let padding_width = self.padding.0 * 2 * self.columns.len();
    let separator_width = if self.collapse_padding {
        self.columns.len() - 1
    } else {
        (self.columns.len() - 1) * (1 + self.padding.0 * 2)
    };

    let overhead = border_width + padding_width + separator_width;
    let available = max_width.saturating_sub(overhead);

    // Step 3: Get initial widths from measurements
    let mut widths: Vec<usize> = measurements.iter()
        .map(|m| m.maximum)
        .collect();

    // Step 4: Apply fixed widths
    for (i, col) in self.columns.iter().enumerate() {
        if let Some(fixed) = col.width {
            widths[i] = fixed;
        }
    }

    // Step 5: If total exceeds available, collapse
    let total: usize = widths.iter().sum();
    if total > available {
        widths = self.collapse_widths(
            &widths,
            &measurements.iter().map(|m| m.minimum).collect::<Vec<_>>(),
            available,
        );
    }

    // Step 6: If expand=true and total < available, expand ratio columns
    if self.expand {
        let total: usize = widths.iter().sum();
        if total < available {
            widths = self.expand_widths(&widths, available);
        }
    }

    widths
}
```

### 9.3 Column Collapse Algorithm

When total width exceeds available space, shrink columns proportionally:

```rust
fn collapse_widths(
    &self,
    widths: &[usize],
    minimums: &[usize],
    available: usize,
) -> Vec<usize> {
    let mut result = widths.to_vec();
    let total: usize = result.iter().sum();
    let mut excess = total.saturating_sub(available);

    // Calculate how much each column can shrink
    let shrinkable: Vec<usize> = result.iter()
        .zip(minimums.iter())
        .map(|(w, m)| w.saturating_sub(*m))
        .collect();

    let total_shrinkable: usize = shrinkable.iter().sum();
    if total_shrinkable == 0 {
        return result;
    }

    // Shrink proportionally
    for (i, shrink) in shrinkable.iter().enumerate() {
        if *shrink > 0 {
            let reduction = (*shrink * excess) / total_shrinkable;
            result[i] = result[i].saturating_sub(reduction);
        }
    }

    // Handle rounding errors
    let new_total: usize = result.iter().sum();
    if new_total > available {
        let diff = new_total - available;
        // Remove from largest shrinkable column
        for i in (0..result.len()).rev() {
            if result[i] > minimums[i] {
                let can_remove = (result[i] - minimums[i]).min(diff);
                result[i] -= can_remove;
                if result.iter().sum::<usize>() <= available {
                    break;
                }
            }
        }
    }

    result
}
```

### 9.4 Column Measurement

```rust
fn measure_column(&self, console: &Console, column: &Column, max_width: usize) -> Measurement {
    let mut cells_to_measure: Vec<&dyn Renderable> = Vec::new();

    // Include header if shown
    if self.show_header && !column.header.is_empty() {
        cells_to_measure.push(&column.header);
    }

    // Include all data cells
    for cell in &column._cells {
        cells_to_measure.push(&*cell.renderable);
    }

    // Include footer if shown
    if self.show_footer && !column.footer.is_empty() {
        cells_to_measure.push(&column.footer);
    }

    // Measure all cells
    let options = ConsoleOptions {
        max_width,
        ..console.options()
    };

    let measurement = measure_renderables(console, &options, &cells_to_measure);

    // Apply column constraints
    measurement
        .clamp(column.min_width, column.max_width)
        .with_maximum(max_width)
}
```

### 9.5 Table Rendering

```rust
impl Renderable for Table {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<RenderItem> {
        let max_width = options.max_width;

        // Calculate column widths
        let widths = self.calculate_column_widths(console, max_width);

        let mut segments = Vec::new();

        // Render title
        if let Some(title) = &self.title {
            segments.extend(self.render_title(console, title, &widths));
        }

        // Top border
        if self.show_edge {
            let top_line = self.box_style.get_row(&widths, RowLevel::Top, true);
            segments.push(Segment::new(&top_line, Some(self.border_style.clone())));
            segments.push(Segment::line());
        }

        // Header
        if self.show_header && !self.columns.is_empty() {
            let header_cells: Vec<_> = self.columns.iter()
                .map(|c| &c.header)
                .collect();
            segments.extend(self.render_row(console, &header_cells, &widths, &self.header_style));

            // Header separator
            let head_sep = self.box_style.get_row(&widths, RowLevel::Head, self.show_edge);
            segments.push(Segment::new(&head_sep, Some(self.border_style.clone())));
            segments.push(Segment::line());
        }

        // Data rows
        for (row_idx, row) in self.rows.iter().enumerate() {
            let row_cells: Vec<_> = self.cells[row_idx].iter()
                .map(|c| &*c.renderable)
                .collect();

            // Get row style (may alternate)
            let row_style = if !self.row_styles.is_empty() {
                &self.row_styles[row_idx % self.row_styles.len()]
            } else {
                &row.style
            };

            segments.extend(self.render_row(console, &row_cells, &widths, row_style));

            // Row separator (if show_lines or end_section)
            if self.show_lines || row.end_section {
                let sep = self.box_style.get_row(&widths, RowLevel::Row, self.show_edge);
                segments.push(Segment::new(&sep, Some(self.border_style.clone())));
                segments.push(Segment::line());
            }
        }

        // Footer
        if self.show_footer && !self.columns.is_empty() {
            // Footer separator
            let foot_sep = self.box_style.get_row(&widths, RowLevel::Foot, self.show_edge);
            segments.push(Segment::new(&foot_sep, Some(self.border_style.clone())));
            segments.push(Segment::line());

            let footer_cells: Vec<_> = self.columns.iter()
                .map(|c| &c.footer)
                .collect();
            segments.extend(self.render_row(console, &footer_cells, &widths, &self.footer_style));
        }

        // Bottom border
        if self.show_edge {
            let bottom_line = self.box_style.get_row(&widths, RowLevel::Bottom, true);
            segments.push(Segment::new(&bottom_line, Some(self.border_style.clone())));
            segments.push(Segment::line());
        }

        // Caption
        if let Some(caption) = &self.caption {
            segments.extend(self.render_caption(console, caption, &widths));
        }

        segments.into_iter().map(RenderItem::Segment).collect()
    }
}
```

### 9.6 Row Rendering (Vertical Alignment)

```rust
fn render_row(
    &self,
    console: &Console,
    cells: &[&dyn Renderable],
    widths: &[usize],
    row_style: &Style,
) -> Vec<Segment> {
    // Render each cell to lines
    let mut cell_lines: Vec<Vec<Vec<Segment>>> = Vec::new();
    let mut max_height = 0;

    for (i, (cell, &width)) in cells.iter().zip(widths.iter()).enumerate() {
        let col = &self.columns[i];
        let cell_options = ConsoleOptions {
            max_width: width,
            justify: Some(col.justify),
            overflow: Some(col.overflow),
            no_wrap: Some(col.no_wrap),
            ..console.options()
        };

        let lines = console.render_lines(*cell, &cell_options, Some(&col.style), true, false);
        max_height = max_height.max(lines.len());
        cell_lines.push(lines);
    }

    // Apply vertical alignment to each cell
    for (i, lines) in cell_lines.iter_mut().enumerate() {
        let col = &self.columns[i];
        let width = widths[i];

        *lines = match col.vertical {
            VerticalAlignMethod::Top => {
                Segment::align_top(std::mem::take(lines), width, max_height, col.style.clone())
            }
            VerticalAlignMethod::Middle => {
                Segment::align_middle(std::mem::take(lines), width, max_height, col.style.clone())
            }
            VerticalAlignMethod::Bottom => {
                Segment::align_bottom(std::mem::take(lines), width, max_height, col.style.clone())
            }
        };
    }

    // Combine cells into row output
    let mut result = Vec::new();
    let (h_pad, v_pad) = self.padding;
    let pad_str = " ".repeat(h_pad);

    for line_idx in 0..max_height {
        // Left edge
        if self.show_edge {
            result.push(Segment::new(&self.box_style.head[0].to_string(), Some(self.border_style.clone())));
        }
        if self.pad_edge {
            result.push(Segment::new(&pad_str, Some(row_style.clone())));
        }

        // Cells
        for (col_idx, cell) in cell_lines.iter().enumerate() {
            result.extend(cell[line_idx].clone());

            // Cell separator
            if col_idx < cell_lines.len() - 1 {
                if self.pad_edge || !self.collapse_padding {
                    result.push(Segment::new(&pad_str, Some(row_style.clone())));
                }
                result.push(Segment::new(&self.box_style.head[2].to_string(), Some(self.border_style.clone())));
                if self.pad_edge || !self.collapse_padding {
                    result.push(Segment::new(&pad_str, Some(row_style.clone())));
                }
            }
        }

        // Right edge
        if self.pad_edge {
            result.push(Segment::new(&pad_str, Some(row_style.clone())));
        }
        if self.show_edge {
            result.push(Segment::new(&self.box_style.head[3].to_string(), Some(self.border_style.clone())));
        }

        result.push(Segment::line());
    }

    result
}
```

---

## 10. Panel & Padding

> Source: `rich/panel.py` (317 lines), `rich/padding.py` (141 lines)

### 10.1 Padding Data Structure

```rust
/// CSS-style padding values
struct PaddingDimensions {
    top: usize,
    right: usize,
    bottom: usize,
    left: usize,
}

impl PaddingDimensions {
    /// Parse CSS-style padding specification
    /// 1 value:  (all,)        -> all sides equal
    /// 2 values: (vert, horiz) -> top/bottom, left/right
    /// 4 values: (top, right, bottom, left) -> individual sides
    fn unpack(pad: impl Into<PaddingInput>) -> Self {
        match pad.into() {
            PaddingInput::Single(n) =>
                PaddingDimensions { top: n, right: n, bottom: n, left: n },
            PaddingInput::Two(v, h) =>
                PaddingDimensions { top: v, right: h, bottom: v, left: h },
            PaddingInput::Four(t, r, b, l) =>
                PaddingDimensions { top: t, right: r, bottom: b, left: l },
        }
    }
}
```

### 10.2 Padding Renderable

```rust
struct Padding {
    renderable: Box<dyn Renderable>,
    pad: PaddingDimensions,
    style: Style,
    expand: bool,
}

impl Padding {
    /// Create indented padding (left indent only)
    fn indent(renderable: impl Renderable, level: usize) -> Self {
        Padding {
            renderable: Box::new(renderable),
            pad: PaddingDimensions { top: 0, right: 0, bottom: 0, left: level },
            style: Style::null(),
            expand: true,
        }
    }
}

impl Renderable for Padding {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<RenderItem> {
        let mut segments = Vec::new();
        let width = options.max_width;

        // Calculate inner width
        let inner_width = width
            .saturating_sub(self.pad.left)
            .saturating_sub(self.pad.right);

        // Create inner options
        let inner_options = options.update_width(inner_width);

        // Create padding strings
        let left_pad = " ".repeat(self.pad.left);
        let right_pad = " ".repeat(self.pad.right);
        let blank_line = " ".repeat(width);

        // Top padding
        for _ in 0..self.pad.top {
            segments.push(Segment::new(&blank_line, Some(self.style.clone())));
            segments.push(Segment::line());
        }

        // Render inner content
        let inner_lines = console.render_lines(
            &*self.renderable,
            &inner_options,
            Some(&self.style),
            self.expand,
            false,
        );

        for line in inner_lines {
            // Left padding
            segments.push(Segment::new(&left_pad, Some(self.style.clone())));

            // Content
            segments.extend(line);

            // Right padding
            segments.push(Segment::new(&right_pad, Some(self.style.clone())));
            segments.push(Segment::line());
        }

        // Bottom padding
        for _ in 0..self.pad.bottom {
            segments.push(Segment::new(&blank_line, Some(self.style.clone())));
            segments.push(Segment::line());
        }

        segments.into_iter().map(RenderItem::Segment).collect()
    }
}
```

### 10.3 Panel Data Structure

```rust
struct Panel {
    renderable: Box<dyn Renderable>,
    box_style: Box,
    safe_box: Option<bool>,
    expand: bool,
    style: Style,
    border_style: Style,
    width: Option<usize>,
    height: Option<usize>,
    padding: PaddingDimensions,
    highlight: bool,

    // Title/subtitle
    title: Option<Text>,
    title_align: JustifyMethod,
    subtitle: Option<Text>,
    subtitle_align: JustifyMethod,
}

impl Panel {
    /// Create panel that fits content width
    fn fit(
        renderable: impl Renderable,
        box_style: Box,
        padding: impl Into<PaddingInput>,
    ) -> Self {
        Panel {
            renderable: Box::new(renderable),
            box_style,
            padding: PaddingDimensions::unpack(padding),
            expand: false,  // Key difference: don't expand
            ..Default::default()
        }
    }

    /// Process title text
    fn make_title(&self, text: &Text, width: usize) -> Text {
        let mut title = text.clone();
        title.truncate(width.saturating_sub(4), OverflowMethod::Ellipsis, false);
        title.plain = format!(" {} ", title.plain);  // Add surrounding spaces
        title
    }
}
```

### 10.4 Panel Rendering

```rust
impl Renderable for Panel {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<RenderItem> {
        let safe_box = self.safe_box.unwrap_or(console.safe_box);
        let box_style = if safe_box {
            self.box_style.substitute_ascii()
        } else {
            self.box_style.clone()
        };

        // Calculate dimensions
        let width = if self.expand {
            options.max_width
        } else if let Some(w) = self.width {
            w
        } else {
            // Measure content
            let inner_options = options.update_width(options.max_width.saturating_sub(4)); // 2 borders + 2 min padding
            let measurement = Measurement::get(console, &inner_options, &*self.renderable);
            measurement.maximum + 4
        };

        let inner_width = width.saturating_sub(2); // Minus border characters
        let content_width = inner_width
            .saturating_sub(self.padding.left)
            .saturating_sub(self.padding.right);

        // Render content
        let content_options = options.update_dimensions(content_width, self.height.unwrap_or(usize::MAX));
        let content_lines = console.render_lines(
            &*self.renderable,
            &content_options,
            None,
            true,
            false,
        );

        let mut segments = Vec::new();

        // Top border with optional title
        let top_border = box_style.get_row(&[inner_width], RowLevel::Top, true);
        if let Some(title) = &self.title {
            let title_text = self.make_title(title, inner_width);
            let title_segments = title_text.render(console, "");

            // Insert title into top border at appropriate position
            segments.extend(self.insert_title_into_border(&top_border, &title_segments, self.title_align, &self.border_style));
        } else {
            segments.push(Segment::new(&top_border, Some(self.border_style.clone())));
        }
        segments.push(Segment::line());

        // Content lines with borders
        let left_pad = " ".repeat(self.padding.left);
        let right_pad = " ".repeat(self.padding.right);

        // Top inner padding
        for _ in 0..self.padding.top {
            segments.push(Segment::new(&box_style.head[0].to_string(), Some(self.border_style.clone())));
            segments.push(Segment::new(&" ".repeat(inner_width), Some(self.style.clone())));
            segments.push(Segment::new(&box_style.head[3].to_string(), Some(self.border_style.clone())));
            segments.push(Segment::line());
        }

        // Content
        for line in content_lines {
            segments.push(Segment::new(&box_style.head[0].to_string(), Some(self.border_style.clone())));
            segments.push(Segment::new(&left_pad, Some(self.style.clone())));
            segments.extend(line);
            segments.push(Segment::new(&right_pad, Some(self.style.clone())));
            segments.push(Segment::new(&box_style.head[3].to_string(), Some(self.border_style.clone())));
            segments.push(Segment::line());
        }

        // Bottom inner padding
        for _ in 0..self.padding.bottom {
            segments.push(Segment::new(&box_style.head[0].to_string(), Some(self.border_style.clone())));
            segments.push(Segment::new(&" ".repeat(inner_width), Some(self.style.clone())));
            segments.push(Segment::new(&box_style.head[3].to_string(), Some(self.border_style.clone())));
            segments.push(Segment::line());
        }

        // Bottom border with optional subtitle
        let bottom_border = box_style.get_row(&[inner_width], RowLevel::Bottom, true);
        if let Some(subtitle) = &self.subtitle {
            let subtitle_text = self.make_title(subtitle, inner_width);
            let subtitle_segments = subtitle_text.render(console, "");
            segments.extend(self.insert_title_into_border(&bottom_border, &subtitle_segments, self.subtitle_align, &self.border_style));
        } else {
            segments.push(Segment::new(&bottom_border, Some(self.border_style.clone())));
        }
        segments.push(Segment::line());

        segments.into_iter().map(RenderItem::Segment).collect()
    }
}
```

---

## 11. Alignment System

> Source: `rich/align.py` (307 lines)

### 11.1 Alignment Types

```rust
/// Horizontal alignment methods
enum AlignMethod {
    Left,
    Center,
    Right,
}

/// Vertical alignment methods
enum VerticalAlignMethod {
    Top,
    Middle,
    Bottom,
}
```

### 11.2 Align Renderable

```rust
struct Align {
    renderable: Box<dyn Renderable>,
    align: AlignMethod,           // Horizontal alignment
    style: Style,                 // Background/fill style
    vertical: VerticalAlignMethod,
    pad: bool,                    // Pad lines to width
    width: Option<usize>,         // Override width
    height: Option<usize>,        // Override height
}

impl Align {
    fn left(renderable: impl Renderable) -> Self {
        Align { align: AlignMethod::Left, ..Self::new(renderable) }
    }

    fn center(renderable: impl Renderable) -> Self {
        Align { align: AlignMethod::Center, ..Self::new(renderable) }
    }

    fn right(renderable: impl Renderable) -> Self {
        Align { align: AlignMethod::Right, ..Self::new(renderable) }
    }
}
```

### 11.3 Alignment Rendering

```rust
impl Renderable for Align {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<RenderItem> {
        let width = self.width.unwrap_or(options.max_width);
        let height = self.height;

        // Render inner content
        let inner_options = options.update_width(width);
        let lines = console.render_lines(
            &*self.renderable,
            &inner_options,
            None,
            false,
            false,
        );

        let mut result_lines = Vec::new();

        for mut line in lines {
            let line_width: usize = line.iter().map(|s| s.cell_length()).sum();
            let excess = width.saturating_sub(line_width);

            match self.align {
                AlignMethod::Left => {
                    // Content on left, padding on right
                    if self.pad && excess > 0 {
                        line.push(Segment::new(&" ".repeat(excess), Some(self.style.clone())));
                    }
                }
                AlignMethod::Center => {
                    // Split padding between left and right
                    let left_pad = excess / 2;
                    let right_pad = excess - left_pad;
                    let mut new_line = Vec::new();
                    if left_pad > 0 {
                        new_line.push(Segment::new(&" ".repeat(left_pad), Some(self.style.clone())));
                    }
                    new_line.extend(line);
                    if self.pad && right_pad > 0 {
                        new_line.push(Segment::new(&" ".repeat(right_pad), Some(self.style.clone())));
                    }
                    line = new_line;
                }
                AlignMethod::Right => {
                    // Padding on left, content on right
                    let mut new_line = Vec::new();
                    if excess > 0 {
                        new_line.push(Segment::new(&" ".repeat(excess), Some(self.style.clone())));
                    }
                    new_line.extend(line);
                    line = new_line;
                }
            }

            result_lines.push(line);
        }

        // Apply vertical alignment if height specified
        if let Some(h) = height {
            if result_lines.len() < h {
                result_lines = match self.vertical {
                    VerticalAlignMethod::Top => {
                        Segment::align_top(result_lines, width, h, self.style.clone())
                    }
                    VerticalAlignMethod::Middle => {
                        Segment::align_middle(result_lines, width, h, self.style.clone())
                    }
                    VerticalAlignMethod::Bottom => {
                        Segment::align_bottom(result_lines, width, h, self.style.clone())
                    }
                };
            }
        }

        // Convert to segments with newlines
        let mut segments = Vec::new();
        for line in result_lines {
            segments.extend(line);
            segments.push(Segment::line());
        }

        segments.into_iter().map(RenderItem::Segment).collect()
    }
}
```

---

## 12. Unicode Cell Width

> Source: `rich/cells.py` (175 lines), `rich/_cell_widths.py` (454 entries)

### 12.1 Cell Width Concept

Terminal cells have fixed width. Most characters occupy 1 cell, but some (CJK, emoji) occupy 2 cells. Rich must calculate cell width accurately for layout.

### 12.2 Cell Width Table

The `CELL_WIDTHS` table contains 454 entries of (start, end, width) tuples that define Unicode ranges with non-standard width:

```rust
/// Cell width lookup table
/// Each entry: (codepoint_start, codepoint_end, cell_width)
const CELL_WIDTHS: &[(u32, u32, usize)] = &[
    (0, 0, 0),           // NULL
    (1, 31, -1),         // C0 control (ignored)
    (127, 159, -1),      // C1 control (ignored)
    (768, 879, 0),       // Combining diacritical marks
    (1155, 1161, 0),     // Combining Cyrillic
    // ... 450+ more entries
    (4352, 4447, 2),     // Hangul Jamo
    (8986, 8987, 2),     // Watch, Hourglass
    (9193, 9203, 2),     // Various symbols
    (9725, 9726, 2),     // Medium squares
    // ... CJK ranges
    (12288, 12288, 2),   // Ideographic space
    (12289, 12350, 2),   // CJK punctuation
    (19968, 40956, 2),   // CJK Unified Ideographs
    // ... Emoji ranges
    (127744, 128591, 2), // Misc symbols/pictographs
    (128640, 128767, 2), // Transport/map symbols
    (129280, 129535, 2), // More emoji
];
```

### 12.3 Fast-Path Detection

For efficiency, single-cell ASCII is detected without table lookup:

```rust
/// Ranges known to be single-cell width
const SINGLE_CELL_RANGES: &[(u32, u32)] = &[
    (0x20, 0x7E),      // Basic ASCII printable
    (0xA0, 0x02FF),    // Latin Extended + IPA
    (0x0370, 0x0482),  // Greek
    // ... more known single-cell ranges
];

fn is_single_cell_fast(c: char) -> bool {
    let cp = c as u32;
    SINGLE_CELL_RANGES.iter().any(|(start, end)| cp >= *start && cp <= *end)
}
```

### 12.4 Cell Width Algorithm

```rust
/// Get cell width of a single character
fn get_character_cell_size(c: char) -> isize {
    let codepoint = c as u32;

    // Binary search in CELL_WIDTHS table
    let idx = CELL_WIDTHS.partition_point(|(start, _, _)| *start <= codepoint);

    if idx > 0 {
        let (start, end, width) = CELL_WIDTHS[idx - 1];
        if codepoint >= start && codepoint <= end {
            return width as isize;
        }
    }

    // Default: 1 cell
    1
}

/// Get total cell width of a string (cached)
fn cell_len(text: &str) -> usize {
    // Use thread-local cache
    CELL_LEN_CACHE.with(|cache| {
        if let Some(&cached) = cache.borrow().get(text) {
            return cached;
        }

        let width: usize = text.chars()
            .map(|c| get_character_cell_size(c).max(0) as usize)
            .sum();

        cache.borrow_mut().insert(text.to_string(), width);
        width
    })
}
```

### 12.5 Cell-Based String Operations

```rust
/// Truncate string to fit within cell width
fn set_cell_size(text: &str, total: usize) -> String {
    let current = cell_len(text);
    if current == total {
        return text.to_string();
    }
    if current < total {
        // Pad with spaces
        return format!("{}{}", text, " ".repeat(total - current));
    }

    // Need to truncate - use binary search
    let chars: Vec<char> = text.chars().collect();
    let mut pos = 0;
    let mut width = 0;

    // Find position where we exceed target
    while pos < chars.len() {
        let char_width = get_character_cell_size(chars[pos]).max(0) as usize;
        if width + char_width > total {
            break;
        }
        width += char_width;
        pos += 1;
    }

    let truncated: String = chars[..pos].iter().collect();

    // Pad if needed (due to wide character not fitting)
    if width < total {
        format!("{}{}", truncated, " ".repeat(total - width))
    } else {
        truncated
    }
}

/// Split string at cell position
fn chop_cells(text: &str, max_size: usize) -> (&str, &str) {
    let mut width = 0;
    let mut byte_pos = 0;

    for (i, c) in text.char_indices() {
        let char_width = get_character_cell_size(c).max(0) as usize;
        if width + char_width > max_size {
            break;
        }
        width += char_width;
        byte_pos = i + c.len_utf8();
    }

    (&text[..byte_pos], &text[byte_pos..])
}
```

---

## 13. Text Wrapping

> Source: `rich/_wrap.py` (94 lines)

### 13.1 Word Tokenizer

```rust
/// Regex pattern for word extraction
/// Matches: optional leading whitespace + non-whitespace + optional trailing whitespace
const RE_WORD: &str = r"\s*\S+\s*";

/// Split text into words (preserving whitespace)
fn words(text: &str) -> Vec<&str> {
    let re = Regex::new(RE_WORD).unwrap();
    re.find_iter(text).map(|m| m.as_str()).collect()
}
```

### 13.2 Line Division Algorithm

```rust
/// Divide a single line of text at specified width
/// Returns: (line_content, remaining_text, has_more)
fn divide_line(text: &str, width: usize, fold: bool) -> Vec<(usize, usize)> {
    let mut breaks = Vec::new();
    let mut line_start = 0;
    let mut line_width = 0;

    for word in words(text) {
        let word_start = word.as_ptr() as usize - text.as_ptr() as usize;
        let word_width = cell_len(word.trim_end());  // Don't count trailing space

        if line_width > 0 && line_width + word_width > width {
            // Word doesn't fit, break here
            breaks.push((line_start, word_start));
            line_start = word_start;
            line_width = 0;
        }

        if fold && word_width > width {
            // Word itself is too wide, must fold within word
            let mut remaining = word;
            while cell_len(remaining) > width {
                let (chunk, rest) = chop_cells(remaining, width);
                let chunk_end = line_start + (chunk.as_ptr() as usize - text.as_ptr() as usize) + chunk.len();
                breaks.push((line_start, chunk_end));
                line_start = chunk_end;
                remaining = rest;
            }
            line_width = cell_len(remaining);
        } else {
            line_width += word_width;
        }
    }

    // Final segment
    if line_start < text.len() {
        breaks.push((line_start, text.len()));
    }

    breaks
}
```

### 13.3 Full Text Wrapping

```rust
/// Wrap text to fit within width
fn wrap_text(text: &str, width: usize, fold: bool) -> Vec<String> {
    let mut lines = Vec::new();

    // Process each existing line
    for line in text.split('\n') {
        if line.is_empty() {
            lines.push(String::new());
            continue;
        }

        let breaks = divide_line(line, width, fold);
        for (start, end) in breaks {
            let segment = &line[start..end];
            // Trim trailing whitespace from wrapped lines
            lines.push(segment.trim_end().to_string());
        }
    }

    lines
}
```

---

## 14. Ratio Distribution

> Source: `rich/_ratio.py` (154 lines)

### 14.1 Edge Protocol

```rust
/// Trait for items that participate in ratio-based distribution
trait Edge {
    fn size(&self) -> Option<usize>;       // Fixed size (None = flexible)
    fn ratio(&self) -> usize;              // Ratio weight (default 1)
    fn minimum_size(&self) -> usize;       // Minimum allowed size (default 1)
}
```

### 14.2 Ratio Resolution Algorithm

This algorithm distributes a total amount among edges based on their ratios:

```rust
use num_rational::Ratio;  // For exact fraction arithmetic

/// Resolve sizes for edges with no fixed size
fn ratio_resolve(total: usize, edges: &[impl Edge]) -> Vec<usize> {
    // Separate fixed and flexible edges
    let mut sizes = vec![0usize; edges.len()];
    let mut flexible_indices = Vec::new();
    let mut fixed_total = 0;
    let mut total_ratio = 0;

    for (i, edge) in edges.iter().enumerate() {
        if let Some(size) = edge.size() {
            sizes[i] = size;
            fixed_total += size;
        } else {
            flexible_indices.push(i);
            total_ratio += edge.ratio();
        }
    }

    // Calculate remaining space for flexible edges
    let remaining = total.saturating_sub(fixed_total);

    if total_ratio == 0 || remaining == 0 {
        // No flexible edges or no space
        for i in flexible_indices {
            sizes[i] = edges[i].minimum_size();
        }
        return sizes;
    }

    // Distribute using exact fractions to avoid rounding errors
    let mut distributed = 0;
    for (idx, &i) in flexible_indices.iter().enumerate() {
        let ratio = Ratio::new(edges[i].ratio(), total_ratio);
        let ideal = ratio * remaining;

        // Round (using nearest integer)
        let size = if idx == flexible_indices.len() - 1 {
            // Last flexible edge gets remainder (avoids accumulation error)
            remaining - distributed
        } else {
            ideal.round().to_integer()
        };

        sizes[i] = size.max(edges[i].minimum_size());
        distributed += sizes[i];
    }

    sizes
}
```

### 14.3 Ratio Reduction Algorithm

When total required exceeds available, reduce proportionally:

```rust
/// Reduce sizes proportionally to fit within total
fn ratio_reduce(
    total: usize,
    ratios: &[usize],
    maximums: &[usize],
    values: &[usize],
) -> Vec<usize> {
    let current_total: usize = values.iter().sum();
    if current_total <= total {
        return values.to_vec();
    }

    let excess = current_total - total;

    // Calculate how much each can shrink (value - 1, weighted by ratio)
    let shrinkable: Vec<usize> = values.iter()
        .zip(ratios.iter())
        .map(|(&v, &r)| (v.saturating_sub(1)) * r)
        .collect();

    let total_shrinkable: usize = shrinkable.iter().sum();
    if total_shrinkable == 0 {
        return values.to_vec();
    }

    // Reduce proportionally
    let mut result = values.to_vec();
    let mut reduced = 0;

    for i in 0..values.len() {
        if shrinkable[i] > 0 {
            let share = Ratio::new(shrinkable[i], total_shrinkable);
            let reduction = (share * excess).round().to_integer().min(values[i] - 1);
            result[i] = values[i] - reduction;
            reduced += reduction;
        }
    }

    // Handle rounding errors by reducing largest values
    while result.iter().sum::<usize>() > total {
        // Find largest value that can still be reduced
        if let Some(i) = result.iter().enumerate()
            .filter(|(_, &v)| v > 1)
            .max_by_key(|(_, &v)| v)
            .map(|(i, _)| i)
        {
            result[i] -= 1;
        } else {
            break;
        }
    }

    result
}
```

### 14.4 Ratio Distribution Algorithm

Distribute extra space among ratio-enabled edges:

```rust
/// Distribute remaining space among edges based on ratio
fn ratio_distribute(
    total: usize,
    edges: &[impl Edge],
    minimums: &[usize],
) -> Vec<usize> {
    let mut sizes = minimums.to_vec();
    let current: usize = sizes.iter().sum();

    if current >= total {
        return sizes;
    }

    let remaining = total - current;

    // Get ratio for flexible edges (ratio > 0)
    let ratios: Vec<usize> = edges.iter()
        .zip(sizes.iter())
        .map(|(e, &s)| if e.ratio() > 0 && s < total { e.ratio() } else { 0 })
        .collect();

    let total_ratio: usize = ratios.iter().sum();
    if total_ratio == 0 {
        return sizes;
    }

    // Distribute using fractions
    let mut distributed = 0;
    let flexible_count = ratios.iter().filter(|&&r| r > 0).count();
    let mut flex_idx = 0;

    for (i, &ratio) in ratios.iter().enumerate() {
        if ratio > 0 {
            flex_idx += 1;
            let share = Ratio::new(ratio, total_ratio);
            let extra = if flex_idx == flexible_count {
                remaining - distributed
            } else {
                (share * remaining).round().to_integer()
            };
            sizes[i] += extra;
            distributed += extra;
        }
    }

    sizes
}
```

---

## 15. Exclusions

The following are explicitly **NOT** being ported:

| Feature | Reason |
|---------|--------|
| Jupyter/IPython integration | Python-specific |
| `inspect` module | Python runtime reflection |
| `traceback` module | Python exception handling |
| `logging` handler | Python stdlib integration |
| Legacy Windows (cmd.exe) | Use modern VT sequences via crossterm |
| Emoji code database (`:smile:`) | Use native unicode |
| Theme .ini files | Code-defined themes only |
| Progress bars | Separate concern, Phase 2+ |
| Live display | Separate concern, Phase 2+ |
| Syntax highlighting | Use syntect, Phase 2+ |
| Markdown rendering | Use pulldown-cmark, Phase 2+ |
| Pretty-printing Python objects | Python-specific |
| Console file export (HTML/SVG) | Phase 2+ |

---

## Appendix A: Rust Trait Summary

```rust
/// Primary rendering trait (equivalent to __rich_console__)
trait Renderable {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<RenderItem>;
}

/// Conversion trait (equivalent to __rich__)
trait RichDisplay {
    fn to_rich(&self) -> Box<dyn Renderable>;
}

/// Measurement trait (equivalent to __rich_measure__)
trait Measurable {
    fn rich_measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement;
}

/// Combined trait for renderables that support measurement
trait MeasurableRenderable: Renderable + Measurable {}
```

---

## Appendix B: Recommended Crate Mappings

| Python | Rust Crate | Purpose |
|--------|------------|---------|
| `colorsys` | `palette` | Color conversion (RGB/HLS) |
| `wcwidth` | `unicode-width` | Character cell width |
| `re` | `regex` | Regular expressions |
| `sys.stdout` | `crossterm` | Terminal detection/manipulation |
| `functools.lru_cache` | `lru` or `cached` | Memoization |
| `dataclasses` | Native structs | Data modeling |
| `typing` | Native types | Type annotations |
| `enum.IntEnum` | `num_enum` | Integer enums |
| `fractions.Fraction` | `num-rational` | Exact ratio arithmetic |

---

*Specification extracted from Python Rich v13.x source code, 2026-01-17*
