import type { EditorState, PaletteColor } from '../state.ts';
import { rgbToHex, textColorForBg } from '../state.ts';

type ColorSelectHandler = (abbrev: string) => void;

function renderSwatchGrid(
  containerId: string,
  colors: PaletteColor[],
  selectedColor: string | null,
  onClick: ColorSelectHandler,
): void {
  const container = document.getElementById(containerId);
  if (!container) return;

  container.innerHTML = '';

  for (const color of colors) {
    // Skip ___ from the regular color display (it's the eraser)
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

    btn.addEventListener('click', () => onClick(color.abbrev));
    container.appendChild(btn);
  }
}

export function renderPalette(
  state: EditorState,
  selectedColor: string | null,
  onClick: ColorSelectHandler,
): void {
  renderSwatchGrid('palette-regular', state.palette.regular, selectedColor, onClick);
  renderSwatchGrid('palette-lock-grid', state.palette.locks, selectedColor, onClick);
  renderSwatchGrid('palette-alias-grid', state.palette.aliases, selectedColor, onClick);

  // Update eraser button state
  const eraserBtn = document.getElementById('eraser-btn');
  if (eraserBtn) {
    if (selectedColor === '___') {
      eraserBtn.classList.add('selected');
    } else {
      eraserBtn.classList.remove('selected');
    }
  }
}
