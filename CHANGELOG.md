# Changelog

All notable changes to this project will be documented in this file.

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
