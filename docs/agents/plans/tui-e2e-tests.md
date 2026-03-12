# TUI E2E Test Plan

## Goal

Add E2E tests for the TUI binary that mirror the 5 web E2E user journeys, adapted for terminal interaction.

## Approach: In-Process Harness

Use ratatui's `TestBackend` (already available — no new deps) to render frames in memory, and drive key events directly through `handle_key`. This avoids spawning a real terminal process.

**Test pipeline per step:**

```
build App → send KeyEvent via handle_key → render to TestBackend → assert on buffer text / App state
```

## Prerequisites (Minimal Refactoring)

Three visibility changes needed before tests can work:

| # | Change | File | Why |
|---|--------|------|-----|
| 1 | `handle_key` → `pub(crate)` | `src/event.rs` | Currently private; tests need to send key events without polling real stdin |
| 2 | Extract `create_test_app()` to shared module | `src/app.rs` → `src/test_helpers.rs` | Currently `#[cfg(test)] mod tests`-private; E2E tests in `tests/` need it |
| 3 | Add `test_helpers` module | `src/main.rs` | `#[cfg(test)] mod test_helpers;` — expose helper for integration tests |

No new crate dependencies required. `ratatui::backend::TestBackend` is part of ratatui 0.28's public API.

## Test Harness

A small `tests/helpers/mod.rs` providing:

```rust
/// Create an App loaded with the sample fixture config
fn create_e2e_app() -> App;

/// Send a key event through the full dispatch pipeline
fn send_key(app: &mut App, code: KeyCode);
fn send_key_modified(app: &mut App, code: KeyCode, modifiers: KeyModifiers);
fn type_str(app: &mut App, text: &str);  // send a sequence of Char keys

/// Render the app to a TestBackend and return the Buffer
fn render(app: &App, width: u16, height: u16) -> Buffer;

/// Assert on rendered output (what the user sees)
fn buffer_contains(buffer: &Buffer, text: &str) -> bool;
fn buffer_text_at(buffer: &Buffer, area: Rect) -> String;
fn cell_bg_color(buffer: &Buffer, col: u16, row: u16) -> Color;
```

## Journey Mapping: Web → TUI

### Journey 1: Paint Keyboard

**Web:** Click swatch → click key → clear → undo → redo → copy config
**TUI:** Navigate cursor → quick color → clear → undo → redo → copy

| # | TUI Test Step | Keys | Assert |
|---|---------------|------|--------|
| 1 | Initial state | — | App has keys, palette, layers; mode is `Normal` |
| 2 | Apply quick color | `0` | Current key color is no longer empty (`___`) |
| 3 | Move and paint another | `→`, `1` | Key colored with palette color 1 |
| 4 | Clear key | `Backspace` | Key color is empty |
| 5 | Undo clear | `Ctrl+Z` | Key color restored |
| 6 | Redo clear | `Ctrl+Y` | Key color is empty again |
| 7 | Copy config | `Ctrl+C` (triggers copy mode), confirm | Status shows copy feedback |

### Journey 2: Manage Layers

**Web:** Add/duplicate/rename/delete layers, adjust fade delay
**TUI:** Same operations via keyboard shortcuts + text input modes

| # | TUI Test Step | Keys | Assert |
|---|---------------|------|--------|
| 1 | Initial state | — | ≥1 layer, first layer active |
| 2 | Add layer | `a`, type `Test_Layer`, `Enter` | Layer count +1, "Test_Layer" exists |
| 3 | Duplicate layer | `d` | Layer count +1 |
| 4 | Rename layer | `r`, type `Renamed`, `Enter` | Active layer name changed |
| 5 | Switch layer | `PageDown` | Active layer index changed |
| 6 | Increase fade | `+` | Fade delay increased |
| 7 | Decrease fade | `-` | Fade delay back to original |
| 8 | Delete layer | `x`, confirm `y` | Layer count −1 |

### Journey 3: Load Config (File-Based Adaptation)

**Web:** Edit textarea → parse error → fix → paste recovery → cancel paste → open file
**TUI:** Loads from file at startup; no live textarea editing or paste/open-file. Adapt to file save/reload flow.

| # | TUI Test Step | Keys | Assert |
|---|---------------|------|--------|
| 1 | Load valid config | — | App loaded, editor has layers, keys have colors |
| 2 | Modify and save | Paint a key, `Ctrl+S` | File written, no error status |
| 3 | Save-as flow | `Ctrl+Shift+S`, type filename, `Enter` | File saved to new path |
| 4 | Cancel save-as | `Ctrl+Shift+S`, `Escape` | Mode returns to Normal, no file written |

> **Note:** Web journey 3 also tests paste-overwrite cancel and open-config-file. These are web-only features (textarea + file chooser) with no TUI equivalent.

### Journey 5: Copy Config (Clipboard)

