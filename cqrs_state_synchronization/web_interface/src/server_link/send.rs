//! Send queued client letters to the server.
//!
//! `client_love_letter` is the queue: each row is one local edit that
//! hasn't been confirmed by the server. The send loop reads rows by id
//! ascending, rebuilds the letter, sends `SendLoveLetter`, and deletes
//! the row on Ok. Transport failures leave the row and stop the loop;
//! the next kick (from a user action or reconnect) tries again. Server
//! errors retry the same row on each kick, and after 3 consecutive
//! failures on the same letter, popup once.

use std::sync::LazyLock;

use client_table_blueprints::tbl_client_love_letter;
use futures_channel::mpsc::{unbounded, UnboundedReceiver};
use futures_util::StreamExt;
use leptos::prelude::*;
use leptos::task::spawn_local;
use love_letter::{build_a_love_letter, deserialize_sketch, love_letter_to_bytes};
use protocol::error::DbError;
use protocol::payload::*;
use protocol::schema_helper::DestructDbReturnCol;
use protocol::serialization::Convert;
use server_protocol::{Request, SendLoveLetterPayload};
use shared_internals_for_server::RouterHandle;
use web_internal_db::db_helper;

use crate::db::kick_send::install_kick_sender;
use crate::db::write_lock::WRITE_LOCK;
use crate::leptos_components::small_components::popup::create_popup;

pub static PENDING_COUNT: LazyLock<ArcRwSignal<usize>> = LazyLock::new(|| ArcRwSignal::new(0));

pub fn spawn_sender(router: RouterHandle) {
    let (tx, rx) = unbounded();
    install_kick_sender(tx);
    spawn_local(async move {
        run_sender(router, rx).await;
    });
}

async fn run_sender(router: RouterHandle, mut rx: UnboundedReceiver<()>) {
    let mut current_letter_id: Option<i64> = None;
    let mut consecutive_failures: u32 = 0;
    let mut popped_this_streak: bool = false;

    while rx.next().await.is_some() {
        loop {
            let next = match next_pending_row().await {
                Ok(n) => n,
                Err(e) => {
                    leptos::logging::error!("send: read failed: {e:?}");
                    break;
                }
            };
            let Some(row) = next else {
                update_pending_count().await;
                break;
            };

            if current_letter_id != Some(row.id) {
                current_letter_id = Some(row.id);
                consecutive_failures = 0;
                popped_this_streak = false;
            }

            match send_one(&router, &row).await {
                SendResult::Ok => {
                    delete_row(row.id).await;
                    consecutive_failures = 0;
                    popped_this_streak = false;
                    update_pending_count().await;
                }
                SendResult::Transport => break,
                SendResult::ServerErr(e) => {
                    consecutive_failures += 1;
                    if consecutive_failures >= 3 && !popped_this_streak {
                        popped_this_streak = true;
                        create_popup(format!(
                            "Server rejected a change (letter {}): {e:?}. Will keep retrying on the next edit or reconnect. If it doesn't clear, copy anything you're afraid of losing, then reload.",
                            row.id
                        ));
                    }
                    break;
                }
            }
        }
    }
}

struct PendingRow {
    id: i64,
    snapshot_of_edit: Vec<u8>,
    love_letter_sketch: Vec<u8>,
}

async fn next_pending_row() -> Result<Option<PendingRow>, DbError> {
    let out = db_helper::get_data_ordered(GetDataOrderedIn {
        table_name: tbl_client_love_letter::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::All),
        columns_to_read: vec![
            "id".to_string(),
            tbl_client_love_letter::SNAPSHOT_OF_EDIT.name.to_string(),
            tbl_client_love_letter::LOVE_LETTER_SKETCH.name.to_string(),
        ],
        order_by: "id ASC".to_string(),
    })
    .await?;

    let Some(row) = out.rows.into_iter().next() else {
        return Ok(None);
    };
    let mut cols = row.cols.into_iter();
    let id_col = cols
        .next()
        .ok_or_else(|| DbError::ColDestructFail("missing id".into()))?;
    let id = *id_col
        .as_int()
        .map_err(|e| DbError::BadCode(format!("id: {e}")))?;
    let snapshot_of_edit = tbl_client_love_letter::SNAPSHOT_OF_EDIT
        .try_destruct_db_col(
            cols.next()
                .ok_or_else(|| DbError::ColDestructFail("missing snapshot".into()))?,
        )
        .map_err(DbError::ColDestructFail)?
        .ok_or_else(|| DbError::ColDestructFail("snapshot null".into()))?;
    let love_letter_sketch = tbl_client_love_letter::LOVE_LETTER_SKETCH
        .try_destruct_db_col(
            cols.next()
                .ok_or_else(|| DbError::ColDestructFail("missing sketch".into()))?,
        )
        .map_err(DbError::ColDestructFail)?
        .ok_or_else(|| DbError::ColDestructFail("sketch null".into()))?;
    Ok(Some(PendingRow {
        id,
        snapshot_of_edit,
        love_letter_sketch,
    }))
}

enum SendResult {
    Ok,
    Transport,
    ServerErr(DbError),
}

async fn send_one(router: &RouterHandle, row: &PendingRow) -> SendResult {
    let sketch = match deserialize_sketch(row.love_letter_sketch.clone()) {
        Ok(s) => s,
        Err(e) => {
            return SendResult::ServerErr(DbError::BadCode(format!("deserialize sketch: {e}")));
        }
    };
    let letter = build_a_love_letter(sketch, row.snapshot_of_edit.clone());
    let bytes = match love_letter_to_bytes(&letter) {
        Ok(b) => b,
        Err(e) => {
            return SendResult::ServerErr(DbError::BadCode(format!("serialize letter: {e}")));
        }
    };

    let payload = SendLoveLetterPayload { love_letter: bytes };
    let payload_bytes = payload.to_payload();
    let reply_bytes = match router
        .send_request(Request::SendLoveLetter, payload_bytes)
        .await
    {
        Ok(b) => b,
        Err(_) => return SendResult::Transport,
    };
    let result: Result<(), DbError> = match Convert::un_payloadify(&reply_bytes) {
        Ok(r) => r,
        Err(e) => {
            return SendResult::ServerErr(DbError::BadCode(format!("decode reply: {e:?}")));
        }
    };
    match result {
        Ok(()) => SendResult::Ok,
        Err(e) => SendResult::ServerErr(e),
    }
}

async fn delete_row(id: i64) {
    let _guard = WRITE_LOCK.acquire().await;
    if let Err(e) = db_helper::delete_row_where(DeleteRowWhereIn {
        table_name: tbl_client_love_letter::TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: "id".to_string(),
            y: id.to_string(),
        }),
    })
    .await
    {
        leptos::logging::error!("send: delete row {id}: {e:?}");
    }
}

async fn update_pending_count() {
    let out = db_helper::get_data(GetDataIn {
        table_name: tbl_client_love_letter::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::All),
        columns_to_read: vec!["id".to_string()],
    })
    .await;
    match out {
        Ok(o) => PENDING_COUNT.set(o.rows.len()),
        Err(e) => leptos::logging::error!("send: count: {e:?}"),
    }
}