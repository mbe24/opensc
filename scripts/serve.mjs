// Tiny static server for the built `dist/`. Node stdlib only — no dependencies.
//
//   node scripts/serve.mjs [port]     # default 8099
//
// Sends the correct MIME types (notably application/wasm) and disables caching,
// so a browser refresh always picks up the latest `npm run build:web`.

import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, join, normalize, extname } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "dist");
const port = Number(process.argv[2] ?? process.env.PORT ?? 8099);

const MIME = {
    ".html": "text/html; charset=utf-8",
    ".js": "text/javascript; charset=utf-8",
    ".wasm": "application/wasm",
    ".png": "image/png",
    ".json": "application/json",
    ".css": "text/css; charset=utf-8",
};

const server = createServer(async (req, res) => {
    const url = decodeURIComponent((req.url ?? "/").split("?")[0]);
    const rel = url === "/" ? "index.html" : url.replace(/^\/+/, "");
    const path = normalize(join(root, rel));
    // Refuse to serve outside dist/.
    if (!path.startsWith(root)) {
        res.writeHead(403).end("forbidden");
        return;
    }
    try {
        const body = await readFile(path);
        res.writeHead(200, {
            "Content-Type": MIME[extname(path)] ?? "application/octet-stream",
            "Cache-Control": "no-store",
        });
        res.end(body);
    } catch {
        res.writeHead(404).end("not found");
    }
});

server.listen(port, () => {
    console.log(`Serving dist/ at http://localhost:${port} (Ctrl-C to stop)`);
});
