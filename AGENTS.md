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
src/                            # TUI binary + library (go60-rgb-editor-tui)
├── lib.rs                      # Library re-exports (app, event, io, ui)
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
├── helpers/
│   └── mod.rs                  # E2E test harness (create_e2e_app, send_key, render, etc.)
├── e2e_paint_keyboard.rs       # Journey 1: paint, clear, undo/redo
├── e2e_manage_layers.rs        # Journey 2: layer CRUD, fade delay
├── e2e_load_config.rs          # Journey 3: config loading, save/save-as
├── e2e_navigation.rs           # Journey 4: keyboard navigation, palette picker, help
├── e2e_copy_config.rs          # Journey 5: clipboard confirm/cancel
├── fixtures/                   # Test fixture files for TUI
└── architecture.rs             # TUI architecture rules (cargo-pup)
pkg/                            # WASM build output (generated, gitignored except .d.ts)
├── go60_rgb_editor_wasm.d.ts   # Type declarations (committed stub, overwritten by build-wasm)
web/                            # Web editor SPA (Vite + TypeScript + WASM)
├── index.html                  # Entry HTML
├── eslint.config.js            # ESLint flat config (strict TS + architecture rules)
├── package.json                # Node dependencies
├── playwright.config.ts        # Playwright E2E test config
├── tsconfig.json               # TypeScript config (strict mode)
├── vite.config.ts              # Vite config (WASM plugin)
├── e2e/                        # E2E tests (Playwright, headless Chrome)
│   ├── paint-keyboard.spec.ts  # Journey 1: paint, clear, undo/redo
│   ├── manage-layers.spec.ts   # Journey 2: layer CRUD, fade delay
│   ├── load-config.spec.ts     # Journey 3: config loading, error recovery
│   └── keyboard-navigation.spec.ts # Journey 4: keyboard navigation, palette picker, help
└── src/
    ├── main.ts                 # App entry, event handlers, orchestration
    ├── editor-bridge.ts        # WASM ↔ JS bridge (wraps Editor handle)
    ├── state.ts                # TypeScript types matching WASM JSON, color utils
    ├── geometry.ts             # Keyboard layout constants (mirrors domain)
    ├── styles.css              # App styles, dark theme
    ├── vite-env.d.ts           # Vite/WASM type declarations
    └── components/
        ├── keyboard.ts         # Keyboard half rendering
        ├── palette.ts          # Color palette swatches
        ├── layers.ts           # Layer list and actions
        ├── toolbar.ts          # Toolbar (undo/redo, fade, modified indicator)
        └── config-text.ts      # Config textarea sync
```

### Dependency Graph

```
go60-rgb-editor-domain (zero deps)
    ↑                          ↑
    |                          |
go60-rgb-editor-tui         go60-rgb-editor-wasm ← web/ SPA
(ratatui, crossterm,        (wasm-bindgen, serde,  (Vite, TypeScript,
 color-eyre, clap)           serde_json)            Playwright)
```

## Tech Stack

- Rust (Cargo workspace)
- Ratatui (TUI framework)
- Crossterm (terminal backend)
- wasm-bindgen (WebAssembly bindings)
- Vite + TypeScript (web editor SPA)
- Playwright (web E2E tests, headless Chrome)
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
mise run web-lint     # Run TypeScript linting (ESLint)
mise run lint-all     # Run linting across all stacks (Rust + TypeScript)
mise run arch-lint    # Run architecture linting with cargo-pup (requires nightly)
mise run coverage     # Test coverage report
mise run coverage-html # Coverage report in browser
mise run run          # Run editor with example config
mise run sbom          # Generate Rust SBOM (CycloneDX JSON)
mise run sbom-web      # Generate web SBOM (CycloneDX JSON)
mise run sbom-all      # Generate SBOMs for all stacks (Rust + Web)
mise run build-wasm   # Build domain-wasm for WebAssembly
mise run web-install  # Install web dependencies
mise run web-dev      # Start web dev server (⚠️ BLOCKING — see warning below)
mise run web-build    # Build web app for production
mise run web-e2e      # Run web E2E tests (Playwright, headless Chrome)
mise run release-build   # Build release binary (set TARGET, USE_CROSS env vars)
mise run release-package # Package release binary + SBOM (set TARGET, VERSION env vars)
mise run tui-e2e      # Run TUI E2E tests (in-process, ratatui TestBackend)
mise run changelog    # Preview unreleased changelog entries
mise run release-patch # Release patch version
mise run release-minor # Release minor version
mise run release-major # Release major version
```

### ⚠️ Blocking Tasks

**NEVER run `mise run web-dev` directly** — it starts a long-running dev server that blocks the agent indefinitely. Use one of these approaches instead:

1. **tmux** (preferred): `tmux new-session -d -s webdev 'mise run web-dev'` — then `tmux kill-session -t webdev` to stop
2. **Subagent**: Delegate to a subagent, but note the server outlives the agent and is hard to clean up
3. **Background**: `mise run web-dev &` works but the process is difficult to stop reliably

The same applies to any other long-running/blocking task (e.g., `mise run run` for the TUI binary).

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

### Rust (cargo-pup)

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

### TypeScript (ESLint)

The web editor uses ESLint with `typescript-eslint` strict type-checked rules, configured in `web/eslint.config.js`. Rules are aligned with the Rust linting and the project's coding style / TypeScript skill definitions.

**Architecture rules** (mirror Rust cargo-pup rules):
- Components (`src/components/`) must not import `editor-bridge` — mirrors `ui_no_io_access`
- `editor-bridge.ts` must not import components — mirrors `io_no_ui_dependency`

**Code hygiene** (mirror Rust rules):
- `max-lines-per-function: 60` — matches Rust `function_length_limit`
- `max-depth: 4` — enforces early returns (coding-style skill)
- `complexity: 15` — keeps functions focused

**TypeScript skill rules:**
- `strictTypeChecked` + `stylisticTypeChecked` presets (includes no-any, no-unsafe-*, no-floating-promises)
- `explicit-function-return-type` — all functions must declare return types
- `consistent-type-imports` — enforce `type` imports for type-only symbols
- `consistent-type-assertions` — minimize `as`, forbid on object literals
- `switch-exhaustiveness-check` — compiler-enforced exhaustive switches
- `consistent-type-definitions: off` — allows both `type` (data) and `interface` (behavior)
- `prefer-readonly` — enforce readonly class fields

**WASM type declarations:**
- `pkg/go60_rgb_editor_wasm.d.ts` is a committed type stub so ESLint can resolve WASM types without building the package
- The `build-wasm` task overwrites it with wasm-bindgen's generated declarations

## Config File Format

The editor parses TailorKey RGB config files. See:
https://sites.google.com/view/tailorkey/how-to/rgb

## Documentation Maintenance

When making changes, keep documentation in sync:
- **AGENTS.md** — update project structure when adding/removing/renaming source files; update mise tasks when adding/removing tasks
- **README.md** — update features, usage, or installation instructions when user-facing behavior changes
