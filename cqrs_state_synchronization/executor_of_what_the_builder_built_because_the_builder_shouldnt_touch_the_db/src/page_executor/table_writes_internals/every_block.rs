use client_table_blueprints::tbl_every_block_in_existence::{
    self, new_every_block_in_existence_row,
};
use data_builder_for_operations_that_need_to_be_correct::{
    BlocksToInsertCtx, EveryBlockInExistenceInsertCtx, EveryBlockInExistenceUpdateCtx,
};
use error_stuff::CqrsErr;
use protocol::error::DbError;
use protocol::payload::{
    DeleteRowWhereIn, InsertDataAndGetColIn, InsertDataIn, SelectArgument, SelectArguments,
};
use protocol::row_col::Col;
use web_internal_db::db_helper;

use super::set_column_where;

pub async fn insert_single_every_block_row(
    insert_ctx: EveryBlockInExistenceInsertCtx,
) -> Result<(), CqrsErr> {
    insert_block_row(
        false,
        insert_ctx.my_id_as_given_by_yrs,
        insert_ctx.content,
        insert_ctx.id_of_page_i_belong_to,
        insert_ctx.frac_position,
        insert_ctx.is_part_of_main_menu_page,
    )
    .await
}

pub async fn insert_into_every_block_in_existence(
    insert_ctx: BlocksToInsertCtx,
) -> Result<i64, CqrsErr> {
    let title = insert_ctx.title_block;
    let title_db_id = insert_block_row_and_get_db_id(
        true,
        title.my_id_as_given_by_yrs,
        title.content,
        title.id_of_page_i_belong_to,
        title.frac_position,
        title.is_part_of_main_menu_page,
    )
    .await?;

    let normal = insert_ctx.normal_block;
    insert_block_row(
        false,
        normal.my_id_as_given_by_yrs,
        normal.content,
        normal.id_of_page_i_belong_to,
        normal.frac_position,
        normal.is_part_of_main_menu_page,
    )
    .await?;

    Ok(title_db_id)
}

pub async fn delete_every_block_row(block_id: String) -> Result<(), CqrsErr> {
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

pub async fn update_block_frac_position(
    block_id: String,
    new_frac_position: String,
) -> Result<(), CqrsErr> {
    set_column_where(
        tbl_every_block_in_existence::TABLE_NAME,
        tbl_every_block_in_existence::MY_ID_AS_GIVEN_BY_YRS.name,
        block_id,
        tbl_every_block_in_existence::FRAC_POSITION.name,
        Col::Text(new_frac_position),
    )
    .await
}

pub async fn update_every_block_content(
    update_ctx: EveryBlockInExistenceUpdateCtx,
) -> Result<(), CqrsErr> {
    set_column_where(
        tbl_every_block_in_existence::TABLE_NAME,
        tbl_every_block_in_existence::MY_ID_AS_GIVEN_BY_YRS.name,
        update_ctx.block_id,
        tbl_every_block_in_existence::CONTENT.name,
        Col::Text(update_ctx.new_content),
    )
    .await
}

pub async fn mark_all_blocks_for_page_disabled(page_id: String) -> Result<(), CqrsErr> {
    set_column_where(
        tbl_every_block_in_existence::TABLE_NAME,
        tbl_every_block_in_existence::ID_OF_PAGE_I_BELONG_TO.name,
        page_id,
        tbl_every_block_in_existence::PAGE_IS_DISABLED.name,
        Col::Text("true".to_string()),
    )
    .await
}

pub async fn insert_block_row(
    is_title: bool,
    my_id_as_given_by_yrs: String,
    content: String,
    id_of_page_i_belong_to: String,
    frac_position: String,
    is_part_of_main_menu_page: bool,
) -> Result<(), CqrsErr> {
    let values = new_every_block_in_existence_row(
        is_title,
        is_part_of_main_menu_page,
        content,
        my_id_as_given_by_yrs,
        id_of_page_i_belong_to,
        frac_position,
    )?;
    db_helper::insert_data(InsertDataIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.into(),
        values,
    })
    .await?;
    Ok(())
}

async fn insert_block_row_and_get_db_id(
    is_title: bool,
    my_id_as_given_by_yrs: String,
    content: String,
    id_of_page_i_belong_to: String,
    frac_position: String,
    is_part_of_main_menu_page: bool,
) -> Result<i64, CqrsErr> {
    let values = new_every_block_in_existence_row(
        is_title,
        is_part_of_main_menu_page,
        content,
        my_id_as_given_by_yrs,
        id_of_page_i_belong_to,
        frac_position,
    )?;
    let out = db_helper::insert_data_and_get_col(InsertDataAndGetColIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.into(),
        values,
        column_to_return: "id".to_string(),
    })
    .await?;
    match out.value {
        Col::Integer(i) => Ok(i),
        _ => Err(CqrsErr::DbErrorContainer(DbError::ColDestructFail(
            "title block id was not an integer".into(),
        ))),
    }
}
