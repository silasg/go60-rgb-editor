# Implementation Plan: Extract Domain into a Workspace Library Crate for Wasm

## Goal

Extract `src/domain/` into a standalone library crate (`crates/domain/`) with **zero external dependencies**. Add a thin `crates/domain-wasm/` wrapper crate that depends on `wasm-bindgen` and exposes the domain to JavaScript. The TUI application becomes a separate binary crate that depends on the domain library.

## Crate Naming

| Crate | Package name | Rust import |
|---|---|---|
| Domain lib | `go60-rgb-editor-domain` | `go60_rgb_editor_domain` |
| Wasm wrapper | `go60-rgb-editor-wasm` | `go60_rgb_editor_wasm` |
| TUI binary | `go60-rgb-editor-tui` | — (binary, not imported) |

The TUI binary crate is renamed to `go60-rgb-editor-tui` in `Cargo.toml`, but the CLI binary output stays `go60-rgb-editor` via an explicit `[[bin]]` section.

## Resulting Structure

```
go60-rgb-editor/
├── Cargo.toml                  # [workspace] root + TUI binary (go60-rgb-editor-tui)
├── crates/
│   ├── domain/                 # Pure domain lib — zero external deps
│   │   ├── Cargo.toml          # name = "go60-rgb-editor-domain"
│   │   ├── tests/
│   │   │   ├── fixtures/
│   │   │   │   └── sample_config.txt
│   │   │   └── architecture.rs
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── color.rs
│   │       ├── config.rs
│   │       ├── cursor.rs
│   │       ├── editor.rs
│   │       ├── geometry.rs
│   │       ├── layer.rs
│   │       ├── undo.rs
│   │       └── parser/
│   │           ├── mod.rs
│   │           ├── reader.rs
│   │           ├── writer.rs
│   │           └── tests.rs
│   └── domain-wasm/            # Thin wasm-bindgen wrapper
│       ├── Cargo.toml          # name = "go60-rgb-editor-wasm"
│       └── src/
│           └── lib.rs          # #[wasm_bindgen] functions wrapping domain API
├── src/                        # TUI binary crate (unchanged module structure minus domain/)
│   ├── main.rs
│   ├── app.rs
│   ├── event.rs
│   ├── tui.rs
│   ├── io/
│   └── ui/
├── tests/
│   ├── fixtures/               # Keep for any TUI-level integration tests
│   └── architecture.rs
└── mise.toml
```

## Steps

### Step 1: Convert to Cargo Workspace

**Files changed:** `Cargo.toml` (root)

Turn the root `Cargo.toml` into a workspace definition. The TUI binary stays at the root (as the default member) so existing `cargo run`, `cargo test`, etc. keep working unchanged.

Rename the package to `go60-rgb-editor-tui` and add an explicit `[[bin]]` to keep the CLI command as `go60-rgb-editor`:

```toml
[package]
name = "go60-rgb-editor-tui"
# ... existing version, edition, license ...

[[bin]]
name = "go60-rgb-editor"
path = "src/main.rs"

[workspace]
members = [".", "crates/domain", "crates/domain-wasm"]

[dependencies]
go60-rgb-editor-domain = { path = "crates/domain" }
# ... ratatui, crossterm, etc. stay here
```

### Step 2: Create `crates/domain/` Library Crate

**New files:** `crates/domain/Cargo.toml`, `crates/domain/src/lib.rs`, and all moved domain source files.

`Cargo.toml`:
```toml
[package]
name = "go60-rgb-editor-domain"
version = "0.2.0"
edition = "2021"
license = "MIT"
description = "Domain model for Go60 RGB underglow editor"

# No [dependencies] section — zero external deps

[dev-dependencies]
cargo_pup_lint_config = "0.1.5"
```

Move source files:
- `src/domain/color.rs` → `crates/domain/src/color.rs`
- `src/domain/config.rs` → `crates/domain/src/config.rs`
- `src/domain/cursor.rs` → `crates/domain/src/cursor.rs`
- `src/domain/editor.rs` → `crates/domain/src/editor.rs`
- `src/domain/geometry.rs` → `crates/domain/src/geometry.rs`
- `src/domain/layer.rs` → `crates/domain/src/layer.rs`
- `src/domain/undo.rs` → `crates/domain/src/undo.rs`
- `src/domain/parser/mod.rs` → `crates/domain/src/parser/mod.rs`
- `src/domain/parser/reader.rs` → `crates/domain/src/parser/reader.rs`
- `src/domain/parser/writer.rs` → `crates/domain/src/parser/writer.rs`
- `src/domain/parser/tests.rs` → `crates/domain/src/parser/tests.rs`
- `src/domain/mod.rs` content → `crates/domain/src/lib.rs` (adapted)

