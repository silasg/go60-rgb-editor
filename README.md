# Go60 RGB Editor

A terminal-based (TUI) RGB underglow editor for ZMK keyboards, specifically designed for the MoErgo Go60 layout.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)

## Features

- **Visual keyboard layout** with real-time color display
- **Layer navigation** - switch between RGB layers with `n`/`p`
- **Color picker** - select from the full color palette
- **Quick color assignment** - use `0-9` for fast color selection
- **Undo/redo** - full edit history support
- **Copy/paste colors** - quickly duplicate color assignments
- **Special color indicators** - lock states and aliases clearly marked
- **Roundtrip parsing** - preserves original file formatting

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

### Key Bindings

| Key | Action |
|-----|--------|
| `h`/`j`/`k`/`l` or arrows | Navigate cursor |
| `Tab` | Switch between left/right half |
| `n`/`p` | Next/previous layer |
| `Enter` | Open color picker |
| `0-9` | Quick select color (first 10) |
| `y` | Copy color at cursor |
| `Y` | Paste color at cursor |
| `u` | Undo |
| `Ctrl+r` | Redo |
| `s` | Save |
| `q` | Quit (prompts if unsaved) |
| `Q` | Force quit without saving |
| `?` | Show help |

### Color Indicators

- Regular colors display as their 3-letter abbreviation (e.g., `RED`, `CYN`)
- Lock indicators show as `*XX*` (toggle between off/on states)
- Aliases show as `→XX` (reference to another color)

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


