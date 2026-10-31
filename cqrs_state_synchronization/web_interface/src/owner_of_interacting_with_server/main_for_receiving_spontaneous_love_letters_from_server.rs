use std::collections::HashSet;
use std::sync::{LazyLock, Mutex};

use client_table_blueprints::{tbl_pages, tbl_seen_server_letters};
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures_util::StreamExt;
use leptos::prelude::*;
use leptos::task::spawn_local;
use love_letter::deserialize_love_letter;
use protocol::error::DbError;
use protocol::payload::{
    DeleteAllRowsIn, GetDataIn, InsertDataIn, SelectArgument, SelectArguments,
};
use protocol::schema_helper::DestructDbReturnCol;
use protocol::serialization::Convert;
use server_protocol::{
    AckPayload, GetAllYrsPagesPayload, GetMyUnsentLettersPayload, PageRow, Request, UnsentLetter,
};
use shared_internals_for_server::RouterHandle;
use web_internal_db::db_helper;

use crate::db::batcher::BatcherHandle;
use crate::db::main_send_unsent_love_letters::main_send_unsent_love_letters;
use crate::db::reads_wrapper::pages::page_exists;
use crate::leptos_components::small_components::popup::create_popup;
use crate::page_state::OpenPages;

use super::main_for_applying_love_letter_from_server_to_app::{apply_letter, ApplyError};

pub static STARTED_FRESH: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));

pub static MENU_TICK: LazyLock<ArcRwSignal<u32>> =
    LazyLock::new(|| ArcRwSignal::new(0));

static SEEN_IDS: LazyLock<Mutex<HashSet<i64>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

static START_OVER_FRESH_ATTEMPTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub fn spawn_task_for_receiving_socket_messages(
    router: RouterHandle,
    rx_take_love_letter_from_server_and_process_it: UnboundedReceiver<UnsentLetter>,
    rx_hear_that_socket_reconnected_so_we_can_relogin: UnboundedReceiver<()>,
    batcher: BatcherHandle,
    open: OpenPages,
) {
    spawn_local(async move {
        task_for_receiving_socket_messages(
            router,
            rx_take_love_letter_from_server_and_process_it,
            rx_hear_that_socket_reconnected_so_we_can_relogin,
            batcher,
            open,
        )
        .await;
    });
}

pub fn make_letters_channel() -> (
    UnboundedSender<UnsentLetter>,
    UnboundedReceiver<UnsentLetter>,
) {
    unbounded()
}

pub async fn clear_seen_server_letters() -> Result<(), DbError> {
    db_helper::delete_all_rows(DeleteAllRowsIn {
        table_name: tbl_seen_server_letters::TABLE_NAME.to_string(),
    })
    .await?;
    Ok(())
}

async fn task_for_receiving_socket_messages(
    router: RouterHandle,
    mut rx_take_love_letter_from_server_and_process_it: UnboundedReceiver<UnsentLetter>,
    mut rx_hear_that_socket_reconnected_so_we_can_relogin: UnboundedReceiver<()>,
    batcher: BatcherHandle,
    open: OpenPages,
) {
    if let Err(e) = load_seen_from_db().await {
        leptos::logging::warn!("load_seen_from_db: {e:?}");
    }

    loop {
        let next_item = rx_take_love_letter_from_server_and_process_it.next();
        let next_reconnect = rx_hear_that_socket_reconnected_so_we_can_relogin.next();
        match futures_util::future::select(next_item, next_reconnect).await {
            futures_util::future::Either::Left((item, _)) => match item {
                Some(i) => {
                    if !handle_one(&batcher, open, &router, i).await {
                        return;
                    }
                }
                None => break,
            },
            futures_util::future::Either::Right((_, _)) => {
                if let Err(e) = super::main_for_login_and_registration::login(&router).await {
                    leptos::logging::error!("login: {e}");
                    create_popup(format!("login failed: {e}"));
                    continue;
                }
                main_send_unsent_love_letters();
                if !START_OVER_FRESH_ATTEMPTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                    match start_over_fresh(&router, &batcher, open).await {
                        Ok(()) => STARTED_FRESH.set(true),
                        Err(e) => {
                            leptos::logging::error!("start_over_fresh failed: {e:?}");
                            create_popup(format!("start_over_fresh failed: {e:?}"));
                        }
                    }
                }
                match pull_unsent(&router).await {
                    Ok(letters) => {
                        for item in letters {
                            if !handle_one(&batcher, open, &router, item).await {
                                return;
                            }
                        }
                    }
                    Err(e) => leptos::logging::warn!("pull failed: {e:?}"),
                }
            }
        }
    }
    leptos::logging::warn!("receive worker exiting: socket channel closed");
}