**Web:** Copy button → clipboard matches config → feedback; paste button → overwrite confirm → config restored
**TUI:** Copy via `Ctrl+C` confirm flow

| # | TUI Test Step | Keys | Assert |
|---|---------------|------|--------|
| 1 | Initial state | — | App loaded with config |
| 2 | Trigger copy | `Ctrl+C` | Mode changes to `ConfirmCopy` |
| 3 | Confirm copy | `y` | Status shows copy feedback, mode back to `Normal` |
| 4 | Trigger copy + cancel | `Ctrl+C`, `n` or `Escape` | Mode back to `Normal`, no copy performed |

> **Note:** Web journey 5 also tests paste-from-clipboard. TUI has no paste-config feature — it loads from file only.

### Journey 4: Keyboard Navigation & Modes

**Web:** Arrow keys, Tab, Enter (palette picker), `?` (help), focus management
**TUI:** Same navigation + modal modes

#### 4a: Navigation & Palette Picker

| # | TUI Test Step | Keys | Assert |
|---|---------------|------|--------|
| 1 | Initial state | — | Cursor at starting position |
| 2 | Arrow navigation | `→` | Cursor column moved |
| 3 | Half switching | `Tab` | Cursor half flipped |
| 4 | Tab back | `Tab` | Back to original half |
| 5 | Quick paint | `0` | Key painted |
| 6 | Clear | `Backspace` | Key cleared |
| 7 | Undo | `Ctrl+Z` | Restored |
| 8 | Copy/paste color | `c`, `→`, `v` | Destination key matches source color |
| 9 | Open palette picker | `Enter` | Mode is `ColorPick` |
| 10 | Navigate palette, cancel | `→`, `Escape` | Mode back to `Normal`, key unchanged |
| 11 | Palette confirm | `Enter` (on key), navigate palette, `Enter` | Key painted with selected color |
| 12 | Layer switch | `PageDown` | Active layer changed |

#### 4b: Help Overlay

| # | TUI Test Step | Keys | Assert |
|---|---------------|------|--------|
| 1 | Open help | `?` | Mode is `Help`; rendered buffer contains "Help" or shortcut text |
| 2 | Close help | `?` | Mode is `Normal` |
| 3 | Open + Escape | `?`, `Escape` | Mode is `Normal` |

## Assertion Strategy

Render-only assertions — assert on what the user sees, never on internal state:

| # | Technique | Example |
|---|-----------|---------|
| 1 | **Buffer text search** | `buffer_contains(&buf, "Help")` — rendered frame contains expected text |
| 2 | **Region text extraction** | `buffer_text_at(&buf, area)` — extract text from a known screen region (e.g., layer list, status bar) |
| 3 | **Cell color inspection** | Check foreground/background color of a specific cell (e.g., cursor key is highlighted, painted key has color) |

This mirrors the web E2E approach: Playwright asserts on visible DOM text and CSS, never on JS state. TUI E2E asserts on rendered buffer text and cell styles, never on `App` fields.

## File Layout

```
tests/
├── helpers/
│   └── mod.rs              # E2E test harness (create_e2e_app, send_key, render, etc.)
├── e2e_paint_keyboard.rs   # Journey 1
├── e2e_manage_layers.rs    # Journey 2
├── e2e_load_config.rs      # Journey 3
├── e2e_navigation.rs       # Journey 4
├── e2e_copy_config.rs      # Journey 5
├── fixtures/
│   └── sample_config.txt   # (existing)
└── architecture.rs         # (existing)
```

## Mise Task

Add a `tui-e2e` task (or fold into existing `test` task since these are standard `cargo test` integration tests in `tests/`). If separate:

```toml
[tasks.tui-e2e]
description = "Run TUI E2E tests"
run = "cargo test --test e2e_ -- --test-threads=1"
```

## Phases

| # | Phase | Scope | Effort |
|---|-------|-------|--------|
| 1 | Refactoring | Make `handle_key` `pub(crate)`, extract `create_test_app`, add `test_helpers` module | Small [x] |
| 2 | Harness | Build `tests/helpers/mod.rs` with key sending + render + assert utilities | Small [x] |
| 3 | Journey 1 | Paint keyboard E2E test | Medium [x] |
| 4 | Journey 2 | Manage layers E2E test | Medium [x] |
| 5 | Journey 3 | Load config (file-based adaptation) E2E test | Small [x] |
| 6 | Journey 4 | Navigation & help overlay E2E test | Medium [x] |
| 7 | Journey 5 | Copy config (clipboard confirm/cancel) E2E test | Small [x] |
| 8 | Docs | Update AGENTS.md project structure, add mise task if separate | Small [x] |

## Out of Scope

- **Snapshot testing** (e.g., `insta`) — possible future enhancement but not needed for parity with web E2E
- **Process-level E2E** (spawn real binary, send PTY input) — too complex for the value; in-process harness gives the same coverage
- **Mouse interaction testing** — TUI is keyboard-only
