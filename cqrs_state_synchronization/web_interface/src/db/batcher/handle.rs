use std::sync::Arc;

use error_stuff::CqrsErr;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_channel::oneshot;
use leptos::task::spawn_local;
use my_yrs_lib::yrs_wrapper::PositionToInsert;
use my_yrs_lib::BossOfYrs;
use protocol::error::DbError;
use text_diff::diff_logic::DiffResult;

use super::run_loop::BatcherMsg;

pub enum PendingWrite {
    Edit {
        boss: Arc<BossOfYrs>,
        block_id: String,
        diff: DiffResult,
    },
    AddBlock {
        boss: Arc<BossOfYrs>,
        block_id: String,
        content: String,
        position: PositionToInsert,
        is_part_of_main_menu_page: bool,
        session_id: String,
    },
}

#[derive(Clone)]
pub struct BatcherHandle {
    tx_send_message_to_batcher_task: UnboundedSender<BatcherMsg>,
}

impl BatcherHandle {
    pub fn spawn_task_and_get_handle() -> Self {
        let (tx_send_message_to_batcher_task, rx_receive_message_from_handle) = unbounded();
        spawn_local(super::run_loop::run_batcher_task(
            rx_receive_message_from_handle,
        ));
        Self {
            tx_send_message_to_batcher_task,
        }
    }

    pub fn push_edit(&self, boss: Arc<BossOfYrs>, block_id: String, diff: DiffResult) {
        let _ = self
            .tx_send_message_to_batcher_task
            .unbounded_send(BatcherMsg::Push(PendingWrite::Edit { boss, block_id, diff }));
    }

    pub fn push_add_block(
        &self,
        boss: Arc<BossOfYrs>,
        block_id: String,
        content: String,
        position: PositionToInsert,
        is_part_of_main_menu_page: bool,
        session_id: String,
    ) {
        let _ = self
            .tx_send_message_to_batcher_task
            .unbounded_send(BatcherMsg::Push(PendingWrite::AddBlock {
                boss,
                block_id,
                content,
                position,
                is_part_of_main_menu_page,
                session_id,
            }));
    }

    pub async fn flush_now(&self) -> Result<(), CqrsErr> {
        let (reply, response) = oneshot::channel();
        if self
            .tx_send_message_to_batcher_task
            .unbounded_send(BatcherMsg::Flush { reply })
            .is_err()
        {
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
}