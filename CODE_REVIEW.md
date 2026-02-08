# Code Review — Go60 RGB Editor

Codebase: ~4050 lines of Rust across 18 source files.  
A TUI application for editing per-key RGB underglow configs for ZMK keyboards.

---

## Resolved Since Last Review

The following issues from the previous review have been addressed:

- **1.1 `lexer.rs` (363 lines dead code)** — Removed entirely ✅
- **1.2 `#[allow(dead_code)]` scattered across codebase** — All removed; test-only methods now use `#[cfg(test)]` ✅
- **1.2 Unused `centered_rect()` (percentage-based)** — Removed ✅
- **1.3 `ColorDef.rgb_name` field** — Removed from model ✅
- **2.2 `move_color_selection()` high complexity** — Refactored with `categorize_palette_colors()`, `move_within_section()`, and section-based navigation ✅
- **3.1 `centered_rect_fixed`** — Renamed to `centered_rect_for_text` ✅
- **3.2 `quick_color()`** — Renamed to `apply_quick_color()` ✅
- **3.3 `parse_rgb_define` unnamed tuple** — Replaced with `RgbDefinition` struct ✅
- **3.4 `copied_color`** — Renamed to `copied_color_abbrev` ✅
- **3.5 `from_visual_col`** — Renamed to `visual_to_data_col` (fixes clippy `wrong_self_convention`) ✅
- **4.1 `move_color_selection()` mixed concerns** — Categorization extracted into `categorize_palette_colors()` ✅
- **5.1 Left/Right navigation duplication** — Unified into `move_within_section()` ✅
- **5.2 Key row formatting in `writer.rs`** — Extracted `format_key_row()` helper ✅
- **5.3 `increase_fade`/`decrease_fade` duplication** — Unified into `adjust_fade(delta)` ✅
- **6.1 Redundant doc comments** — Removed from `app.rs`, `model/config.rs`, `model/layer.rs` ✅
- **6.2 `push_undo()` stack limit** — Now uses `MAX_UNDO_HISTORY` constant ✅
- **7.1 Magic numbers (partial)** — Constants added: `MAX_UNDO_HISTORY`, `FADE_STEP_MS`, `STATUS_TIMEOUT_SECS`, `COLORS_PER_PICKER_ROW`, `KEY_CELL_WIDTH`, `HALF_GAP`, `MIN_TERMINAL_WIDTH`, `MIN_TERMINAL_HEIGHT`, `TICK_RATE_MS` ✅
- **7.3 `lexer.rs` vs `grammar.rs` parallel parsers** — Resolved by removing `lexer.rs` ✅
- **`tick()` renamed** — Renamed to `clear_expired_status()` ✅
- **Clippy auto-fixable warnings** — Fixed `Error::other`, `map_or` → `== Some(…)` / `ends_with` / `.is_some_and(…)`, `if let` → `.map(…)` ✅
- **`COLORS_PER_PICKER_ROW` duplicated** — Deduplicated into `model/layer.rs`, imported by `app.rs` and `color_picker.rs` ✅
- **Hardcoded color picker row limit** — Replaced `if row > 1` with `MAX_REGULAR_COLOR_ROWS` constant ✅

---

## Remaining Issues

### 1. Clippy Warnings (4 remaining)

`cargo clippy` reports 4 warnings (down from 9):

| Location | Warning | Fix |
|---|---|---|
| `keyboard.rs:29` | `render_half_row` — too many arguments (8/7) | Group into a struct or reduce params |
| `keyboard.rs:34` | Needless range loop with manual indexing | Use iterator + enumerate |
| `color_picker.rs:27` | `render_labeled_section` — too many arguments (9/7) | Group into a struct or reduce params |
| `ui/mod.rs:121` | `render_fixed_modal` — too many arguments (8/7) | Group into a struct or reduce params |

### 2. Single Level of Abstraction Violations

