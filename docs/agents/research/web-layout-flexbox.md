# Web Editor Layout — Flexbox Concept

How the Go60 RGB Editor web app uses nested CSS flexbox to fill the viewport, constrain width on large screens, and remain scrollable on small ones.

## DOM Structure

```
<body>                          ← flex column, height: 100%
  <section #hero>               ← flex-shrink: 0 (natural height)
  <div #app>                    ← flex: 1 (fills remaining height)
    <header #header>            ← flex-shrink: 0
    <main #main>                ← flex: 1, flex row
      <aside #layer-panel>      ← fixed width: 180px
        <h2>                    ← heading
        <div #layer-actions>    ← action buttons (fixed)
        <div #layer-list>       ← flex: 1, overflow-y: auto (scrolls)
      <section #editor-panel>   ← flex: 1, flex column
        <div #palette-section>  ← natural height
        <div #keyboard-section> ← natural height
        <div #config-section>   ← flex: 1 (fills remaining height)
          <div #config-header>  ← natural height
          <textarea #config-text> ← flex: 1 (fills remaining height)
```

## Four Nested Flex Containers

The layout uses four levels of flexbox nesting. Each level solves a different problem.

### Level 1: body (vertical)

```css
body {
  display: flex;
  flex-direction: column;
  height: 100%;
}
```

**Problem:** The hero section sits outside the app. We need the app to fill whatever height remains after the hero.

**How it works:** `body` is a flex column at full viewport height. The hero gets its natural size (`flex-shrink: 0`), and `#app` gets the rest (`flex: 1`).

### Level 2: #app (vertical)

```css
#app {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 600px;
  overflow: hidden;
}
```

**Problem:** The app has a fixed-height header bar and a main area that should fill the rest.

**How it works:** Another flex column. `#header` has `flex-shrink: 0` so it keeps its natural size. `#main` gets `flex: 1` to fill the rest.

**Key properties:**
- `overflow: hidden` — prevents the app itself from scrolling. This forces all flex children to fit within the available height. Without this, `flex: 1` children would expand to their content size and the app would scroll.
- `min-height: 600px` — safety net for small screens. If the viewport is too short for hero + 600px, the **body** overflows and the page gets a scrollbar. This ensures the app content is never squished to zero.

### Level 3: #main (horizontal)

```css
#main {
  display: flex;       /* row is the default direction */
  flex: 1;
  overflow: hidden;
}
```

**Problem:** Layer panel on the left, editor area filling the rest.

**How it works:** Flex row. `#layer-panel` has fixed `width: 180px` with `flex-shrink: 0`. `#editor-panel` gets `flex: 1` for the remaining horizontal space.

### Level 4: #editor-panel (vertical)

```css
#editor-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}
```

**Problem:** Palette and keyboard have fixed heights, but the config textarea should fill whatever vertical space remains.

**How it works:** Flex column. Palette and keyboard take their natural height. `#config-section` gets `flex: 1`. Inside that, `#config-text` (the textarea) also gets `flex: 1`, making it expand to fill remaining space. The textarea's native scrollbar handles overflowing config text.

**Key property:**
- `min-height: 0` — **critical for nested flex.** By default, flex items have `min-height: auto`, which means they refuse to shrink below their content size. In a nested flex layout, this cascades up: if a child is tall, the parent won't shrink either, breaking the constraint chain. Setting `min-height: 0` lets the element shrink below its content size, allowing the parent's `overflow: hidden` to actually constrain it.

## Width Constraint

```css
#app {
  max-width: 1300px;
  width: 100%;
  margin: 0 auto;
  border-left: 1px solid var(--border);
  border-right: 1px solid var(--border);
}
```

On wide screens, `max-width` prevents the UI from spreading across the full viewport. `margin: 0 auto` centers it. The borders give a visual frame.

This single constraint on `#app` keeps layers, toolbar, palette, keyboard, and textarea all within the same width — no need for separate width limits on each section.

## The min-height: 0 Pattern

This comes up twice (`#editor-panel` and `#config-section`) and is the most non-obvious part.

**The problem:** In CSS flexbox, the default `min-height` is `auto`, not `0`. This means a flex item will never shrink below the size its content needs. For a simple single-level flex layout, this is usually fine. But when you nest flex containers, it breaks:

```
#app (overflow: hidden, height from flex: 1 = e.g. 700px)
  └─ #main (flex: 1, but min-height: auto)
       └─ #editor-panel (flex: 1, but min-height: auto)
            ├─ palette (100px)
            ├─ keyboard (250px)
            └─ #config-section (flex: 1)
                 └─ textarea (content = 2000px)
```

Without `min-height: 0`, the textarea's content size (2000px) propagates upward: config-section refuses to shrink below 2000px, editor-panel refuses to shrink below 2350px (100+250+2000), and the whole layout overflows despite `overflow: hidden` on `#app`.

With `min-height: 0` at each nested level, the flex algorithm can shrink containers below their content size, and `overflow: hidden` (or the textarea's native scroll) handles the overflow.

## Small Screen Fallback

```
Viewport too short for hero + app content
  ↓
body height: 100% (= viewport)
  hero:   ~180px (flex-shrink: 0)
  #app:   min-height: 600px wins over flex: 1
  total:  ~780px > viewport
  ↓
body overflows → page scrollbar appears
```

The `min-height: 600px` on `#app` guarantees a usable minimum. The flex layout inside `#app` still works because `overflow: hidden` keeps the internal flex math constrained to 600px.

## Layer Panel: Pinned Actions

```
#layer-panel (flex column)
  ├─ <h2> Layers           ← natural height
  ├─ #layer-actions         ← natural height, always visible
  └─ #layer-list            ← flex: 1, overflow-y: auto
```

Action buttons sit above the scrollable layer list so they're always visible regardless of how many layers exist. `#layer-list` gets `flex: 1` + `overflow-y: auto` — it takes remaining space and scrolls independently.

## Summary of Key Techniques

| Technique | Where | Why |
|---|---|---|
| `flex: 1` | app, main, editor-panel, config-section, textarea | Fill remaining space at each level |
| `flex-shrink: 0` | hero, header, layer-panel | Keep fixed-size elements from shrinking |
| `overflow: hidden` | app, editor-panel | Force flex children to fit, don't scroll |
| `min-height: 0` | editor-panel, config-section | Allow shrinking below content size in nested flex |
| `min-height: 600px` | app | Guarantee usable minimum, triggers page scroll on small viewports |
| `max-width` + `margin: 0 auto` | app | Constrain width on large screens, center content |
| `overflow-y: auto` | layer-list | Independent scroll for long lists |
