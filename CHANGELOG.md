# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2026-02-12

### ✨ Features

- Add layer management: add, duplicate, rename, and delete layers
  - `a` — add a new empty layer
  - `d` — duplicate the current layer
  - `n` — rename the current layer
  - `x` — delete the current layer (with confirmation)
- Layer name validation: alphanumeric and underscores only, max 50 characters, unique names enforced
- Auto-generated unique names for duplicated layers (e.g., `Cursor_copy`, `Cursor_copy_2`)
- Full undo/redo support for all layer operations

### ♻️ Refactor

- Move model, parser, cursor, geometry, undo into domain module
- Extract ratatui color helpers from RgbColor to UI module
- Extract IO into dedicated module, remove IO from Config
- Create EditorState to encapsulate domain logic
- Move color picker navigation to UI module
- Extract guard logic from event.rs into App methods
- Reorganized help popup into a two-column layout

### 🎨 Style

- Add Arrange-Act-Assert comments to all tests

### 📚 Documentation

- Add prebuilt binaries section and changelog link to README

### 🔧 Miscellaneous

- Remove duplicated Changelog section from README
- Add .agency and .sandbox folders to gitignore
## [0.1.1] - 2026-02-08

### 🐛 Bug Fixes

- Add Windows clipboard support (clip.exe)
- **ci:** Add --execute to cargo-release mise tasks

### 👷 CI/CD

- Include version in release artifact filenames

### 📚 Documentation

- Update AGENTS.md with current project structure and mise tasks
- Add documentation maintenance instructions to AGENTS.md

### 🔧 Miscellaneous

- Add local/ dir and .nvimlog to .gitignore
## [0.1.0] - 2026-02-08

Initial release of the Go60 RGB Editor.

### ✨ Features

- Visual keyboard layout matching the physical MoErgo Go60 split layout
- Layer navigation with fade duration control
- Color picker with full palette and keyboard navigation
- Quick color assignment via number keys
- Undo/redo with full edit history
- Copy/paste colors between keys
- Clear colors (reset to black)
- Save and Save As with overwrite confirmation
- Clipboard export of configuration
- Special color support for lock indicators (CapsLock/NumLock/ScrollLock) and mouse speed aliases
- Roundtrip parsing preserving original file formatting and comments
- In-app help overlay with all key bindings
