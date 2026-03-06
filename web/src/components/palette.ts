import type { EditorState, PaletteColor } from '../state.ts';
import { rgbToHex, textColorForBg } from '../state.ts';

type ColorSelectHandler = (abbrev: string) => void;

function createClearSwatch(
  selectedColor: string | null,
  onClick: ColorSelectHandler,
): HTMLButtonElement {
  const btn = document.createElement('button');
  btn.className = 'swatch';
  btn.textContent = '___';
  btn.title = 'Clear key';
  btn.style.backgroundColor = 'var(--key-off)';
  btn.style.color = 'var(--text-secondary)';

  if (selectedColor === '___') {
    btn.classList.add('selected');
  }

  btn.addEventListener('click', () => { onClick('___'); });
  return btn;
}

function renderSwatchGrid(
  containerId: string,
  colors: PaletteColor[],
  selectedColor: string | null,
  onClick: ColorSelectHandler,
  paletteCursorIndex: number | null,
  flatStartIndex: number,
): void {
  const container = document.getElementById(containerId);
  if (!container) return;

  container.innerHTML = '';

  let localIdx = 0;
  for (const color of colors) {
    if (color.abbrev === '___') continue;

    const btn = document.createElement('button');
    btn.className = 'swatch';
    btn.textContent = color.abbrev;
    btn.title = color.abbrev;

    const bg = rgbToHex(color.r, color.g, color.b);
    btn.style.backgroundColor = bg;
    btn.style.color = textColorForBg(color.r, color.g, color.b);

    if (selectedColor === color.abbrev) {
      btn.classList.add('selected');
    }

    if (paletteCursorIndex !== null && (flatStartIndex + localIdx) === paletteCursorIndex) {
      btn.classList.add('palette-cursor');
    }

    btn.addEventListener('click', () => onClick(color.abbrev));
    container.appendChild(btn);
    localIdx++;
  }
}

export function renderPalette(
  state: EditorState,
  selectedColor: string | null,
  onClick: ColorSelectHandler,
  paletteCursorIndex: number | null,
): void {
  let flatIndex = 0;

  renderSwatchGrid('palette-regular', state.palette.regular, selectedColor, onClick, paletteCursorIndex, flatIndex);
  flatIndex += state.palette.regular.filter(c => c.abbrev !== '___').length;

  // Prepend clear swatch to the regular colors grid
  const regularGrid = document.getElementById('palette-regular');
  if (regularGrid) {
    regularGrid.prepend(createClearSwatch(selectedColor, onClick));
  }

  renderSwatchGrid('palette-lock-grid', state.palette.locks, selectedColor, onClick, paletteCursorIndex, flatIndex);
  flatIndex += state.palette.locks.length;

  renderSwatchGrid('palette-alias-grid', state.palette.aliases, selectedColor, onClick, paletteCursorIndex, flatIndex);
}
