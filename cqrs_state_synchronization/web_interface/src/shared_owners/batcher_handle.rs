use std::sync::Arc;
use std::sync::OnceLock;

use error_stuff::CqrsErr;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_channel::oneshot;
use leptos::task::spawn_local;
use my_yrs_lib::yrs_wrapper::PositionToInsert;
use my_yrs_lib::BossOfYrs;
use protocol::error::DbError;
use text_diff::diff_logic::DiffResult;

use crate::db::batcher::run_batcher_task;
use crate::db::batcher::{BatcherMsg, PendingWrite};

struct BatcherHandle {
    tx_send_message_to_batcher_task: UnboundedSender<BatcherMsg>,
}

static BATCHER_HANDLE: OnceLock<BatcherHandle> = OnceLock::new();

pub fn init() {
    let (tx_send_message_to_batcher_task, rx_receive_message_from_handle) = unbounded();
    spawn_local(run_batcher_task(rx_receive_message_from_handle));
    if BATCHER_HANDLE
        .set(BatcherHandle {
            tx_send_message_to_batcher_task,
        })
        .is_err()
    {
        panic!("batcher was initialized twice");
    }
}

pub fn push_edit(boss: Arc<BossOfYrs>, block_id: String, diff: DiffResult) {
    let _ = tx().unbounded_send(BatcherMsg::Push(PendingWrite::Edit {
        boss,
        block_id,
        diff,
    }));
}

pub fn push_add_block(
    boss: Arc<BossOfYrs>,
    block_id: String,
    content: String,
    position: PositionToInsert,
    is_part_of_main_menu_page: bool,
    session_id: String,
) {
    let _ = tx().unbounded_send(BatcherMsg::Push(PendingWrite::AddBlock {
        boss,
        block_id,
        content,
        position,
        is_part_of_main_menu_page,
        session_id,
    }));
}

pub async fn flush_now() -> Result<(), CqrsErr> {
    let (reply, response) = oneshot::channel();
    if tx().unbounded_send(BatcherMsg::Flush { reply }).is_err() {
        return Err(CqrsErr::DbErrorContainer(DbError::BadCode(
            "batcher run loop is gone".to_string(),
        )));
    }
    match response.await {
        Ok(result) => result,
        Err(_canceled) => Err(CqrsErr::DbErrorContainer(DbError::BadCode(
            "batcher dropped the flush reply".to_string(),
        ))),
    }
}

pub async fn run_exclusive<Fut, F, T>(work: F) -> Result<T, CqrsErr>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, CqrsErr>>,
{
    flush_now().await?;
    let guard = crate::db::write_lock::WRITE_LOCK.acquire().await;
    let result = work().await;
    drop(guard);
    result
}

fn tx() -> &'static UnboundedSender<BatcherMsg> {
    &BATCHER_HANDLE
        .get()
        .expect("batcher not initialized; call batcher_handle::init in main")
        .tx_send_message_to_batcher_task
}
