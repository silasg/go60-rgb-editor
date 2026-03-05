# Code Review: Web App (commits d046cd0..b2686c6)

Review ran in a clean subagent context, checked against **code-review**, **typescript**, **coding-style**, and **conventional-commits** skills.

## Commits Reviewed

| # | Hash | Message |
|---|------|---------|
| 1 | `d046cd0` | `fix: add WASM build dependencies and fix wasm-bindgen version mismatch` |
| 2 | `25c05d3` | `feat: add web editor SPA with WASM integration` |
| 3 | `d3e4dd1` | `feat(web): load default config on startup and move cursor on key click` |
| 4 | `4445e14` | `refactor(web): move clear button into palette grid as regular swatch` |
| 5 | `b2686c6` | `chore: add wasm setup to mise and ignore .rodney directory` |

## Critical — fix before merging (8)

| # | Category | Location | Finding |
|---|----------|----------|---------|
| 1 | CI | `deploy-web.yml:40` | **CI will fail** — `npm ci` requires `package-lock.json` which is not committed |
| 2 | CI | `deploy-web.yml:42` | **CI skips type checking** — runs `npx vite build` instead of `npm run build`, bypassing `tsc` |
| 3 | CI | `deploy-web.yml:29` | **CI version drift** — `wasm-bindgen-cli` not pinned; the exact bug commit #1 was fixing |
| 4 | Dead code | `keyboard.ts:23` | `selectedColor` param passed to `renderHalf`/`renderKeyboard` but **never used** — `tsc` would flag this with current tsconfig |
| 5 | Encapsulation | `domain-wasm/src/lib.rs:375-397` | `set_color_at()` **bypasses domain API** — reimplements undo/modified logic instead of calling `EditorState::set_key_color()` |
| 6 | Function length | `domain-wasm/src/lib.rs:185-268`, `keyboard.ts:18-82` | `get_state_json` (84 lines) and `renderHalf` (65 lines) exceed the project's 60-line limit |
| 7 | Duplication | `domain-wasm/src/lib.rs:192-350` | Palette/layer serialization **copy-pasted** between `get_state_json()`, `get_layers_json()`, `get_palette_json()` (~120 duplicated lines) |
| 8 | Dead code | Multiple | `getLastSetValue()`, `getConfigText()`, `get_layers_json()`, `get_palette_json()` — exported but never called anywhere |

## Suggestions — consider improving (12)

| # | Category | Location | Finding |
|---|----------|----------|---------|
| 1 | TypeScript skill | `state.ts` | All interface fields lack `readonly` — skill requires immutable-by-default |
| 2 | TypeScript skill | `layers.ts:3`, `main.ts`, `editor-bridge.ts` | String params should be literal unions (`'left' \| 'right'`, `'add' \| 'duplicate' \| ...`) |
| 3 | TypeScript skill | `main.ts:60`, `config-text.ts:4,18` | `as HTMLTextAreaElement` assertions without justifying comments |
| 4 | Abstraction | `main.ts:110-141` | `onLayerAction()` mixes UI interaction (`prompt()`, `confirm()`) with domain bridge calls |
| 5 | Conventional commits | Commits #1,#2,#3,#5 | Missing scopes, "and" in subjects (multiple changes per commit), wrong type on #1 (`fix:` → `build:`) — **not actionable**, commits already pushed |
| 6 | Duplication | `geometry.ts` vs `geometry.rs` | Constants manually duplicated from Rust domain — will drift. Expose via WASM or add cross-reference test |
| 7 | ADR | `docs/adr/` | No ADR for the web architecture decision (SPA + WASM + JSON state bridge) |
| 8 | Docs | `AGENTS.md` | Not updated with `web/` directory structure or new mise tasks |
| 9 | Mutable state | `editor-bridge.ts:4` | Module-level `let editor: Editor \| null` — global mutable singleton |
| 10 | Dead code | `config-text.ts:1,13,22-24` | `lastSetValue` + its getter — never consumed |
| 11 | Abstraction | `keyboard.ts:18-82` | `renderHalf` mixes DOM creation, grid math, color lookup, cursor highlighting, and event binding |
| 12 | CI | `mise.toml` | `wasm-bindgen-cli` version should be pinned in mise setup task too, not just CI |

## Resolved

### ~~Critical #9: Zero tests~~ → resolved in `ed102e3`

E2E test suite added with Playwright (headless Chrome). Three user journey tests:

| # | Journey | File | Covers |
|---|---------|------|--------|
| 1 | Paint keyboard | `web/e2e/paint-keyboard.spec.ts` | Color painting, clearing, cursor, undo/redo, modified indicator, config output |
| 2 | Manage layers | `web/e2e/manage-layers.spec.ts` | Add, duplicate, rename, switch, fade delay ±, delete layers |
| 3 | Load config | `web/e2e/load-config.spec.ts` | Default config load, invalid config error, recovery with valid config |

Each test was verified red-green: production code was broken, test caught the breakage, code was reverted, test passed.

Run with `mise run web-e2e` or `cd web && npm run test:e2e`.

## Passed

- `strict: true` in tsconfig with `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`
- Zero `any` usage across all TypeScript files
- `type` used for data shapes (no misuse of `interface`)
- No barrel `index.ts` re-exports
- Clean component separation (`keyboard`, `palette`, `layers`, `toolbar`, `config-text`)
- Domain change (`editor.rs`) is minimal — only adds `set_cursor()` method
- `editor-bridge.ts` provides clean isolation between WASM and UI
- Vite + WASM plugin configuration is correct

## Summary

**Files reviewed:** 21 | **Findings:** 20 (8 critical, 12 suggestions)

The biggest themes: CI is broken as-is (#1–#3), the WASM crate has encapsulation and duplication issues (#5, #7). TypeScript standards are mostly followed but need `readonly` fields and narrower types.

Tests are now covered (resolved). Remaining critical items are CI fixes and code quality in the WASM crate.
