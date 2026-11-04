use crate::yrs_error::YrsError;
use crate::yrs_wrapper::{BossOfYrs, PositionToInsert};

use super::destruct_input::destruct_input;

#[derive(uniffi::Record)]
pub struct FrozenBlock {
    pub block_id: String,
    pub content: String,
    pub meta: String,
}

#[derive(uniffi::Record)]
pub struct FreezeDocInput {
    pub page_id: String,
    pub user_id: String,
    pub time: String,
    pub blocks: Vec<FrozenBlock>,
}

pub(crate) fn freeze_doc_internals(input: FreezeDocInput) -> Result<BossOfYrs, YrsError> {
    let (page_id, user_id, time, blocks) = destruct_input(input);

    let boss = std::sync::Arc::new(BossOfYrs {
        doc: std::sync::RwLock::new(yrs::Doc::new()),
        page_id,
        user_id,
        time,
    });
    for block in blocks {
        boss.clone().insert_new_block_with_preexisting_id(
            block.block_id,
            block.content,
            block.meta,
            PositionToInsert::AtEnd,
        )?;
    }

    std::sync::Arc::try_unwrap(boss).map_err(|_| {
        crate::generic_error!("freeze_doc: unexpected Arc ref count", "freeze_doc_internals")
    })
}