/**
 * GitHub / Open Graph social preview (1280×640).
 * Layout like chaya2mqtt: icon left + stacked title/tagline, soft accent glow, no frames.
 *
 *   node scripts/render-social-preview.mjs
 */
import { createRequire } from "node:module";
import { writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const out = path.join(root, ".github/pics/social-preview.png");
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

/** Dark tokens — chaya-style layout; flyscanner dark shell + sky accent */
const colors = {
	bg: "#0a0a0a",
	text: "#ffffff",
	muted: "#8a8a8a",
	accent: "#38bdf8",
};

const W = 1280;
const H = 640;

/** Lucide "plane" (24×24 viewBox) */
const planePath =
	"M17.8 19.2 16 11l3.5-3.5C21 6 21.5 4 21 3c-1-.5-3 0-4.5 1.5L13 8 4.8 6.2c-.5-.1-.9.1-1.1.5l-.3.5c-.2.5-.1 1 .3 1.3L9 12l-2 3H4l-1 1 3 2 2 3 1-1v-3l3-2 3.5 5.3c.3.4.8.5 1.3.3l.5-.2c.4-.3.6-.7.5-1.2z";

const iconSize = 160;
const iconScale = iconSize / 24;
const title = "flyscanner";
const subtitle = "MODERN ADS-B ALL-IN-ONE";
const titleSize = 72;
const subSize = 28;

// Horizontal group centered like chaya (icon + gap + text)
const gap = 48;
const textBlockW = 520;
const groupW = iconSize + gap + textBlockW;
const groupX = (W - groupW) / 2;
const groupY = (H - iconSize) / 2;
const textX = groupX + iconSize + gap;
const titleY = groupY + iconSize / 2 - 18;
const subY = groupY + iconSize / 2 + 36;

const svg = `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}">
  <rect width="${W}" height="${H}" fill="${colors.bg}"/>

  <!-- plane icon only — no glow/shadow -->
  <g transform="translate(${groupX}, ${groupY})">
    <g
      transform="scale(${iconScale})"
      fill="${colors.accent}"
      stroke="none"
    >
      <path d="${planePath}"/>
    </g>
  </g>

  <text
    x="${textX}"
    y="${titleY}"
    text-anchor="start"
    font-family="JetBrains Mono"
    font-weight="700"
    font-size="${titleSize}"
    fill="${colors.text}"
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
