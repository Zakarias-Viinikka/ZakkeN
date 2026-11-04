use std::sync::Arc;

use crate::yrs_error::YrsError;
use crate::yrs_wrapper::BossOfYrs;

use super::freeze_internals::{FreezeDocInput, FrozenBlock};

#[uniffi::export]
pub fn freeze_doc_builder(boss: Arc<BossOfYrs>) -> Result<FreezeDocInput, YrsError> {
    crate::guarded!("freeze_doc_builder", move || {
        let blocks = boss.clone().get_entire_page()?;

        let mut frozen_blocks = Vec::with_capacity(blocks.len());
        for block in blocks {
            frozen_blocks.push(FrozenBlock {
                block_id: block.id_in_yrs,
                content: block.text,
                meta: block.metadata,
            });
        }

        Ok(FreezeDocInput {
            page_id: boss.page_id.clone(),
            user_id: boss.user_id.clone(),
            time: boss.time.clone(),
            blocks: frozen_blocks,
        })
    })
}