**Changes in the moved files:**
- `parser/reader.rs` line 1: `use crate::domain::{...}` → `use crate::{...}` (the crate root is now the domain)
- `parser/writer.rs` line 1: `use crate::domain::Config` → `use crate::Config`
- `parser/writer.rs` line 33: `&crate::domain::Layer` → `&crate::Layer`
- `parser/tests.rs` line 3: `use crate::domain::{...}` → `use crate::{...}`
- `parser/tests.rs` line 4: `use crate::domain::parser::{...}` → `use crate::parser::{...}`
- `parser/tests.rs` line 6: fix `include_str!` path to `../../../tests/fixtures/sample_config.txt` (relative from `crates/domain/src/parser/tests.rs` → `crates/domain/tests/fixtures/sample_config.txt`)
- `editor.rs`: uses `super::` paths which still work since `lib.rs` defines the same modules
- `layer.rs`: uses `super::` paths — still works
- `cursor.rs`: uses `super::` paths — still works

`crates/domain/src/lib.rs` (adapted from current `domain/mod.rs`):
```rust
pub mod color;
pub mod config;
pub mod cursor;
pub mod editor;
pub mod geometry;
pub mod layer;
pub mod parser;
pub mod undo;

pub use color::{ColorDef, ColorKind, ColorPalette, RgbColor};
pub use config::Config;
pub use cursor::Direction;
pub use editor::EditorState;
pub use geometry::{Half, RgbPos};
pub use layer::Layer;
pub use parser::{parse_config, write_config};
```

### Step 3: Update TUI Binary to Use Domain as Dependency

**Files changed:** `src/main.rs`, `src/app.rs`, `src/event.rs`, `src/io/`, `src/ui/`

- Delete `src/domain/` directory entirely (it now lives in `crates/domain/`)
- Remove `mod domain;` from `src/main.rs`
- All `use crate::domain::...` imports throughout the TUI crate become `use go60_rgb_editor_domain::...`

Affected files (grep for `crate::domain`):
- `src/app.rs` — `use crate::domain::cursor::Direction` → `use go60_rgb_editor_domain::cursor::Direction` (and similar)
- `src/event.rs` — any domain imports
- `src/io/` — `parse_config`, `write_config` imports
- `src/ui/` — color types, layer types, geometry types

### Step 4: Move Test Fixture

**Files moved:** `tests/fixtures/sample_config.txt`

Copy the fixture to `crates/domain/tests/fixtures/sample_config.txt` so the domain crate's parser integration tests remain self-contained. Keep the original in `tests/fixtures/` for any TUI-level integration tests that may exist or be added later.

The `include_str!` path in `crates/domain/src/parser/tests.rs` becomes:
```rust
const SAMPLE_CONFIG: &str = include_str!("../../../tests/fixtures/sample_config.txt");
```
This resolves from `crates/domain/src/parser/tests.rs` → up 3 to `crates/domain/` → `tests/fixtures/sample_config.txt`.

### Step 5: Create `crates/domain-wasm/` Wrapper Crate

**New files:** `crates/domain-wasm/Cargo.toml`, `crates/domain-wasm/src/lib.rs`

`Cargo.toml`:
```toml
[package]
name = "go60-rgb-editor-wasm"
version = "0.2.0"
edition = "2021"
license = "MIT"
description = "WebAssembly bindings for Go60 RGB underglow editor"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
go60-rgb-editor-domain = { path = "../domain" }
wasm-bindgen = "0.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

`src/lib.rs` — thin wrapper exposing an opaque `Editor` handle:

```rust
use wasm_bindgen::prelude::*;
use go60_rgb_editor_domain::{EditorState, Direction, Half, RgbPos};
use go60_rgb_editor_domain::parser::{parse_config, write_config};

#[wasm_bindgen]
pub struct Editor {
    inner: EditorState,
}

#[wasm_bindgen]
impl Editor {
    /// Create an Editor from a config file string.
    #[wasm_bindgen(constructor)]
    pub fn new(config_text: &str) -> Result<Editor, String> {
        let config = parse_config(config_text)?;
        Ok(Editor { inner: EditorState::new(config) })
    }

    /// Serialize current state back to config file format.
    pub fn serialize(&self) -> String {
        write_config(&self.inner.config)
    }

    /// Get the full state as JSON (for rendering in the web UI).
    pub fn to_json(&self) -> Result<String, String> {
        // Custom serialization — only what the UI needs:
        // layers, palette, cursor position, current layer index
        // (implemented as manual JSON building or via serde on mirror structs)
        todo!("implement JSON serialization of editor state")
    }

