import { EditorView } from "@codemirror/view";

// Dark theme. Font, size, line height and text color come from the surrounding
// .ps-content css (so titles stay big). Styling only, no logic.
export const plainTextareaTheme = EditorView.theme(
  {
    "&": {
      backgroundColor: "transparent",
      color: "inherit",
      fontFamily: "inherit",
      fontSize: "inherit",
      lineHeight: "inherit",
    },
    "&.cm-focused": { outline: "none" },
    ".cm-scroller": { fontFamily: "inherit", lineHeight: "inherit", overflow: "visible" },
    ".cm-content": { padding: "0", caretColor: "#e4e9f2" },
    ".cm-cursor, .cm-dropCursor": { borderLeftColor: "#e4e9f2" },
    ".cm-line": { padding: "0" },
    "&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, .cm-selectionBackground, ::selection":
      { backgroundColor: "rgba(138, 180, 255, 0.3)" },
    ".cm-placeholder": { color: "rgba(228, 233, 242, 0.35)" },
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
      backgroundColor: "rgba(255, 255, 255, 0.08)",
      padding: "0 3px",
      borderRadius: "3px",
      fontSize: "0.92em",
    },
    ".cm-wikilink": {
      color: "#8ab4ff",
      backgroundColor: "rgba(138, 180, 255, 0.15)",
      padding: "0 4px",
      borderRadius: "3px",
      cursor: "pointer",
      textDecoration: "underline",
    },
    ".cm-wikilink-missing": {
      color: "#ff9b9b",
      backgroundColor: "rgba(255, 110, 110, 0.15)",
    },
    ".cm-wikilink-disabled": {
      color: "rgba(228, 233, 242, 0.5)",
      backgroundColor: "rgba(255, 255, 255, 0.06)",
      textDecoration: "line-through",
    },
  },
  { dark: true }
);