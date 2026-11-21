import { Annotation, Compartment, EditorState, Prec, Transaction } from "@codemirror/state";
import { EditorView, keymap, placeholder } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import { autocompletion, completionStatus } from "@codemirror/autocomplete";
import {
  createMarkdownDecorationsExtension,
  setTitlesEffect,
  titlesField,
} from "./decorations.js";

const fromRustAnnotation = Annotation.define();
const editors = new Map();

function isHighSurrogate(code) {
  return code >= 0xd800 && code <= 0xdbff;
}

function isLowSurrogate(code) {
  return code >= 0xdc00 && code <= 0xdfff;
}

// Compute minimal common prefix/suffix in UTF-16 code units without splitting
// any surrogate pair so emoji edits remain valid UTF-16/UTF-8.
function minimalDiffRange(oldText, newText) {
  const minLen = Math.min(oldText.length, newText.length);
  let prefix = 0;
  while (prefix < minLen && oldText.charCodeAt(prefix) === newText.charCodeAt(prefix)) {
    prefix++;
  }
  if (prefix > 0 && isHighSurrogate(oldText.charCodeAt(prefix - 1))) {
    prefix--;
  }

  let suffix = 0;
  while (
    suffix < oldText.length - prefix &&
    suffix < newText.length - prefix &&
    oldText.charCodeAt(oldText.length - 1 - suffix) ===
      newText.charCodeAt(newText.length - 1 - suffix)
  ) {
    suffix++;
  }
  if (suffix > 0) {
    const oldAtBoundary = oldText.charCodeAt(oldText.length - suffix);
    const newAtBoundary = newText.charCodeAt(newText.length - suffix);
    if (isLowSurrogate(oldAtBoundary) || isLowSurrogate(newAtBoundary)) {
      suffix--;
    }
  }

  return {
    from: prefix,
    to: oldText.length - suffix,
    insert: newText.slice(prefix, newText.length - suffix),
  };
}

function wikilinkCompletionSource(context) {
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
          view.dispatch({
            changes: { from, to, insert: `[[${cleanTitle}|${id}]]` },
          });
        },
      };
    });

  return {
    from: match.from,
    options,
    validFor: /^\[\[[^\]\r\n]*$/,
  };
}

function createIntentKeymap(onIntent) {
  return Prec.high(
    keymap.of([
      {
        key: "ArrowUp",
        run(view) {
          if (completionStatus(view.state) === "active") return false;
          const sel = view.state.selection;
          if (sel.ranges.length === 1 && sel.main.empty && sel.main.head === 0) {
            onIntent(JSON.stringify({ kind: "nav", direction: "up" }));
            return true;
          }
          return false;
        },
      },
      {
        key: "ArrowDown",
        run(view) {
          if (completionStatus(view.state) === "active") return false;
          const sel = view.state.selection;
          if (
            sel.ranges.length === 1 &&
            sel.main.empty &&
            sel.main.head === view.state.doc.length
          ) {
            onIntent(JSON.stringify({ kind: "nav", direction: "down" }));
            return true;
          }
          return false;
        },
      },
      {
        key: "Enter",
        run(view) {
          if (completionStatus(view.state) === "active") return false;
          const doc = view.state.doc;
          if (doc.length === 0) {
            onIntent(JSON.stringify({ kind: "enter_passthrough" }));
            return false;
          }
          const main = view.state.selection.main;
          const before = doc.sliceString(0, main.from);
          const after = doc.sliceString(main.to, doc.length);
          onIntent(JSON.stringify({ kind: "split", before, after }));
          return true;
        },
      },
      {
        key: "Backspace",
        run(view) {
          const sel = view.state.selection;
          if (sel.ranges.length === 1 && sel.main.empty && sel.main.head === 0) {
            onIntent(JSON.stringify({ kind: "merge_up" }));
            return true;
          }
          return false;
        },
      },
      {
        key: "Delete",
        run(view) {
          if (view.state.doc.length === 0) {
            onIntent(JSON.stringify({ kind: "merge_up" }));
            return true;
          }
          return false;
        },
      },
    ])
  );
}

const plainTextareaTheme = EditorView.theme({
  "&": {
    border: "1px solid #ccc",
    borderRadius: "4px",
    backgroundColor: "#fff",
    fontFamily: "system-ui, -apple-system, sans-serif",
    fontSize: "15px",
    lineHeight: "1.5",
  },
  "&.cm-focused": {
    outline: "2px solid #4a90e2",
    outlineOffset: "-1px",
  },
  ".cm-scroller": {
    fontFamily: "inherit",
    lineHeight: "inherit",
  },
  ".cm-content": {
    padding: "6px 8px",
    caretColor: "#111",
  },
  ".cm-line": {
    padding: "0",
  },
  ".cm-md-heading": {
    fontWeight: "700",
  },
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
  ".cm-wikilink-missing": {
    color: "#a51d2d",
    backgroundColor: "#fde8e8",
  },
  ".cm-wikilink-disabled": {
    color: "#777",
    backgroundColor: "#eee",
    textDecoration: "line-through",
  },
});

