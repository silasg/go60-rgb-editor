import { initWasm, loadConfig, hasEditor, getState, getLayerGrid, serialize, setLayer, setCursor, setColorAt, clearColorAt, editorUndo, editorRedo, adjustFade, addLayer, duplicateLayer, renameLayer, deleteLayer } from './editor-bridge.ts';
import { type AppState, createAppState } from './state.ts';
import { renderKeyboard } from './components/keyboard.ts';
import { renderPalette } from './components/palette.ts';
import { renderLayers, type LayerAction } from './components/layers.ts';
import { updateConfigText } from './components/config-text.ts';
import { renderToolbar } from './components/toolbar.ts';
import './styles.css';
import defaultConfig from '../../Go60 TK Latest RGB scheme.txt?raw';

const appState: AppState = createAppState();
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

async function main(): Promise<void> {
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
  renderToolbar(state, onUndo, onRedo, onFadeChange);
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

function onLayerAction(action: LayerAction): void {
  switch (action) {
    case 'add': {
      const name = prompt('New layer name:');
      if (name) {
        const err = addLayer(name);
        if (err) alert(err);
      }
      break;
    }
    case 'duplicate': {
      const err = duplicateLayer();
      if (err) alert(err);
      break;
    }
    case 'rename': {
      const name = prompt('New name:');
      if (name) {
        const err = renameLayer(name);
        if (err) alert(err);
      }
      break;
    }
    case 'delete': {
      if (confirm('Delete this layer?')) {
        const err = deleteLayer();
        if (err) alert(err);
      }
      break;
    }
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
