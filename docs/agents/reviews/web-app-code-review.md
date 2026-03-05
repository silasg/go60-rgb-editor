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

## Open — remaining suggestions (4)

| # | Category | Location | Finding | Status |
|---|----------|----------|---------|--------|
| 1 | Conventional commits | Commits #1,#2,#3,#5 | Missing scopes, "and" in subjects, wrong type on #1 | **Not actionable** — commits already pushed |
| 2 | Duplication | `geometry.ts` vs `geometry.rs` | Constants manually duplicated from Rust domain — drift risk | **Accepted** — add cross-reference test if drift occurs |
| 3 | ADR | `docs/adr/` | No ADR for web architecture decision (SPA + WASM + JSON state bridge) | **Deferred** — discuss with team first |
| 4 | Mutable state | `editor-bridge.ts:4` | Module-level `let editor` global singleton | **Accepted** — pragmatic for SPA |

## Resolved

### Critical #1–3: CI fixes → `d506935`

- Pinned `wasm-bindgen-cli --version 0.2.114` in CI and mise setup
- Changed CI build from `npx vite build` to `npm run build` (runs `tsc` first)
- `package-lock.json` committed in `ed102e3`

### Critical #4, #8 + Suggestion #10: Dead code removal → `b18771f`

- Removed unused `selectedColor` parameter from `renderHalf`/`renderKeyboard`
- Removed dead exports: `getLastSetValue()`, `getConfigText()`, `lastSetValue`
- Removed dead import of `getConfigText` in `main.ts`

### Critical #5: Encapsulation violation → `f441654`

- Added `set_key_color_at()` to domain `EditorState`
- WASM `set_color_at()` now delegates to domain API instead of reimplementing undo/modified logic

### Critical #6–7 + Dead code: WASM duplication and function length → `8790e8c`

- Extracted `serialize_layers()`, `serialize_palette()`, `serialize_color()` helpers
- Removed dead `get_layers_json()` and `get_palette_json()` (never called)
- `get_state_json()` reduced from 84 lines to ~20 lines

### Critical #9 (former): Zero tests → `ed102e3`

- Added Playwright E2E suite with 3 user journey tests (headless Chrome)
- Each verified red-green with production code breakage
- Added `web-e2e` mise task

### Suggestions #1–3: TypeScript type safety → `63da1bd`

- Added `readonly` to all WASM-facing interface fields in `state.ts`
- Narrowed string params to literal unions: `Half`, `LayerAction`, `'left' | 'right'`
- Added justifying comment for `as HTMLTextAreaElement` assertion

### Suggestions #4, #11: Abstraction fixes → `e2fc702`

- Split `onLayerAction()` into dedicated `handleAddLayer/Duplicate/Rename/DeleteLayer` handlers
- Extracted `createKeyButton()` and `applyKeyStyle()` from `renderHalf()`
- `renderHalf` reduced from 65 lines to ~25 lines

### Suggestion #8: Documentation → `6769691`

- Updated AGENTS.md with `web/` directory structure, dependency graph, tech stack, and new mise tasks

### Suggestion #12: mise.toml pin → `d506935`

- Pinned `wasm-bindgen-cli --version 0.2.114` in `[tasks.setup]`

## Passed (unchanged)

- `strict: true` in tsconfig with `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`
- Zero `any` usage across all TypeScript files
- `type` used for data shapes (no misuse of `interface`)
- No barrel `index.ts` re-exports
- Clean component separation (`keyboard`, `palette`, `layers`, `toolbar`, `config-text`)
- Domain change (`editor.rs`) is minimal — only adds `set_cursor()` and `set_key_color_at()` methods
- `editor-bridge.ts` provides clean isolation between WASM and UI
- Vite + WASM plugin configuration is correct

## Summary

**Original findings:** 20 (8 critical, 12 suggestions)
**Resolved:** 16 (8 critical, 8 suggestions)
**Remaining:** 4 suggestions (1 not actionable, 2 accepted, 1 deferred)

All critical findings fixed. All E2E tests pass.
