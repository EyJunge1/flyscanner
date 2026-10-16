/**
 * GitHub / Open Graph social preview (1280×640).
 * Colors = dark tokens from frontend/src/routes/layout.css (same as Web-UI).
 * Layout like chaya2mqtt: icon left + stacked title/tagline, no frames/glow.
 *
 *   node scripts/render-social-preview.mjs
 */
import { createRequire } from "node:module";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const out = path.join(root, ".github/pics/social-preview.png");
const cssPath = path.join(root, "frontend/src/routes/layout.css");
const fontDir = "/tmp/jetbrains-mono/fonts/ttf";

/** Parse `.dark { --ui-*: #hex; }` from layout.css — keep pics + UI in sync. */
function loadDarkUiColors() {
	const css = readFileSync(cssPath, "utf8");
	const block = css.match(/\.dark\s*\{([^}]+)\}/);
	if (!block) throw new Error(`no .dark block in ${cssPath}`);
	const colors = {};
	for (const m of block[1].matchAll(/--ui-([a-z0-9-]+)\s*:\s*(#[0-9a-fA-F]{3,8})\s*;/g)) {
		colors[m[1]] = m[2].toLowerCase();
	}
	for (const key of ["bg", "text", "text-bright", "muted", "accent"]) {
		if (!colors[key]) throw new Error(`missing --ui-${key} in .dark`);
	}
	return colors;
}

async function loadResvg() {
	const candidates = [
		path.join(root, "node_modules/@resvg/resvg-js/index.js"),
		path.join(root, "frontend/node_modules/@resvg/resvg-js/index.js"),
		"/Users/arneleder/Development/opencode-mem/web/node_modules/@resvg/resvg-js/index.js",
	];
	for (const p of candidates) {
		try {
			return await import(pathToFileURL(p).href);
		} catch {
			/* try next */
		}
	}
	const require = createRequire(import.meta.url);
	return { Resvg: require("@resvg/resvg-js").Resvg };
}

const { Resvg } = await loadResvg();
const ui = loadDarkUiColors();

const colors = {
	bg: ui.bg,
	surface: ui.surface,
	title: ui["text-bright"],
	muted: ui.muted,
	accent: ui.accent,
};

const W = 1280;
const H = 640;

/** Lucide "plane" (24×24) — outline like README banner */
const planePaths = `
  <path d="M17.8 19.2 16 11l3.5-3.5C21 6 21.5 4 21 3c-1-.5-3 0-4.5 1.5L13 8 4.8 6.2c-.5-.1-.9.1-1.1.5l-.3.5c-.2.5-.1 1 .3 1.3L9 12l-2 3H4l-1 1 3 2 2 3 1-1v-3l3-2 3.5 5.3c.3.4.8.5 1.3.3l.5-.2c.4-.3.6-.7.5-1.2z"/>
`;

const logo = 160;
const pad = 28;
const planeScale = (logo - pad * 2) / 24;
const title = "flyscanner";
const subtitle = "MODERN ADS-B ALL-IN-ONE";
const titleSize = 72;
const subSize = 28;

const gap = 48;
const textBlockW = 520;
const groupW = logo + gap + textBlockW;
const groupX = (W - groupW) / 2;
const groupY = (H - logo) / 2;
const textX = groupX + logo + gap;
const titleY = groupY + logo / 2 - 18;
const subY = groupY + logo / 2 + 36;

const svg = `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}">
  <rect width="${W}" height="${H}" fill="${colors.bg}"/>

  <!-- rounded tile like README banner -->
  <g transform="translate(${groupX}, ${groupY})">
    <rect width="${logo}" height="${logo}" rx="36" fill="${colors.surface}"/>
    <rect width="${logo}" height="${logo}" rx="36" fill="${colors.accent}" fill-opacity="0.15"/>
    <g
      transform="translate(${pad}, ${pad})"
      stroke="${colors.accent}"
      stroke-width="2"
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

const resvg = new Resvg(svg, {
	fitTo: { mode: "width", value: W },
	font: {
		fontFiles: [
			`${fontDir}/JetBrainsMono-Bold.ttf`,
			`${fontDir}/JetBrainsMono-Regular.ttf`,
			`${fontDir}/JetBrainsMono-SemiBold.ttf`,
			`${fontDir}/JetBrainsMono-Medium.ttf`,
		],
		loadSystemFonts: false,
		defaultFontFamily: "JetBrains Mono",
	},
});

writeFileSync(out, resvg.render().asPng());
console.log("wrote", out, `${resvg.width}x${resvg.height}`);
console.log("colors from layout.css .dark:", colors);
