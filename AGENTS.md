# Go60 RGB Editor

TUI RGB underglow editor for ZMK keyboards (MoErgo Go60 layout).

## Project Structure

This is a Cargo workspace with three crates:

| Crate | Package name | Purpose |
|---|---|---|
| Domain lib | `go60-rgb-editor-domain` | Pure domain model — zero external dependencies |
| Wasm wrapper | `go60-rgb-editor-wasm` | WebAssembly bindings (wasm-bindgen + serde) |
| TUI binary | `go60-rgb-editor-tui` | Terminal UI application |

```
Cargo.toml                      # Workspace root + TUI binary (go60-rgb-editor-tui)
crates/
├── domain/                     # go60-rgb-editor-domain (zero deps)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs              # Re-exports
│   │   ├── color.rs            # Color definitions, palette
│   │   ├── config.rs           # Overall config
│   │   ├── cursor.rs           # Cursor position for keyboard navigation
│   │   ├── editor.rs           # Editor state & logic (undo, layers, colors)
│   │   ├── geometry.rs         # Keyboard layout geometry definitions
│   │   ├── layer.rs            # Layer structure
│   │   ├── undo.rs             # Undo/redo history
│   │   └── parser/
│   │       ├── mod.rs          # Re-exports
│   │       ├── reader.rs       # Config file parser
│   │       ├── writer.rs       # Serialize back to file
│   │       └── tests.rs        # Parser integration tests
│   └── tests/
│       ├── fixtures/           # Test fixture files for domain
│       └── architecture.rs     # Domain architecture rules (cargo-pup)
├── domain-wasm/                # go60-rgb-editor-wasm
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs              # #[wasm_bindgen] wrapper around domain
src/                            # TUI binary (go60-rgb-editor-tui)
├── main.rs                     # Entry point, CLI args
├── app.rs                      # Application state & logic
├── event.rs                    # Key event handling
├── tui.rs                      # Terminal setup/teardown
├── io/                         # File I/O, clipboard
├── ui/                         # UI widgets
│   ├── mod.rs                  # Re-exports
│   ├── layout.rs               # Screen layout and modal rendering
│   ├── color_render.rs         # Color-to-terminal rendering utilities
│   ├── keyboard.rs             # Keyboard layout widget
│   ├── layer_list.rs
│   ├── color_picker.rs
│   ├── status_bar.rs
│   └── help.rs
tests/
├── fixtures/                   # Test fixture files for TUI
└── architecture.rs             # TUI architecture rules (cargo-pup)
```

### Dependency Graph

```
go60-rgb-editor-domain (zero deps)
    ↑                          ↑
    |                          |
go60-rgb-editor-tui         go60-rgb-editor-wasm
(ratatui, crossterm,        (wasm-bindgen, serde,
 color-eyre, clap)           serde_json)
```

## Tech Stack

