import type { EditorState, InteractionMode } from '../state.ts';

type VoidHandler = () => void;
type FadeHandler = (delta: number) => void;

function createFadeControls(state: EditorState, onFadeChange: FadeHandler): HTMLElement {
  const fadeGroup = document.createElement('div');
  fadeGroup.className = 'toolbar-group';

  const fadeLabel = document.createElement('span');
  fadeLabel.className = 'fade-label';
  const currentLayer = state.layers[state.currentLayerIndex];
  fadeLabel.textContent = `Fade: ${currentLayer.fadeDelay}ms`;
  fadeGroup.appendChild(fadeLabel);

  const fadeDown = document.createElement('button');
  fadeDown.className = 'toolbar-btn';
  fadeDown.textContent = '−';
  fadeDown.title = 'Decrease fade delay';
  fadeDown.addEventListener('click', () => { onFadeChange(-5); });
  fadeGroup.appendChild(fadeDown);

  const fadeUp = document.createElement('button');
  fadeUp.className = 'toolbar-btn';
  fadeUp.textContent = '+';
  fadeUp.title = 'Increase fade delay';
  fadeUp.addEventListener('click', () => { onFadeChange(5); });
  fadeGroup.appendChild(fadeUp);

  return fadeGroup;
}

function createModeToggle(
  mode: InteractionMode,
  onToggle: VoidHandler,
): HTMLElement {
  const group = document.createElement('div');
  group.className = 'toolbar-group mode-toggle-group';

  const selectBtn = document.createElement('button');
  selectBtn.className = 'mode-toggle-btn';
  selectBtn.textContent = '🔘 Select';
  selectBtn.title = 'Select mode (m): click a key to select it, then click a color to apply';
  if (mode === 'select') selectBtn.classList.add('mode-active');
  selectBtn.addEventListener('click', () => { if (mode !== 'select') onToggle(); });

  const paintBtn = document.createElement('button');
  paintBtn.className = 'mode-toggle-btn';
  paintBtn.textContent = '🎨 Paint';
  paintBtn.title = 'Paint mode (m): choose a color, then click keys to paint them';
  if (mode === 'paint') paintBtn.classList.add('mode-active');
  paintBtn.addEventListener('click', () => { if (mode !== 'paint') onToggle(); });

  group.appendChild(selectBtn);
  group.appendChild(paintBtn);
  return group;
}

export function renderToolbar(
  state: EditorState,
  onUndo: VoidHandler,
  onRedo: VoidHandler,
  onFadeChange: FadeHandler,
  onThemeToggle: VoidHandler,
  onHelpToggle: VoidHandler,
  interactionMode: InteractionMode,
  onModeToggle: VoidHandler,
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

  toolbar.appendChild(createModeToggle(interactionMode, onModeToggle));
  toolbar.appendChild(createFadeControls(state, onFadeChange));

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

  // Theme toggle
  const themeBtn = document.createElement('button');
  themeBtn.className = 'theme-toggle';
  const isDark = document.documentElement.getAttribute('data-theme') !== 'light';
  themeBtn.textContent = isDark ? '☀️' : '🌙';
  themeBtn.title = isDark ? 'Switch to light mode' : 'Switch to dark mode';
  themeBtn.addEventListener('click', onThemeToggle);
  toolbar.appendChild(themeBtn);

  // Help toggle
  const helpBtn = document.createElement('button');
  helpBtn.className = 'toolbar-btn help-toggle';
  helpBtn.textContent = '? Help';
  helpBtn.title = 'Toggle keyboard shortcuts';
  helpBtn.addEventListener('click', onHelpToggle);
  toolbar.appendChild(helpBtn);
}
