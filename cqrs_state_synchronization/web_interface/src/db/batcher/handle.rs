use std::sync::Arc;

use error_stuff::CqrsErr;
use futures_channel::mpsc::UnboundedSender;
use futures_channel::oneshot;
use leptos::prelude::*;
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
    pub(crate) pending: RwSignal<Vec<PendingWrite>>,
    pub(crate) sender: UnboundedSender<BatcherMsg>,
}

impl BatcherHandle {
    pub fn push_edit(&self, boss: Arc<BossOfYrs>, block_id: String, diff: DiffResult) {
        self.pending
            .update(|list| list.push(PendingWrite::Edit { boss, block_id, diff }));
        let _ = self.sender.unbounded_send(BatcherMsg::Poke);
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
        self.pending.update(|list| {
            list.push(PendingWrite::AddBlock {
                boss,
                block_id,
                content,
                position,
                is_part_of_main_menu_page,
                session_id,
            })
        });
        let _ = self.sender.unbounded_send(BatcherMsg::Poke);
    }

    pub async fn flush_now(&self) -> Result<(), CqrsErr> {
        let (reply, response) = oneshot::channel();
        if self
            .sender
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
