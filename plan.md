# Implementation Plan: Extract Domain into a Workspace Library Crate for Wasm

## Goal

Extract `src/domain/` into a standalone library crate (`crates/domain/`) with **zero external dependencies**. Add a thin `crates/domain-wasm/` wrapper crate that depends on `wasm-bindgen` and exposes the domain to JavaScript. The TUI application becomes a separate binary crate that depends on the domain library.

## Resulting Structure

```
go60-rgb-editor/
├── Cargo.toml                  # [workspace] root
├── crates/
│   ├── domain/                 # Pure domain lib — zero external deps
│   │   ├── Cargo.toml          # name = "go60-domain"
│   │   └── src/
│   │       ├── lib.rs          # Re-exports (current domain/mod.rs content)
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
│       ├── Cargo.toml          # depends on go60-domain + wasm-bindgen + serde/serde_json
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
│   ├── fixtures/
│   └── architecture.rs
└── mise.toml
```

## Steps

### Step 1: Convert to Cargo Workspace

**Files changed:** `Cargo.toml` (root)

Turn the root `Cargo.toml` into a workspace definition. The TUI binary stays at the root (as the default member) so existing `cargo run`, `cargo test`, etc. keep working unchanged.

```toml
[workspace]
members = [".", "crates/domain", "crates/domain-wasm"]
```

The root `Cargo.toml` keeps its current `[package]`, `[dependencies]`, etc. but adds a dependency on the domain crate:

```toml
[dependencies]
go60-domain = { path = "crates/domain" }
# ... ratatui, crossterm, etc. stay here
```

### Step 2: Create `crates/domain/` Library Crate

**New files:** `crates/domain/Cargo.toml`, `crates/domain/src/lib.rs`, and all moved domain source files.

`Cargo.toml`:
```toml
[package]
name = "go60-domain"
version = "0.2.0"
edition = "2021"
license = "MIT"
description = "Domain model for Go60 RGB editor"

# No [dependencies] section — zero external deps
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
- `src/domain/mod.rs` → `crates/domain/src/lib.rs` (adjust content)

**Changes in the moved files:**
- `parser/reader.rs` line 1: `use crate::domain::{...}` → `use crate::{...}` (the crate root is now the domain)
- `parser/writer.rs` line 1: `use crate::domain::Config` → `use crate::Config`
- `parser/writer.rs` line 33: `&crate::domain::Layer` → `&crate::Layer`
- `parser/tests.rs` line 3: `use crate::domain::{...}` → `use crate::{...}`
- `parser/tests.rs` line 4: `use crate::domain::parser::{...}` → `use crate::parser::{...}`
- `parser/tests.rs` line 6: fix `include_str!` path — `../../../tests/fixtures/sample_config.txt` needs to be adjusted since the crate root moved. Either:
  - Move the fixture file to `crates/domain/tests/fixtures/` (preferred — domain tests own their fixtures)
  - Or adjust the relative path to `../../../../tests/fixtures/sample_config.txt`
- `editor.rs`: uses `super::` paths which still work since `lib.rs` re-exports the same modules
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
pub use geometry::{Half, RgbPos};
pub use layer::Layer;
```

**Also export types that the wasm wrapper and TUI both need:**
```rust
pub use cursor::Direction;
pub use editor::EditorState;
pub use parser::{parse_config, write_config};
```

### Step 3: Update TUI Binary to Use Domain as Dependency

**Files changed:** `src/main.rs`, `src/app.rs`, `src/event.rs`, `src/io/`, `src/ui/`

- Delete `src/domain/` directory entirely (it now lives in `crates/domain/`)
- Remove `mod domain;` from `src/main.rs`
- All `use crate::domain::...` imports throughout the TUI crate become `use go60_domain::...`

Affected files (grep for `crate::domain`):
- `src/app.rs` — `use crate::domain::cursor::Direction` → `use go60_domain::cursor::Direction` (and similar)
- `src/event.rs` — any domain imports
- `src/io/` — `parse_config`, `write_config` imports
- `src/ui/` — color types, layer types, geometry types

### Step 4: Move Test Fixture

**Files moved/changed:** `tests/fixtures/sample_config.txt`

Copy the fixture to `crates/domain/tests/fixtures/sample_config.txt` so the domain crate's parser integration tests remain self-contained. The original can stay for any TUI-level integration tests.

Update `include_str!` in `crates/domain/src/parser/tests.rs`:
```rust
const SAMPLE_CONFIG: &str = include_str!("../../../tests/fixtures/sample_config.txt");
```
This path goes from `crates/domain/src/parser/tests.rs` → up 3 levels to `crates/domain/` → `tests/fixtures/sample_config.txt`.

### Step 5: Create `crates/domain-wasm/` Wrapper Crate

**New files:** `crates/domain-wasm/Cargo.toml`, `crates/domain-wasm/src/lib.rs`

`Cargo.toml`:
```toml
[package]
name = "go60-domain-wasm"
version = "0.2.0"
edition = "2021"
license = "MIT"
description = "WebAssembly bindings for Go60 RGB domain model"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
go60-domain = { path = "../domain" }
wasm-bindgen = "0.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

`src/lib.rs` — thin wrapper exposing an opaque `Editor` handle plus free functions for parse/write:

```rust
use wasm_bindgen::prelude::*;
use go60_domain::{EditorState, Config, Direction, Half, RgbPos};
use go60_domain::parser::{parse_config, write_config};

/// Parse a TailorKey config string. Returns the editor state as an opaque handle.
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

