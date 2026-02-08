# Go60 RGB Editor

TUI RGB underglow editor for ZMK keyboards (MoErgo Go60 layout).

## Project Structure

```
src/
├── main.rs          # Entry point, CLI args
├── app.rs           # Application state & logic
├── event.rs         # Key event handling
├── tui.rs           # Terminal setup/teardown
├── cursor.rs        # Cursor position for keyboard navigation
├── geometry.rs      # Keyboard layout geometry definitions
├── undo.rs          # Undo/redo history
├── model/           # Data models
│   ├── mod.rs       # Re-exports
│   ├── color.rs     # Color definitions, palette
│   ├── layer.rs     # Layer structure
│   └── config.rs    # Overall config
├── parser/          # Config file parsing
│   ├── mod.rs       # Re-exports
│   ├── reader.rs    # Main parser (nom combinators)
│   ├── writer.rs    # Serialize back to file
│   └── tests.rs     # Parser tests
└── ui/              # UI widgets
    ├── mod.rs       # Re-exports
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

## Development

Uses [mise](https://mise.jdx.dev/) for task management. Run `mise tasks ls` for available tasks.

```bash
mise install          # Install tools (cargo-llvm-cov, git-cliff, cargo-release)
mise run setup        # One-time setup (rustup components)
mise run build        # Build (debug)
mise run build-release # Build (release)
mise run test         # Run clippy + tests
mise run lint         # Run clippy lints
mise run coverage     # Test coverage report
mise run coverage-html # Coverage report in browser
mise run run          # Run editor with example config
mise run changelog    # Preview unreleased changelog entries
mise run release-patch # Release patch version
mise run release-minor # Release minor version
mise run release-major # Release major version
```

## Commits

This project uses [Conventional Commits](https://www.conventionalcommits.org/). All commit messages **must** follow this format:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat:` — new feature (bumps minor version)
- `fix:` — bug fix (bumps patch version)
- `docs:` — documentation only
- `style:` — formatting, no code change
- `refactor:` — code restructuring, no behavior change
- `perf:` — performance improvement
- `test:` — adding/updating tests
- `ci:` — CI/CD changes
- `chore:` — maintenance tasks
- `build:` — build system changes

**Breaking changes:** Add `!` after the type (e.g., `feat!: redesign config format`) or add a `BREAKING CHANGE:` footer. This bumps the major version.

## Releases

Releases are managed with `cargo-release` and `git-cliff` (changelog generation). Both are installed via mise.

```bash
mise run changelog        # Preview/generate CHANGELOG.md
mise run release-patch    # Release patch version (bug fixes)
mise run release-minor    # Release minor version (features)
mise run release-major    # Release major version (breaking)
```

`cargo-release` bumps the version in `Cargo.toml`, generates the changelog via `git-cliff`, commits, and tags. Push the tag manually to trigger the GitHub Actions release workflow which builds binaries for all platforms.

## Config File Format

The editor parses TailorKey RGB config files. See:
https://sites.google.com/view/tailorkey/how-to/rgb

## Documentation Maintenance

When making changes, keep documentation in sync:
- **AGENTS.md** — update project structure when adding/removing/renaming source files; update mise tasks when adding/removing tasks
- **README.md** — update features, usage, or installation instructions when user-facing behavior changes
