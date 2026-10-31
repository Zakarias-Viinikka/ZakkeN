pub mod pages;
pub mod every_block;
pub mod love_letter;

use error_stuff::CqrsErr;
use protocol::payload::*;
use protocol::row_col::Col;
use web_internal_db::db_helper;

pub async fn set_column_where(
    table: &str,
    key_col: &str,
    key_value: String,
    column: &str,
    new_value: Col,
) -> Result<(), CqrsErr> {
    db_helper::edit_col_in_row_where(EditColInRowWhereIn {
        table_name: table.to_string(),
        where_clause: SelectArguments::Single(SelectArgument::XEqualY {
            x: key_col.to_string(),
            y: key_value,
        }),
        column: column.to_string(),
        new_value,
    })
    .await?;
    Ok(())
}
