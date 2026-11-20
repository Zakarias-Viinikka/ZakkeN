// Keep in lockstep with `src/link_parser.rs` and `test_vectors/links.json`.
// Regex: `[[Title|id]]` where Title has no `]` or newline (greedy up to LAST `|`),
// and id has no whitespace, `]`, or `|`.
const WIKILINK_RE = /\[\[([^\]\r\n]+)\|([^\s\]|]+)\]\]/g;

export function parseLinks(raw) {
  const results = [];
  WIKILINK_RE.lastIndex = 0;
  let match;
  while ((match = WIKILINK_RE.exec(raw)) !== null) {
    results.push({
      from: match.index,
      to: match.index + match[0].length,
      title: match[1],
      pageId: match[2],
    });
  }
  return results;
}
