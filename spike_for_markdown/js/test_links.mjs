import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parseLinks } from "./src/link_parser.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const vectorsPath = path.resolve(__dirname, "../test_vectors/links.json");
const vectors = JSON.parse(fs.readFileSync(vectorsPath, "utf8"));

for (const vec of vectors) {
  const actual = parseLinks(vec.input).map((l) => [l.title, l.pageId]);
  assert.deepEqual(actual, vec.expected, `Vector failed: ${vec.name}`);
}

console.log(`JS link_parser passed ${vectors.length} vectors.`);
