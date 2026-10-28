use client_table_blueprints::tbl_every_block_in_existence::{
    self, CONTENT, ID_OF_PAGE_I_BELONG_TO, IS_PART_OF_MAIN_MENU_PAGE, IS_TITLE,
    MY_ID_AS_GIVEN_BY_YRS, PAGE_IS_DISABLED,
};
use protocol::error::DbError;
use protocol::payload::{GetDataIn, JoinType, SelectArgument, SelectArguments};
use web_internal_db::db_helper;

use crate::complicated_signal_flow::LocalPages;
use super::destruct::destruct_entire_get_all_menu_pages;

pub async fn get_all_menu_pages() -> Result<Vec<LocalPages>, DbError> {
    let get_data_in = GetDataIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.into(),
        arguments: SelectArguments::Three {
            first: SelectArgument::XEqualY {
                x: IS_TITLE.name.to_string(),
                y: "true".to_string(),
            },
            join: JoinType::And,
            second: SelectArgument::XEqualY {
                x: IS_PART_OF_MAIN_MENU_PAGE.name.to_string(),
                y: "true".to_string(),
            },
            join2: JoinType::And,
            third: SelectArgument::XEqualY {
                x: PAGE_IS_DISABLED.name.to_string(),
                y: "false".to_string(),
            },
        },
        columns_to_read: vec![
            "id".to_string(),
            CONTENT.name.to_string(),
            MY_ID_AS_GIVEN_BY_YRS.name.to_string(),
            ID_OF_PAGE_I_BELONG_TO.name.to_string(),
        ],
    };
    let out = db_helper::get_data(get_data_in).await?;
    let mut result = Vec::new();
    for row in out.rows {
        result.push(destruct_entire_get_all_menu_pages(row)?);
    }
    Ok(result)
}