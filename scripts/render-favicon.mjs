/**
 * App favicon — same tile colors as banner / social (layout.css .dark).
 *
 *   node scripts/render-favicon.mjs
 */
import { writeFileSync } from "node:fs";
import path from "node:path";
import { faviconSvg, root } from "./render-pics-shared.mjs";

const out = path.join(root, "frontend/src/lib/assets/favicon.svg");
writeFileSync(out, faviconSvg(32));
console.log("wrote", out);
