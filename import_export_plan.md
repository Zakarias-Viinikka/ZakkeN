# Import / export — project context and plan

Last updated: 2026-11-25

This document records the current understanding of the import/export work so a future session can continue without reconstructing this conversation.

## Goal

Add a way to export the user's data as a portable JSON backup and import that backup into a local client.

This is **not** intended to be a direct dump of the SQLite database. The Yrs page documents are the source of truth; SQLite rows such as `every_block_in_existence` are queryable mirrors of those documents.

The important constraint is **Yrs lineage**. Rebuilding a page from block text creates a new Yrs document lineage, even when its visible content is identical. Updates made against the old lineage must never be merged into the rebuilt document. After an import, the client must remain gated from normal sync until it reconciles with the server or adopts one dataset as the source of truth.

## Current state (evidence available on 2026-11-25)

### Confirmed implemented

- The Yrs crate already has the hard-copy primitives:
  - `freeze_doc` builds a fresh `BossOfYrs` from a list of frozen blocks.
  - `freeze_doc_builder` reads a live `BossOfYrs` into a frozen-block list.
  - `FrozenBlock` contains `block_id`, `content`, and `meta`. It has no fractional-position field; list order carries the block order.
- The web app has an Import/Export route and a dev-sidebar button that navigates to it.
- The current page component is only a placeholder. Its entire visible content is `imp_or_export`.
- A server-side export request/protocol handler scaffold was added around 2026-11-15. The project notes explicitly say the client side of that exchange was not built. This is not evidence of a working export feature.
- The raw server wipe request exists and was tested separately. That is a server operation, not the complete client-side data-management flow.
- Reconcile was subsequently implemented end-to-end (recorded in `todo.md` as completed around 2026-11-11). Import depends on having a safe way to resolve the new local lineage against the server.

### Not confirmed implemented

There is no evidence in the files or history reviewed in this conversation that the following exist yet:

- A versioned backup-file data model and JSON serializer.
- A builder that extracts every page's blocks from its Yrs blob for export.
- A working browser download of a backup file.
- An import parser and validation pipeline.
- Rebuilding imported pages with `freeze_doc` and applying them to the local database.
- Writing imported backlinks or replacing local data as one transaction.
- A working import UI or export UI.

A separate import/export spike has **not** been located so far. The Git search for import/export commit messages and the repository tree did not reveal one, but that does not prove that no spike exists under an unrelated name.

## Relevant project documents and authority

1. `combining_connecting_template_with_cqrs_and_server.md` — written 2026-11-04. This is the newer implementation plan and explicitly overrides the older plan where they disagree. It is the main design reference for export/import.
2. `todo.md` — contains later status notes. It says the Yrs hard-copy primitive exists, raw server wipe exists, and reconcile is implemented. It also describes export/import as not yet built.
3. `cqrs_state_synchronization/CURRENT_STATE.md` — records the Import/Export route as a placeholder and says the client-side exchange had not been built. This note is older (2026-11-14), so use it as historical confirmation rather than a guarantee of today's code.
4. `plan_for_general_data_management_ops.md` was deleted. Its contents were recovered from Git using `b64b9d9^:plan_for_general_data_management_ops.md`. It is an older design reference only; the 2026-11-04 plan supersedes it.

The old plan can be recovered again with:

```bash
git -C /home/zakke/ProgStuff/ZakkeN show b64b9d9^:plan_for_general_data_management_ops.md
```

## Intended backup format

The 2026-11-04 plan specifies a versioned JSON file with this conceptual shape:

```json
{
  "format_version": 1,
  "pages": [
    {
      "page_id": "...",
      "is_main_menu_page": true,
      "blocks": [
        {
          "block_id": "...",
          "content": "...",
          "meta": "..."
        }
      ]
    }
  ],
  "backlinks": [
    {
      "page_that_holds_link_id": "...",
      "page_being_linked_to_id": "..."
    }
  ]
}
```

Important format rules:

- `format_version` is currently `1`.
- Pages carry `page_id`, `is_main_menu_page`, and their ordered blocks.
- Blocks carry `block_id`, `content`, and `meta`.
- Block order in the file is authoritative. Do not store fractional positions; rebuilt documents calculate their own fractions.
- Do not store `is_title` or a per-block main-menu flag. Those are derived using the existing builder rules when the database mirror rows are written.
- Disabled pages are omitted from an export entirely.
- Backlinks are included in the local backup. The older plan notes that backlinks are not currently sent to or stored on the server.

The actual Rust type and function signatures must be checked before implementation. In particular, read the real `freeze_doc`, `freeze_doc_builder`, `FreezeDocInput`, and `FrozenBlock` definitions rather than relying only on this description.

## Export: intended flow

Export should work without contacting the server and should be allowed in any sync mode, including while the client is gated. The 2026-11-04 plan also says it may run from a `TwoTabs` tab.

1. Flush the batcher so pending edits are written.
2. Under `run_exclusive`, read the page rows and their Yrs blobs, plus the backlinks. The plan calls for one query for pages and one for backlinks.
3. Release the exclusive section before doing the potentially heavier conversion/serialization work.
4. Use `freeze_doc_builder` on each active page blob to obtain its ordered blocks.
5. Omit disabled pages.
6. Serialize the versioned backup as pretty JSON.
7. Trigger a browser download with a name like `zakken-backup-<date>.json`.

