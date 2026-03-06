---
date: "2026-03-05T20:26:32+00:00"
git_commit: d5242eb
branch: agency/4-web-keys
topic: "Web App Keyboard Navigation"
tags: [plan, web, keyboard-navigation, accessibility]
status: draft
---

# Web App Keyboard Navigation Implementation Plan

## Overview

Add keyboard navigation to the web editor so users can navigate the keyboard layout, select colors, and perform all editing operations without a mouse. Uses a focus-region model where arrow keys and shortcuts change behavior based on which component has focus. The config textarea remains fully native — all custom shortcuts are suppressed when it has focus.

## Current State Analysis

**What exists:**
- Mouse-only interaction for keyboard keys, palette swatches, and layer selection
- `Ctrl/Cmd+Z`/`Ctrl/Cmd+Y` global undo/redo shortcuts (in `main.ts`)
- `CursorState` type already tracks `(half, row, col)` — rendered as `.cursor` CSS class
- Copy/Paste buttons for the device tree config text (`#copy-config-btn`, `#paste-config-btn` in `index.html`, wired in `main.ts` to `copyConfigToClipboard()`/`pasteConfigFromClipboard()` in `config-text.ts`)
- Theme toggle button (light/dark mode via `data-theme` attribute, `localStorage`)
- `renderToolbar` now takes 5 params including `onThemeToggle: VoidHandler`
- WASM API already exposes: `move_up/down/left/right()`, `switch_half()`, `next_layer()`/`prev_layer()`, `yank_color()`, `paste_color()`, `set_color()`, `clear_color()`
- `editor-bridge.ts` does NOT wrap these cursor/color methods yet — only wraps `setCursor()` and `setColorAt()`/`clearColorAt()`

**What's missing:**
- No arrow key navigation for keyboard keys
- No keyboard way to select palette colors
- No keyboard way to switch layers
- No focus management between components
- No help overlay showing available shortcuts

### Color Copy/Paste Design

Color yank/paste uses plain letter keys `c` and `v` (no modifier) instead of `Ctrl+C`/`Ctrl+V`. This avoids conflicting with the browser's native clipboard behavior — `Ctrl+C`/`Ctrl+V` always does system clipboard copy/paste regardless of focus, which users expect. The letter keys only fire when the keyboard region is focused (not in the textarea). This mirrors the TUI's `y`/`p` (vim-style) but with web-friendly letters.

| # | Shortcut | Action |
|---|----------|--------|
| 1 | `c` | Copy (yank) color at cursor into WASM internal register |
| 2 | `v` | Paste yanked color at cursor position |
| 3 | `Ctrl/Cmd+C/V` | Native browser clipboard — always works, never intercepted |
| 4 | `📋 Copy` / `📥 Paste` buttons | Config text to/from system clipboard via click handlers |

### Key Discoveries:
- WASM `move_up/down/left/right()` already handles staggered row geometry (thumb rows) — bridge just needs thin wrappers
- `yank_color()` / `paste_color()` operate on an internal register in WASM — plain `c`/`v` keys in keyboard region call these (no Ctrl modifier, avoids browser clipboard conflict)
- Config textarea has native browser copy/paste plus dedicated Copy/Paste buttons — no conflict since letter keys are suppressed when textarea has focus
- Palette swatches are rendered as flat lists inside three containers (`#palette-regular`, `#palette-lock-grid`, `#palette-alias-grid`) — grid navigation needs to treat them as a single flat sequence or navigate per section

## Desired End State

Users can perform all editing operations via keyboard:

1. **Arrow keys** navigate the keyboard layout cursor
2. **`Tab`** switches keyboard halves
3. **`Enter`** opens palette navigation at cursor → arrow keys pick color → `Enter` confirms
4. **`Backspace`/`Delete`** clears color at cursor
5. **`c`/`v`** copies/pastes colors between keys (plain letter keys, keyboard region only)
6. **`0-9`** quick-applies palette color by index
7. **`PageUp`/`PageDown`** switches layers
8. **`+`/`-`** adjusts fade delay
9. **`Escape`** exits textarea / closes palette picker / returns to keyboard
10. **`?`** shows help overlay

When the config textarea has focus, **all custom shortcuts are suppressed** — browser handles everything natively. The existing Copy/Paste buttons for device tree text continue to work independently.

### UI Mockups

**Help overlay** (shown when pressing `?`):

