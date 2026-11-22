import { EditorView } from "@codemirror/view";

// Looks like a plain textarea. Styling only, no logic.
export const plainTextareaTheme = EditorView.theme({
  "&": {
    border: "1px solid #ccc",
    borderRadius: "4px",
    backgroundColor: "#fff",
    fontFamily: "system-ui, -apple-system, sans-serif",
    fontSize: "15px",
    lineHeight: "1.5",
  },
  "&.cm-focused": { outline: "2px solid #4a90e2", outlineOffset: "-1px" },
  ".cm-scroller": { fontFamily: "inherit", lineHeight: "inherit" },
  ".cm-content": { padding: "6px 8px", caretColor: "#111" },
  ".cm-line": { padding: "0" },
  ".cm-md-heading": { fontWeight: "700" },
  ".cm-md-h1": { fontSize: "1.45em" },
  ".cm-md-h2": { fontSize: "1.3em" },
  ".cm-md-h3": { fontSize: "1.15em" },
  ".cm-md-h4": { fontSize: "1.05em" },
  ".cm-md-h5": { fontSize: "1.0em" },
  ".cm-md-h6": { fontSize: "0.95em" },
  ".cm-md-strong": { fontWeight: "700" },
  ".cm-md-em": { fontStyle: "italic" },
  ".cm-md-code": {
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
    backgroundColor: "#f2f3f5",
    padding: "0 3px",
    borderRadius: "3px",
    fontSize: "0.92em",
  },
  ".cm-wikilink": {
    color: "#1a5fb4",
    backgroundColor: "#e8f0fe",
    padding: "0 4px",
    borderRadius: "3px",
    cursor: "pointer",
    textDecoration: "underline",
  },
  ".cm-wikilink-missing": { color: "#a51d2d", backgroundColor: "#fde8e8" },
  ".cm-wikilink-disabled": {
    color: "#777",
    backgroundColor: "#eee",
    textDecoration: "line-through",
  },
});
