import type { EditorState } from '../state.ts';

type LayerSelectHandler = (index: number) => void;
export type LayerAction = 'add' | 'duplicate' | 'rename' | 'delete';
type LayerActionHandler = (action: LayerAction) => void;

export function renderLayers(
  state: EditorState,
  onSelect: LayerSelectHandler,
  onAction: LayerActionHandler,
): void {
  const listEl = document.getElementById('layer-list');
  const actionsEl = document.getElementById('layer-actions');
  if (!listEl || !actionsEl) return;

  listEl.innerHTML = '';

  for (let i = 0; i < state.layers.length; i++) {
    const layer = state.layers[i];
    const item = document.createElement('button');
    item.className = 'layer-item';
    if (i === state.currentLayerIndex) {
      item.classList.add('active');
    }

    const nameSpan = document.createElement('span');
    nameSpan.className = 'layer-name';
    nameSpan.textContent = `${i} ${layer.name}`;
    item.appendChild(nameSpan);

    const fadeSpan = document.createElement('span');
    fadeSpan.className = 'layer-fade';
    fadeSpan.textContent = `${layer.fadeDelay}ms`;
    item.appendChild(fadeSpan);

    item.addEventListener('click', () => onSelect(i));
    listEl.appendChild(item);
  }

  // Layer action buttons
  actionsEl.innerHTML = '';

  const actions: { label: string; action: LayerAction }[] = [
    { label: '+ Add', action: 'add' },
    { label: 'Dup', action: 'duplicate' },
    { label: 'Ren', action: 'rename' },
    { label: 'Del', action: 'delete' },
  ];

  for (const { label, action } of actions) {
    const btn = document.createElement('button');
    btn.className = 'layer-action-btn';
    btn.textContent = label;
    btn.addEventListener('click', () => onAction(action));
    actionsEl.appendChild(btn);
  }
}