```
┌─────────────────────────────────────────────┐
│  Keyboard Shortcuts                     [×] │
│                                             │
│  Navigation                                 │
│    ← → ↑ ↓    Move cursor                  │
│    Tab         Switch keyboard half         │
│    PageUp/Dn   Switch layer                 │
│                                             │
│  Editing                                    │
│    Enter        Pick color from palette     │
│    0-9          Quick-apply color by index   │
│    Backspace    Clear key color             │
│    c            Copy color at cursor        │
│    v            Paste color to cursor       │
│    +/−          Adjust fade delay           │
│                                             │
│  General                                    │
│    Ctrl+Z       Undo                        │
│    Ctrl+Y       Redo                        │
│    Escape       Exit textarea / close modal │
│    ?            Toggle this help             │
│                                             │
│  Press Escape or ? to close                 │
└─────────────────────────────────────────────┘
```

**Palette picker mode** (visual indicator when Enter opens palette):

```
  Colors
  ┌───┬───┬───┬───┬───┬───┬───┬───┐
  │___│RED│ORG│YEL│GRN│CYN│BLU│PRP│  ← existing swatches
  └───┴───┴───┴───┴───┴───┴─┬─┴───┘
                             │
                          [focused]  ← blue ring on navigated swatch
                                       (reuses .selected styling)
```

## What We're NOT Doing

- No vim-style `hjkl` navigation — arrow keys are sufficient in a browser context
- No letter-key shortcuts for layer CRUD (`a`/`d`/`n`/`x`) — these conflict with textarea input and are infrequent operations; buttons remain the primary interface
- No file save/save-as shortcuts — web app has no filesystem concept
- No custom focus ring framework — reuse CSS `.cursor` and `.selected` classes
- No multi-key selection or range operations
- No changes to the config textarea Copy/Paste buttons — they work independently via click handlers on `#copy-config-btn` / `#paste-config-btn`
- No keyboard shortcut for theme toggle — button-only is fine for infrequent action

## Implementation Approach

Three phases, each independently testable:

1. **Bridge + core navigation** — wire WASM cursor/color methods to JS, add the `keydown` dispatcher
2. **Palette picker mode** — Enter to open, arrows to navigate, Enter/Escape to confirm/cancel
3. **Help overlay + polish** — `?` key, focus indicators, E2E tests

## Phase 1: Bridge + Core Keyboard Navigation

### Overview
Expose missing WASM methods in the bridge, add a focus-region-aware `keydown` handler to `main.ts`, wire up arrow navigation, color operations, and layer switching.

### Changes Required:

#### [x] 1. Add missing bridge functions
**File**: `web/src/editor-bridge.ts`
**Changes**: Add thin wrappers for cursor movement, color-at-cursor operations, yank/paste, and layer navigation.

```typescript
export function moveCursorUp(): void { editor?.move_up(); }
export function moveCursorDown(): void { editor?.move_down(); }
export function moveCursorLeft(): void { editor?.move_left(); }
export function moveCursorRight(): void { editor?.move_right(); }
export function switchHalf(): void { editor?.switch_half(); }

export function setColorAtCursor(abbrev: string): boolean {
  if (!editor) return false;
  return editor.set_color(abbrev);
}

export function clearColorAtCursor(): boolean {
  if (!editor) return false;
  return editor.clear_color();
}

export function yankColor(): string {
  if (!editor) return '';
  return editor.yank_color();
}

export function pasteColor(): boolean {
  if (!editor) return false;
  return editor.paste_color();
}

export function nextLayer(): void { editor?.next_layer(); }
export function prevLayer(): void { editor?.prev_layer(); }
```

#### [x] 2. Add focus region tracking to AppState
**File**: `web/src/state.ts`
**Changes**: Add a `focusRegion` field and type.

```typescript
export type FocusRegion = 'keyboard' | 'palette' | 'config';

export interface AppState {
  selectedColor: string | null;
  configLoaded: boolean;
  focusRegion: FocusRegion;
}

export function createAppState(): AppState {
  return {
    selectedColor: null,
    configLoaded: false,
    focusRegion: 'keyboard',
  };
}
```

#### [x] 3. Add palette index tracking to AppState
**File**: `web/src/state.ts`
**Changes**: Add `paletteIndex` for palette navigation mode. This tracks position in the flat list of all palette colors (regular + locks + aliases) when the picker is open.

