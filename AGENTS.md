# Go60 RGB Editor

TUI RGB underglow editor for ZMK keyboards (MoErgo Go60 layout).

## Project Structure

```
src/
├── main.rs          # Entry point, CLI args
├── app.rs           # Application state & logic
├── event.rs         # Key event handling
├── tui.rs           # Terminal setup/teardown
├── model/           # Data models
│   ├── color.rs     # Color definitions, palette
│   ├── layer.rs     # Layer structure
│   └── config.rs    # Overall config
├── parser/          # Config file parsing
│   ├── grammar.rs   # Main parser
│   ├── lexer.rs     # Nom combinators (utility)
│   └── writer.rs    # Serialize back to file
└── ui/              # UI widgets
    ├── keyboard.rs  # Keyboard layout widget
    ├── layer_list.rs
    ├── color_picker.rs
    ├── status_bar.rs
    └── help.rs
```

## Tech Stack

- Rust
- Ratatui (TUI framework)
- Crossterm (terminal backend)
- Nom (parser combinators)

## Building & Testing

```bash
cargo build           # Dev build
cargo build --release # Release build
cargo test            # Run tests
```

## Running

```bash
cargo run -- "Go60 TK Latest RGB scheme.txt"
```

## Config File Format

The editor parses TailorKey RGB config files. See:
https://sites.google.com/view/tailorkey/how-to/rgb
