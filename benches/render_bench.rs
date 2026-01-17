//! Benchmarks for rich_rust rendering.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rich_rust::text::Text;
use rich_rust::style::Style;

fn benchmark_text_render(c: &mut Criterion) {
    let mut text = Text::new("Hello, World! This is a test string for benchmarking.");
    text.stylize(0, 5, Style::new().bold());
    text.stylize(7, 12, Style::new().italic());

    c.bench_function("text_render", |b| {
        b.iter(|| {
            black_box(text.render(""))
        });
    });
}

fn benchmark_text_wrap(c: &mut Criterion) {
    let text = Text::new("This is a longer string that needs to be wrapped to fit within a certain width. It contains multiple words and should demonstrate the wrapping algorithm.");

    c.bench_function("text_wrap_80", |b| {
        b.iter(|| {
            black_box(text.wrap(80))
        });
    });

    c.bench_function("text_wrap_40", |b| {
        b.iter(|| {
            black_box(text.wrap(40))
        });
    });
}

fn benchmark_style_parse(c: &mut Criterion) {
    c.bench_function("style_parse_simple", |b| {
        b.iter(|| {
            black_box(Style::parse("bold red"))
        });
    });

    c.bench_function("style_parse_complex", |b| {
        b.iter(|| {
            black_box(Style::parse("bold italic underline red on blue"))
        });
    });
}

criterion_group!(
    benches,
    benchmark_text_render,
    benchmark_text_wrap,
    benchmark_style_parse,
);
criterion_main!(benches);