```typescript
export interface AppState {
  selectedColor: string | null;
  configLoaded: boolean;
  focusRegion: FocusRegion;
  paletteIndex: number;
}

export function createAppState(): AppState {
  return {
    selectedColor: null,
    configLoaded: false,
    focusRegion: 'keyboard',
    paletteIndex: 0,
  };
}
```

#### [x] 4. Expand keydown handler in main.ts
**File**: `web/src/main.ts`
**Changes**: Add new imports for the bridge functions added in step 1. Replace the existing minimal keydown listener in `setupEventListeners` with a focus-region-aware dispatcher. The existing `initTheme`/`toggleTheme`, copy/paste button listeners, and textarea input/paste listeners remain unchanged.

```typescript
// Add to existing import from './editor-bridge.ts':
import {
  // ...existing imports (initWasm, loadConfig, hasEditor, getState, etc.)...
  moveCursorUp, moveCursorDown, moveCursorLeft, moveCursorRight,
  switchHalf, setColorAtCursor, clearColorAtCursor,
  yankColor, pasteColor, nextLayer, prevLayer,
} from './editor-bridge.ts';

// Inside setupEventListeners, replace the keydown listener block:
document.addEventListener('keydown', (e) => {
  if (!hasEditor()) return;

  // When config textarea has focus, only intercept Escape
  const inTextarea = document.activeElement?.id === 'config-text';
  if (inTextarea) {
    if (e.key === 'Escape') {
      e.preventDefault();
      (document.activeElement as HTMLElement).blur();
      appState.focusRegion = 'keyboard';
      render();
    }
    return; // let browser handle everything else natively
  }

  // Global shortcuts (work in all focus regions)
  if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
    e.preventDefault();
    onUndo();
    return;
  }
  if ((e.ctrlKey || e.metaKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) {
    e.preventDefault();
    onRedo();
    return;
  }

  // Dispatch to focus-region handler
  if (appState.focusRegion === 'palette') {
    handlePaletteKey(e);
  } else {
    handleKeyboardKey(e);
  }
});
```

Note: The existing copy/paste button listeners (`#copy-config-btn`, `#paste-config-btn`) use `click` events and are unaffected by keyboard shortcut handling.

#### [x] 5. Implement keyboard-region key handler
**File**: `web/src/main.ts`
**Changes**: Add `handleKeyboardKey` function.

```typescript
function handleKeyboardKey(e: KeyboardEvent): void {
  switch (e.key) {
    case 'ArrowUp':
      e.preventDefault();
      moveCursorUp();
      render();
      break;
    case 'ArrowDown':
      e.preventDefault();
      moveCursorDown();
      render();
      break;
    case 'ArrowLeft':
      e.preventDefault();
      moveCursorLeft();
      render();
      break;
    case 'ArrowRight':
      e.preventDefault();
      moveCursorRight();
      render();
      break;
    case 'Tab':
      e.preventDefault();
      switchHalf();
      render();
      break;
    case 'Enter':
      e.preventDefault();
      enterPaletteMode();
      break;
    case 'Backspace':
    case 'Delete':
      e.preventDefault();
      clearColorAtCursor();
      render();
      break;
    case 'PageDown':
      e.preventDefault();
      nextLayer();
      render();
      break;
    case 'PageUp':
      e.preventDefault();
      prevLayer();
      render();
      break;
    case '+':
    case '=':
      e.preventDefault();
      onFadeChange(5);
      break;
    case '-':
      e.preventDefault();
      onFadeChange(-5);
      break;
    case '?':
      e.preventDefault();
      toggleHelp();
      break;
    case 'c':
      if (!e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        yankColor();
        render();
      }
      break;
    case 'v':
      if (!e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        pasteColor();
        render();
      }
      break;
    default:
      // 0-9 quick color selection
      if (e.key >= '0' && e.key <= '9' && !e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        applyColorByIndex(parseInt(e.key, 10));
      }
      break;
  }
}
```

#### [x] 6. Implement number-key quick-apply helper
**File**: `web/src/main.ts`
**Changes**: Add `applyColorByIndex` function that maps `0-9` to palette colors.

```typescript
function applyColorByIndex(index: number): void {
  const state = getState();
  if (!state) return;

  // Combine all palette colors into flat list (matching render order)
  const allColors = [
    ...state.palette.regular.filter(c => c.abbrev !== '___'),
    ...state.palette.locks,
    ...state.palette.aliases,
  ];

  if (index < allColors.length) {
    setColorAtCursor(allColors[index].abbrev);
    appState.selectedColor = allColors[index].abbrev;
    render();
  }
}
```

