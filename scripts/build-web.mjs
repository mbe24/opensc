// Build the WebAssembly game and assemble a static `dist/` ready to serve or
// deploy to GitHub Pages. Node stdlib only — no dependencies.
//
//   node scripts/build-web.mjs
//
// macroquad compiles straight to wasm32-unknown-unknown (no wasm-bindgen); the
// vendored `web/mq_js_bundle.js` loads the raw `.wasm`. All paths in index.html
// are relative, so it works under the GitHub Pages `/<repo>/` subpath.

import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, copyFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const PROFILE = "wasm-release";
const TARGET = "wasm32-unknown-unknown";
const WASM = "stuntcopter.wasm";
const dist = join(root, "dist");

const built = join(root, "target", TARGET, PROFILE, WASM);
const web = join(root, "web");

// Optional cargo features, comma-separated, via `FEATURES=debug-controls`. The
// deployed build passes none, so debug/test controls never ship.
const features = (process.env.FEATURES ?? "").trim();
const featureArgs = features ? ["--features", features] : [];

console.log("› cargo build --profile", PROFILE, "--target", TARGET, ...featureArgs);
execFileSync(
    "cargo",
    ["build", "--profile", PROFILE, "--target", TARGET, ...featureArgs],
    { cwd: root, stdio: "inherit" },
);

if (!existsSync(built)) {
    throw new Error(`expected wasm at ${built} — did the build target change?`);
}

// Overwrite files in place (don't remove dist/ — a running static server may
// have it as its working directory, which Windows won't let us delete). The
// output filenames are fixed, so overwriting is a clean refresh.
mkdirSync(dist, { recursive: true });
copyFileSync(built, join(dist, WASM));
copyFileSync(join(web, "index.html"), join(dist, "index.html"));
copyFileSync(join(web, "mq_js_bundle.js"), join(dist, "mq_js_bundle.js"));
// GitHub Pages runs Jekyll otherwise, which drops underscore-prefixed paths.
writeFileSync(join(dist, ".nojekyll"), "");

console.log(`✓ dist/ assembled — serve over http (e.g. \`npx serve dist\`).`);
