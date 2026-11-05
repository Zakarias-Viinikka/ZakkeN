use std::sync::Arc;

use error_stuff::CqrsErr;
use futures_channel::oneshot;
use my_yrs_lib::yrs_wrapper::PositionToInsert;
use my_yrs_lib::BossOfYrs;
use text_diff::diff_logic::DiffResult;

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

pub(crate) enum BatcherMsg {
    Push(PendingWrite),
    Flush {
        reply: oneshot::Sender<Result<(), CqrsErr>>,
    },
}
