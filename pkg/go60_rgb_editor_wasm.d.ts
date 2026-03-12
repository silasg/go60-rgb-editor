/* tslint:disable */
/* eslint-disable */

/**
 * Opaque editor handle exposed to JavaScript via WebAssembly.
 *
 * Wraps `EditorState` in Wasm memory. JS calls methods on this handle
 * to mutate editor state, and reads the current state via getter methods.
 * Undo/redo history stays internal in Rust memory.
 */
export class Editor {
    free(): void;
    [Symbol.dispose](): void;
    add_layer(name: string): void;
    /**
     * Adjust fade delay by delta milliseconds. Returns the new value, or -1 if no layer.
     */
    adjust_fade(delta: number): number;
    /**
     * Clear the color at the current cursor position.
     */
    clear_color(): boolean;
    /**
     * Clear color at a specific position.
     */
    clear_color_at(half: string, row: number, col: number): boolean;
    /**
     * Get the color abbreviation at the current cursor position.
     */
    current_color(): string;
    /**
     * Get the current layer index.
     */
    current_layer_index(): number;
    /**
     * Get the current cursor column.
     */
    cursor_col(): number;
    /**
     * Get the current cursor half ("left" or "right").
     */
    cursor_half(): string;
    /**
     * Get the current cursor row.
     */
    cursor_row(): number;
    delete_layer(): string;
    duplicate_layer(): string;
    /**
     * Get the current layer's fade delay.
     */
    fade_delay(): number;
    /**
     * Current layer's key grid as JSON.
     * Returns: { left: [[abbrev]], right: [[abbrev]], fadeDelay }
     */
    get_layer_grid_json(index: number): string;
    /**
     * Full UI state as JSON (call after any mutation to re-render).
     * Returns: { cursor, currentLayerIndex, layerCount, modified, layers, palette }
     */
    get_state_json(): string;
    /**
     * Whether the config has been modified since last save.
     */
    is_modified(): boolean;
    /**
     * Get the number of layers.
     */
    layer_count(): number;
    mark_saved(): void;
    move_down(): void;
    move_left(): void;
    move_right(): void;
    move_up(): void;
    /**
     * Create an Editor by parsing a TailorKey config file string.
     */
    constructor(config_text: string);
    next_layer(): void;
    /**
     * Paste the previously yanked color. Returns true on success.
     */
    paste_color(): boolean;
    prev_layer(): void;
    redo(): boolean;
    rename_layer(new_name: string): void;
    /**
     * Serialize the current config back to the TailorKey file format.
     */
    serialize(): string;
    /**
     * Set the color at the current cursor position. Returns true on success.
     */
    set_color(abbrev: string): boolean;
    /**
     * Set color at a specific position (bypasses cursor).
     */
    set_color_at(half: string, row: number, col: number, abbrev: string): boolean;
    /**
     * Set the cursor to a specific position.
     */
    set_cursor(half: string, row: number, col: number): void;
    /**
     * Set the current layer by index.
     */
    set_layer(index: number): void;
    switch_half(): void;
    undo(): boolean;
    /**
     * Yank (copy) the color at the current cursor position.
     * Returns the abbreviation, or empty string if no color.
     */
    yank_color(): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_editor_free: (a: number, b: number) => void;
    readonly editor_add_layer: (a: number, b: number, c: number) => [number, number];
    readonly editor_adjust_fade: (a: number, b: number) => number;
    readonly editor_clear_color: (a: number) => number;
    readonly editor_clear_color_at: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly editor_current_color: (a: number) => [number, number];
    readonly editor_current_layer_index: (a: number) => number;
    readonly editor_cursor_col: (a: number) => number;
    readonly editor_cursor_half: (a: number) => [number, number];
    readonly editor_cursor_row: (a: number) => number;
    readonly editor_delete_layer: (a: number) => [number, number, number, number];
    readonly editor_duplicate_layer: (a: number) => [number, number, number, number];
    readonly editor_fade_delay: (a: number) => number;
    readonly editor_get_layer_grid_json: (a: number, b: number) => [number, number];
    readonly editor_get_state_json: (a: number) => [number, number];
    readonly editor_is_modified: (a: number) => number;
    readonly editor_layer_count: (a: number) => number;
    readonly editor_mark_saved: (a: number) => void;
    readonly editor_move_down: (a: number) => void;
    readonly editor_move_left: (a: number) => void;
    readonly editor_move_right: (a: number) => void;
    readonly editor_move_up: (a: number) => void;
    readonly editor_new: (a: number, b: number) => [number, number, number];
    readonly editor_next_layer: (a: number) => void;
    readonly editor_paste_color: (a: number) => number;
    readonly editor_prev_layer: (a: number) => void;
    readonly editor_redo: (a: number) => number;
    readonly editor_rename_layer: (a: number, b: number, c: number) => [number, number];
    readonly editor_serialize: (a: number) => [number, number];
    readonly editor_set_color: (a: number, b: number, c: number) => number;
    readonly editor_set_color_at: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
    readonly editor_set_cursor: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly editor_set_layer: (a: number, b: number) => void;
    readonly editor_switch_half: (a: number) => void;
    readonly editor_undo: (a: number) => number;
    readonly editor_yank_color: (a: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
