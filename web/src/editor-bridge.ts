import init, { Editor } from '../../pkg/go60_rgb_editor_wasm.js';
import type { EditorState, LayerGrid } from './state.ts';

let editor: Editor | null = null;

export async function initWasm(): Promise<void> {
  await init();
}

export function loadConfig(text: string): string | null {
  try {
    editor = new Editor(text);
    return null;
  } catch (e) {
    return String(e);
  }
}

export function hasEditor(): boolean {
  return editor !== null;
}

export function getState(): EditorState | null {
  if (!editor) return null;
  // Trust boundary: JSON shape guaranteed by Rust serialization in domain-wasm
  return JSON.parse(editor.get_state_json()) as EditorState;
}

export function getLayerGrid(index: number): LayerGrid | null {
  if (!editor) return null;
  // Trust boundary: JSON shape guaranteed by Rust serialization in domain-wasm
  return JSON.parse(editor.get_layer_grid_json(index)) as LayerGrid;
}

export function serialize(): string {
  if (!editor) return '';
  return editor.serialize();
}

export function setLayer(index: number): void {
  editor?.set_layer(index);
}

export function setCursor(half: 'left' | 'right', row: number, col: number): void {
  editor?.set_cursor(half, row, col);
}

export function setColorAt(half: 'left' | 'right', row: number, col: number, abbrev: string): boolean {
  if (!editor) return false;
  return editor.set_color_at(half, row, col, abbrev);
}

export function clearColorAt(half: 'left' | 'right', row: number, col: number): boolean {
  if (!editor) return false;
  return editor.clear_color_at(half, row, col);
}

export function editorUndo(): boolean {
  if (!editor) return false;
  return editor.undo();
}

export function editorRedo(): boolean {
  if (!editor) return false;
  return editor.redo();
}

export function adjustFade(delta: number): number {
  if (!editor) return -1;
  return editor.adjust_fade(delta);
}

export function addLayer(name: string): string | null {
  if (!editor) return 'No editor';
  try {
    editor.add_layer(name);
    return null;
  } catch (e) {
    return String(e);
  }
}

export function duplicateLayer(): string | null {
  if (!editor) return 'No editor';
  try {
    editor.duplicate_layer();
    return null;
  } catch (e) {
    return String(e);
  }
}

export function renameLayer(newName: string): string | null {
  if (!editor) return 'No editor';
  try {
    editor.rename_layer(newName);
    return null;
  } catch (e) {
    return String(e);
  }
}

export function deleteLayer(): string | null {
  if (!editor) return 'No editor';
  try {
    editor.delete_layer();
    return null;
  } catch (e) {
    return String(e);
  }
}
