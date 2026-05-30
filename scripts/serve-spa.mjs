#!/usr/bin/env node

import { createReadStream, existsSync, statSync } from "node:fs";
import { createServer } from "node:http";
import { extname, join, normalize, resolve, sep } from "node:path";

const root = resolve(process.argv[2] || "dist");
const port = Number(process.argv[3] || "4173");

const types = new Map([
  [".css", "text/css; charset=utf-8"],
  [".html", "text/html; charset=utf-8"],
  [".js", "text/javascript; charset=utf-8"],
  [".json", "application/json; charset=utf-8"],
  [".svg", "image/svg+xml"],
  [".wasm", "application/wasm"],
]);

function fileFor(urlPath) {
  const decoded = decodeURIComponent(urlPath.split("?")[0] || "/");
  const clean = normalize(decoded).replace(/^(\.\.[/\\])+/, "");
  const candidate = resolve(join(root, clean));
  if (!candidate.startsWith(root + sep) && candidate !== root) {
    return join(root, "index.html");
  }
  if (existsSync(candidate) && statSync(candidate).isFile()) {
    return candidate;
  }
  return join(root, "index.html");
}

const server = createServer((req, res) => {
  const file = fileFor(req.url || "/");
  const contentType = types.get(extname(file)) || "application/octet-stream";
  res.writeHead(200, { "Content-Type": contentType });
  createReadStream(file).pipe(res);
});

server.listen(port, "127.0.0.1", () => {
  console.log(`serving ${root} on http://127.0.0.1:${port}`);
});
