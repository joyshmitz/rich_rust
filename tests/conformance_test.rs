//! Conformance test suite for rich_rust.
//!
//! This test file provides triple-duty testing infrastructure:
//! 1. Integration tests - verify correct behavior
//! 2. Conformance tests - compare against Python Rich expectations
//! 3. Performance baseline - reusable in benchmarks
//!
//! # Running Tests
//!
//! ```bash
//! # Run all conformance tests
//! cargo test --test conformance_test
//!
//! # Run with output
//! cargo test --test conformance_test -- --nocapture
//! ```

mod conformance;

use conformance::text_tests;
use conformance::{run_test, TestCase};

// =============================================================================
// Text Conformance Tests
// =============================================================================

#[test]
fn conformance_text_plain() {
    let test = text_tests::MarkupTextTest {
        name: "plain_text",
        markup: "Hello, World!",
        width: 80,
    };
    let output = run_test(&test);
    assert_eq!(output, "Hello, World!");
}

#[test]
fn conformance_text_bold() {
    let test = text_tests::MarkupTextTest {
        name: "bold_text",
        markup: "[bold]Bold text[/]",
        width: 80,
    };
    let output = run_test(&test);
    assert_eq!(conformance::strip_ansi(&output), "Bold text");
}

#[test]
fn conformance_text_colors() {
    let test = text_tests::MarkupTextTest {
        name: "colored_text",
        markup: "[red]Red[/] and [green]Green[/]",
        width: 80,
    };
    let output = run_test(&test);
    assert_eq!(conformance::strip_ansi(&output), "Red and Green");
}

#[test]
fn conformance_text_nested_styles() {
    let test = text_tests::MarkupTextTest {
        name: "nested_styles",
        markup: "[bold]Bold [italic]and italic[/italic] text[/bold]",
        width: 80,
    };
    let output = run_test(&test);
    assert_eq!(
        conformance::strip_ansi(&output),
        "Bold and italic text"
    );
}

// =============================================================================
// All Standard Tests
// =============================================================================

#[test]
fn conformance_all_text_tests() {
    for test in text_tests::standard_text_tests() {
        let test_ref: &dyn TestCase = test.as_ref();
        let output = run_test(test_ref);
        println!("Test '{}': {} chars", test_ref.name(), output.len());
        assert!(
            !output.is_empty(),
            "Test '{}' produced empty output",
            test_ref.name()
        );
    }
}

// =============================================================================
// Python Rich Comparison (Manual Verification)
// =============================================================================

/// Print Python Rich equivalent code for manual verification.
/// Run with: cargo test --test conformance_test print_python_equivalents -- --nocapture --ignored
#[test]
#[ignore]
fn print_python_equivalents() {
    println!("\n=== Python Rich Equivalent Code ===\n");

    for test in text_tests::standard_text_tests() {
        let test_ref: &dyn TestCase = test.as_ref();
        println!("--- {} ---", test_ref.name());
        if let Some(code) = test_ref.python_rich_code() {
            println!("{}\n", code);
        } else {
            println!("(No Python equivalent)\n");
        }
    }
}
