/**
 * GitHub / Open Graph social preview (1280×640) — same brand layout as banner.
 *
 *   node scripts/render-social-preview.mjs
 */
import { createRequire } from "node:module";
import { writeFileSync } from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { brandLayout, fontDir, root } from "./render-pics-shared.mjs";

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
const out = path.join(root, ".github/pics/social-preview.png");
const { svg, colors } = brandLayout({ W: 1280, H: 640 });

const resvg = new Resvg(svg, {
	fitTo: { mode: "width", value: 1280 },
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
console.log("colors:", colors);