async fn start_over_fresh(
    router: &RouterHandle,
    batcher: &BatcherHandle,
    open: OpenPages,
) -> Result<(), DbError> {
    if local_db_has_pages().await? {
        return Err(DbError::IllegalInput(
            "start_over_fresh refused: local db already has pages. Wipe locally, or use reconcile/adopt."
                .to_string(),
        ));
    }

    let bytes = router
        .send_request(Request::GetAllYrsPages, Vec::new())
        .await
        .map_err(|e| DbError::BadCode(format!("GetAllYrsPages: {e:?}")))?;
    let payload: Result<GetAllYrsPagesPayload, DbError> = Convert::un_payloadify(&bytes)
        .map_err(|e| DbError::BadCode(format!("decode pages: {e:?}")))?;
    let payload = payload?;

    for row in payload.pages {
        apply_bootstrap_row(batcher, open, row).await?;
    }
    Ok(())
}

async fn apply_bootstrap_row(
    batcher: &BatcherHandle,
    open: OpenPages,
    row: PageRow,
) -> Result<(), DbError> {
    let is_disabled = !page_status_is_active(&row.page_status, &row.page_id);

    let exists = page_exists(row.page_id.clone()).await?;

    if !exists {
        let letter = love_letter::LoveLetter::CreateNewPage {
            page_id: row.page_id.clone(),
            is_main_menu_page: row.is_main_menu_page == "true",
            snapshot_of_edit: row.blobbed_page,
        };
        apply_letter(batcher, open, letter)
            .await
            .map_err(apply_error_to_db_error)?;
        if is_disabled {
            let letter = love_letter::LoveLetter::DisablePage {
                page_id: row.page_id.clone(),
                snapshot_of_edit: row.page_status,
            };
            apply_letter(batcher, open, letter)
                .await
                .map_err(apply_error_to_db_error)?;
        }
    } else {
        let letter = love_letter::LoveLetter::EditBlock {
            text_edit: my_yrs_lib::TextEdit::Insert {
                text: String::new(),
                position: 0,
            },
            edit_target: my_yrs_lib::EditTarget::Text,
            target_page_id: row.page_id.clone(),
            block_id: String::new(),
            snapshot_of_edit: row.blobbed_page,
        };
        apply_letter(batcher, open, letter)
            .await
            .map_err(apply_error_to_db_error)?;
        if is_disabled {
            let letter = love_letter::LoveLetter::DisablePage {
                page_id: row.page_id.clone(),
                snapshot_of_edit: row.page_status,
            };
            apply_letter(batcher, open, letter)
                .await
                .map_err(apply_error_to_db_error)?;
        }
    }
    MENU_TICK.update(|n| *n = n.wrapping_add(1));
    Ok(())
}

fn page_status_is_active(status_blob: &[u8], page_id: &str) -> bool {
    if status_blob.is_empty() {
        return true;
    }
    match my_yrs_lib::YrsActivePages::new(status_blob.to_vec()) {
        Ok(active) => std::sync::Arc::new(active)
            .is_page_active(page_id.to_string())
            .unwrap_or(true),
        Err(_) => true,
    }
}

fn apply_error_to_db_error(e: ApplyError) -> DbError {
    match e {
        ApplyError::UnknownPage(p) => {
            DbError::IllegalInput(format!("start_over_fresh: unknown page {p}"))
        }
        ApplyError::Fatal(c) => DbError::BadCode(format!("start_over_fresh apply: {c:?}")),
    }
}

