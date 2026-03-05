# Go60 RGB Editor

A visual RGB underglow editor for the MoErgo Go60 keyboard running the `community.pr36.per-key-rgb` community firmware. Available as a terminal app (TUI) and a [web editor](https://silasg.github.io/go60-rgb-editor/).

![CI](https://github.com/silasg/go60-rgb-editor/actions/workflows/ci.yml/badge.svg)
![GitHub Release](https://img.shields.io/github/v/release/silasg/go60-rgb-editor)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)

![Go60 RGB Editor Screenshot](screenshot.png)

## Features

- [x] **Visual keyboard layout** - color display matching the physical Go60 layout
- [x] **Layer management** - add, duplicate, rename, and delete layers
- [x] **Layer navigation** - navigate between RGB layers, adjust fade duration
- [x] **Color picker** - select from the full color palette with keyboard navigation
- [x] **Quick color assignment** - number keys for fast color selection
- [x] **Undo/redo** - full edit history support
- [x] **Copy/paste colors** - duplicate color assignments between keys
- [x] **Clear colors** - quickly reset keys to black
- [x] **Clipboard export** - copy configuration to system clipboard
- [x] **Special color support** - lock indicators (CapsLock/NumLock/ScrollLock) and mouse speed aliases
- [x] **Roundtrip parsing** - preserves original file formatting and comments

- [ ] **Color palette management** - add, edit, remove color definitions

Press `?` in the editor to see all key bindings.

## Supported Keyboards

**Currently supported:**
- MoErgo Go60 layout

**Not supported (alternatives available):**
- Glove80 - see the [TailorKey RGB documentation](https://sites.google.com/view/tailorkey/how-to/rgb) for alternative tools

## Web Editor

Try the editor directly in your browser — no installation required:

**[https://silasg.github.io/go60-rgb-editor/](https://silasg.github.io/go60-rgb-editor/)**

The web editor supports the same core features: painting per-key colors, layer management, undo/redo, and copy/paste of [TailorKey RGB](https://sites.google.com/view/tailorkey/how-to/rgb) config text.

## Installation (TUI)

### Prebuilt binaries

Download the latest release for your platform from the [GitHub Releases](https://github.com/silasg/go60-rgb-editor/releases) page. Binaries are available for Linux (x86_64, aarch64), macOS (Intel, Apple Silicon), and Windows.

### From source

```bash
git clone https://github.com/silasg/go60-rgb-editor.git
cd go60-rgb-editor
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

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a list of all changes.

## Disclaimer

This software is provided as-is, without any warranty or guarantee of any kind. I cannot predict how different firmware versions may react to specific configuration file formats. Use this tool at your own risk — I accept no liability for any problems, damage, or unexpected behavior that may result from using it.

## Development

Requires Rust 1.70+ and [mise](https://mise.jdx.dev/) for task management.

### Setup

```bash
# Install tools and dependencies
mise install
mise run setup
```

> **Note:** `mise run setup` installs a pinned Rust nightly toolchain and [cargo-pup](https://github.com/datadog/cargo-pup) for architecture linting. This is separate from your normal Rust installation and does not affect regular builds.

### Available Tasks

```bash
mise tasks ls
```

This will show all available development tasks with descriptions (build, test, coverage, etc.).
