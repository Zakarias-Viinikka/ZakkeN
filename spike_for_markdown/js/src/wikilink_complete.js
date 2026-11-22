import { titlesField } from "./decorations.js";

// Autocomplete after "[[": options come from the titles map Rust supplied.
export function wikilinkCompletionSource(context) {
  const match = context.matchBefore(/\[\[[^\]\r\n]*/);
  if (!match) return null;

  const titlesMap = context.state.field(titlesField);
  const options = Object.entries(titlesMap)
    .filter(([, entry]) => entry && !entry.disabled)
    .map(([id, entry]) => {
      const cleanTitle = String(entry.title || "").replace(/[\[\]\r\n]/g, "");
      return {
        label: cleanTitle || id,
        detail: id,
        apply: (view, _completion, from, to) => {
          view.dispatch({ changes: { from, to, insert: `[[${cleanTitle}|${id}]]` } });
        },
      };
    });

  return { from: match.from, options, validFor: /^\[\[[^\]\r\n]*$/ };
}
