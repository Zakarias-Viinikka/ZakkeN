use data_builder_for_operations_that_need_to_be_correct::{
    EveryBlockInExistenceUpdateCtx, MergePlan, PagesInsertCtx, PagesUpdateCtx, ReconcileOp,
};
use error_stuff::CqrsErr;

use super::table_writes::every_block::{
    delete_every_block_row, insert_block_row, mark_all_blocks_for_page_disabled,
    update_block_frac_position, update_every_block_content,
};
use super::table_writes::pages::{insert_into_pages, update_pages_blob, update_pages_status};
use super::transaction::run_in_transaction;

pub async fn apply_merge_plan(plan: MergePlan) -> Result<(), CqrsErr> {
    run_in_transaction(|| async move {
        write_page_row(&plan).await?;
        write_page_status(&plan).await?;
        write_disabled_marker(&plan).await?;
        for op in plan.reconcile {
            apply_reconcile_op(&plan.page_id, plan.is_main_menu_page, op).await?;
        }
        Ok(())
    })
    .await
}

async fn write_page_row(plan: &MergePlan) -> Result<(), CqrsErr> {
    if let Some(new_page) = &plan.is_new_page {
        insert_into_pages(PagesInsertCtx {
            page_id: plan.page_id.clone(),
            blobbed_page: plan.new_blob.clone().unwrap_or_default(),
            version: new_page.version.clone(),
            is_main_menu_page: new_page.is_main_menu_page,
        })
        .await?;
    } else if let Some(blob) = &plan.new_blob {
        update_pages_blob(PagesUpdateCtx {
            page_id: plan.page_id.clone(),
            yrs_blob: blob.clone(),
        })
        .await?;
    }
    Ok(())
}

async fn write_page_status(plan: &MergePlan) -> Result<(), CqrsErr> {
    if let Some(status) = &plan.new_status {
        update_pages_status(plan.page_id.clone(), status.clone()).await?;
    }
    Ok(())
}

async fn write_disabled_marker(plan: &MergePlan) -> Result<(), CqrsErr> {
    if plan.set_page_disabled {
        mark_all_blocks_for_page_disabled(plan.page_id.clone()).await?;
    }
    Ok(())
}

async fn apply_reconcile_op(
    page_id: &str,
    is_main_menu_page: bool,
    op: ReconcileOp,
) -> Result<(), CqrsErr> {
    match op {
        ReconcileOp::Insert {
            block_id,
            content,
            frac_position,
            is_title,
        } => {
            insert_block_row(
                is_title,
                block_id,
                content,
                page_id.to_string(),
                frac_position,
                is_main_menu_page,
            )
            .await?;
        }
        ReconcileOp::UpdateContent {
            block_id,
            new_content,
        } => {
            update_every_block_content(EveryBlockInExistenceUpdateCtx {
                block_id,
                new_content,
            })
            .await?;
        }
        ReconcileOp::UpdateFrac {
            block_id,
            new_frac_position,
        } => {
            update_block_frac_position(block_id, new_frac_position).await?;
        }
        ReconcileOp::Delete { block_id } => {
            delete_every_block_row(block_id).await?;
        }
    }
    Ok(())
}
