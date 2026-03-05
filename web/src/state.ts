// ---- JSON types matching WASM API output ----

export interface CursorState {
  readonly row: number;
  readonly col: number;
  readonly half: 'left' | 'right';
}

export interface LayerInfo {
  readonly name: string;
  readonly fadeDelay: number;
}

export interface PaletteColor {
  readonly abbrev: string;
  readonly r: number;
  readonly g: number;
  readonly b: number;
}

export interface PaletteLock extends PaletteColor {
  readonly offColor: string;
  readonly onColor: string;
}

export interface PaletteAlias extends PaletteColor {
  readonly target: string;
}

export interface PaletteState {
  readonly regular: PaletteColor[];
  readonly locks: PaletteLock[];
  readonly aliases: PaletteAlias[];
}

export interface EditorState {
  readonly cursor: CursorState;
  readonly currentLayerIndex: number;
  readonly layerCount: number;
  readonly modified: boolean;
  readonly layers: LayerInfo[];
  readonly palette: PaletteState;
}

export interface LayerGrid {
  readonly left: string[][];
  readonly right: string[][];
  readonly fadeDelay: number;
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
