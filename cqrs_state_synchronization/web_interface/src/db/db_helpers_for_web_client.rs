use client_table_blueprints::tbl_every_block_in_existence::*;
use protocol::{
    error::DbError,
    payload::{GetDataIn, GetDataOut, JoinType, SelectArgument, SelectArguments},
};
use web_internal_db::db_helper;

pub async fn get_title_and_id_of_all_menu_pages() -> Result<GetDataOut, DbError> {
    let arguments = SelectArguments::Two {
        first: SelectArgument::XEqualY {
            x: IS_TITLE.name.to_string(),
            y: "true".to_string(),
        },
        join: JoinType::And,
        second: SelectArgument::XEqualY {
            x: IS_PART_OF_MAIN_MENU_PAGE.name.to_string(),
            y: "true".to_string(),
        },
    };
    let get_data_in = GetDataIn {
        table_name: get_table_name_every_block_in_existence(),
        arguments,
        columns_to_read: vec![
            IS_TITLE.name.to_string(),
            MY_ID_AS_GIVEN_BY_YRS.name.to_string(),
        ],
    };
    db_helper::get_data(get_data_in).await
}
