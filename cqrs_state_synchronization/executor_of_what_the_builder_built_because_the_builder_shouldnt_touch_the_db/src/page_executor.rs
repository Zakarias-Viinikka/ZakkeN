use client_table_blueprints::tbl_every_block_in_existence::{
    self, new_every_block_in_existence_row,
};
use client_table_blueprints::tbl_pages::{self, new_page_row};
use client_table_blueprints::tbl_client_love_letter::{self, new_client_love_letter_row};
use data_builder_for_operations_that_need_to_be_correct::*;
use error_stuff::cqrs_err::CqrsErr;
use protocol::error::DbError;
use protocol::payload::*;
use protocol::row_col::*;
use web_internal_db::db_helper;

#[macro_export]
macro_rules! unwrap_or_bail {
    ($method:expr) => {
        if let Err(result) = $method {
            return Err(result.into());
        }
    };
}

#[macro_export]
macro_rules! unwrap_or_bail_and_regret {
    ($method:expr, $own:expr) => {
        match $method {
            Ok(v) => v,
            Err(result) => {
                if $own {
                    web_internal_db::db_helper::regret_everything().await?;
                }
                return Err(result.into());
            }
        }
    };
}

#[macro_export]
macro_rules! start_transaction_if_not_in_one {
    () => {{
        let we_own = !db_helper::are_we_in_middle_of_transaction().await?.value;
        if we_own {
            db_helper::begin_all_or_nothing().await?;
        }
        we_own
    }};
}

#[macro_export]
macro_rules! end_transaction_if_i_own_the_transaction {
    ($we_own:expr) => {
        if $we_own {
            db_helper::everything_went_perfectly().await?;
        }
    };
}

pub async fn insert_page_requires_three_db_inserts(
    everything_to_insert_ctx: EverythingToInsertForNewPage,
) -> Result<i64, CqrsErr> {
    // returns the title block's row id in every_block_in_existence

    let do_i_own_the_transaction = start_transaction_if_not_in_one!();

    unwrap_or_bail_and_regret!(
        insert_into_pages(everything_to_insert_ctx.page_to_insert).await,
        do_i_own_the_transaction
    );
    let title_db_id = unwrap_or_bail_and_regret!(
        insert_into_every_block_in_existence(everything_to_insert_ctx.blocks_to_insert).await,
        do_i_own_the_transaction
    );
    unwrap_or_bail_and_regret!(
        insert_into_client_love_letter(everything_to_insert_ctx.client_love_letter).await,
        do_i_own_the_transaction
    );

    end_transaction_if_i_own_the_transaction!(do_i_own_the_transaction);

    Ok(title_db_id)
}

pub async fn edit_block_requires_three_db_inserts(
    edit_block_ctx: EverythingForEditBlock,
) -> Result<(), CqrsErr> {
    let do_i_own_the_transaction = start_transaction_if_not_in_one!();

    unwrap_or_bail_and_regret!(
        update_pages_blob(edit_block_ctx.pages_update).await,
        do_i_own_the_transaction
    );
    unwrap_or_bail_and_regret!(
        update_every_block_content(edit_block_ctx.every_block_in_existence_update).await,
        do_i_own_the_transaction
    );
    unwrap_or_bail_and_regret!(
        insert_into_client_love_letter(edit_block_ctx.client_love_letter).await,
        do_i_own_the_transaction
    );

    end_transaction_if_i_own_the_transaction!(do_i_own_the_transaction);

    Ok(())
}

pub async fn disabled_page_requires_three_db_actions(
    everything_to_disable: EverythingForDisablePage,
) -> Result<(), CqrsErr> {
    let page_id = everything_to_disable.pages_update.page_id.clone();
    let new_active_pages_blob = everything_to_disable.pages_update.yrs_blob;

    let do_i_own_the_transaction = start_transaction_if_not_in_one!();

    unwrap_or_bail_and_regret!(
        update_pages_status(page_id.clone(), new_active_pages_blob).await,
        do_i_own_the_transaction
    );
    unwrap_or_bail_and_regret!(
        mark_all_blocks_for_page_disabled(page_id).await,
        do_i_own_the_transaction
    );
    unwrap_or_bail_and_regret!(
        insert_into_client_love_letter(everything_to_disable.client_love_letter).await,
        do_i_own_the_transaction
    );

    end_transaction_if_i_own_the_transaction!(do_i_own_the_transaction);

    Ok(())
}