- Rust (Cargo workspace)
- Ratatui (TUI framework)
- Crossterm (terminal backend)
- wasm-bindgen (WebAssembly bindings)
- [cargo-pup](https://github.com/datadog/cargo-pup) (architecture linting)

## Development

Uses [mise](https://mise.jdx.dev/) for task management. Run `mise tasks ls` for available tasks.

```bash
mise install          # Install tools (cargo-llvm-cov, git-cliff, cargo-release)
mise run setup        # One-time setup (rustup components, nightly for cargo-pup)
mise run build        # Build (debug)
mise run build-release # Build (release)
mise run test         # Run clippy + tests
mise run lint         # Run clippy lints
mise run arch-lint    # Run architecture linting with cargo-pup (requires nightly)
mise run coverage     # Test coverage report
mise run coverage-html # Coverage report in browser
mise run run          # Run editor with example config
mise run build-wasm   # Build domain-wasm for WebAssembly
mise run changelog    # Preview unreleased changelog entries
mise run release-patch # Release patch version
mise run release-minor # Release minor version
mise run release-major # Release major version
```

## Commits

This project uses [Conventional Commits](https://www.conventionalcommits.org/). All commit messages **must** follow this format:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat:` — new feature (bumps minor version)
- `fix:` — bug fix (bumps patch version)
- `docs:` — documentation only
- `style:` — formatting, no code change
- `refactor:` — code restructuring, no behavior change
- `perf:` — performance improvement
- `test:` — adding/updating tests
- `ci:` — CI/CD changes
- `chore:` — maintenance tasks
- `build:` — build system changes

**Breaking changes:** Add `!` after the type (e.g., `feat!: redesign config format`) or add a `BREAKING CHANGE:` footer. This bumps the major version.

## Releases

Releases are managed with `cargo-release` and `git-cliff` (changelog generation). Both are installed via mise.

```bash
mise run changelog        # Preview/generate CHANGELOG.md
mise run release-patch    # Release patch version (bug fixes)
mise run release-minor    # Release minor version (features)
mise run release-major    # Release major version (breaking)
```

`cargo-release` bumps the version in `Cargo.toml`, generates the changelog via `git-cliff`, commits, and tags. Push the tag manually to trigger the GitHub Actions release workflow which builds binaries for all platforms.

## Tests

- All tests **must** use Arrange-Act-Assert (AAA) block comments to structure test bodies:
  - `// Arrange` — set up test data and preconditions
  - `// Act` — execute the code under test
  - `// Assert` — verify the expected outcome
  - Use `// Act & Assert` when act and assert are interleaved or inseparable (e.g., multiple inline checks)
- Omit a section comment if that phase is empty (e.g., no arrange needed)
- Keep any existing explanatory comments; AAA comments are added alongside them, not as replacements

## Architecture Linting

This project uses [cargo-pup](https://github.com/datadog/cargo-pup) (an ArchUnit alternative for Rust) to enforce architectural boundaries and code hygiene.

Architecture rules are defined in two places:
- `tests/architecture.rs` — rules for the TUI crate
- `crates/domain/tests/architecture.rs` — rules for the domain crate

Each generates a `pup.ron` config file. The `arch-lint` mise task runs both, then invokes `cargo pup` for each crate.

### Rules

**TUI crate (`tests/architecture.rs`):**
- `ui_no_io_access` — UI must not import IO modules
- `io_no_ui_dependency` — IO must not import UI or presentation crates

**Domain crate (`crates/domain/tests/architecture.rs`):**
- `domain_has_no_external_imports` — defense-in-depth: domain must not import ratatui, crossterm, color-eyre, clap, wasm-bindgen, serde, or serde-json (primary enforcement is Cargo.toml having zero dependencies)

**Code hygiene (both crates):**
- `clean_mod_files` — mod.rs files should only contain mod declarations and re-exports
- `no_wildcard_imports` — no `use something::*`
- `function_length_limit` — functions should not exceed 60 lines

### Nightly Toolchain Requirement

cargo-pup hooks into `rustc` compiler internals (`rustc_private` API) to analyze module structure, imports, and function bodies. These internals are only available on nightly Rust. This does **not** affect the normal build — cargo-pup performs a separate analysis build in `.pup/`.

The pinned nightly version is defined once in `mise.toml` as `PUP_NIGHTLY`. When upgrading cargo-pup, update:
1. `PUP_NIGHTLY` in `mise.toml`
2. `cargo_pup_lint_config` version in both `Cargo.toml` (root) and `crates/domain/Cargo.toml`

Then run `mise run setup` to install the new toolchain and cargo-pup version.

## Config File Format

The editor parses TailorKey RGB config files. See:
https://sites.google.com/view/tailorkey/how-to/rgb

## Documentation Maintenance

When making changes, keep documentation in sync:
- **AGENTS.md** — update project structure when adding/removing/renaming source files; update mise tasks when adding/removing tasks
- **README.md** — update features, usage, or installation instructions when user-facing behavior changes
