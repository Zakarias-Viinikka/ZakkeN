# Spike Progress

## Current Task
Browser harness verification via `trunk serve` (`c r`).

## Done
- Created `spike_for_markdown/` directory, `SPIKE_BRIEF.md`, `SPIKE_PROGRESS.md`.
- Read `coding_style.md` and reference crates (`web_interface`, `yrs_wrapper`).
- Created `Cargo.toml`, `rust-toolchain.toml`, `test_vectors/links.json`, `src/lib.rs`, `src/main.rs`, `src/link_parser.rs`, `src/edit_translation.rs`, `src/authority_doc.rs`.
- Created `js/package.json`, `js/src/link_parser.js`, `js/test_links.mjs`.
- Added `local_cargo.sh` (`run`, `test`, `check` including wasm32).

## Next
- Confirm in the browser: (a) markers in a heading/bold span reveal when the caret enters and hide when it leaves; (b) typing `[[` shows the autocomplete popup with Cats (and Cats & Kittens when toggled); (c) undo (Ctrl+Z) after typing works without breaking sync; (d) focus-hop: Down at end of block 1 → focus lands in block 2; Up at start of block 2 → focus in block 1.

## Files Touched
- `spike_for_markdown/Cargo.toml`
- `spike_for_markdown/rust-toolchain.toml`
- `spike_for_markdown/test_vectors/links.json`
- `spike_for_markdown/src/lib.rs`
- `spike_for_markdown/src/main.rs`
- `spike_for_markdown/src/link_parser.rs`
- `spike_for_markdown/src/edit_translation.rs`
- `spike_for_markdown/src/authority_doc.rs`
- `spike_for_markdown/js/package.json`
- `spike_for_markdown/js/src/link_parser.js`
- `spike_for_markdown/js/test_links.mjs`
- `spike_for_markdown/local_cargo.sh`
- `spike_for_markdown/FINDINGS.md`
- `spike_for_markdown/assets/.gitkeep`
- `spike_for_markdown/js/src/decorations.js`
- `spike_for_markdown/js/src/bridge.js`
- `spike_for_markdown/Trunk.toml`
- `spike_for_markdown/index.html`
- `spike_for_markdown/src/fuzz.rs`
- `spike_for_markdown/src/bridge_ffi.rs`
- `spike_for_markdown/src/harness_app.rs`
- `spike_for_markdown/README.md`
- `spike_for_markdown/SPIKE_PROGRESS.md`

## Decisions Made
- Used `OffsetKind::Utf16` in `AuthorityBlockDoc` so CodeMirror UTF-16 offsets pass directly without translation.
- Implemented `parse_links` in Rust with a small dependency-free UTF-8 scanner matching `/\[\[([^\]\r\n]+)\|([^\s\]|]+)\]\]/g`, avoiding adding a `regex` crate dependency.
- Kept `Cargo.toml` aligned with `web_interface/Cargo.toml` (`edition = "2024"`, `nightly`, Leptos `0.8.20` CSR, `yrs = "0.27.0"`).

## Open Questions
- Whether `yrs` 0.27 compiles cleanly for `wasm32-unknown-unknown` without extra `getrandom` flags.
- Browser-side visual rendering of decorations, focus-hop intents, autocomplete, and undo round-trip still need user confirmation in the browser.

## Verification Status
- `cargo test`: passed (verified via `c t`).
- `cargo check --target wasm32-unknown-unknown`: passed (verified via `c t`).
- `node js/test_links.mjs`: passed (verified via `c t`).
- esbuild bundle (`assets/cm_bridge.js`): built cleanly (verified via `c t`).
- Browser harness initial mount: verified (all 3 blocks rendered, heights measured by ResizeObserver, CM and authority in sync).
- Fuzz 500x in browser (block 3, seed 12345): PASS 500/500, all 3 blocks IN SYNC after fuzz (verified via user-pasted Copy report). UTF-16 offsets hold up against emoji, ZWJ family emoji, CJK, and wikilinks mixed into garbage fuzz text.
- Visual decorations (heading size, bold/italic/code styling, wikilink chips blue vs red): confirmed by user browsing the page — looks right.
- All 5 remote-edit buttons on block 2: confirmed working (all applied cleanly, block 2 stayed IN SYNC). Clicking them left-to-right in one pass yields the expected composition — "delete link" deletes whichever link the previous "break link" step left parseable, so the ` <MID> ` insertion from the middle step ends up inside the deleted second token.
- Rename page-cats / Toggle disable page-cats buttons: clicked, no errors, chip rendered correctly per user; final titles state not separately captured in the report.
- Hidden-marker-on-caret-away, focus-hop intents, `[[` autocomplete, undo (Ctrl+Z) round-trip: not yet explicitly confirmed.