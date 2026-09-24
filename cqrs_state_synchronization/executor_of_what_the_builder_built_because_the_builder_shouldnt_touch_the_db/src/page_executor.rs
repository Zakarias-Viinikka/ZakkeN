use data_builder_for_operations_that_need_to_be_correct::EverythingToInsertForNewPage;
use error_stuff::cqrs_err::CqrsErr;

/*
*
* data_builder_for_operations_that_need_to_be_correct::insert_structs
pub struct EverythingToInsertForNewPage {
    pub page_to_insert: PagesInsertCtx,
    pub blocks_to_insert: BlocksToInsertCtx,
    pub uncommitted_diffs: UncommitedDiffsInsertCtx,
}
*/

pub fn insert_page_requires_three_db_inserts(
    everything_to_insert_ctx: EverythingToInsertForNewPage,
) -> Result<(), CqrsErr> {
    insert_into_pages(everything_to_insert_ctx.page_to_insert)?;
    insert_into_every_block_in_existence(everything_to_insert_ctx.blocks_to_insert)?;
    insert_into_uncommitted_diffs(everything_to_insert_ctx.uncommitted_diffs)?;
}

fn insert_into_pages(insert_ctx: PagesInsertCtx) -> Result<(), CqrsErr> {
    unimplemented!()
}

fn insert_into_every_block_in_existence(insert_ctx: BlocksToInsertCtx) -> Result<(), CqrsErr> {
    unimplemented!()
}

fn insert_into_uncommitted_diffs(insert_ctx: UncommitedDiffsInsertCtx) -> Result<(), CqrsErr> {
    unimplemented!()
}