#### 2.1 `parse_colors()` in `grammar.rs` — Two-Pass Logic (~90 lines)

The first pass (collecting `_RGB` defines) uses a helper `parse_rgb_define()`, but the second pass has all parsing logic inline with raw `parts[2]`/`parts[3]` indexing. The three binding types (`&ug`, `&ug_sl`/`&ug_nl`/`&ug_cl`, alias) should each be extracted.

**Recommendation:** Extract `parse_underglow_binding()`, `parse_lock_indicator()`, and `parse_alias()` as counterparts to `parse_rgb_define()`.

#### 2.2 `draw()` in `ui/mod.rs` — Modal Rendering

The top-level `draw()` function (lines 21–104) mixes layout setup with inline modal rendering for 5 dialog types. The `render_modal()` and `render_fixed_modal()` helpers exist but are called inline with string literals and parameter lists.

#### 2.3 `copy_to_clipboard()` in `app.rs`

Mixes platform detection (trying pbcopy → xclip → xsel) with pipe writing and error handling in one method. Could extract `spawn_clipboard_process()`.

### 3. Naming Issues

No remaining naming issues.

### 4. Code Duplication

#### 4.1 Left/Right Half Rendering in `keyboard.rs`

The refactored `render_half_row()` helper resolved much duplication, but the three row groups in `Widget::render` (main rows, row 4, row 5) each compute their own X offsets with inline arithmetic. The coordinate computation could be further clarified.

#### 4.2 Lock Indicator / Alias Sections in `color_picker.rs`

The `render_labeled_section()` helper handles this now but has 9 parameters — a sign it may benefit from a `LabeledSection` struct.

### 5. Complexity / Structure

#### 5.1 `App` as God Object (1489 lines)

`App` still holds all state and behavior: cursor, config, undo/redo, clipboard, file I/O, color picker navigation, fade management. While the method count is reasonable, the struct is growing. As the application grows, consider separating:
- Cursor management (including visual↔data mapping)
- Undo/redo state machine
- File operations

#### 5.2 `parse_colors()` in `grammar.rs` (~90 lines)

Still has two-pass logic at mixed abstraction levels (see 2.1 above).

### 6. Other Findings

#### 6.1 `render_half_row` Ignores Out-of-Range `row` Silently

In `keyboard.rs:33`, if `row >= half_keys.len()`, the method silently does nothing. This is fine defensively, but the caller always passes valid row indices, making the guard redundant noise. Consider either removing it or documenting why it's there.

#### 6.2 `parser/tests.rs` Uses `include_str!` on External File

The test file `Go60 TK Latest RGB scheme.txt` is included via `include_str!` at compile time. If the file is moved or renamed, compilation fails with an opaque error. The path has spaces, which is fragile. Consider moving it to a `tests/fixtures/` directory with a conventional name.

#### 6.3 Potential Panic in `parse_colors()` Second Pass

In `grammar.rs:91`, `parts[3]` and `parts[4]` are accessed after only checking `parts.len() >= 5`, but `parts` comes from `line.split_whitespace()` which includes the `#define` token. If the line has exactly 5 whitespace-separated tokens, `parts[3]` and `parts[4]` are the 4th and 5th — this is correct, but fragile. Named indices or destructuring would make intent clearer.

---

## Summary of Priorities

| Priority | Issue | Impact |
|---|---|---|
| 🟡 Medium | Extract `parse_colors()` second pass into named functions (2.1) | Readability |
| 🟡 Medium | Reduce argument count on `render_half_row`, `render_labeled_section`, `render_fixed_modal` (1) | Clippy compliance |
| 🟡 Medium | Move test fixture to `tests/fixtures/` (6.2) | Build robustness |
| 🟢 Low | Extract `spawn_clipboard_process()` from `copy_to_clipboard()` (2.3) | Single level of abstraction |
| 🟢 Low | Consider splitting `App` as it grows (5.1) | Long-term maintainability |
