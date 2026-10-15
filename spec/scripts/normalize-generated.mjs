import { readdir, readFile, writeFile } from "node:fs/promises";
import { extname, join, resolve } from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const root = resolve(import.meta.dirname, "..", "..");
const targets = [
  "runtimes/python/src/quilldown_spec",
  "runtimes/python/tests",
];
const rustTargets = [
  "runtimes/rust/quilldown/src/typra_spec",
  "runtimes/rust/quilldown/tests",
];
const execFileAsync = promisify(execFile);

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

async function rustFiles(dir) {
  const files = [];
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...await rustFiles(path));
    } else if (extname(entry.name) === ".rs") {
      files.push(path);
    }
  }
  return files;
}

const generatedRustFiles = (await Promise.all(
  rustTargets.map((target) => rustFiles(join(root, target)))
)).flat();
if (generatedRustFiles.length) {
  await execFileAsync("rustfmt", ["--edition", "2021", ...generatedRustFiles], {
    cwd: root,
  });
}
