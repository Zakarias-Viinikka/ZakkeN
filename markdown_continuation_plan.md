TASK: make `[[command|rest]]` in a block do something custom when clicked. One folder holds one file per command, plus one main file that picks the file by command name. Setup only: dummy actions that log, plus a real PageNav. Zakke wants to add a new command later by making one new file.

Written for an AI with zero memory. If code and this file disagree, code wins.

Context
- Real app: /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface (call it web_interface). All paths below are relative to it unless absolute.
- The markdown editor (CodeMirror 6) is already in and working. Its Rust side is src/markdown_conversion/ (cm.rs is the ONLY file that talks to JS). Its JS is markdown_js/src/*.js, built into markdown_web_output/cm_bridge.js. Details of what exists: /home/zakke/ProgStuff/ZakkeN/markdown_progress.md.
- Zakke only runs commands you give him, does not read code, does not debug. Keep replies short, plain language, no bullet lists in replies unless he asks, no code until he says go. Ask for files with ONE cat line using absolute paths (use `t` for trees). Never claim something is verified without pasted output or him saying so. If unsure, say so.
- Zakke's coding style is in /home/zakke/ProgStuff/ZakkeN/coding_style.md (long descriptive names, `main` and `core` words, lib.rs/mod.rs are plumbing only, entry function at top of file). Error rules: never unwrap(), return errors to the caller, errors carry err_msg, method, filepath; never swallow an error silently (log it at least).
- Do not edit crates outside web_interface (yrs, text_diff, zakken_server, z_db, Android). Report problems instead.
- The å/ä/ö/emoji position bug is known, outside this task, and Zakke said to ignore it.

Decisions already made by Zakke
- Link format is [[command|rest]]. The FIRST part is the command. Everything after the FIRST `|` is one raw string ("rest"), which may contain spaces. The code for each command parses `rest` itself.
- The old format was [[Title|id]] (first part = shown text, second = page id). Zakke has no existing notes using it, so the parser is simply changed. No migration.
- Shown text for now is the command word itself (PageNav, Install x). Later, Rust may decide how each command is shown.
- Folder: src/markdown_conversion/main_for_on_link_click/. One file per command inside it. `core.rs` is the entry (holds the dispatcher). mod.rs is plumbing only.
- Actions receive only (command, rest). No app state passed, unless a real need shows up (then ask Zakke).
- Dummy commands: "Install x", "Install y", "Install c", "Install all things" (calls the others). They only log a message.
- Real command: [[PageNav|pageid]] navigates to that page. Look at how the app navigates first (src/expose_methods_for_testing/method_navigate.rs and src/page_state.rs were seen only as file names). If navigating is not simple, ask Zakke before building anything extra.
- Adding a new command later = make a new file in the folder + one line in the list in core.rs + `pub mod` in mod.rs. This is accepted; auto-registration was rejected as risky in wasm.
- An unknown command must not be swallowed: log it visibly (err_msg, method, filepath).

Design

A. Parser + click event (JS side, stays thin, JS never decides meaning)
- markdown_js/src/link_parser.js: today regex is /\[\[([^\]\r\n]+)\|([^\s\]|]+)\]\]/g with fields title and pageId. Change so command = text up to the first `|`, rest = everything after it up to `]]`, no newline. Not decided: whether [[command]] with no `|` is allowed. Default: require the `|` unless Zakke says otherwise.
- markdown_js/src/decorations.js: WikiLinkWidget currently takes pageId, displayTitle, missing, disabled and looks up titlesField by pageId. Change it to take command and rest, show the command as the text, and drop the red "missing" and "disabled" look for now (it only made sense for page ids). The widget's click calls onLinkClick(command, rest).
- markdown_js/src/editor.js: emit "link_click" with {command, rest} instead of {pageId}.
- markdown_js/src/wikilink_complete.js was NOT read. It is the autocomplete when typing [[. It probably assumes the old format. Read it and decide with Zakke (likely: disable or adapt later, not part of this setup).
- Photos/other widgets: NOT part of this setup. Note for later: the widget is built in decorations.js (toDOM), and the clean way is a Rust-filled map command -> how to show it (text, image, non-clickable), reusing the unwired `titles` prop path. Nothing in this setup blocks that.

B. Rust side
- src/markdown_conversion/cm.rs (not re-read this session): the link_click event parsing and the on_link_click callback type must change from one String to (command, rest).
- src/markdown_conversion/markdown_conversion.rs: prop on_link_click: Option<Callback<String>> becomes a callback of (String, String). link_cb forwards it.
- src/bbox_drag_reorder_container/my_block_content.rs: pass the dispatcher from main_for_on_link_click as on_link_click. This is the only place outside the new folder that learns about it.
- main_for_on_link_click/core.rs: one function, entry for a click. It matches the command name against the list, calls that file's function with `rest`, and logs the returned error if any. Unknown command logs an error.
- Each command file: one function taking `rest: &str`, returning Result. Parsing `rest` is its own job.

Order of work (one build command per step, wait for output)
1. Parser, decorations, editor.js, cm.rs, converter, adapter changes so a click reaches Rust as (command, rest), with a temporary log. Build: `cd /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/markdown_js && npm run build`, then `c c`.
2. Create main_for_on_link_click/ with core.rs, mod.rs, the four dummy install files, and wire it into my_block_content.rs. `c c`.
3. PageNav (after looking at how navigation works). `c c`.
4. Zakke tests in the browser (`c r`): type [[Install x|hello world]], click it, see the log. Type [[PageNav|<a real page id>]], click it, page opens. Type [[Nope|x]], click it, see the unknown-command error. Check the link looks right on the dark theme and that typing around it still works.
5. Update /home/zakke/ProgStuff/ZakkeN/markdown_progress.md.
