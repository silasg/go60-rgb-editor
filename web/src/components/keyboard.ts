import type { EditorState, LayerGrid, PaletteColor, PaletteLock, PaletteAlias } from '../state.ts';
import { rgbToHex, textColorForBg } from '../state.ts';
import { ROW_COUNT, colsForRow, gridColumn, gridRow, GRID_COLS } from '../geometry.ts';

type KeyClickHandler = (half: string, row: number, col: number) => void;

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

function renderHalf(
  containerId: string,
  half: 'left' | 'right',
  grid: string[][],
  state: EditorState,
  selectedColor: string | null,
  onClick: KeyClickHandler,
): void {
  const container = document.getElementById(containerId);
  if (!container) return;

  // Keep the label, clear keys
  const label = container.querySelector('.half-label');
  container.innerHTML = '';
  if (label) container.appendChild(label);

  container.style.display = 'grid';
  container.style.gridTemplateColumns = `repeat(${GRID_COLS}, 1fr)`;
  container.style.gridTemplateRows = `repeat(${ROW_COUNT}, 1fr)`;
  container.style.gap = '4px';

  for (let row = 0; row < ROW_COUNT && row < grid.length; row++) {
    const cols = colsForRow(row);
    for (let col = 0; col < cols && col < grid[row].length; col++) {
      const abbrev = grid[row][col];
      const btn = document.createElement('button');
      btn.className = 'key';
      btn.textContent = abbrev;
      btn.dataset.half = half;
      btn.dataset.row = String(row);
      btn.dataset.col = String(col);

      const gCol = gridColumn(half, row, col);
      const gRow = gridRow(row);
      btn.style.gridColumn = String(gCol);
      btn.style.gridRow = String(gRow);

      // Color styling
      if (abbrev !== '___') {
        const colorDef = findColor(abbrev, state.palette);
        if (colorDef) {
          const bg = rgbToHex(colorDef.r, colorDef.g, colorDef.b);
          btn.style.backgroundColor = bg;
          btn.style.color = textColorForBg(colorDef.r, colorDef.g, colorDef.b);
        }
      } else {
        btn.style.backgroundColor = '#1a1a2e';
        btn.style.color = '#666';
      }

      // Cursor highlight
      if (
        state.cursor.half === half &&
        state.cursor.row === row &&
        state.cursor.col === col
      ) {
        btn.classList.add('cursor');
      }

      btn.addEventListener('click', () => onClick(half, row, col));
      container.appendChild(btn);
    }
  }
}

export function renderKeyboard(
  state: EditorState,
  grid: LayerGrid,
  selectedColor: string | null,
  onClick: KeyClickHandler,
): void {
  renderHalf('keyboard-left', 'left', grid.left, state, selectedColor, onClick);
  renderHalf('keyboard-right', 'right', grid.right, state, selectedColor, onClick);
}
