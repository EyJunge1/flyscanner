/**
 * Shared layout + colors for banner / social preview.
 * Colors from frontend/src/routes/layout.css (.dark).
 */
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
export const root = path.resolve(__dirname, "..");
export const cssPath = path.join(root, "frontend/src/routes/layout.css");
export const fontDir = "/tmp/jetbrains-mono/fonts/ttf";

export function loadDarkUiColors() {
	const css = readFileSync(cssPath, "utf8");
	const block = css.match(/\.dark\s*\{([^}]+)\}/);
	if (!block) throw new Error(`no .dark block in ${cssPath}`);
	const colors = {};
	for (const m of block[1].matchAll(/--ui-([a-z0-9-]+)\s*:\s*(#[0-9a-fA-F]{3,8})\s*;/g)) {
		colors[m[1]] = m[2].toLowerCase();
	}
	return colors;
}

export const planePaths = `
  <path d="M17.8 19.2 16 11l3.5-3.5C21 6 21.5 4 21 3c-1-.5-3 0-4.5 1.5L13 8 4.8 6.2c-.5-.1-.9.1-1.1.5l-.3.5c-.2.5-.1 1 .3 1.3L9 12l-2 3H4l-1 1 3 2 2 3 1-1v-3l3-2 3.5 5.3c.3.4.8.5 1.3.3l.5-.2c.4-.3.6-.7.5-1.2z"/>
`;

/**
 * Same visual recipe for both sizes:
 * logo left (rounded tile + stroke frame), title + subtitle stacked right.
 */
export function brandLayout({ W, H }) {
	const ui = loadDarkUiColors();
	const colors = {
		bg: ui.bg,
		surface: ui.surface,
		border: ui.border,
		title: ui["text-bright"],
		muted: ui.muted,
		accent: ui.accent,
	};

	const logo = Math.round(H * 0.32);
	const rx = Math.round(logo * 0.22);
	const pad = Math.round(logo * 0.188);
	const planeScale = (logo - pad * 2) / 24;
	// stroke is on the unscaled parent `g` (same as old banner: width 2 @ logo~68)
	const strokePlane = Math.max(2, logo * 0.03);
	const frameStroke = Math.max(1.5, logo * 0.02);

	const title = "flyscanner";
	const subtitle = "ADS-B ALL-IN-ONE";
	const titleSize = Math.round(H * 0.173);
	const subSize = Math.round(H * 0.06);
	const gap = Math.round(logo * 0.33);
	const textBlockW = Math.round(titleSize * 10.5);

	const groupW = logo + gap + textBlockW;
	const groupX = (W - groupW) / 2;
	const groupY = (H - logo) / 2;
	const textX = groupX + logo + gap;
	const titleY = groupY + logo / 2 - Math.round(titleSize * 0.28);
	const subY = groupY + logo / 2 + Math.round(titleSize * 0.48);

	const svg = `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}">
  <rect width="${W}" height="${H}" fill="${colors.bg}"/>

  <g transform="translate(${groupX}, ${groupY})">
    <rect width="${logo}" height="${logo}" rx="${rx}" fill="${colors.surface}"/>
    <rect width="${logo}" height="${logo}" rx="${rx}" fill="${colors.accent}" fill-opacity="0.15"/>
    <rect
      x="${frameStroke / 2}"
      y="${frameStroke / 2}"
      width="${logo - frameStroke}"
      height="${logo - frameStroke}"
      rx="${Math.max(0, rx - frameStroke / 2)}"
      fill="none"
      stroke="${colors.border}"
      stroke-width="${frameStroke}"
    />
    <g
      transform="translate(${pad}, ${pad})"
      stroke="${colors.accent}"
      stroke-width="${strokePlane}"
      stroke-linecap="round"
      stroke-linejoin="round"
      fill="none"
    >
      <g transform="scale(${planeScale})">${planePaths}</g>
    </g>
  </g>

  <text
    x="${textX}"
    y="${titleY}"
    text-anchor="start"
    font-family="JetBrains Mono"
    font-weight="700"
    font-size="${titleSize}"
    fill="${colors.title}"
    dominant-baseline="middle"
  >${title}</text>

  <text
    x="${textX}"
    y="${subY}"
    text-anchor="start"
    font-family="JetBrains Mono"
    font-weight="400"
    font-size="${subSize}"
    fill="${colors.muted}"
    letter-spacing="2"
    dominant-baseline="middle"
  >${subtitle}</text>
</svg>`;

	return { svg, colors, logo };
}
