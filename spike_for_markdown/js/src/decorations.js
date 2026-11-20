import { StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView, ViewPlugin, WidgetType } from "@codemirror/view";
import { syntaxTree } from "@codemirror/language";
import { parseLinks } from "./link_parser.js";

export const setTitlesEffect = StateEffect.define();

export const titlesField = StateField.define({
  create() {
    return {};
  },
  update(value, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setTitlesEffect)) {
        return effect.value || {};
      }
    }
    return value;
  },
});

const ALLOWED_TREE_NODES = new Set([
  "ATXHeading1",
  "ATXHeading2",
  "ATXHeading3",
  "ATXHeading4",
  "ATXHeading5",
  "ATXHeading6",
  "HeaderMark",
  "StrongEmphasis",
  "Emphasis",
  "EmphasisMark",
  "InlineCode",
  "CodeMark",
]);

class WikiLinkWidget extends WidgetType {
  constructor(pageId, displayTitle, missing, disabled, onLinkClick) {
    super();
    this.pageId = pageId;
    this.displayTitle = displayTitle;
    this.missing = missing;
    this.disabled = disabled;
    this.onLinkClick = onLinkClick;
  }

  eq(other) {
    return (
      other instanceof WikiLinkWidget &&
      other.pageId === this.pageId &&
      other.displayTitle === this.displayTitle &&
      other.missing === this.missing &&
      other.disabled === this.disabled
    );
  }

  toDOM() {
    const span = document.createElement("span");
    let cls = "cm-wikilink";
    if (this.missing) cls += " cm-wikilink-missing";
    if (this.disabled) cls += " cm-wikilink-disabled";
    span.className = cls;
    span.textContent = this.displayTitle;
    span.dataset.pageId = this.pageId;
    span.title = this.missing
      ? `Missing page (${this.pageId})`
      : this.disabled
        ? `Disabled page (${this.pageId})`
        : `Open ${this.pageId}`;

    // Prevent CodeMirror from moving caret/focus before click fires.
    span.addEventListener("mousedown", (e) => {
      e.preventDefault();
      e.stopPropagation();
    });
    span.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      if (this.onLinkClick) {
        this.onLinkClick(this.pageId);
      }
    });
    return span;
  }

  ignoreEvent() {
    return true;
  }
}

function selectionTouches(ranges, from, to) {
  for (const r of ranges) {
    if (r.from <= to && r.to >= from) {
      return true;
    }
  }
  return false;
}

function insideAnyRange(from, to, ranges) {
  for (const r of ranges) {
    if (from >= r.from && to <= r.to) {
      return true;
    }
  }
  return false;
}

function overlapsAnyRange(from, to, ranges) {
  for (const r of ranges) {
    if (from < r.to && to > r.from) {
      return true;
    }
  }
  return false;
}

function buildDecorations(view, onLinkClick) {
  const { state } = view;
  const doc = state.doc;
  const raw = doc.toString();
  const titlesMap = state.field(titlesField);
  const selRanges = state.selection.ranges;

  const links = parseLinks(raw);
  const decos = [];
  const atomicDecos = [];

  for (const link of links) {
    const entry = Object.prototype.hasOwnProperty.call(titlesMap, link.pageId)
      ? titlesMap[link.pageId]
      : null;
    const missing = entry === null || entry === undefined;
    const disabled = Boolean(entry && entry.disabled);
    const displayTitle = entry && entry.title ? entry.title : link.title;

    const widgetDeco = Decoration.replace({
      widget: new WikiLinkWidget(link.pageId, displayTitle, missing, disabled, onLinkClick),
    });
    decos.push(widgetDeco.range(link.from, link.to));
    atomicDecos.push(widgetDeco.range(link.from, link.to));
  }

  const seenHeadingLines = new Set();
  const tree = syntaxTree(state);

  for (const { from, to } of view.visibleRanges) {
    tree.iterate({
      from,
      to,
      enter(node) {
        const name = node.name;
        if (!ALLOWED_TREE_NODES.has(name)) {
          return;
        }
        // Skip anything completely inside a wikilink range.
        if (insideAnyRange(node.from, node.to, links)) {
          return false;
        }

        if (name.startsWith("ATXHeading")) {
          const level = name.slice("ATXHeading".length);
          const line = doc.lineAt(node.from);
          if (!seenHeadingLines.has(line.from)) {
            seenHeadingLines.add(line.from);
            decos.push(
              Decoration.line({
                class: `cm-md-heading cm-md-h${level}`,
              }).range(line.from)
            );
          }
          return;
        }

        if (name === "HeaderMark") {
          const parent = node.node.parent;
          if (!parent || !parent.name.startsWith("ATXHeading")) return;
          if (selectionTouches(selRanges, parent.from, parent.to)) return;

          let hideTo = node.to;
          if (hideTo < parent.to && doc.sliceString(hideTo, hideTo + 1) === " ") {
            hideTo += 1;
          }
          if (!overlapsAnyRange(node.from, hideTo, links)) {
            decos.push(Decoration.replace({}).range(node.from, hideTo));
          }
          return;
        }

        if (name === "StrongEmphasis") {
          decos.push(Decoration.mark({ class: "cm-md-strong" }).range(node.from, node.to));
          return;
        }

        if (name === "Emphasis") {
          decos.push(Decoration.mark({ class: "cm-md-em" }).range(node.from, node.to));
          return;
        }

        if (name === "EmphasisMark") {
          const parent = node.node.parent;
          if (!parent) return;
          if (parent.name !== "StrongEmphasis" && parent.name !== "Emphasis") return;
          if (selectionTouches(selRanges, parent.from, parent.to)) return;
          if (!overlapsAnyRange(node.from, node.to, links)) {
            decos.push(Decoration.replace({}).range(node.from, node.to));
          }
          return;
        }

        if (name === "InlineCode") {
          decos.push(Decoration.mark({ class: "cm-md-code" }).range(node.from, node.to));
          return;
        }

        if (name === "CodeMark") {
          const parent = node.node.parent;
          if (!parent || parent.name !== "InlineCode") return;
          if (selectionTouches(selRanges, parent.from, parent.to)) return;
          if (!overlapsAnyRange(node.from, node.to, links)) {
            decos.push(Decoration.replace({}).range(node.from, node.to));
          }
        }
      },
    });
  }

  return {
    decorations: Decoration.set(decos, true),
    atomicLinks: Decoration.set(atomicDecos, true),
  };
}

export function createMarkdownDecorationsExtension(onLinkClick) {
  const plugin = ViewPlugin.fromClass(
    class {
      constructor(view) {
        const built = buildDecorations(view, onLinkClick);
        this.decorations = built.decorations;
        this.atomicLinks = built.atomicLinks;
      }

      update(update) {
        const titlesChanged = update.transactions.some((tr) =>
          tr.effects.some((e) => e.is(setTitlesEffect))
        );
        if (
          update.docChanged ||
          update.selectionSet ||
          update.viewportChanged ||
          titlesChanged
        ) {
          const built = buildDecorations(update.view, onLinkClick);
          this.decorations = built.decorations;
          this.atomicLinks = built.atomicLinks;
        }
      }
    },
    {
      decorations: (v) => v.decorations,
    }
  );

  return [
    plugin,
    EditorView.atomicRanges.of((view) => {
      const p = view.plugin(plugin);
      return p ? p.atomicLinks : Decoration.none;
    }),
  ];
}
