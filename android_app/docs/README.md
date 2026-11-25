# Android app documentation

Start with `AI_READ_THIS.md`. Everything else is grouped by purpose below. This folder is a set of living documents, not a manual; if the code contradicts a doc, the code is true.

## Onboarding, read first

- **GETTING_STARTED.md** — 0-context entry point. Task-to-files map ("I want to work on X, these files matter"). Read this if you know what you want to do but not where it lives.
- **AI_READ_THIS.md** — primary reference. What the code is, where things live, the rules. Read this once fully; then use it as lookup.
- **AI_LAB_RULE.md** — how the experimental lab (`z.zndroid.lab`) is allowed to relate to production code.

## Reference

- **INVENTED_ON_ANDROID.md** — every place Android does something the web side does not, or does it differently, with the reason and a pointer to the equivalent. If you design something without a web reference, add an entry here.
- **MISSING_AND_STALE.md** — known gaps, known stale entries, and a trailing audit log of what has been checked against the code. Entries are snapshots; re-verify against the tree before acting on any entry.
- **SHARED_MEMORY.md** — the shared understanding of the yrs CRDT layer, including gotchas both web and Android rely on.
- **OUTSIDE_ANDROID_APP.md** — what lives outside this repo (Rust crates, web) and must not be edited from here.

## How-to

- **TESTING_ON_PC.md** — TODO. How to run JVM unit tests and the Android emulator, and what currently blocks both. Not started.

## Design

- **UX_INTENT.md** — interaction rules (Enter, Backspace, arrows, delete, selection) that the editor implements, and why.
- **LAB_WISHES.md** — specs for lab experiments that may or may not reach production.

## Historical, do not follow instructions as if current

- **IMPLEMENTATION_PLAN.md** — an old plan. Its body still tells an AI to request approvals up front and never ask the human, which contradicts how this project actually works. Kept for history only.

## Files that are not docs

- `FORME.txt` — dev Logcat filter. Not part of the doc set.

## Adding a doc

- Describes what the code is right now → Reference.
- How to do a task → How-to.
- Intent or reasoning → Design.
- Superseded → Historical.
- Cross-cutting rule the AI must follow → add it to `AI_READ_THIS.md` or `AI_LAB_RULE.md`; do not create a new top-level doc.
