/* tslint:disable */
/* eslint-disable */
/**
 * Type declarations for the WASM-generated Editor module.
 *
 * Committed as a stub so TypeScript and ESLint can resolve types
 * without building the WASM package. The `build-wasm` task overwrites
 * this with the exact generated declarations from wasm-bindgen.
 */
export class Editor {
  constructor(config_text: string);
  free(): void;
  serialize(): string;
  move_up(): void;
  move_down(): void;
  move_left(): void;
  move_right(): void;
  switch_half(): void;
  set_cursor(half: string, row: number, col: number): void;
  next_layer(): void;
  prev_layer(): void;
  set_color(abbrev: string): boolean;
  clear_color(): boolean;
  yank_color(): string;
  paste_color(): boolean;
  undo(): boolean;
  redo(): boolean;
  adjust_fade(delta: number): number;
  add_layer(name: string): void;
  duplicate_layer(): string;
  rename_layer(new_name: string): void;
  delete_layer(): string;
  is_modified(): boolean;
  mark_saved(): void;
  cursor_row(): number;
  cursor_col(): number;
  cursor_half(): string;
  current_layer_index(): number;
  current_color(): string;
  layer_count(): number;
  get_state_json(): string;
  get_layer_grid_json(index: number): string;
  set_layer(index: number): void;
  set_color_at(half: string, row: number, col: number, abbrev: string): boolean;
  clear_color_at(half: string, row: number, col: number): boolean;
  fade_delay(): number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export default function init(module_or_path?: InitInput | Promise<InitInput>): Promise<void>;
