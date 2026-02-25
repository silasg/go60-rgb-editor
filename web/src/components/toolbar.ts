import type { EditorState } from '../state.ts';

type VoidHandler = () => void;
type FadeHandler = (delta: number) => void;

export function renderToolbar(
  state: EditorState,
  onUndo: VoidHandler,
  onRedo: VoidHandler,
  onFadeChange: FadeHandler,
): void {
  const toolbar = document.getElementById('toolbar');
  if (!toolbar) return;

  toolbar.innerHTML = '';

  // Modified indicator
  if (state.modified) {
    const dot = document.createElement('span');
    dot.className = 'modified-indicator';
    dot.textContent = '● Modified';
    dot.title = 'Config has unsaved changes';
    toolbar.appendChild(dot);
  }

  // Fade delay controls
  const fadeGroup = document.createElement('div');
  fadeGroup.className = 'toolbar-group';

  const fadeLabel = document.createElement('span');
  fadeLabel.className = 'fade-label';
  const currentLayer = state.layers[state.currentLayerIndex];
  fadeLabel.textContent = `Fade: ${currentLayer?.fadeDelay ?? 0}ms`;
  fadeGroup.appendChild(fadeLabel);

  const fadeDown = document.createElement('button');
  fadeDown.className = 'toolbar-btn';
  fadeDown.textContent = '−';
  fadeDown.title = 'Decrease fade delay';
  fadeDown.addEventListener('click', () => onFadeChange(-5));
  fadeGroup.appendChild(fadeDown);

  const fadeUp = document.createElement('button');
  fadeUp.className = 'toolbar-btn';
  fadeUp.textContent = '+';
  fadeUp.title = 'Increase fade delay';
  fadeUp.addEventListener('click', () => onFadeChange(5));
  fadeGroup.appendChild(fadeUp);

  toolbar.appendChild(fadeGroup);

  // Undo/Redo buttons
  const undoBtn = document.createElement('button');
  undoBtn.className = 'toolbar-btn';
  undoBtn.textContent = '↩ Undo';
  undoBtn.addEventListener('click', onUndo);
  toolbar.appendChild(undoBtn);

  const redoBtn = document.createElement('button');
  redoBtn.className = 'toolbar-btn';
  redoBtn.textContent = '↪ Redo';
  redoBtn.addEventListener('click', onRedo);
  toolbar.appendChild(redoBtn);
}
