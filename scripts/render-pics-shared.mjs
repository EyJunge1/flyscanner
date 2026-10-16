/**
 * Shared layout + colors for banner / social / favicon.
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

/** Lucide plane path (24×24) — outline only */
const planeD =
	"M17.8 19.2 16 11l3.5-3.5C21 6 21.5 4 21 3c-1-.5-3 0-4.5 1.5L13 8 4.8 6.2c-.5-.1-.9.1-1.1.5l-.3.5c-.2.5-.1 1 .3 1.3L9 12l-2 3H4l-1 1 3 2 2 3 1-1v-3l3-2 3.5 5.3c.3.4.8.5 1.3.3l.5-.2c.4-.3.6-.7.5-1.2z";

/** JetBrains Mono advance ≈ 0.6em */
function monoWidth(text, size, letterSpacing = 0) {
	return text.length * size * 0.6 + Math.max(0, text.length - 1) * letterSpacing;
}

/** Logo tile markup — identical for banner, social, and favicon. */
export function logoTileInner(logo, accent, surface) {
	const rx = Math.round(logo * (16 / 68));
	const pad = Math.round(logo * (10 / 68));
	const planeScale = 2 * (logo / 68);
	return `
    <rect width="${logo}" height="${logo}" rx="${rx}" fill="${surface}"/>
    <rect width="${logo}" height="${logo}" rx="${rx}" fill="${accent}" fill-opacity="0.15"/>
    <g transform="translate(${pad}, ${pad}) scale(${planeScale})">
      <path
        d="${planeD}"
        fill="none"
        stroke="${accent}"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        vector-effect="non-scaling-stroke"
      />
    </g>`;
}

/**
 * Same visual recipe for both sizes:
 * logo left (rounded tile), crisp outline plane, title + subtitle stacked right.
 */
export function brandLayout({ W, H }) {
	const ui = loadDarkUiColors();
	const colors = {
		bg: ui.bg,
		surface: ui.surface,
		title: ui["text-bright"],
		muted: ui.muted,
		accent: ui.accent,
	};

	// Match original banner ratios (logo 68 / H 300, pad 10, stroke 2, scale 2)
	const logo = Math.round(H * (68 / 300));

	const title = "flyscanner";
	const subtitle = "ADS-B ALL-IN-ONE";
	const titleSize = Math.round(H * (48 / 300));
	const subSize = Math.round(H * (15 / 300));
	const gap = Math.round(logo * (20 / 68));
	const letterSpacing = 2;
	const textBlockW = Math.max(
		monoWidth(title, titleSize),
		monoWidth(subtitle, subSize, letterSpacing),
	);

	const groupW = logo + gap + textBlockW;
	const groupX = (W - groupW) / 2;
	const groupY = (H - logo) / 2;
	const textX = groupX + logo + gap;
	const titleY = groupY + logo / 2 - Math.round(titleSize * 0.28);
	const subY = groupY + logo / 2 + Math.round(titleSize * 0.48);
	const tile = logoTileInner(logo, colors.accent, colors.surface);

	const svg = `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}">
  <rect width="${W}" height="${H}" fill="${colors.bg}"/>

  <g transform="translate(${groupX}, ${groupY})">${tile}
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

/** Favicon = same logo tile as banner (same surface / accent / opacity). */
export function faviconSvg(size = 32) {
	const ui = loadDarkUiColors();
	const tile = logoTileInner(size, ui.accent, ui.surface);
	return `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${size} ${size}" width="${size}" height="${size}" fill="none">${tile}
</svg>
`;
}
