# Go60 RGB Editor

A terminal-based (TUI) RGB underglow editor for ZMK keyboards, specifically designed for the MoErgo Go60 layout.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)

## Features

- **Visual keyboard layout** - color display matching the physical Go60 layout
- **Layer management** - navigate between RGB layers, adjust fade duration
- **Color picker** - select from the full color palette with keyboard navigation
- **Quick color assignment** - number keys for fast color selection
- **Undo/redo** - full edit history support
- **Copy/paste colors** - duplicate color assignments between keys
- **Clear colors** - quickly reset keys to black
- **Clipboard export** - copy configuration to system clipboard
- **Special color support** - lock indicators (CapsLock/NumLock/ScrollLock) and mouse speed aliases
- **Roundtrip parsing** - preserves original file formatting and comments

Press `?` in the editor to see all key bindings.

## Supported Keyboards

**Currently supported:**
- MoErgo Go60 layout

**Not supported (alternatives available):**
- Go80 - see the [TailorKey RGB documentation](https://sites.google.com/view/tailorkey/how-to/rgb) for alternative tools

## Installation

### From source

```bash
git clone <repo-url>
cd Go60
cargo build --release
```

The binary will be at `target/release/go60-rgb-editor`.

## Usage

```bash
go60-rgb-editor <path-to-rgb-config.txt>
```

### Example

```bash
go60-rgb-editor "Go60 TK Latest RGB scheme.txt"
```

## Configuration File Format

The editor works with TailorKey RGB configuration files. For detailed documentation on the file format and RGB configuration options, see:

**[TailorKey RGB Documentation](https://sites.google.com/view/tailorkey/how-to/rgb)**

The included `Go60 TK Latest RGB scheme.txt` is an example configuration from TailorKey.

## Building

Requires Rust 1.70 or later.

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test
```