pub async fn add_block_requires_three_db_actions(
    add_block_ctx: EverythingForAddBlock,
) -> Result<(), CqrsErr> {
    let do_i_own_the_transaction = start_transaction_if_not_in_one!();

    unwrap_or_bail_and_regret!(
        update_pages_blob(add_block_ctx.pages_update).await,
        do_i_own_the_transaction
    );
    unwrap_or_bail_and_regret!(
        insert_single_every_block_row(add_block_ctx.every_block_in_existence_insert).await,
        do_i_own_the_transaction
    );
    unwrap_or_bail_and_regret!(
        insert_into_client_love_letter(add_block_ctx.client_love_letter).await,
        do_i_own_the_transaction
    );

    end_transaction_if_i_own_the_transaction!(do_i_own_the_transaction);

    Ok(())
}

pub async fn move_block_requires_three_db_actions(
    move_block_ctx: EverythingForMoveBlock,
) -> Result<(), CqrsErr> {
    let do_i_own_the_transaction = start_transaction_if_not_in_one!();

    let block_id = move_block_ctx.block_id.clone();
    let new_frac_position = move_block_ctx.new_frac_position.clone();

    unwrap_or_bail_and_regret!(
        update_pages_blob(move_block_ctx.pages_update).await,
        do_i_own_the_transaction
    );
    unwrap_or_bail_and_regret!(
        update_block_frac_position(block_id, new_frac_position).await,
        do_i_own_the_transaction
    );
    unwrap_or_bail_and_regret!(
        insert_into_client_love_letter(move_block_ctx.client_love_letter).await,
        do_i_own_the_transaction
    );

    end_transaction_if_i_own_the_transaction!(do_i_own_the_transaction);

    Ok(())
}

pub async fn delete_block_requires_three_db_actions(
    delete_block_ctx: EverythingForDeleteBlock,
) -> Result<(), CqrsErr> {
    let do_i_own_the_transaction = start_transaction_if_not_in_one!();

    unwrap_or_bail_and_regret!(
        update_pages_blob(delete_block_ctx.pages_update).await,
        do_i_own_the_transaction
    );
    unwrap_or_bail_and_regret!(
        delete_every_block_row(delete_block_ctx.block_id).await,
        do_i_own_the_transaction
    );
    unwrap_or_bail_and_regret!(
        insert_into_client_love_letter(delete_block_ctx.client_love_letter).await,
        do_i_own_the_transaction
    );

    end_transaction_if_i_own_the_transaction!(do_i_own_the_transaction);

    Ok(())
}

async fn delete_every_block_row(block_id: String) -> Result<(), CqrsErr> {
    db_helper::delete_row_where(DeleteRowWhereIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_every_block_in_existence::MY_ID_AS_GIVEN_BY_YRS
                .name
                .to_string(),
            y: block_id,
        }),
    })
    .await?;
    Ok(())
}

async fn insert_single_every_block_row(
    insert_ctx: EveryBlockInExistenceInsertCtx,
) -> Result<(), CqrsErr> {
    let values = new_every_block_in_existence_row(
        false,
        insert_ctx.is_part_of_main_menu_page,
        insert_ctx.content,
        insert_ctx.my_id_as_given_by_yrs,
        insert_ctx.id_of_page_i_belong_to,
        insert_ctx.frac_position,
    )?;

    db_helper::insert_data(InsertDataIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.into(),
        values,
    })
    .await?;

    Ok(())
}


async fn update_pages_status(
    page_id: String,
    new_active_pages_blob: Vec<u8>,
) -> Result<(), CqrsErr> {
    db_helper::edit_col_in_row_where(EditColInRowWhereIn {
        table_name: tbl_pages::TABLE_NAME.to_string(),
        where_clause: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_pages::PAGE_ID.name.to_string(),
            y: page_id,
        }),
        column: tbl_pages::PAGE_STATUS.name.to_string(),
        new_value: Col::Blob(new_active_pages_blob),
    })
    .await?;
    Ok(())
}

async fn mark_all_blocks_for_page_disabled(page_id: String) -> Result<(), CqrsErr> {
    db_helper::edit_col_in_row_where(EditColInRowWhereIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.to_string(),
        where_clause: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_every_block_in_existence::ID_OF_PAGE_I_BELONG_TO
                .name
                .to_string(),
            y: page_id,
        }),
        column: tbl_every_block_in_existence::PAGE_IS_DISABLED
            .name
            .to_string(),
        new_value: Col::Text("true".to_string()),
    })
    .await?;
    Ok(())
}

