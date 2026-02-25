// ---- JSON types matching WASM API output ----

export interface CursorState {
  row: number;
  col: number;
  half: 'left' | 'right';
}

export interface LayerInfo {
  name: string;
  fadeDelay: number;
}

export interface PaletteColor {
  abbrev: string;
  r: number;
  g: number;
  b: number;
}

export interface PaletteLock extends PaletteColor {
  offColor: string;
  onColor: string;
}

export interface PaletteAlias extends PaletteColor {
  target: string;
}

export interface PaletteState {
  regular: PaletteColor[];
  locks: PaletteLock[];
  aliases: PaletteAlias[];
}

export interface EditorState {
  cursor: CursorState;
  currentLayerIndex: number;
  layerCount: number;
  modified: boolean;
  layers: LayerInfo[];
  palette: PaletteState;
}

export interface LayerGrid {
  left: string[][];
  right: string[][];
  fadeDelay: number;
}

// ---- Application state (frontend-only) ----

export interface AppState {
  selectedColor: string | null;
  configLoaded: boolean;
}

export function createAppState(): AppState {
  return {
    selectedColor: null,
    configLoaded: false,
  };
}

// ---- Color utilities ----

export function rgbToHex(r: number, g: number, b: number): string {
  const toHex = (v: number) => v.toString(16).padStart(2, '0');
  return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
}

export function luminance(r: number, g: number, b: number): number {
  return 0.299 * r + 0.587 * g + 0.114 * b;
}

export function textColorForBg(r: number, g: number, b: number): string {
  return luminance(r, g, b) > 140 ? '#000000' : '#ffffff';
}