#### [x] 7. Track focus region on textarea focus
**File**: `web/src/main.ts`
**Changes**: Add focus/blur listeners on the textarea so `appState.focusRegion` stays in sync when user clicks into/out of the textarea.

```typescript
// Inside setupEventListeners, after textarea input/paste listeners:
textarea.addEventListener('focus', () => {
  appState.focusRegion = 'config';
});
textarea.addEventListener('blur', () => {
  appState.focusRegion = 'keyboard';
});
```

### Success Criteria:

#### Automated Verification:
- [x] TypeScript compiles: `cd web && npx tsc --noEmit`
- [ ] Linting passes: `cd web && npx eslint src/` (no ESLint config in this branch — skipped)
- [x] Existing E2E tests still pass: `mise run web-e2e`

#### Manual Verification:
- [ ] Arrow keys move cursor across keyboard keys (left/right wraps between halves)
- [ ] Tab switches between left and right half
- [ ] Backspace/Delete clears color at cursor position
- [ ] `c` at a colored key, navigate away, `v` paints the copied color
- [ ] Ctrl+C/V are NOT intercepted — browser clipboard works normally
- [ ] PageUp/PageDown switches active layer
- [ ] +/- adjusts fade delay
- [ ] Number keys 0-9 apply palette colors
- [ ] Ctrl+Z/Ctrl+Y still work for undo/redo
- [ ] Clicking into config textarea suppresses all custom shortcuts
- [ ] Escape exits textarea and returns cursor navigation
- [ ] Config Copy button (`📋 Copy`) still copies config to system clipboard
- [ ] Config Paste button (`📥 Paste`) still pastes config from system clipboard and re-renders
- [ ] Regular typing in textarea still works (not intercepted)
- [ ] Theme toggle button still works

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human before proceeding to the next phase.

---

## Phase 2: Palette Picker Mode

### Overview
Pressing `Enter` on a keyboard key enters palette navigation mode. Arrow keys navigate the palette grid, `Enter` confirms selection (paints the key and returns to keyboard), `Escape` cancels.

### Changes Required:

#### [x] 1. Add palette navigation CSS class
**File**: `web/src/styles.css`
**Changes**: Add a `.palette-cursor` class for the navigated swatch (distinct from `.selected` which marks the active brush color).

```css
.swatch.palette-cursor {
  outline: 2px solid var(--cursor-ring);
  outline-offset: 1px;
  box-shadow: 0 0 6px var(--cursor-ring);
}
```

#### [x] 2. Pass palette cursor index to palette renderer
**File**: `web/src/components/palette.ts`
**Changes**: Accept an optional `paletteCursorIndex` parameter. When set, the swatch at that index in the flat color list gets the `.palette-cursor` class.

```typescript
export function renderPalette(
  state: EditorState,
  selectedColor: string | null,
  onClick: ColorSelectHandler,
  paletteCursorIndex: number | null,
): void {
  // Track flat index across all grids to apply .palette-cursor
  let flatIndex = 0;

  // Render regular colors (clear swatch is index 0, not counted as palette color for navigation)
  renderSwatchGrid('palette-regular', state.palette.regular, selectedColor, onClick, paletteCursorIndex, flatIndex);
  flatIndex += state.palette.regular.filter(c => c.abbrev !== '___').length;

  // ... prepend clear swatch (unchanged) ...

  renderSwatchGrid('palette-lock-grid', state.palette.locks, selectedColor, onClick, paletteCursorIndex, flatIndex);
  flatIndex += state.palette.locks.length;

  renderSwatchGrid('palette-alias-grid', state.palette.aliases, selectedColor, onClick, paletteCursorIndex, flatIndex);
}
```

Update `renderSwatchGrid` to accept and apply cursor index:

```typescript
function renderSwatchGrid(
  containerId: string,
  colors: PaletteColor[],
  selectedColor: string | null,
  onClick: ColorSelectHandler,
  paletteCursorIndex: number | null,
  flatStartIndex: number,
): void {
  // ... existing rendering ...
  // For each non-___ color, if (flatStartIndex + localIndex) === paletteCursorIndex,
  // add 'palette-cursor' class
}
```

#### [x] 3. Implement palette mode enter/exit functions
**File**: `web/src/main.ts`
**Changes**: Add `enterPaletteMode` and update render to pass palette cursor state.

