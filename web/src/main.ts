import { initWasm, loadConfig, hasEditor, getState, getLayerGrid, serialize, setLayer, setCursor, setColorAt, clearColorAt, editorUndo, editorRedo, adjustFade, addLayer, duplicateLayer, renameLayer, deleteLayer, moveCursorUp, moveCursorDown, moveCursorLeft, moveCursorRight, switchHalf, setColorAtCursor, clearColorAtCursor, yankColor, pasteColor, nextLayer, prevLayer } from './editor-bridge.ts';
import { type AppState, type EditorState, type PaletteColor, createAppState } from './state.ts';
import { renderKeyboard } from './components/keyboard.ts';
import { renderPalette } from './components/palette.ts';
import { renderLayers, type LayerAction } from './components/layers.ts';
import { updateConfigText, copyConfigToClipboard, pasteConfigFromClipboard, openConfigFile } from './components/config-text.ts';
import { renderToolbar } from './components/toolbar.ts';
import './styles.css';
import defaultConfig from '../../Go60 TK Latest RGB scheme.txt?raw';

const appState: AppState = createAppState();
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

function initTheme(): void {
  const stored = localStorage.getItem('theme');
  if (stored === 'light' || stored === 'dark') {
    document.documentElement.setAttribute('data-theme', stored);
  } else if (window.matchMedia('(prefers-color-scheme: light)').matches) {
    document.documentElement.setAttribute('data-theme', 'light');
  } else {
    document.documentElement.setAttribute('data-theme', 'dark');
  }
}

function toggleTheme(): void {
  const current = document.documentElement.getAttribute('data-theme');
  const next = current === 'light' ? 'dark' : 'light';
  document.documentElement.setAttribute('data-theme', next);
  localStorage.setItem('theme', next);
  render();
}

async function main(): Promise<void> {
  initTheme();
  await initWasm();
  setupEventListeners();
  renderFromConfig(defaultConfig);
}

function render(): void {
  if (!hasEditor()) return;

  const state = getState();
  if (!state) return;

  const grid = getLayerGrid(state.currentLayerIndex);
  if (!grid) return;

  renderKeyboard(state, grid, onKeyClick);

  const paletteCursorIndex = appState.focusRegion === 'palette'
    ? appState.paletteIndex
    : null;
  renderPalette(state, appState.selectedColor, onColorSelect, paletteCursorIndex);

  const paletteSection = document.getElementById('palette-section');
  if (paletteSection) {
    paletteSection.classList.toggle('palette-active', appState.focusRegion === 'palette');
  }

  renderLayers(state, onLayerSelect, onLayerAction);
  renderToolbar(state, onUndo, onRedo, onFadeChange, toggleTheme, toggleHelp, appState.interactionMode, toggleInteractionMode);
  updateConfigText(serialize());
}

function renderFromConfig(text: string): void {
  const error = loadConfig(text);
  const statusEl = document.getElementById('parse-status');

  if (error) {
    if (statusEl) {
      statusEl.textContent = `Parse error: ${error}`;
      statusEl.className = 'error';
    }
    return;
  }

  if (statusEl) {
    statusEl.textContent = '';
    statusEl.className = '';
  }

  appState.configLoaded = true;
  render();
}

// ---- Event handlers ----

function setupEventListeners(): void {
  // DOM query returns generic Element; textarea needs specific type for .value access
  const textarea = document.getElementById('config-text') as HTMLTextAreaElement | null;
  if (textarea) {
    textarea.addEventListener('input', () => {
      if (debounceTimer) clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => {
        renderFromConfig(textarea.value);
      }, 500);
    });

    // Also handle paste
    textarea.addEventListener('paste', () => {
      setTimeout(() => {
        renderFromConfig(textarea.value);
      }, 50);
    });

    // Focus tracking on textarea
    textarea.addEventListener('focus', () => {
      appState.focusRegion = 'config';
    });
    textarea.addEventListener('blur', () => {
      appState.focusRegion = 'keyboard';
    });
  }

  // Copy/Paste config buttons
  const copyBtn = document.getElementById('copy-config-btn');
  if (copyBtn) {
    copyBtn.addEventListener('click', () => { void copyConfigToClipboard(); });
  }

  const pasteBtn = document.getElementById('paste-config-btn');
  if (pasteBtn) {
    pasteBtn.addEventListener('click', () => {
      void pasteConfigFromClipboard().then((text) => {
        if (text) renderFromConfig(text);
      });
    });
  }

  const openBtn = document.getElementById('open-config-btn');
  if (openBtn) {
    openBtn.addEventListener('click', () => {
      void openConfigFile().then((text) => {
        if (text) renderFromConfig(text);
      });
    });
  }

  // Help overlay close button
  const helpClose = document.getElementById('help-close');
  if (helpClose) {
    helpClose.addEventListener('click', toggleHelp);
  }

  // Keyboard shortcuts
  document.addEventListener('keydown', handleKeyDown);
}

