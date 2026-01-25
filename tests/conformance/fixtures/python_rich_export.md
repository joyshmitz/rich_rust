# Python Rich Export Fixtures (HTML/SVG)

These fixtures capture **Python Rich** export output for manual comparison
against `rich_rust`'s minimal exporters.

Generated on: 2026-01-25  
Source: `legacy_rich` snapshot (repo-local)  
Console config: `width=40`, `color_system=truecolor`, `force_terminal=True`, `record=True`

Input sequence:

```
console.print("Plain")
console.print("[bold red]Error[/]")
console.print("[link=https://example.com]Link[/]")
```

Notes:
- Python Rich SVG output includes a `terminal-<id>` prefix. This may vary by run.
- HTML output is inline-styles mode (`export_html(inline_styles=True)`).

## HTML (inline styles)

```html
<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<style>

body {
    color: #000000;
    background-color: #ffffff;
}
</style>
</head>
<body>
    <pre style="font-family:Menlo,'DejaVu Sans Mono',consolas,'Courier New',monospace"><code style="font-family:inherit">Plain
<span style="color: #800000; text-decoration-color: #800000; font-weight: bold">Error</span>
<a href="https://example.com">Link</a>
</code></pre>
</body>
</html>
```

## SVG

```svg
<svg class="rich-terminal" viewBox="0 0 506 74.4" xmlns="http://www.w3.org/2000/svg">
    <!-- Generated with Rich https://www.textualize.io -->
    <style>

    @font-face {
        font-family: "Fira Code";
        src: local("FiraCode-Regular"),
                url("https://cdnjs.cloudflare.com/ajax/libs/firacode/6.2.0/woff2/FiraCode-Regular.woff2") format("woff2"),
                url("https://cdnjs.cloudflare.com/ajax/libs/firacode/6.2.0/woff/FiraCode-Regular.woff") format("woff");
        font-style: normal;
        font-weight: 400;
    }
    @font-face {
        font-family: "Fira Code";
        src: local("FiraCode-Bold"),
                url("https://cdnjs.cloudflare.com/ajax/libs/firacode/6.2.0/woff2/FiraCode-Bold.woff2") format("woff2"),
                url("https://cdnjs.cloudflare.com/ajax/libs/firacode/6.2.0/woff/FiraCode-Bold.woff") format("woff");
        font-style: bold;
        font-weight: 700;
    }

    .terminal-62194055-matrix {
        font-family: Fira Code, monospace;
        font-size: 20px;
        line-height: 24.4px;
        font-variant-east-asian: full-width;
    }

    .terminal-62194055-title {
        font-size: 18px;
        font-weight: bold;
        font-family: arial;
    }

    
    </style>

    <defs>
    <clipPath id="terminal-62194055-clip-terminal">
      <rect x="0" y="0" width="487.0" height="23.4" />
    </clipPath>
    
    </defs>

    <rect fill="#292929" stroke="rgba(255,255,255,0.35)" stroke-width="1" x="1" y="1" width="504" height="72.4" rx="8"/><text class="terminal-62194055-title" fill="#c5c8c6" text-anchor="middle" x="252" y="27">Rich</text>
            <g transform="translate(26,22)">
            <circle cx="0" cy="0" r="7" fill="#ff5f57"/>
            <circle cx="22" cy="0" r="7" fill="#febc2e"/>
            <circle cx="44" cy="0" r="7" fill="#28c840"/>
            </g>
        
    <g transform="translate(9, 41)" clip-path="url(#terminal-62194055-clip-terminal)">
    
    <g class="terminal-62194055-matrix">
    
    </g>
    </g>
</svg>
```