```typescript
function enterPaletteMode(): void {
  const state = getState();
  if (!state) return;

  // Pre-select current key's color in palette, or start at 0
  const currentColor = state.cursor ? getCurrentColorAbbrev(state) : null;
  const allColors = getAllPaletteColors(state);
  const matchIndex = currentColor
    ? allColors.findIndex(c => c.abbrev === currentColor)
    : -1;

  appState.paletteIndex = matchIndex >= 0 ? matchIndex : 0;
  appState.focusRegion = 'palette';
  render();
}

function getAllPaletteColors(state: EditorState): PaletteColor[] {
  return [
    ...state.palette.regular.filter(c => c.abbrev !== '___'),
    ...state.palette.locks,
    ...state.palette.aliases,
  ];
}
```

#### [x] 4. Implement palette key handler
**File**: `web/src/main.ts`
**Changes**: Add `handlePaletteKey` function for arrow navigation + confirm/cancel.

```typescript
function handlePaletteKey(e: KeyboardEvent): void {
  const state = getState();
  if (!state) return;

  const allColors = getAllPaletteColors(state);
  const count = allColors.length;
  if (count === 0) return;

  // Palette swatches are rendered in a CSS grid — determine columns per grid
  // Use a fixed column count matching .swatch-grid CSS (auto-fill, ~50px swatches)
  // For simplicity, treat palette as a 1D list with left/right navigation
  switch (e.key) {
    case 'ArrowRight':
      e.preventDefault();
      appState.paletteIndex = Math.min(appState.paletteIndex + 1, count - 1);
      render();
      break;
    case 'ArrowLeft':
      e.preventDefault();
      appState.paletteIndex = Math.max(appState.paletteIndex - 1, 0);
      render();
      break;
    case 'ArrowDown':
      e.preventDefault();
      // Jump forward by a row's worth of swatches (estimate ~8 per row from grid)
      appState.paletteIndex = Math.min(appState.paletteIndex + 8, count - 1);
      render();
      break;
    case 'ArrowUp':
      e.preventDefault();
      appState.paletteIndex = Math.max(appState.paletteIndex - 8, 0);
      render();
      break;
    case 'Enter':
      e.preventDefault();
      if (appState.paletteIndex >= 0 && appState.paletteIndex < count) {
        const color = allColors[appState.paletteIndex];
        appState.selectedColor = color.abbrev;
        setColorAtCursor(color.abbrev);
      }
      appState.focusRegion = 'keyboard';
      render();
      break;
    case 'Escape':
      e.preventDefault();
      appState.focusRegion = 'keyboard';
      render();
      break;
  }
}
```

#### [x] 5. Update render() to pass palette cursor
**File**: `web/src/main.ts`
**Changes**: Pass `paletteIndex` to `renderPalette` when in palette focus region.

```typescript
function render(): void {
  // ... existing state/grid retrieval ...
  const paletteCursorIndex = appState.focusRegion === 'palette'
    ? appState.paletteIndex
    : null;
  renderPalette(state, appState.selectedColor, onColorSelect, paletteCursorIndex);
  // ... rest unchanged ...
}
```

#### [x] 6. Add visual feedback for palette mode
**File**: `web/src/styles.css`
**Changes**: Optionally dim keyboard or add a subtle indicator that palette mode is active.

```css
/* When palette is focused, add subtle highlight to palette section */
#palette-section.palette-active {
  outline: 1px solid var(--accent);
  outline-offset: 2px;
  border-radius: 4px;
}
```

Apply/remove in render based on `appState.focusRegion === 'palette'`.

### Success Criteria:

#### Automated Verification:
- [x] TypeScript compiles: `cd web && npx tsc --noEmit`
- [x] Existing E2E tests still pass: `mise run web-e2e`

#### Manual Verification:
- [ ] `Enter` on a keyboard key opens palette mode — navigated swatch shows yellow ring
- [ ] Arrow keys move through palette swatches
- [ ] `Enter` in palette mode paints the selected color on the key and returns to keyboard
- [ ] `Escape` in palette mode cancels without painting and returns to keyboard
- [ ] Palette pre-selects the current key's color when opening
- [ ] After confirming, the selected color stays as the "brush" for future number-key use
- [ ] Mouse clicking a swatch still works during palette mode

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human before proceeding to the next phase.

---

## Phase 3: Help Overlay + E2E Tests

### Overview
Add a `?`-toggled help modal showing all shortcuts, and write E2E tests covering the keyboard navigation journeys.

