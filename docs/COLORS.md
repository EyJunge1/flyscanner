# UI colors (Flyscanner)

Neutrals match **opencode-mem** 1:1 (`web/src/app.css`). Accent pair is the same idea (light + dark), but **sky blue** instead of violet.

Theme: `dark` class on `<html>` (same as opencode-mem). Persistence: `localStorage` key `flyscanner-theme`.

**Single source of truth:** [`frontend/src/routes/layout.css`](../frontend/src/routes/layout.css)

## Tokens

Swatches = Light | Dark.

| Token | Light | Dark | Role |
|-------|-------|------|------|
| `--ui-bg` | ![](https://img.shields.io/badge/-f5f4f8?style=flat-square&color=f5f4f8) `#f5f4f8` | ![](https://img.shields.io/badge/-0a0a0a?style=flat-square&color=0a0a0a) `#0a0a0a` | `--background` |
| `--ui-text` | ![](https://img.shields.io/badge/-1b1a1f?style=flat-square&color=1b1a1f) `#1b1a1f` | ![](https://img.shields.io/badge/-d4d4d4?style=flat-square&color=d4d4d4) `#d4d4d4` | `--foreground` |
| `--ui-text-bright` | ![](https://img.shields.io/badge/-0a0a0a?style=flat-square&color=0a0a0a) `#0a0a0a` | ![](https://img.shields.io/badge/-ffffff?style=flat-square&color=ffffff) `#ffffff` | `--foreground-bright` |
| `--ui-surface` | ![](https://img.shields.io/badge/-ffffff?style=flat-square&color=ffffff) `#ffffff` | ![](https://img.shields.io/badge/-141414?style=flat-square&color=141414) `#141414` | `--card` / `--popover` |
| `--ui-surface-hover` | ![](https://img.shields.io/badge/-ebeaf0?style=flat-square&color=ebeaf0) `#ebeaf0` | ![](https://img.shields.io/badge/-1a1a1a?style=flat-square&color=1a1a1a) `#1a1a1a` | `--surface-hover` |
| `--ui-sidebar` | ![](https://img.shields.io/badge/-f0eef5?style=flat-square&color=f0eef5) `#f0eef5` | ![](https://img.shields.io/badge/-111111?style=flat-square&color=111111) `#111111` | `--sidebar` |
| `--ui-border` | ![](https://img.shields.io/badge/-dddce3?style=flat-square&color=dddce3) `#dddce3` | ![](https://img.shields.io/badge/-2a2a2a?style=flat-square&color=2a2a2a) `#2a2a2a` | `--border` |
| `--ui-muted` | ![](https://img.shields.io/badge/-6b6874?style=flat-square&color=6b6874) `#6b6874` | ![](https://img.shields.io/badge/-8a8a8a?style=flat-square&color=8a8a8a) `#8a8a8a` | `--muted-foreground` |
| `--ui-accent` | ![](https://img.shields.io/badge/-0284c7?style=flat-square&color=0284c7) `#0284c7` | ![](https://img.shields.io/badge/-38bdf8?style=flat-square&color=38bdf8) `#38bdf8` | Sky blue ↔ `--primary` |
| `--ui-accent-fg` | ![](https://img.shields.io/badge/-f0f9ff?style=flat-square&color=f0f9ff) `#f0f9ff` | ![](https://img.shields.io/badge/-0c4a6e?style=flat-square&color=0c4a6e) `#0c4a6e` | ↔ `--primary-foreground` |
| `--ui-accent-soft` | ![](https://img.shields.io/badge/-e0f2fe?style=flat-square&color=e0f2fe) `#e0f2fe` | ![](https://img.shields.io/badge/-0a1628?style=flat-square&color=0a1628) `#0a1628` | ↔ `--accent` |
| `--ui-danger` | ![](https://img.shields.io/badge/-dc2626?style=flat-square&color=dc2626) `#dc2626` | ![](https://img.shields.io/badge/-f44336?style=flat-square&color=f44336) `#f44336` | `--destructive` |
| `--ui-warn` | ![](https://img.shields.io/badge/-d97706?style=flat-square&color=d97706) `#d97706` | ![](https://img.shields.io/badge/-f0b429?style=flat-square&color=f0b429) `#f0b429` | `--warning` |
| `--ui-ring` | ![](https://img.shields.io/badge/-0284c7?style=flat-square&color=0284c7) `#0284c7` | ![](https://img.shields.io/badge/-38bdf8?style=flat-square&color=38bdf8) `#38bdf8` | `--ring` |
| `--ui-ok` | ![](https://img.shields.io/badge/-059669?style=flat-square&color=059669) `#059669` | ![](https://img.shields.io/badge/-34d399?style=flat-square&color=34d399) `#34d399` | *(Flyscanner only: live)* |

### Quick look (Dark)

![](https://img.shields.io/badge/-0a0a0a?style=for-the-badge&color=0a0a0a)
![](https://img.shields.io/badge/-141414?style=for-the-badge&color=141414)
![](https://img.shields.io/badge/-2a2a2a?style=for-the-badge&color=2a2a2a)
![](https://img.shields.io/badge/-d4d4d4?style=for-the-badge&color=d4d4d4)
![](https://img.shields.io/badge/-8a8a8a?style=for-the-badge&color=8a8a8a)
![](https://img.shields.io/badge/-38bdf8?style=for-the-badge&color=38bdf8)
![](https://img.shields.io/badge/-34d399?style=for-the-badge&color=34d399)
![](https://img.shields.io/badge/-f44336?style=for-the-badge&color=f44336)
![](https://img.shields.io/badge/-f0b429?style=for-the-badge&color=f0b429)

### Quick look (Light)

![](https://img.shields.io/badge/-f5f4f8?style=for-the-badge&color=f5f4f8)
![](https://img.shields.io/badge/-ffffff?style=for-the-badge&color=ffffff)
![](https://img.shields.io/badge/-dddce3?style=for-the-badge&color=dddce3)
![](https://img.shields.io/badge/-1b1a1f?style=for-the-badge&color=1b1a1f)
![](https://img.shields.io/badge/-6b6874?style=for-the-badge&color=6b6874)
![](https://img.shields.io/badge/-0284c7?style=for-the-badge&color=0284c7)
![](https://img.shields.io/badge/-059669?style=for-the-badge&color=059669)
![](https://img.shields.io/badge/-dc2626?style=for-the-badge&color=dc2626)
![](https://img.shields.io/badge/-d97706?style=for-the-badge&color=d97706)

## Rules

1. Change hex values **only** in `layout.css`. Banner + social preview read `.dark` from there (`npm run pics`).
2. Markup: use tokens / Tailwind aliases — no `sky-*` / `slate-*` hardcodes.
3. MapLibre overlays: `cssVar('--ui-…')` and reset on theme change.
