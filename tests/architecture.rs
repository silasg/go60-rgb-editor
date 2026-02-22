//! Architecture rules enforced by cargo-pup for the TUI crate.
//!
//! This test uses the cargo_pup_lint_config builder API to generate `pup.ron`.
//! The mise `arch-lint` task runs this first, then invokes `cargo pup` to check.
//!
//! Note: The domain crate has its own architecture rules in crates/domain/tests/.
//! Domain isolation is enforced structurally (zero deps in Cargo.toml) + defense-in-depth
//! via the domain crate's own cargo-pup rules.
//!
//! Workflow:
//!   1. `cargo test --test architecture` → writes `pup.ron`
//!   2. `cargo +nightly-2026-01-22 pup`  → lints the project using `pup.ron`

use cargo_pup_lint_config::{FunctionLintExt, LintBuilder, ModuleLintExt, Severity};

fn build_architecture_rules() -> LintBuilder {
    let mut builder = LintBuilder::new();

    // ── Layer isolation ────────────────────────────────────────────────

    // UI must not perform IO operations directly.
    builder
        .module_lint()
        .lint_named("ui_no_io_access")
        .matching(|m| m.module(".*::ui::.*"))
        .with_severity(Severity::Error)
        .restrict_imports(None, Some(vec![".*::io::.*".to_string()]))
        .build();

    // IO must not depend on UI or presentation crates.
    builder
        .module_lint()
        .lint_named("io_no_ui_dependency")
        .matching(|m| m.module(".*::io::.*"))
        .with_severity(Severity::Error)
        .restrict_imports(
            None,
            Some(vec![
                ".*::ui::.*".to_string(),
                "ratatui::.*".to_string(),
            ]),
        )
        .build();

    // TUI must not import the wasm wrapper crate — only the domain crate.
    builder
        .module_lint()
        .lint_named("tui_no_wasm_dependency")
        .matching(|m| m.module(".*"))
        .with_severity(Severity::Error)
        .restrict_imports(None, Some(vec!["go60_rgb_editor_wasm::.*".to_string()]))
        .build();

    // ── Module hygiene ─────────────────────────────────────────────────

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

    // ── Function hygiene ───────────────────────────────────────────────

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
/// Run via: `cargo test --test architecture`
#[test]
fn generate_pup_config() {
    // Act
    let builder = build_architecture_rules();
    builder
        .write_to_file("pup.ron")
        .expect("Failed to write pup.ron");
}