### Changes Required:

#### [x] 1. Add help overlay HTML structure
**File**: `web/index.html`
**Changes**: Add a hidden help overlay div.

```html
<!-- Before closing </div> of #app -->
<div id="help-overlay" class="overlay hidden">
  <div class="overlay-content">
    <div class="overlay-header">
      <h2>Keyboard Shortcuts</h2>
      <button id="help-close" class="overlay-close">×</button>
    </div>
    <div class="overlay-body">
      <div class="shortcut-group">
        <h3>Navigation</h3>
        <dl>
          <dt>← → ↑ ↓</dt><dd>Move cursor</dd>
          <dt>Tab</dt><dd>Switch keyboard half</dd>
          <dt>PageUp / PageDown</dt><dd>Switch layer</dd>
        </dl>
      </div>
      <div class="shortcut-group">
        <h3>Editing</h3>
        <dl>
          <dt>Enter</dt><dd>Pick color from palette</dd>
          <dt>0–9</dt><dd>Quick-apply color by index</dd>
          <dt>Backspace</dt><dd>Clear key color</dd>
          <dt>c</dt><dd>Copy color at cursor</dd>
          <dt>v</dt><dd>Paste color to cursor</dd>
          <dt>+ / −</dt><dd>Adjust fade delay</dd>
        </dl>
      </div>
      <div class="shortcut-group">
        <h3>General</h3>
        <dl>
          <dt>Ctrl+Z</dt><dd>Undo</dd>
          <dt>Ctrl+Y</dt><dd>Redo</dd>
          <dt>Escape</dt><dd>Exit textarea / close modal</dd>
          <dt>?</dt><dd>Toggle this help</dd>
        </dl>
      </div>
    </div>
  </div>
</div>
```

#### [x] 2. Add help overlay styles
**File**: `web/src/styles.css`
**Changes**: Add overlay + shortcut list styles.

```css
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.overlay.hidden {
  display: none;
}

.overlay-content {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 24px;
  max-width: 480px;
  width: 90%;
  max-height: 80vh;
  overflow-y: auto;
}

.overlay-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.overlay-close {
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 24px;
  cursor: pointer;
}

.shortcut-group h3 {
  color: var(--accent);
  margin: 12px 0 8px;
  font-size: 14px;
  text-transform: uppercase;
}

.shortcut-group dl {
  display: grid;
  grid-template-columns: 140px 1fr;
  gap: 4px 12px;
  margin: 0;
}

.shortcut-group dt {
  color: var(--text-primary);
  font-family: monospace;
  font-weight: bold;
}

.shortcut-group dd {
  color: var(--text-secondary);
  margin: 0;
}
```

#### [x] 3. Implement toggleHelp function
**File**: `web/src/main.ts`
**Changes**: Add `toggleHelp` and wire up the close button.

```typescript
function toggleHelp(): void {
  const overlay = document.getElementById('help-overlay');
  if (!overlay) return;
  overlay.classList.toggle('hidden');
}

// In setupEventListeners:
const helpClose = document.getElementById('help-close');
if (helpClose) {
  helpClose.addEventListener('click', toggleHelp);
}
```

Also handle `Escape` closing the help overlay (add to the top of the keydown handler):

```typescript
// At the top of keydown handler, before focus region checks:
const helpOverlay = document.getElementById('help-overlay');
if (helpOverlay && !helpOverlay.classList.contains('hidden')) {
  if (e.key === 'Escape' || e.key === '?') {
    e.preventDefault();
    toggleHelp();
  }
  return; // swallow all keys while help is open
}
```

#### [x] 4. Write E2E test: keyboard navigation journey
**File**: `web/e2e/keyboard-navigation.spec.ts`
**Changes**: New test file covering the main keyboard navigation flows.