function handleKeyDown(e: KeyboardEvent): void {
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
    return;
  }

  // Help overlay intercepts all keys when open
  const helpOverlay = document.getElementById('help-overlay');
  if (helpOverlay && !helpOverlay.classList.contains('hidden')) {
    if (e.key === 'Escape' || e.key === '?') {
      e.preventDefault();
      toggleHelp();
    }
    return;
  }

  if (handleGlobalShortcuts(e)) return;

  // Dispatch to focus-region handler
  if (appState.focusRegion === 'palette') {
    handlePaletteKey(e);
  } else {
    handleKeyboardKey(e);
  }
}

function handleGlobalShortcuts(e: KeyboardEvent): boolean {
  if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
    e.preventDefault();
    onUndo();
    return true;
  }
  if ((e.ctrlKey || e.metaKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) {
    e.preventDefault();
    onRedo();
    return true;
  }
  return false;
}

// ---- Keyboard navigation handlers ----

function getAllPaletteColors(state: EditorState): PaletteColor[] {
  return [
    ...state.palette.regular.filter(c => c.abbrev !== '___'),
    ...state.palette.locks,
    ...state.palette.aliases,
  ];
}

function handleKeyboardNav(e: KeyboardEvent): boolean {
  switch (e.key) {
    case 'ArrowUp': e.preventDefault(); moveCursorUp(); render(); return true;
    case 'ArrowDown': e.preventDefault(); moveCursorDown(); render(); return true;
    case 'ArrowLeft': e.preventDefault(); moveCursorLeft(); render(); return true;
    case 'ArrowRight': e.preventDefault(); moveCursorRight(); render(); return true;
    case 'Tab': e.preventDefault(); switchHalf(); render(); return true;
    case 'PageDown': e.preventDefault(); nextLayer(); render(); return true;
    case 'PageUp': e.preventDefault(); prevLayer(); render(); return true;
    default: return false;
  }
}

function handleKeyboardEdit(e: KeyboardEvent): boolean {
  switch (e.key) {
    case 'Enter': e.preventDefault(); enterPaletteMode(); return true;
    case 'Backspace':
    case 'Delete': e.preventDefault(); clearColorAtCursor(); render(); return true;
    case '+':
    case '=': e.preventDefault(); onFadeChange(5); return true;
    case '-': e.preventDefault(); onFadeChange(-5); return true;
    case 'c':
      if (!e.ctrlKey && !e.metaKey) { e.preventDefault(); yankColor(); render(); return true; }
      return false;
    case 'v':
      if (!e.ctrlKey && !e.metaKey) { e.preventDefault(); pasteColor(); render(); return true; }
      return false;
    default: return false;
  }
}

function handleKeyboardMisc(e: KeyboardEvent): boolean {
  if (e.key === '?') {
    e.preventDefault();
    toggleHelp();
    return true;
  }
  if (e.key === 't' && !e.ctrlKey && !e.metaKey) {
    e.preventDefault();
    toggleTheme();
    return true;
  }
  if (e.key === 'm' && !e.ctrlKey && !e.metaKey) {
    e.preventDefault();
    toggleInteractionMode();
    return true;
  }
  return false;
}