The wrapper crate is the **only** place `wasm-bindgen` and `serde` appear. The domain crate stays dependency-free.

**Design decision — opaque handle vs JSON round-trips:**
- The `Editor` struct wraps `EditorState` as an opaque handle on the Wasm side
- JS calls methods on it (move cursor, set color, undo, etc.)
- JS reads the current state via `to_json()` which returns a JSON snapshot for rendering
- This avoids serializing/deserializing on every keystroke — only state reads go through JSON
- Undo/redo stays internal (in Rust memory) as you mentioned — no need to expose the history

### Step 6: Update Architecture Tests

**Files changed:** `tests/architecture.rs`

The architecture test file lives in the TUI binary's `tests/` directory. It uses `cargo_pup_lint_config` to generate `pup.ron`. Since cargo-pup analyzes the crate being linted, and the workspace now has multiple crates, we need to decide the scope.

**Option A (recommended):** Keep the architecture test in the root `tests/architecture.rs` for the TUI crate. The existing rules already work because they match module paths like `.*::domain::.*` — but since domain is now an external crate, cargo-pup won't see its internals from the TUI crate's perspective. This means:

1. **`domain_is_self_contained`** — This rule can be **moved to the domain crate itself** as `crates/domain/tests/architecture.rs`. However, since the domain has zero dependencies, this rule is **enforced structurally by the Cargo.toml** — if someone adds `ratatui` to the domain's `Cargo.toml`, it would be an explicit, reviewable change. The arch lint becomes a defense-in-depth check rather than the primary guard.

2. **`ui_no_io_access`** and **`io_no_ui_dependency`** — These stay in the TUI crate's arch tests unchanged (they reference TUI-internal modules).

3. **`clean_mod_files`**, **`no_wildcard_imports`**, **`function_length_limit`** — These are universal hygiene rules. They should apply to both crates.

**Implementation:**

a. **Root `tests/architecture.rs`** — update module path patterns:
   - Remove `domain_is_self_contained` (no longer applicable — domain is external)
   - Keep `ui_no_io_access`, `io_no_ui_dependency`
   - Keep hygiene rules

b. **New `crates/domain/tests/architecture.rs`** — add domain-specific rules:
   - `domain_has_no_external_imports`: Ensure domain modules don't import anything outside the crate (defense-in-depth)
   - Hygiene rules (clean mod files, no wildcards, function length)

c. **Add `cargo_pup_lint_config` as dev-dependency** to the domain crate:
   ```toml
   [dev-dependencies]
   cargo_pup_lint_config = "0.1.5"
   ```
   (This is the one dev-dependency — acceptable since it's test-only.)

d. **New arch rule for the domain crate:**
   ```rust
   // domain must not import anything outside its own crate
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
   ```

e. **Update `mise.toml`** — the `arch-lint` task needs to run cargo-pup for both crates:
   ```toml
   [tasks.arch-lint]
   description = "Run architecture linting with cargo-pup (requires nightly)"
   run = """
   cargo test --test architecture -- --nocapture
   cargo test -p go60-domain --test architecture -- --nocapture
   NIGHTLY_BIN="$(dirname "$(rustup which --toolchain {{vars.PUP_NIGHTLY}} cargo)")"
   PATH="$NIGHTLY_BIN:$PATH" cargo pup
   PATH="$NIGHTLY_BIN:$PATH" cargo pup -p go60-domain
   """
   ```

### Step 7: Add Wasm Build Task to mise.toml

**Files changed:** `mise.toml`

```toml
[tasks.build-wasm]
description = "Build domain-wasm for WebAssembly"
run = """
cargo build -p go60-domain-wasm --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir pkg/ target/wasm32-unknown-unknown/release/go60_domain_wasm.wasm
"""
```

### Step 8: Verify Everything Compiles and Tests Pass

Run in order:
1. `cargo build` — TUI binary still builds
2. `cargo test` — all existing tests pass (TUI + domain via workspace)
3. `cargo test -p go60-domain` — domain tests pass in isolation
4. `cargo build -p go60-domain-wasm --target wasm32-unknown-unknown` — Wasm build succeeds
5. `mise run arch-lint` — architecture rules pass for both crates

### Step 9: Update Documentation

**Files changed:** `CLAUDE.md`, `README.md` (if applicable)

- Update project structure in CLAUDE.md to reflect workspace layout
- Add `crates/domain/` and `crates/domain-wasm/` descriptions
- Document the new `build-wasm` mise task
- Update architecture linting section to mention both crates

---

## Summary of Dependency Graph

```
go60-domain (zero deps)
    ↑                    ↑
    |                    |
go60-rgb-editor       go60-domain-wasm
(ratatui, crossterm,  (wasm-bindgen, serde,
 color-eyre, clap)     serde_json)
```

The domain stays completely free of any external dependency. The wasm-bindgen/serde concerns are isolated in the wrapper crate. The TUI crate's dependencies don't leak into the domain.

## Risk / Considerations

1. **`include_str!` paths for test fixtures** — These are relative to the source file. Need to verify the path after moving files.
2. **cargo-pup workspace support** — Need to verify cargo-pup can lint individual workspace members. If not, may need to run it separately per crate.
3. **Workspace version sync** — Both crates start at `0.2.0`. Consider using `workspace.package.version` to keep them in sync, or version them independently.
4. **`cargo release`** — Currently configured for a single crate. May need `cargo-release` workspace configuration to handle multi-crate releases (or release domain separately).
