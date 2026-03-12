import type { AppState } from './state.ts';
import type { LayerAction } from './components/layers.ts';
import { setCursor, setColorAt, clearColorAt, setColorAtCursor, clearColorAtCursor, setLayer, addLayer, duplicateLayer, renameLayer, deleteLayer, editorUndo, editorRedo, adjustFade } from './editor-bridge.ts';

export function handleKeyClick(appState: AppState, half: 'left' | 'right', row: number, col: number): void {
  setCursor(half, row, col);

  if (appState.interactionMode === 'paint' && appState.selectedColor) {
    if (appState.selectedColor === '___') {
      clearColorAt(half, row, col);
    } else {
      setColorAt(half, row, col, appState.selectedColor);
    }
  }
}

export function handleColorSelect(appState: AppState, abbrev: string): void {
  appState.selectedColor = abbrev;

  if (appState.interactionMode === 'select') {
    if (abbrev === '___') {
      clearColorAtCursor();
    } else {
      setColorAtCursor(abbrev);
    }
  }
}

export function handleLayerSelect(index: number): void {
  setLayer(index);
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

export function handleLayerAction(action: LayerAction): void {
  switch (action) {
    case 'add': handleAddLayer(); break;
    case 'duplicate': handleDuplicateLayer(); break;
    case 'rename': handleRenameLayer(); break;
    case 'delete': handleDeleteLayer(); break;
  }
}

export function handleUndo(): void {
  editorUndo();
}

export function handleRedo(): void {
  editorRedo();
}

export function handleFadeChange(delta: number): void {
  adjustFade(delta);
}

export function toggleInteractionMode(appState: AppState): void {
  appState.interactionMode = appState.interactionMode === 'select' ? 'paint' : 'select';
}

export function toggleHelp(): void {
  const overlay = document.getElementById('help-overlay');
  if (!overlay) return;
  overlay.classList.toggle('hidden');
}
