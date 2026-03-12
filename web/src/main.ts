import { initWasm, loadConfig, hasEditor, getState, getLayerGrid, serialize } from './editor-bridge.ts';
import { type AppState, createAppState } from './state.ts';
import { renderKeyboard } from './components/keyboard.ts';
import { renderPalette } from './components/palette.ts';
import { renderLayers, type LayerAction } from './components/layers.ts';
import { updateConfigText, copyConfigToClipboard, pasteConfigFromClipboard, openConfigFile } from './components/config-text.ts';
import { renderToolbar } from './components/toolbar.ts';
import { initTheme, toggleTheme } from './theme.ts';
import { handleKeyDown } from './key-handler.ts';
import { handleKeyClick, handleColorSelect, handleLayerSelect, handleLayerAction, handleUndo, handleRedo, handleFadeChange, toggleInteractionMode, toggleHelp } from './actions.ts';
import './styles/index.css';
import defaultConfig from '../../Go60 TK Latest RGB scheme.txt?raw';

const appState: AppState = createAppState();
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

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
  renderToolbar(state, onUndo, onRedo, onFadeChange, onToggleTheme, onToggleHelp, appState.interactionMode, onToggleMode);
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

// ---- Callbacks (stable references for components) ----

function onKeyClick(half: 'left' | 'right', row: number, col: number): void {
  handleKeyClick(appState, half, row, col);
  render();
}

function onColorSelect(abbrev: string): void {
  handleColorSelect(appState, abbrev);
  render();
}

function onLayerSelect(index: number): void {
  handleLayerSelect(index);
  render();
}

function onLayerAction(action: LayerAction): void {
  handleLayerAction(action);
  render();
}

function onUndo(): void {
  handleUndo();
  render();
}

function onRedo(): void {
  handleRedo();
  render();
}

function onFadeChange(delta: number): void {
  handleFadeChange(delta);
  render();
}

function onToggleTheme(): void {
  toggleTheme();
  render();
}

function onToggleHelp(): void {
  toggleHelp();
}

function onToggleMode(): void {
  toggleInteractionMode(appState);
  render();
}

// ---- Event setup ----

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

    textarea.addEventListener('paste', () => {
      setTimeout(() => {
        renderFromConfig(textarea.value);
      }, 50);
    });

    textarea.addEventListener('focus', () => {
      appState.focusRegion = 'config';
    });
    textarea.addEventListener('blur', () => {
      appState.focusRegion = 'keyboard';
    });
  }

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

  const helpClose = document.getElementById('help-close');
  if (helpClose) {
    helpClose.addEventListener('click', onToggleHelp);
  }

  document.addEventListener('keydown', (e) => {
    const shouldRender = handleKeyDown(e, appState, renderFromConfig);
    if (shouldRender) render();
  });
}

void main();
