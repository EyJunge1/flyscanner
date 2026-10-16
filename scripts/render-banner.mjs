/**
 * GitHub README banner — same recipe as opencode-mem/web/scripts/render-banner.mjs
 * Needs JetBrains Mono TTFs under /tmp/jetbrains-mono/fonts/ttf/
 * and @resvg/resvg-js (e.g. from opencode-mem/web/node_modules).
 */
import { createRequire } from "node:module";
import { writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const out = path.join(root, ".github/pics/banner.png");
const fontDir = "/tmp/jetbrains-mono/fonts/ttf";

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

/** Keep in sync with frontend/src/routes/layout.css */
const colors = {
	panel: "#111111",
	card: "#141414",
	foregroundBright: "#ffffff",
	muted: "#8a8a8a",
	primary: "#38bdf8", // sky accent
};

const W = 1200;
const H = 300;
const CX = W / 2;

/** JetBrains Mono advance ≈ 0.6em */
function monoWidth(text, size) {
	return text.length * size * 0.6;
}

/** Lucide "plane" (24×24) */
const planePaths = `
  <path d="M17.8 19.2 16 11l3.5-3.5C21 6 21.5 4 21 3c-1-.5-3 0-4.5 1.5L13 8 4.8 6.2c-.5-.1-.9.1-1.1.5l-.3.5c-.2.5-.1 1 .3 1.3L9 12l-2 3H4l-1 1 3 2 2 3 1-1v-3l3-2 3.5 5.3c.3.4.8.5 1.3.3l.5-.2c.4-.3.6-.7.5-1.2z"/>
`;

const brand = "flyscanner";
const brandSize = 48;
const logo = 68;
const brandGap = 20;
const brandTextW = monoWidth(brand, brandSize);
const brandGroupW = logo + brandGap + brandTextW;
const brandX = CX - brandGroupW / 2;
const brandY = 95;

const subtitle = "MODERN ADS-B ALL-IN-ONE STACK";

const svg = `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}">
  <rect width="${W}" height="${H}" fill="${colors.panel}"/>
  <rect x="1" y="1" width="${W - 2}" height="${H - 2}" rx="12" fill="${colors.panel}" stroke="${colors.primary}" stroke-width="2"/>

  <g transform="translate(${brandX}, ${brandY})">
    <rect width="${logo}" height="${logo}" rx="16" fill="${colors.card}"/>
    <rect width="${logo}" height="${logo}" rx="16" fill="${colors.primary}" fill-opacity="0.15"/>
    <g transform="translate(10, 10)" stroke="${colors.primary}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none">
      <g transform="scale(2)">${planePaths}</g>
    </g>
    <text
      x="${logo + brandGap}"
      y="${logo / 2}"
      text-anchor="start"
      font-family="JetBrains Mono"
      font-weight="700"
      font-size="${brandSize}"
      fill="${colors.foregroundBright}"
      dominant-baseline="middle"
    >${brand}</text>
  </g>

  <text
    x="${CX}"
    y="210"
    text-anchor="middle"
    font-family="JetBrains Mono"
    font-weight="400"
    font-size="15"
    fill="${colors.muted}"
    letter-spacing="3"
  >${subtitle}</text>
</svg>`;

const resvg = new Resvg(svg, {
	fitTo: { mode: "width", value: W },
	font: {
		fontFiles: [
			`${fontDir}/JetBrainsMono-Bold.ttf`,
			`${fontDir}/JetBrainsMono-Regular.ttf`,
			`${fontDir}/JetBrainsMono-SemiBold.ttf`,
		],
		loadSystemFonts: false,
		defaultFontFamily: "JetBrains Mono",
	},
});

writeFileSync(out, resvg.render().asPng());
console.log("wrote", out, `${resvg.width}x${resvg.height}`);
