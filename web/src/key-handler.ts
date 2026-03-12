import type { AppState, EditorState, PaletteColor } from './state.ts';
import { hasEditor, getState, getLayerGrid, moveCursorUp, moveCursorDown, moveCursorLeft, moveCursorRight, switchHalf, setColorAtCursor, clearColorAtCursor, yankColor, pasteColor, nextLayer, prevLayer } from './editor-bridge.ts';
import { copyConfigToClipboard, pasteConfigFromClipboard, openConfigFile } from './components/config-text.ts';
import { toggleHelp, toggleInteractionMode, handleLayerAction, handleUndo, handleRedo, handleFadeChange } from './actions.ts';
import { toggleTheme } from './theme.ts';

function getAllPaletteColors(state: EditorState): PaletteColor[] {
  return [
    ...state.palette.regular.filter(c => c.abbrev !== '___'),
    ...state.palette.locks,
    ...state.palette.aliases,
  ];
}

function getCurrentColorAbbrev(state: EditorState): string | null {
  const grid = getLayerGrid(state.currentLayerIndex);
  if (!grid) return null;

  const half = state.cursor.half === 'left' ? grid.left : grid.right;
  const abbrev = half[state.cursor.row]?.[state.cursor.col];
  return abbrev && abbrev !== '___' ? abbrev : null;
}

function handleGlobalShortcuts(e: KeyboardEvent): boolean {
  if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
    e.preventDefault();
    handleUndo();
    return true;
  }
  if ((e.ctrlKey || e.metaKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) {
    e.preventDefault();
    handleRedo();
    return true;
  }
  return false;
}

function handleKeyboardNav(e: KeyboardEvent): boolean {
  switch (e.key) {
    case 'ArrowUp': e.preventDefault(); moveCursorUp(); return true;
    case 'ArrowDown': e.preventDefault(); moveCursorDown(); return true;
    case 'ArrowLeft': e.preventDefault(); moveCursorLeft(); return true;
    case 'ArrowRight': e.preventDefault(); moveCursorRight(); return true;
    case 'Tab': e.preventDefault(); switchHalf(); return true;
    case 'PageDown': e.preventDefault(); nextLayer(); return true;
    case 'PageUp': e.preventDefault(); prevLayer(); return true;
    default: return false;
  }
}

function enterPaletteMode(appState: AppState): void {
  const state = getState();
  if (!state) return;

  const allColors = getAllPaletteColors(state);
  const currentColor = getCurrentColorAbbrev(state);
  const matchIndex = currentColor
    ? allColors.findIndex(c => c.abbrev === currentColor)
    : -1;

  appState.paletteIndex = matchIndex >= 0 ? matchIndex : 0;
  appState.focusRegion = 'palette';
}

function handleKeyboardEdit(e: KeyboardEvent, appState: AppState): boolean {
  switch (e.key) {
    case 'Enter': e.preventDefault(); enterPaletteMode(appState); return true;
    case 'Backspace':
    case 'Delete': e.preventDefault(); clearColorAtCursor(); return true;
    case '+':
    case '=': e.preventDefault(); handleFadeChange(5); return true;
    case '-': e.preventDefault(); handleFadeChange(-5); return true;
    case 'c':
      if (!e.ctrlKey && !e.metaKey) { e.preventDefault(); yankColor(); return true; }
      return false;
    case 'v':
      if (!e.ctrlKey && !e.metaKey) { e.preventDefault(); pasteColor(); return true; }
      return false;
    default: return false;
  }
}

function handleKeyboardMisc(e: KeyboardEvent, appState: AppState): boolean {
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
    toggleInteractionMode(appState);
    return true;
  }
  return false;
}

function applyColorByIndex(index: number, appState: AppState): void {
  const state = getState();
  if (!state) return;

  const allColors = getAllPaletteColors(state);
  if (index < allColors.length) {
    setColorAtCursor(allColors[index].abbrev);
    appState.selectedColor = allColors[index].abbrev;
  }
}

function handleKeyboardShortcuts(e: KeyboardEvent, appState: AppState, renderFromConfig: (text: string) => void): boolean {
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
    case 'a': e.preventDefault(); handleLayerAction('add'); return true;
    case 'd': e.preventDefault(); handleLayerAction('duplicate'); return true;
    case 'r': e.preventDefault(); handleLayerAction('rename'); return true;
    case 'x': e.preventDefault(); handleLayerAction('delete'); return true;
    default:
      if (e.key >= '0' && e.key <= '9') {
        e.preventDefault();
        applyColorByIndex(parseInt(e.key, 10), appState);
        return true;
      }
      return false;
  }
}

function handleKeyboardKey(e: KeyboardEvent, appState: AppState, renderFromConfig: (text: string) => void): void {
  if (handleKeyboardNav(e)) return;
  if (handleKeyboardEdit(e, appState)) return;
  if (handleKeyboardMisc(e, appState)) return;
  handleKeyboardShortcuts(e, appState, renderFromConfig);
}

function handlePaletteKey(e: KeyboardEvent, appState: AppState): void {
  const state = getState();
  if (!state) return;

  const allColors = getAllPaletteColors(state);
  const count = allColors.length;
  if (count === 0) return;

  switch (e.key) {
    case 'ArrowRight':
      e.preventDefault();
      appState.paletteIndex = Math.min(appState.paletteIndex + 1, count - 1);
      break;
    case 'ArrowLeft':
      e.preventDefault();
      appState.paletteIndex = Math.max(appState.paletteIndex - 1, 0);
      break;
    case 'ArrowDown':
      e.preventDefault();
      appState.paletteIndex = Math.min(appState.paletteIndex + 8, count - 1);
      break;
    case 'ArrowUp':
      e.preventDefault();
      appState.paletteIndex = Math.max(appState.paletteIndex - 8, 0);
      break;
    case 'Enter':
      e.preventDefault();
      if (appState.paletteIndex >= 0 && appState.paletteIndex < count) {
        const color = allColors[appState.paletteIndex];
        appState.selectedColor = color.abbrev;
        setColorAtCursor(color.abbrev);
      }
      appState.focusRegion = 'keyboard';
      break;
    case 'Escape':
      e.preventDefault();
      appState.focusRegion = 'keyboard';
      break;
  }
}

export function handleKeyDown(e: KeyboardEvent, appState: AppState, renderFromConfig: (text: string) => void): boolean {
  if (!hasEditor()) return false;

  // When config textarea has focus, only intercept Escape
  const inTextarea = document.activeElement?.id === 'config-text';
  if (inTextarea) {
    if (e.key === 'Escape') {
      e.preventDefault();
      (document.activeElement as HTMLElement).blur();
      appState.focusRegion = 'keyboard';
      return true;
    }
    return false;
  }

  // Help overlay intercepts all keys when open
  const helpOverlay = document.getElementById('help-overlay');
  if (helpOverlay && !helpOverlay.classList.contains('hidden')) {
    if (e.key === 'Escape' || e.key === '?') {
      e.preventDefault();
      toggleHelp();
    }
    return false;
  }

  if (handleGlobalShortcuts(e)) return true;

  // Dispatch to focus-region handler
  if (appState.focusRegion === 'palette') {
    handlePaletteKey(e, appState);
  } else {
    handleKeyboardKey(e, appState, renderFromConfig);
  }
  return true;
}