async fn local_db_has_pages() -> Result<bool, DbError> {
    let out = db_helper::get_data(GetDataIn {
        table_name: tbl_pages::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::All),
        columns_to_read: vec![tbl_pages::PAGE_ID.name.to_string()],
    })
    .await?;
    Ok(!out.rows.is_empty())
}

async fn pull_unsent(router: &RouterHandle) -> Result<Vec<UnsentLetter>, DbError> {
    let bytes = router
        .send_request(Request::GetMyUnsentLetters, Vec::new())
        .await
        .map_err(|e| DbError::BadCode(format!("GetMyUnsentLetters: {e:?}")))?;
    let payload: Result<GetMyUnsentLettersPayload, DbError> = Convert::un_payloadify(&bytes)
        .map_err(|e| DbError::BadCode(format!("decode unsent: {e:?}")))?;
    payload.map(|p| p.letters)
}

async fn handle_one(
    batcher: &BatcherHandle,
    open: OpenPages,
    router: &RouterHandle,
    item: UnsentLetter,
) -> bool {
    if SEEN_IDS.lock().unwrap().contains(&item.letter_id) {
        send_ack(router, item.letter_id).await;
        return true;
    }
    SEEN_IDS.lock().unwrap().insert(item.letter_id);

    let letter = match deserialize_love_letter(item.love_letter) {
        Ok(l) => l,
        Err(e) => {
            leptos::logging::error!("decode letter {}: {e}", item.letter_id);
            return true;
        }
    };

    let created_new_page = matches!(letter, love_letter::LoveLetter::CreateNewPage { .. });

    match apply_letter(batcher, open, letter).await {
        Ok(outcome) => {
            if let Err(e) = persist_seen(item.letter_id).await {
                leptos::logging::warn!("persist_seen {}: {e:?}", item.letter_id);
            }
            send_ack(router, item.letter_id).await;
            if outcome.changed {
                MENU_TICK.update(|n| *n = n.wrapping_add(1));
            }
            if created_new_page {
                if let Ok(more) = pull_unsent(router).await {
                    for extra in more {
                        if !Box::pin(handle_one(batcher, open, router, extra)).await {
                            return false;
                        }
                    }
                }
            }
            true
        }
        Err(ApplyError::UnknownPage(pid)) => {
            leptos::logging::warn!(
                "letter {} targets unknown page {pid}; leaving unacked",
                item.letter_id
            );
            SEEN_IDS.lock().unwrap().remove(&item.letter_id);
            true
        }
        Err(ApplyError::Fatal(e)) => {
            leptos::logging::error!("apply letter {}: {e:?}", item.letter_id);
            create_popup(
                "Data problem while receiving an edit. Copy anything you're afraid of losing, then reload."
                    .to_string(),
            );
            false
        }
    }
}

async fn send_ack(router: &RouterHandle, letter_id: i64) {
    let payload = AckPayload { letter_id };
    let payload_bytes = payload.to_payload();
    let router = router.clone();
    spawn_local(async move {
        let _ = router.send_request(Request::Ack, payload_bytes).await;
    });
}

async fn load_seen_from_db() -> Result<(), DbError> {
    let out = db_helper::get_data(GetDataIn {
        table_name: tbl_seen_server_letters::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::All),
        columns_to_read: vec![tbl_seen_server_letters::LETTER_ID.name.to_string()],
    })
    .await?;
    let mut seen = SEEN_IDS.lock().unwrap();
    seen.clear();
    for row in out.rows {
        if let Some(col) = row.cols.into_iter().next() {
            if let Ok(Some(id)) = tbl_seen_server_letters::LETTER_ID.try_destruct_db_col(col) {
                seen.insert(id);
            }
        }
    }
    Ok(())
}

async fn persist_seen(id: i64) -> Result<(), DbError> {
    db_helper::insert_data(InsertDataIn {
        table_name: tbl_seen_server_letters::TABLE_NAME.into(),
        values: tbl_seen_server_letters::new_seen_server_letter_row(id),
    })
    .await?;
    Ok(())
}