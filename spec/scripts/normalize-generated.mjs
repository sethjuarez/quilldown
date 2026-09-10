import { readdir, readFile, writeFile } from "node:fs/promises";
import { extname, join, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..", "..");
const targets = [
  "runtimes/python/src/quilldown_spec",
  "runtimes/python/tests",
];

async function normalizeTree(dir) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      await normalizeTree(path);
    } else if (extname(entry.name) === ".py") {
      const text = await readFile(path, "utf8");
      const normalized = text.replace(/\r\n/g, "\n");
      if (normalized !== text) {
        await writeFile(path, normalized, "utf8");
      }
    }
  }
}

await Promise.all(targets.map((target) => normalizeTree(join(root, target))));