Keep the browser-download operation in a small separate function so Android can later use a file-writing implementation instead.

## Import: intended flow

Import replaces the local dataset with the contents of a backup. It does not write to the server. Because `freeze_doc` creates new Yrs lineages, the client must enter `NeedsReconcile` and stay gated until a later reconcile or adopt succeeds.

1. Read the selected file as text.
2. Run `prepare_import(text)` entirely in memory. It should parse, validate, rebuild the documents with `freeze_doc`, and prepare the merge plans. It must not write to the database.
3. Validation requirements from the plan:
   - The JSON must parse.
   - `format_version` must equal `1`; unsupported versions should produce a clear error.
   - Page IDs must be unique.
   - Block IDs must be unique within each page.
   - Every page must contain at least one block; its first block is the title.
   - Backlinks that point to unknown pages are dropped and counted rather than making the whole import fail.
   - Any page-build or merge-plan failure aborts before any database write.
4. Acquire the two-tab lock and set sync mode to `NeedsReconcile` with an import-related reason before destructive work.
5. Flush the batcher. Then perform one `run_exclusive` closure containing one database transaction:
   - Delete the existing local data in child-before-parent order: block mirror rows, backlinks, `client_love_letter`, `seen_server_letters`, then pages.
   - Apply the prepared page merge plans.
   - Write the valid imported backlinks.
   - Clear the in-memory seen-letter set as well as the database table.
6. Preserve the local user/session identity values in `key_value_storage`. The 2026-11-04 plan also says not to change the stored server dataset-generation value during import.
7. Call `restart_sync_stack()` after the local replacement. The imported client remains gated in `NeedsReconcile`; it must not start ordinary sync against the old server lineage.
8. The user can then reconcile with the server or adopt the imported dataset as the server's new dataset.

The database replacement must be all-or-nothing. A rejected or invalid file must leave the existing local data intact.

## Lineage rule — do not weaken this

Never merge a Yrs update or snapshot from one lineage into a document from another lineage.

For import, the rebuilt pages have new lineages. They must stay gated. Reconciliation must compare/replace blobs or build edits on top of the server's actual blob; it must not simply merge the imported document into the existing server document.

This is the reason import is not just “read JSON and insert text into the current pages.”

## Related data-management work

These items matter to import/export but are not themselves the export/import feature:

- **Reconcile:** `todo.md` records this as implemented end-to-end around 2026-11-11. Re-read the current implementation before changing it; the original October plan's `ReconcilePlan`/`diff_page` design was later superseded. The 2026-11-04 plan describes reconcile as a running process rather than a stored plan object.
- **Adopt:** lets the server take the client's page blobs as-is, preserving their lineage. The current `todo.md` still lists adopt as not built.
- **Wipe server:** the raw server request exists, but the complete client verb and generation/seen-letter handling are separate work. Do not mistake the raw wipe request for the full safe user flow.
- **Dataset generation:** the 2026-11-04 plan's step 8 is a design discussion that must be settled with the human before implementing that step. `CURRENT_STATE.md` (2026-11-14) says the `NoSuchUser` clear-seen behaviour was still a stopgap “until the dataset-generation id lands.” The project later added drift-version and spontaneous server-message/lonewolf behaviour, but that is not by itself proof that the specific dataset-generation-in-login design is complete. Inspect current server and client code before deciding what remains.
- **Pause/resume:** present in the deleted October plan, but explicitly removed by the newer 2026-11-04 plan. Do not implement it as part of this work.
- **Automatic backups:** mentioned as a broader future intention in project notes, but no concrete automatic-backup design is established here.

## Suggested continuation order

1. Re-check the current code and exact signatures for `freeze_doc` / `freeze_doc_builder` / `FrozenBlock`, and locate the target-agnostic builder crate named in the 2026-11-04 plan (`data_builder_for_operations_that_need_to_be_correct`). Confirm whether any backup-related work has appeared since the notes above.
2. Implement and test the pure backup types and builders: read blocks from blobs, build the backup file, and prepare/validate an import entirely in memory. Follow the `FreezeDocInput` / `FrozenBlock` API; do not add fractional positions to the file.
3. Implement export: database read helpers, active-page filtering, pretty JSON, browser download.
4. Implement import: file reading, validation, safe local transactional replacement, seen-set clearing, and the `NeedsReconcile` transition.
5. Verify the generation/reconcile/adopt path before claiming imported data can safely resume syncing. Do not assume the old plan's server-generation step is already complete.
6. Add/verify end-to-end tests for export, import into a fresh profile, and import followed by reconcile/adopt.

Work one step at a time. Before editing an unfamiliar file, read it. At the end of each step, report what changed and how to verify it, then wait for the human before moving on.

## Current UI files inspected

The current component is only a placeholder:

- `cqrs_state_synchronization/web_interface/src/leptos_components/imp_or_export/imp_or_export.rs`
- `cqrs_state_synchronization/web_interface/src/leptos_components/imp_or_export/mod.rs`

The component currently renders only the text `imp_or_export`. The route and navigation button exist, but no import/export controls or behaviour were present in the version inspected on 2026-11-25.