    // Editing operations — thin delegation:
    pub fn move_cursor(&mut self, direction: &str) { ... }
    pub fn set_color(&mut self, abbrev: &str) -> bool { ... }
    pub fn clear_color(&mut self) -> bool { ... }
    pub fn next_layer(&mut self) { ... }
    pub fn prev_layer(&mut self) { ... }
    pub fn undo(&mut self) -> bool { ... }
    pub fn redo(&mut self) -> bool { ... }
    // ... etc
}
```

**Design decisions:**
- `Editor` wraps `EditorState` as an opaque handle in Wasm memory
- JS calls methods on it (move cursor, set color, etc.)
- JS reads the current state via `to_json()` for rendering
- Undo/redo stays internal in Rust memory — no need to expose the history
- `wasm-bindgen` and `serde` only appear in this wrapper crate

### Step 6: Update Architecture Tests

**Files changed:** `tests/architecture.rs`, new `crates/domain/tests/architecture.rs`

Since the domain is now a separate crate with zero dependencies, architecture enforcement works at two levels:

**a. Root `tests/architecture.rs`** — update for TUI crate:
- Remove `domain_is_self_contained` rule (domain is now external; this is enforced structurally by Cargo.toml)
- Keep `ui_no_io_access`, `io_no_ui_dependency` (these reference TUI-internal modules)
- Keep hygiene rules (`clean_mod_files`, `no_wildcard_imports`, `function_length_limit`)

**b. New `crates/domain/tests/architecture.rs`** — domain-specific rules:
- `domain_has_no_external_imports` — defense-in-depth: block imports of `ratatui`, `crossterm`, `color_eyre`, `clap`, `wasm_bindgen`, `serde`, `serde_json`
- Hygiene rules (`clean_mod_files`, `no_wildcard_imports`, `function_length_limit`)

```rust
// crates/domain/tests/architecture.rs
use cargo_pup_lint_config::{FunctionLintExt, LintBuilder, ModuleLintExt, Severity};

fn build_architecture_rules() -> LintBuilder {
    let mut builder = LintBuilder::new();

    // Defense-in-depth: domain must not import any external crate.
    // Primary enforcement is Cargo.toml having no [dependencies].
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

    // Module hygiene
    builder
        .module_lint()
        .lint_named("clean_mod_files")
        .matching(|m| m.module(".*"))
        .with_severity(Severity::Error)
        .must_have_empty_mod_file()
        .build();

    builder
        .module_lint()
        .lint_named("no_wildcard_imports")
        .matching(|m| m.module(".*"))
        .with_severity(Severity::Error)
        .no_wildcard_imports()
        .build();

    // Function hygiene
    builder
        .function_lint()
        .lint_named("function_length_limit")
        .matching(|m| m.name_regex(".*"))
        .with_severity(Severity::Error)
        .max_length(60)
        .build();

    builder
}

#[test]
fn generate_pup_config() {
    // Act
    let builder = build_architecture_rules();
    builder
        .write_to_file("pup.ron")
        .expect("Failed to write pup.ron");
}
```

### Step 7: Update mise.toml

**Files changed:** `mise.toml`

Update `arch-lint` to lint both crates and add `build-wasm` task:

```toml
[tasks.arch-lint]
description = "Run architecture linting with cargo-pup (requires nightly)"
run = """
cargo test --test architecture -- --nocapture
cargo test -p go60-rgb-editor-domain --test architecture -- --nocapture
NIGHTLY_BIN="$(dirname "$(rustup which --toolchain {{vars.PUP_NIGHTLY}} cargo)")"
PATH="$NIGHTLY_BIN:$PATH" cargo pup
cd crates/domain && PATH="$NIGHTLY_BIN:$PATH" cargo pup
"""

[tasks.build-wasm]
description = "Build domain-wasm for WebAssembly"
run = """
cargo build -p go60-rgb-editor-wasm --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir pkg/ target/wasm32-unknown-unknown/release/go60_rgb_editor_wasm.wasm
"""
```

### Step 8: Verify Everything Compiles and Tests Pass

Run in order:
1. `cargo build` — TUI binary still builds
2. `cargo test` — all existing tests pass (workspace-wide)
3. `cargo test -p go60-rgb-editor-domain` — domain tests pass in isolation
4. `cargo build -p go60-rgb-editor-wasm --target wasm32-unknown-unknown` — Wasm build succeeds
5. `mise run test` — clippy + tests pass
6. `mise run arch-lint` — architecture rules pass for both crates

### Step 9: Update Documentation

**Files changed:** `CLAUDE.md`

- Update project structure to reflect workspace layout
- Add `crates/domain/` and `crates/domain-wasm/` descriptions
- Document the new `build-wasm` mise task
- Update architecture linting section to mention both crates
- Note the crate naming convention

---

## Summary of Dependency Graph

```
go60-rgb-editor-domain (zero deps)
    ↑                          ↑
    |                          |
go60-rgb-editor-tui         go60-rgb-editor-wasm
(ratatui, crossterm,        (wasm-bindgen, serde,
 color-eyre, clap)           serde_json)
```

The domain stays completely free of any external dependency. The wasm-bindgen/serde concerns are isolated in the wrapper crate. The TUI crate's dependencies don't leak into the domain.

## Risk / Considerations

1. **`include_str!` paths for test fixtures** — These are relative to the source file. Need to verify the path after moving files.
2. **cargo-pup workspace support** — cargo-pup may need `cd` into the domain crate directory to lint it. The mise task accounts for this.
3. **Workspace version sync** — All crates start at `0.2.0`. Consider using `workspace.package.version` to keep them in sync, or version them independently.
4. **`cargo release`** — Currently configured for a single crate. May need `cargo-release` workspace configuration to handle multi-crate releases (or release domain independently).
5. **Binary name vs crate name** — The `[[bin]]` section ensures `cargo install` and `cargo run` still produce/run `go60-rgb-editor` despite the crate being named `go60-rgb-editor-tui`.
