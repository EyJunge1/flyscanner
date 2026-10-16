# UI-Farben (Flyscanner)

Kleine **semantische** Palette. Neutrals und Status an **opencode-mem** (Dark-Shell) angelehnt; Accent ist **Blau** statt Lila (`#c4b5fd` → `#38bdf8`).

**Single source of truth:** [`frontend/src/routes/layout.css`](../frontend/src/routes/layout.css)

Dark-only.

## Tokens

| CSS-Variable | Hex | Herkunft / Verwendung |
|--------------|-----|------------------------|
| `--ui-bg` | `#0a0a0a` | opencode-mem `--background` |
| `--ui-surface` | `#141414` | opencode-mem `--card` |
| `--ui-border` | `#2a2a2a` | opencode-mem `--border` |
| `--ui-text` | `#d4d4d4` | opencode-mem `--foreground` |
| `--ui-muted` | `#8a8a8a` | opencode-mem `--muted-foreground` |
| `--ui-accent` | `#38bdf8` | Sky-Blau (statt Violet `#c4b5fd`) |
| `--ui-ok` | `#34d399` | positiv / live |
| `--ui-danger` | `#f44336` | opencode-mem `--destructive` |
| `--ui-warn` | `#f0b429` | opencode-mem `--warning` |

## Regeln

1. Hex-Werte **nur** in `layout.css` ändern.
2. Markup/SVG: `var(--ui-…)` oder Tailwind-Aliase (`bg-bg`, `text-accent`, …).
3. MapLibre: `getComputedStyle(...).getPropertyValue('--ui-…')`.
