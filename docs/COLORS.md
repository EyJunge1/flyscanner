# UI-Farben (Flyscanner)

Neutrals 1:1 von **opencode-mem** (`web/src/app.css`). Accent-Paar wie dort (hell + dunkel), aber **Sky-Blau** statt Violett.

Theme: Klasse `dark` auf `<html>` (wie opencode-mem). Persistenz: `localStorage` Key `flyscanner-theme`.

**Single source of truth:** [`frontend/src/routes/layout.css`](../frontend/src/routes/layout.css)

## Tokens

| CSS-Variable | Light | Dark | Rolle |
|--------------|-------|------|-------|
| `--ui-bg` | `#f5f4f8` | `#0a0a0a` | `--background` |
| `--ui-text` | `#1b1a1f` | `#d4d4d4` | `--foreground` |
| `--ui-text-bright` | `#0a0a0a` | `#ffffff` | `--foreground-bright` |
| `--ui-surface` | `#ffffff` | `#141414` | `--card` / `--popover` |
| `--ui-surface-hover` | `#ebeaf0` | `#1a1a1a` | `--surface-hover` |
| `--ui-sidebar` | `#f0eef5` | `#111111` | `--sidebar` |
| `--ui-border` | `#dddce3` | `#2a2a2a` | `--border` |
| `--ui-muted` | `#6b6874` | `#8a8a8a` | `--muted-foreground` |
| `--ui-accent` | `#0284c7` | `#38bdf8` | Sky-Blau ↔ `--primary` |
| `--ui-accent-fg` | `#f0f9ff` | `#0c4a6e` | ↔ `--primary-foreground` |
| `--ui-accent-soft` | `#e0f2fe` | `#0a1628` | ↔ `--accent` |
| `--ui-danger` | `#dc2626` | `#f44336` | `--destructive` |
| `--ui-warn` | `#d97706` | `#f0b429` | `--warning` |
| `--ui-ring` | `#0284c7` | `#38bdf8` | `--ring` |
| `--ui-ok` | `#059669` | `#34d399` | *(nur Flyscanner: live)* |

## Regeln

1. Hex-Werte **nur** in `layout.css` ändern (Banner/Favicon: Dark-Accent `#38bdf8`).
2. Markup: Token / Tailwind-Aliase — keine `sky-*` / `slate-*` Hardcodes.
3. MapLibre-Overlays: `cssVar('--ui-…')` und bei Theme-Wechsel neu setzen.
