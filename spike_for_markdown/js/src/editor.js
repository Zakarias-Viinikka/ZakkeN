// Thin wrapper around CodeMirror 6. NO app logic here.
// Rust -> JS: window.cmBridge.<fn>(...)   (plain function calls)
// JS -> Rust: window CustomEvent "cm", detail = JSON string {id, kind, ...}.
//             Cancelable: if a Rust listener calls prevent_default(), emit() returns false.
// Event kinds: changes {changes, head}, key {...}, link_click {pageId}.
import { Annotation, Compartment, EditorState, Prec, Transaction } from "@codemirror/state";
import { EditorView, keymap, placeholder } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import { autocompletion } from "@codemirror/autocomplete";
import { plainTextareaTheme } from "./theme.js";
import {
  createMarkdownDecorationsExtension,
  setTitlesEffect,
  titlesField,
} from "./decorations.js";
import { wikilinkCompletionSource } from "./wikilink_complete.js";

const editors = new Map();
const fromRust = Annotation.define();

function emit(id, kind, payload) {
  const detail = JSON.stringify({ id, kind, ...payload });
  return window.dispatchEvent(new CustomEvent("cm", { detail, cancelable: true }));
}

const isHigh = (c) => c >= 0xd800 && c <= 0xdbff;
const isLow = (c) => c >= 0xdc00 && c <= 0xdfff;

// Minimal common prefix/suffix in UTF-16 units, never splitting a surrogate pair.
function minimalDiff(a, b) {
  const minLen = Math.min(a.length, b.length);
  let prefix = 0;
  while (prefix < minLen && a.charCodeAt(prefix) === b.charCodeAt(prefix)) prefix++;
  if (prefix > 0 && isHigh(a.charCodeAt(prefix - 1))) prefix--;

  let suffix = 0;
  while (
    suffix < a.length - prefix &&
    suffix < b.length - prefix &&
    a.charCodeAt(a.length - 1 - suffix) === b.charCodeAt(b.length - 1 - suffix)
  ) suffix++;
  if (suffix > 0) {
    if (isLow(a.charCodeAt(a.length - suffix)) || isLow(b.charCodeAt(b.length - suffix))) suffix--;
  }
  return { from: prefix, to: a.length - suffix, insert: b.slice(prefix, b.length - suffix) };
}

// Keys Rust wants to see. JS decides NOTHING: it reports and asks.
// (The autocomplete popup's own keymap runs before this one, so an open popup keeps its keys.)
const REPORTED_KEYS = ["ArrowUp", "ArrowDown", "Enter", "Backspace", "Delete"];

function keyReporter(id) {
  return Prec.high(
    keymap.of(
      REPORTED_KEYS.map((key) => ({
        key,
        run(view) {
          const sel = view.state.selection;
          const proceed = emit(id, "key", {
            key,
            anchor: sel.main.anchor,
            head: sel.main.head,
            docLen: view.state.doc.length,
            ranges: sel.ranges.length,
          });
          return !proceed; // Rust called preventDefault() => consumed
        },
      }))
    )
  );
}

window.cmBridge = {
  createEditor(id, container, initialText, titlesJson) {
    if (editors.has(id)) window.cmBridge.destroy(id);

    const titles = titlesJson ? JSON.parse(titlesJson) : {};
    const editableCompartment = new Compartment();
    const placeholderCompartment = new Compartment();

    const updateListener = EditorView.updateListener.of((update) => {
      if (!update.docChanged) return;
      // Changes that Rust itself pushed in are not user edits: no echo.
      if (update.transactions.some((tr) => tr.annotation(fromRust) === true)) return;
      // from/to are UTF-16 offsets in the OLD document.
      const changes = [];
      update.changes.iterChanges((fromA, toA, _fromB, _toB, inserted) => {
        changes.push({ from: fromA, to: toA, insert: inserted.toString() });
      });
      emit(id, "changes", { changes, head: update.state.selection.main.head });
    });

    const view = new EditorView({
      state: EditorState.create({
        doc: initialText || "",
        extensions: [
          plainTextareaTheme,
          EditorView.lineWrapping,
          editableCompartment.of([
            EditorView.editable.of(true),
            EditorState.readOnly.of(false),
          ]),
          placeholderCompartment.of([]),
          keyReporter(id),
          history(),
          keymap.of([...defaultKeymap, ...historyKeymap]),
          markdown(),
          titlesField.init(() => titles),
          createMarkdownDecorationsExtension((pageId) => {
            emit(id, "link_click", { pageId });
          }),
          autocompletion({ override: [wikilinkCompletionSource] }),
          updateListener,
        ],
      }),
      parent: container,
    });
    editors.set(id, { view, editableCompartment, placeholderCompartment });
  },

  // Replace the document with newText using ONE minimal change, so CM keeps the caret.
  setText(id, newText) {
    const rec = editors.get(id);
    if (!rec) return;
    const oldText = rec.view.state.doc.toString();
    if (oldText === newText) return;
    rec.view.dispatch({
      changes: minimalDiff(oldText, newText),
      annotations: [
        fromRust.of(true),
        Transaction.remote.of(true),
        Transaction.addToHistory.of(false),
      ],
    });
  },

  getText(id) {
    const rec = editors.get(id);
    return rec ? rec.view.state.doc.toString() : "";
  },

  getSelection(id) {
    const rec = editors.get(id);
    if (!rec) return [0, 0];
    const m = rec.view.state.selection.main;
    return [m.anchor, m.head];
  },

  setSelection(id, anchor, head) {
    const rec = editors.get(id);
    if (!rec) return;
    const len = rec.view.state.doc.length;
    const clamp = (n) => Math.max(0, Math.min(n, len));
    rec.view.dispatch({
      selection: { anchor: clamp(anchor), head: clamp(head) },
      scrollIntoView: true,
    });
  },

  focus(id, edge) {
    const rec = editors.get(id);
    if (!rec) return;
    const pos = edge === "start" ? 0 : rec.view.state.doc.length;
    rec.view.dispatch({ selection: { anchor: pos, head: pos } });
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

  setTitles(id, titlesJson) {
    const rec = editors.get(id);
    if (!rec) return;
    rec.view.dispatch({
      effects: setTitlesEffect.of(titlesJson ? JSON.parse(titlesJson) : {}),
    });
  },

  destroy(id) {
    const rec = editors.get(id);
    if (!rec) return;
    rec.view.destroy();
    editors.delete(id);
  },
};
