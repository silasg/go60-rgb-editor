//! Architecture rules enforced by cargo-pup for the domain crate.
//!
//! The domain crate has zero production dependencies (enforced structurally by Cargo.toml).
//! These rules provide defense-in-depth: even if someone adds a dependency, cargo-pup will
//! catch forbidden imports.
//!
//! Workflow:
//!   1. `cargo test -p go60-rgb-editor-domain --test architecture` → writes `pup.ron`
//!   2. `cd crates/domain && cargo +nightly-2026-01-22 pup` → lints using `pup.ron`

use cargo_pup_lint_config::{FunctionLintExt, LintBuilder, ModuleLintExt, Severity};

fn build_architecture_rules() -> LintBuilder {
    let mut builder = LintBuilder::new();

    // ── Domain isolation (defense-in-depth) ──────────────────────────

    // Domain must not import any external crate. Primary enforcement is
    // Cargo.toml having no [dependencies]; this catches it at the import level.
    builder
        .module_lint()
        .lint_named("domain_has_no_external_imports")
        .matching(|m| m.module(".*"))
        .with_severity(Severity::Error)
        .restrict_imports(
            None,
            Some(vec![
                "ratatui::.*".to_string(),
                "crossterm::.*".to_string(),
                "color_eyre::.*".to_string(),
                "clap::.*".to_string(),
                "wasm_bindgen::.*".to_string(),
                "serde::.*".to_string(),
                "serde_json::.*".to_string(),
            ]),
        )
        .build();

    // ── Module hygiene ───────────────────────────────────────────────

    // mod.rs files should only contain mod declarations and re-exports.
    builder
        .module_lint()
        .lint_named("clean_mod_files")
        .matching(|m| m.module(".*"))
        .with_severity(Severity::Error)
        .must_have_empty_mod_file()
        .build();

    // Prevent wildcard imports (use something::*).
    builder
        .module_lint()
        .lint_named("no_wildcard_imports")
        .matching(|m| m.module(".*"))
        .with_severity(Severity::Error)
        .no_wildcard_imports()
        .build();

    // ── Function hygiene ─────────────────────────────────────────────

    // Keep functions at a reasonable length.
    builder
        .function_lint()
        .lint_named("function_length_limit")
        .matching(|m| m.name_regex(".*"))
        .with_severity(Severity::Error)
        .max_length(60)
        .build();

    builder
}

/// Generate `pup.ron` from the builder rules.
/// Run via: `cargo test -p go60-rgb-editor-domain --test architecture`
#[test]
fn generate_pup_config() {
    // Act
    let builder = build_architecture_rules();
    builder
        .write_to_file("pup.ron")
        .expect("Failed to write pup.ron");
}
