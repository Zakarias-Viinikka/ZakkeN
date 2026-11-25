# Import progress (nov 25)

Read nov25_import_plan.md first for the why. This file is where we are.

## Decisions
- Pagination, not streaming. The client owns the cursor, the server is stateless, every request gets one reply.
- `GetAllYrsPages` is left alone for now.
- Import UI: the imp_or_export page gets a text field for the zip path plus an Import button. The path is part of every request.
- Import wipes `every_block_in_existence` and `pages` first, then applies pages as they arrive, all in one client-side transaction (`run_in_transaction`, which stays open across awaits). If a request fails midway, nothing is committed.
- Mid-fetch edit problem is noted in ZakkeN/potential_issue.md. Not solving it now.
- Style: no `Get` prefix in server_protocol, entry function at the top of the file, long names.

## Done (assumed applied, the compile hasn't been checked)
- server_protocol: `Request::ImportPages`, `ImportPagesPayload { zip_path, after_page_id, limit }`, `Block`, `ImportedPage`, `ImportPagesReply { pages, has_more }`, all exported from lib.rs.
- logging.rs: `request_name` has the new variant.
- exp_and_import.rs: uses the shared `Block` from server_protocol.
- server_core: `allowed_requests/import_pages.rs` opens the zip, sorts page ids, returns the next `limit` pages after the cursor. Wired into allowed_requests/mod.rs and core.rs.
- shared_internals: `ClientCommand::ImportPages(ImportPagesPayload)`.
- web_interface: `owner_of_interacting_with_server/import_requests.rs` with `fetch_import_pages(zip_path, after_page_id, limit)`. `decode_reply` in reconcile_requests.rs is now `pub(super)`. `import_requests` is registered in mod.rs.
- my_yrs_lib: `BossOfYrs::new_with_preexisting_page_id`.
- builder crate: `import_page.rs` with `build_import_page(page_id, blocks_in_order, is_main_menu_page, user_id, time) -> MergePlan`. Wraps `build_merge_new_page`, run it with `apply_merge_plan`. No love letters.

## Not done
- Mapping each server `Block` to `my_yrs_lib::Block` (content to text, block_id to id_in_yrs) before calling `build_import_page`. Part of the client import function below.
- Compile-check the client import function (`main_for_importing_pages_from_export_zip.rs`, written but not compiled). Assumed `get_time()` and `current_user_id()` return String.
- The ImpOrExport UI: text field and Import button.
- A test with a lot of pages (export, then import).

## Open questions
- Decided: import is local only and ends in lonewolf. No love letters. It wipes pages, every_block_in_existence, client_love_letter and seen_server_letters, applies pages via `build_import_page` + `apply_merge_plan`, then calls `enter_lone_wolf`. `ImportPages` is exempt from the server's drift check.
- The export doesn't carry is_main_menu_page or disabled status. Import currently marks every page as main menu.
- The wipe takes WRITE_LOCK. write_lock.rs says never call `flush_now()` while holding it, so flush first.
