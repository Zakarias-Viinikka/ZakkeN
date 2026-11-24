// Link parser, rendering only. JS never decides what a link means.
// Format: `[[command|rest]]`.
// command = text up to the FIRST `|` (no `]`, `|` or newline).
// rest    = everything after that first `|` up to `]]` (may contain spaces and `|`, no newline).
// The `|` is required: `[[command]]` is not a link.
const LINK_RE = /\[\[([^\]\r\n|]+)\|((?:[^\]\r\n]|\](?!\]))*)\]\]/g;

export function parseLinks(raw) {
  const results = [];
  LINK_RE.lastIndex = 0;
  let match;
  while ((match = LINK_RE.exec(raw)) !== null) {
    results.push({
      from: match.index,
      to: match.index + match[0].length,
      command: match[1],
      rest: match[2],
    });
  }
  return results;
}