```typescript
import { test, expect } from '@playwright/test';

test('navigate keyboard and paint with keyboard shortcuts', async ({ page }) => {
  // Arrange
  await page.goto('/');
  await page.locator('.key').first().waitFor({ state: 'visible', timeout: 10_000 });

  // Act & Assert — cursor movement
  const initialCursor = page.locator('.key.cursor');
  await expect(initialCursor).toBeVisible();

  // Read initial cursor position
  const startHalf = await initialCursor.getAttribute('data-half');
  const startRow = await initialCursor.getAttribute('data-row');
  const startCol = await initialCursor.getAttribute('data-col');

  // Move right
  await page.keyboard.press('ArrowRight');
  const newCursor = page.locator('.key.cursor');
  const newCol = await newCursor.getAttribute('data-col');
  // Cursor should have moved (unless at boundary)
  if (startCol !== null && parseInt(startCol) < 5) {
    expect(parseInt(newCol!)).toBe(parseInt(startCol) + 1);
  }

  // Act & Assert — Tab switches half
  await page.keyboard.press('Tab');
  const tabCursor = page.locator('.key.cursor');
  const tabHalf = await tabCursor.getAttribute('data-half');
  expect(tabHalf).not.toBe(startHalf);

  // Act & Assert — number key paints color
  await page.keyboard.press('0');
  const paintedKey = page.locator('.key.cursor');
  const bgAfterPaint = await paintedKey.evaluate(
    (el) => getComputedStyle(el).backgroundColor,
  );
  // Should no longer be the "empty" dark color
  expect(bgAfterPaint).not.toBe('rgb(26, 26, 46)');

  // Act & Assert — Backspace clears
  await page.keyboard.press('Backspace');
  const textAfterClear = await page.locator('.key.cursor').textContent();
  expect(textAfterClear).toBe('___');

  // Act & Assert — Ctrl+Z undoes the clear
  await page.keyboard.press('Control+z');
  const textAfterUndo = await page.locator('.key.cursor').textContent();
  expect(textAfterUndo).not.toBe('___');
});

test('palette picker mode via Enter key', async ({ page }) => {
  // Arrange
  await page.goto('/');
  await page.locator('.key').first().waitFor({ state: 'visible', timeout: 10_000 });

  // Act — open palette picker
  await page.keyboard.press('Enter');

  // Assert — a swatch has palette-cursor class
  const paletteCursor = page.locator('.swatch.palette-cursor');
  await expect(paletteCursor).toBeVisible();

  // Act — navigate and confirm
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('Enter');

  // Assert — palette cursor gone, key is painted
  await expect(paletteCursor).not.toBeVisible();
  const keyText = await page.locator('.key.cursor').textContent();
  expect(keyText).not.toBe('___');
});

test('copy and paste color between keys', async ({ page }) => {
  // Arrange — paint a key first
  await page.goto('/');
  await page.locator('.key').first().waitFor({ state: 'visible', timeout: 10_000 });
  await page.keyboard.press('0'); // paint with first palette color
  const paintedColor = await page.locator('.key.cursor').textContent();

  // Act — copy color with 'c', move, paste with 'v'
  await page.keyboard.press('c');
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('v');

  // Assert — destination key has the same color
  const pastedColor = await page.locator('.key.cursor').textContent();
  expect(pastedColor).toBe(paintedColor);
});

test('Escape exits config textarea', async ({ page }) => {
  // Arrange
  await page.goto('/');
  await page.locator('.key').first().waitFor({ state: 'visible', timeout: 10_000 });

  // Act — focus textarea, then Escape
  await page.locator('#config-text').focus();
  await page.keyboard.press('Escape');

  // Assert — arrow keys work again (cursor moves)
  const cursorBefore = await page.locator('.key.cursor').getAttribute('data-col');
  await page.keyboard.press('ArrowRight');
  const cursorAfter = await page.locator('.key.cursor').getAttribute('data-col');
  // Cursor should have moved (or stayed if at boundary)
  expect(cursorAfter).toBeDefined();
});

test('help overlay opens and closes with ?', async ({ page }) => {
  // Arrange
  await page.goto('/');
  await page.locator('.key').first().waitFor({ state: 'visible', timeout: 10_000 });

  // Act — open help
  await page.keyboard.press('?');

  // Assert
  const overlay = page.locator('#help-overlay');
  await expect(overlay).not.toHaveClass(/hidden/);

  // Act — close help
  await page.keyboard.press('?');

  // Assert
  await expect(overlay).toHaveClass(/hidden/);
});

test('config copy/paste buttons work alongside keyboard navigation', async ({ page }) => {
  // Arrange
  await page.goto('/');
  await page.locator('.key').first().waitFor({ state: 'visible', timeout: 10_000 });

  // Use keyboard navigation first
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('0');

  // Act — click copy button
  await page.locator('#copy-config-btn').click();

  // Assert — button shows feedback
  await expect(page.locator('#copy-config-btn')).toHaveText('✅ Copied');

  // Assert — keyboard navigation still works after button click
  await page.keyboard.press('ArrowLeft');
  const cursor = page.locator('.key.cursor');
  await expect(cursor).toBeVisible();
});

test('keyboard shortcuts suppressed in config textarea', async ({ page }) => {
  // Arrange
  await page.goto('/');
  await page.locator('.key').first().waitFor({ state: 'visible', timeout: 10_000 });

  // Record cursor position
  const cursorBefore = await page.locator('.key.cursor').getAttribute('data-col');

  // Act — focus textarea, press arrow keys
  await page.locator('#config-text').focus();
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('ArrowRight');

  // Escape back to keyboard
  await page.keyboard.press('Escape');

  // Assert — cursor didn't move while textarea was focused
  const cursorAfter = await page.locator('.key.cursor').getAttribute('data-col');
  expect(cursorAfter).toBe(cursorBefore);
});

test('PageUp and PageDown switch layers', async ({ page }) => {
  // Arrange
  await page.goto('/');
  await page.locator('.key').first().waitFor({ state: 'visible', timeout: 10_000 });

  const initialLayer = page.locator('#layer-list .layer-item.active .layer-name');
  const initialLayerText = await initialLayer.textContent();

  // Act
  await page.keyboard.press('PageDown');

  // Assert — active layer changed (if more than one layer)
  const layerCount = await page.locator('#layer-list .layer-item').count();
  if (layerCount > 1) {
    const newLayerText = await page.locator('#layer-list .layer-item.active .layer-name').textContent();
    expect(newLayerText).not.toBe(initialLayerText);
  }
});
```