function handleKeyboardShortcuts(e: KeyboardEvent): boolean {
  if (e.ctrlKey || e.metaKey) return false;

  switch (e.key) {
    case 'C': e.preventDefault(); void copyConfigToClipboard(); return true;
    case 'V':
      e.preventDefault();
      void pasteConfigFromClipboard().then((text) => {
        if (text) renderFromConfig(text);
      });
      return true;
    case 'O':
      e.preventDefault();
      void openConfigFile().then((text) => {
        if (text) renderFromConfig(text);
      });
      return true;
    case 'a': e.preventDefault(); onLayerAction('add'); return true;
    case 'd': e.preventDefault(); onLayerAction('duplicate'); return true;
    case 'r': e.preventDefault(); onLayerAction('rename'); return true;
    case 'x': e.preventDefault(); onLayerAction('delete'); return true;
    default:
      if (e.key >= '0' && e.key <= '9') {
        e.preventDefault();
        applyColorByIndex(parseInt(e.key, 10));
        return true;
      }
      return false;
  }
}

function handleKeyboardKey(e: KeyboardEvent): void {
  if (handleKeyboardNav(e)) return;
  if (handleKeyboardEdit(e)) return;
  if (handleKeyboardMisc(e)) return;
  handleKeyboardShortcuts(e);
}

function applyColorByIndex(index: number): void {
  const state = getState();
  if (!state) return;

  const allColors = getAllPaletteColors(state);
  if (index < allColors.length) {
    setColorAtCursor(allColors[index].abbrev);
    appState.selectedColor = allColors[index].abbrev;
    render();
  }
}

function enterPaletteMode(): void {
  const state = getState();
  if (!state) return;

  const allColors = getAllPaletteColors(state);
  const currentColor = getCurrentColorAbbrev(state);
  const matchIndex = currentColor
    ? allColors.findIndex(c => c.abbrev === currentColor)
    : -1;

  appState.paletteIndex = matchIndex >= 0 ? matchIndex : 0;
  appState.focusRegion = 'palette';
  render();
}

function getCurrentColorAbbrev(state: EditorState): string | null {
  const grid = getLayerGrid(state.currentLayerIndex);
  if (!grid) return null;

  const half = state.cursor.half === 'left' ? grid.left : grid.right;
  const abbrev = half[state.cursor.row]?.[state.cursor.col];
  return abbrev && abbrev !== '___' ? abbrev : null;
}

function handlePaletteKey(e: KeyboardEvent): void {
  const state = getState();
  if (!state) return;

  const allColors = getAllPaletteColors(state);
  const count = allColors.length;
  if (count === 0) return;

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

function toggleInteractionMode(): void {
  appState.interactionMode = appState.interactionMode === 'select' ? 'paint' : 'select';
  render();
}

function toggleHelp(): void {
  const overlay = document.getElementById('help-overlay');
  if (!overlay) return;
  overlay.classList.toggle('hidden');
}

function onKeyClick(half: 'left' | 'right', row: number, col: number): void {
  setCursor(half, row, col);

  if (appState.interactionMode === 'paint' && appState.selectedColor) {
    if (appState.selectedColor === '___') {
      clearColorAt(half, row, col);
    } else {
      setColorAt(half, row, col, appState.selectedColor);
    }
  }

  render();
}

function onColorSelect(abbrev: string): void {
  appState.selectedColor = abbrev;

  if (appState.interactionMode === 'select') {
    if (abbrev === '___') {
      clearColorAtCursor();
    } else {
      setColorAtCursor(abbrev);
    }
  }

  render();
}

function onLayerSelect(index: number): void {
  setLayer(index);
  render();
}

function handleAddLayer(): void {
  const name = prompt('New layer name:');
  if (name) {
    const err = addLayer(name);
    if (err) alert(err);
  }
}

function handleDuplicateLayer(): void {
  const err = duplicateLayer();
  if (err) alert(err);
}

function handleRenameLayer(): void {
  const name = prompt('New name:');
  if (name) {
    const err = renameLayer(name);
    if (err) alert(err);
  }
}

function handleDeleteLayer(): void {
  if (confirm('Delete this layer?')) {
    const err = deleteLayer();
    if (err) alert(err);
  }
}

function onLayerAction(action: LayerAction): void {
  switch (action) {
    case 'add': handleAddLayer(); break;
    case 'duplicate': handleDuplicateLayer(); break;
    case 'rename': handleRenameLayer(); break;
    case 'delete': handleDeleteLayer(); break;
  }
  render();
}

function onUndo(): void {
  editorUndo();
  render();
}

function onRedo(): void {
  editorRedo();
  render();
}

function onFadeChange(delta: number): void {
  adjustFade(delta);
  render();
}

void main();
