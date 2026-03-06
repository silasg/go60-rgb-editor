import type { EditorState, LayerGrid, PaletteColor, PaletteLock, PaletteAlias } from '../state.ts';
import { rgbToHex, textColorForBg } from '../state.ts';
import { ROW_COUNT, colsForRow, gridColumn, gridRow, GRID_COLS } from '../geometry.ts';

type Half = 'left' | 'right';
type KeyClickHandler = (half: Half, row: number, col: number) => void;

function findColor(
  abbrev: string,
  palette: { regular: PaletteColor[]; locks: PaletteLock[]; aliases: PaletteAlias[] },
): PaletteColor | PaletteLock | PaletteAlias | undefined {
  return (
    palette.regular.find((c) => c.abbrev === abbrev) ??
    palette.locks.find((c) => c.abbrev === abbrev) ??
    palette.aliases.find((c) => c.abbrev === abbrev)
  );
}

function applyKeyStyle(
  btn: HTMLButtonElement,
  abbrev: string,
  state: EditorState,
): void {
  if (abbrev !== '___') {
    const colorDef = findColor(abbrev, state.palette);
    if (colorDef) {
      btn.style.backgroundColor = rgbToHex(colorDef.r, colorDef.g, colorDef.b);
      btn.style.color = textColorForBg(colorDef.r, colorDef.g, colorDef.b);
    }
  } else {
    btn.style.backgroundColor = 'var(--key-off)';
    btn.style.color = 'var(--text-secondary)';
  }
}

function createKeyButton(
  half: Half,
  row: number,
  col: number,
  abbrev: string,
  state: EditorState,
  onClick: KeyClickHandler,
): HTMLButtonElement {
  const btn = document.createElement('button');
  btn.className = 'key';
  btn.textContent = abbrev;
  btn.dataset.half = half;
  btn.dataset.row = String(row);
  btn.dataset.col = String(col);

  applyKeyStyle(btn, abbrev, state);

  if (state.cursor.half === half && state.cursor.row === row && state.cursor.col === col) {
    btn.classList.add('cursor');
  }

  btn.addEventListener('click', () => { onClick(half, row, col); });
  return btn;
}

function renderHalf(
  containerId: string,
  half: Half,
  grid: string[][],
  state: EditorState,
  onClick: KeyClickHandler,
): void {
  const container = document.getElementById(containerId);
  if (!container) return;

  container.innerHTML = '';

  container.style.display = 'grid';
  container.style.gridTemplateColumns = `repeat(${GRID_COLS}, 1fr)`;
  container.style.gridTemplateRows = `repeat(${ROW_COUNT}, 1fr)`;
  container.style.gap = '4px';

  for (let row = 0; row < ROW_COUNT && row < grid.length; row++) {
    const cols = colsForRow(row);
    for (let col = 0; col < cols && col < grid[row].length; col++) {
      const btn = createKeyButton(half, row, col, grid[row][col], state, onClick);
      btn.style.gridColumn = String(gridColumn(half, row, col));
      btn.style.gridRow = String(gridRow(row));
      container.appendChild(btn);
    }
  }
}

export function renderKeyboard(
  state: EditorState,
  grid: LayerGrid,
  onClick: KeyClickHandler,
): void {
  renderHalf('keyboard-left', 'left', grid.left, state, onClick);
  renderHalf('keyboard-right', 'right', grid.right, state, onClick);
}