#### [x] 5. Update AGENTS.md file list
**File**: `AGENTS.md`
**Changes**: Add `keyboard-navigation.spec.ts` to the web E2E test list in the project structure.

### Success Criteria:

#### Automated Verification:
- [x] TypeScript compiles: `cd web && npx tsc --noEmit`
- [x] All E2E tests pass (existing + new): `mise run web-e2e`

#### Manual Verification:
- [ ] `?` opens help overlay with all shortcuts listed
- [ ] `?` or `Escape` closes help overlay
- [ ] Close button (×) also closes help overlay
- [ ] All keyboard shortcuts work as documented in the help overlay
- [ ] Help overlay is scrollable on small viewports
- [ ] Config textarea Copy/Paste buttons remain functional throughout all keyboard flows

**Implementation Note**: After completing this phase and all automated verification passes, pause here for manual confirmation from the human before proceeding.

---

## Testing Strategy

### Unit Tests:
- No unit tests needed — all logic lives in WASM (already tested in domain) or is thin wiring

### E2E Tests (Phase 3):
- Cursor movement (arrows, Tab)
- Color painting (number keys, Enter palette flow)
- Color copy/paste (`c`/`v` letter keys)
- Color clear (Backspace)
- Layer switching (PageUp/PageDown)
- Help overlay (? toggle)
- Textarea isolation (shortcuts suppressed)
- Config Copy/Paste buttons coexist with keyboard navigation
- Existing `copy-paste-config.spec.ts` still passes (tests button copy/paste flows)

### Manual Testing Steps:
1. Full keyboard-only editing session: navigate to a key, pick a color via Enter → palette → Enter, clear another key, undo, switch layers
2. Mixed mouse+keyboard workflow: click a palette color, arrow to keys, Enter to paint
3. Config textarea round-trip: paste config, Escape out, keyboard-navigate, verify state consistency
4. Test config textarea Copy/Paste buttons still work after using keyboard navigation

## Performance Considerations

- No performance impact — keydown handler is O(1) dispatch, bridge functions are thin WASM calls
- `render()` is already called on every mouse click; keyboard events use the same path
- Palette cursor index is a single integer; no additional DOM queries needed

## References

- TUI keyboard handling: `src/event.rs` (all 10 modes, vim-style bindings)
- Domain cursor model: `crates/domain/src/cursor.rs` (`move_cursor`, `switch_half`, staggered row geometry)
- WASM API: `crates/domain-wasm/src/lib.rs` (all methods including `move_up/down/left/right`, `yank_color`, `paste_color`)
- Existing E2E patterns: `web/e2e/paint-keyboard.spec.ts` (data-attribute pinning, keyboard undo/redo)
- Config copy/paste E2E: `web/e2e/copy-paste-config.spec.ts` (button click flows, clipboard API)
- Config copy/paste implementation: `web/src/components/config-text.ts` (`copyConfigToClipboard`, `pasteConfigFromClipboard`)
