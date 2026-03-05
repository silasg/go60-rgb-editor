import { initWasm, loadConfig, hasEditor, getState, getLayerGrid, serialize, setLayer, setCursor, setColorAt, clearColorAt, editorUndo, editorRedo, adjustFade, addLayer, duplicateLayer, renameLayer, deleteLayer } from './editor-bridge.ts';
import { type AppState, createAppState } from './state.ts';
import { renderKeyboard } from './components/keyboard.ts';
import { renderPalette } from './components/palette.ts';
import { renderLayers, type LayerAction } from './components/layers.ts';
import { updateConfigText, copyConfigToClipboard, pasteConfigFromClipboard } from './components/config-text.ts';
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
  renderPalette(state, appState.selectedColor, onColorSelect);
  renderLayers(state, onLayerSelect, onLayerAction);
  renderToolbar(state, onUndo, onRedo, onFadeChange, toggleTheme);
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

  // Keyboard shortcuts
  document.addEventListener('keydown', (e) => {
    if (!hasEditor()) return;

    if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
      e.preventDefault();
      onUndo();
    } else if ((e.ctrlKey || e.metaKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) {
      e.preventDefault();
      onRedo();
    }
  });


}

function onKeyClick(half: 'left' | 'right', row: number, col: number): void {
  setCursor(half, row, col);

  if (appState.selectedColor) {
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

main();