async fn update_pages_blob(update_ctx: PagesUpdateCtx) -> Result<(), CqrsErr> {
    db_helper::edit_col_in_row_where(EditColInRowWhereIn {
        table_name: tbl_pages::TABLE_NAME.to_string(),
        where_clause: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_pages::PAGE_ID.name.to_string(),
            y: update_ctx.page_id,
        }),
        column: tbl_pages::BLOBBED_PAGE.name.to_string(),
        new_value: Col::Blob(update_ctx.yrs_blob),
    })
    .await?;

    Ok(())
}

async fn update_block_frac_position(
    block_id: String,
    new_frac_position: String,
) -> Result<(), CqrsErr> {
    db_helper::edit_col_in_row_where(EditColInRowWhereIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.to_string(),
        where_clause: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_every_block_in_existence::MY_ID_AS_GIVEN_BY_YRS
                .name
                .to_string(),
            y: block_id,
        }),
        column: tbl_every_block_in_existence::FRAC_POSITION
            .name
            .to_string(),
        new_value: Col::Text(new_frac_position),
    })
    .await?;

    Ok(())
}

async fn update_every_block_content(
    update_ctx: EveryBlockInExistenceUpdateCtx,
) -> Result<(), CqrsErr> {
    db_helper::edit_col_in_row_where(EditColInRowWhereIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.to_string(),
        where_clause: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_every_block_in_existence::MY_ID_AS_GIVEN_BY_YRS
                .name
                .to_string(),
            y: update_ctx.block_id,
        }),
        column: tbl_every_block_in_existence::CONTENT.name.to_string(),
        new_value: Col::Text(update_ctx.new_content),
    })
    .await?;

    Ok(())
}

async fn insert_into_pages(insert_ctx: PagesInsertCtx) -> Result<(), CqrsErr> {
    let values = new_page_row(
        insert_ctx.page_id,
        insert_ctx.is_main_menu_page,
        insert_ctx.blobbed_page,
        insert_ctx.version,
    )?;

    db_helper::insert_data(InsertDataIn {
        table_name: tbl_pages::TABLE_NAME.to_string(),
        values,
    })
    .await?;

    Ok(())
}

async fn insert_into_every_block_in_existence(
    insert_ctx: BlocksToInsertCtx,
) -> Result<i64, CqrsErr> {
    let title_values = new_every_block_in_existence_row(
        true,
        insert_ctx.title_block.is_part_of_main_menu_page,
        insert_ctx.title_block.content,
        insert_ctx.title_block.my_id_as_given_by_yrs,
        insert_ctx.title_block.id_of_page_i_belong_to,
        insert_ctx.title_block.frac_position,
    )?;

    let title_out = db_helper::insert_data_and_get_col(InsertDataAndGetColIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.into(),
        values: title_values,
        column_to_return: "id".to_string(),
    })
    .await?;

    let title_db_id = match title_out.value {
        Col::Integer(i) => i,
        _ => {
            return Err(CqrsErr::DbErrorContainer(DbError::ColDestructFail(
                "title block id was not an integer".into(),
            )))
        }
    };

    let normal_values = new_every_block_in_existence_row(
        false,
        insert_ctx.normal_block.is_part_of_main_menu_page,
        insert_ctx.normal_block.content,
        insert_ctx.normal_block.my_id_as_given_by_yrs,
        insert_ctx.normal_block.id_of_page_i_belong_to,
        insert_ctx.normal_block.frac_position,
    )?;

    db_helper::insert_data(InsertDataIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.into(),
        values: normal_values,
    })
    .await?;

    Ok(title_db_id)
}

async fn insert_into_client_love_letter(
    insert_ctx: ClientLoveLetterInsertCtx,
) -> Result<(), CqrsErr> {
    let values = new_client_love_letter_row(
        insert_ctx.snapshot_of_edit,
        insert_ctx.love_letter_sketch,
        insert_ctx.session_id,
        insert_ctx.target_id,
    )?;

    db_helper::insert_data(InsertDataIn {
        table_name: tbl_client_love_letter::TABLE_NAME.into(),
        values,
    })
    .await?;

    Ok(())
}