window.cmBridge = {
  createEditor(id, container, initialText, titlesJson, callbacks) {
    if (editors.has(id)) {
      window.cmBridge.destroy(id);
    }

    const parsedTitles = titlesJson ? JSON.parse(titlesJson) : {};
    const editableCompartment = new Compartment();
    const placeholderCompartment = new Compartment();

    const updateListener = EditorView.updateListener.of((update) => {
      if (!update.docChanged) return;
      const isFromRust = update.transactions.some(
        (tr) => tr.annotation(fromRustAnnotation) === true
      );
      if (isFromRust) return;

      const changes = [];
      update.changes.iterChanges((fromA, toA, _fromB, _toB, inserted) => {
        changes.push({
          from: fromA,
          to: toA,
          insert: inserted.toString(),
        });
      });
      if (changes.length > 0 && callbacks && callbacks.onChanges) {
        callbacks.onChanges(JSON.stringify(changes));
      }
    });

    const state = EditorState.create({
      doc: initialText || "",
      extensions: [
        plainTextareaTheme,
        EditorView.lineWrapping,
        editableCompartment.of([
          EditorView.editable.of(true),
          EditorState.readOnly.of(false),
        ]),
        placeholderCompartment.of([]),
        history(),
        createIntentKeymap((json) => {
          if (callbacks && callbacks.onIntent) callbacks.onIntent(json);
        }),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        markdown(),
        titlesField.init(() => parsedTitles),
        createMarkdownDecorationsExtension((pageId) => {
          if (callbacks && callbacks.onLinkClick) callbacks.onLinkClick(pageId);
        }),
        autocompletion({
          override: [wikilinkCompletionSource],
        }),
        updateListener,
      ],
    });

    const view = new EditorView({
      state,
      parent: container,
    });

    let observer = null;
    if (typeof ResizeObserver !== "undefined") {
      observer = new ResizeObserver((entries) => {
        for (const entry of entries) {
          if (callbacks && callbacks.onHeight) {
            callbacks.onHeight(Math.round(entry.contentRect.height));
          }
        }
      });
      observer.observe(view.dom);
    }

    editors.set(id, { view, observer, editableCompartment, placeholderCompartment });
  },

  setTitles(id, titlesJson) {
    const rec = editors.get(id);
    if (!rec) return;
    const parsed = titlesJson ? JSON.parse(titlesJson) : {};
    rec.view.dispatch({
      effects: setTitlesEffect.of(parsed),
    });
  },

  applyExternalText(id, newText) {
    const rec = editors.get(id);
    if (!rec) return;
    const oldText = rec.view.state.doc.toString();
    if (oldText === newText) return;
    const diff = minimalDiffRange(oldText, newText);
    rec.view.dispatch({
      changes: diff,
      annotations: [
        fromRustAnnotation.of(true),
        Transaction.remote.of(true),
        Transaction.addToHistory.of(false),
      ],
    });
  },

  // Non-remote multi-change dispatch used by the fuzz harness to simulate paste/multi-cursor.
  dispatchChanges(id, changesJson) {
    const rec = editors.get(id);
    if (!rec) return;
    const changes = JSON.parse(changesJson);
    rec.view.dispatch({ changes });
  },

  focus(id, edge) {
    const rec = editors.get(id);
    if (!rec) return;
    const docLen = rec.view.state.doc.length;
    const pos = edge === "start" ? 0 : docLen;
    rec.view.dispatch({
      selection: { anchor: pos, head: pos },
    });
    rec.view.focus();
  },

  setEditable(id, editable) {
    const rec = editors.get(id);
    if (!rec) return;
    rec.view.dispatch({
      effects: rec.editableCompartment.reconfigure([
        EditorView.editable.of(!!editable),
        EditorState.readOnly.of(!editable),
      ]),
    });
  },

  setPlaceholder(id, text) {
    const rec = editors.get(id);
    if (!rec) return;
    rec.view.dispatch({
      effects: rec.placeholderCompartment.reconfigure(text ? placeholder(text) : []),
    });
  },

  getCursor(id) {
    const rec = editors.get(id);
    if (!rec) return 0;
    return rec.view.state.selection.main.head;
  },

  getText(id) {
    const rec = editors.get(id);
    if (!rec) return "";
    return rec.view.state.doc.toString();
  },

  destroy(id) {
    const rec = editors.get(id);
    if (!rec) return;
    if (rec.observer) rec.observer.disconnect();
    rec.view.destroy();
    editors.delete(id);
  },
};
