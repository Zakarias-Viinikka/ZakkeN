use crate::ask_js::ask;
use wasm_bindgen::JsValue;
use wasm_rusqlite::create_table::ColumnDef;
use wasm_rusqlite::public_data_shapes::*;
use wasm_rusqlite::table_row::Col;

pub async fn list_tables() -> Result<ListTablesOut, JsValue> {
    let bytes = ask("list_tables", None).await?;
    ListTablesOut::deserialize_wrapper(&bytes).map_err(|e| JsValue::from_str(&format!("{e:?}")))
}

pub async fn create_table(table_name: &str, columns: &[ColumnDef]) -> Result<(), JsValue> {
    let input = CreateTableIn {
        table_name: table_name.to_string(),
        columns: columns.to_vec(),
    };
    let bytes = ask("create_table", Some(input.serialize_wrapper())).await?;
    deserialize_unit(&bytes)
}

pub async fn drop_table(table_name: &str) -> Result<(), JsValue> {
    let input = DropTableIn {
        table_name: table_name.to_string(),
    };
    let bytes = ask("drop_table", Some(input.serialize_wrapper())).await?;
    deserialize_unit(&bytes)
}

pub async fn check_table(table_name: &str) -> Result<CheckTableOut, JsValue> {
    let input = CheckTableIn {
        table_name: table_name.to_string(),
    };
    let bytes = ask("check_table", Some(input.serialize_wrapper())).await?;
    CheckTableOut::deserialize_wrapper(&bytes).map_err(|e| JsValue::from_str(&format!("{e:?}")))
}

pub async fn get_data(
    table_name: &str,
    arguments: Vec<SelectArgument>,
    columns_to_read: Vec<String>,
) -> Result<GetDataOut, JsValue> {
    let input = GetDataIn {
        table_name: table_name.to_string(),
        arguments,
        columns_to_read,
    };
    let bytes = ask("get_data", Some(input.serialize_wrapper())).await?;
    GetDataOut::deserialize_wrapper(&bytes).map_err(|e| JsValue::from_str(&format!("{e:?}")))
}

pub async fn get_data_ordered(
    table_name: &str,
    arguments: Vec<SelectArgument>,
    columns_to_read: Vec<String>,
    order_by: &str,
) -> Result<GetDataOut, JsValue> {
    let input = GetDataOrderedIn {
        table_name: table_name.to_string(),
        arguments,
        columns_to_read,
        order_by: order_by.to_string(),
    };
    let bytes = ask("get_data_ordered", Some(input.serialize_wrapper())).await?;
    GetDataOut::deserialize_wrapper(&bytes).map_err(|e| JsValue::from_str(&format!("{e:?}")))
}

pub async fn insert_data(table_name: &str, values: Vec<ColumnValue>) -> Result<(), JsValue> {
    let input = InsertDataIn {
        table_name: table_name.to_string(),
        values,
    };
    let bytes = ask("insert_data", Some(input.serialize_wrapper())).await?;
    deserialize_unit(&bytes)
}

pub async fn edit_col_in_row(
    table_name: &str,
    row_id: &str,
    column: &str,
    new_value: Col,
) -> Result<(), JsValue> {
    let input = EditColInRowIn {
        table_name: table_name.to_string(),
        row_id: row_id.to_string(),
        column: column.to_string(),
        new_value,
    };
    let bytes = ask("edit_col_in_row", Some(input.serialize_wrapper())).await?;
    deserialize_unit(&bytes)
}

pub async fn delete_row(table_name: &str, row_id: &str) -> Result<(), JsValue> {
    let input = DeleteRowIn {
        table_name: table_name.to_string(),
        row_id: row_id.to_string(),
    };
    let bytes = ask("delete_row", Some(input.serialize_wrapper())).await?;
    deserialize_unit(&bytes)
}

pub async fn swap_columns(
    table_name: &str,
    row_id_1: &str,
    row_id_2: &str,
    column: &str,
) -> Result<(), JsValue> {
    let input = SwapColumnsIn {
        table_name: table_name.to_string(),
        row_id_1: row_id_1.to_string(),
        row_id_2: row_id_2.to_string(),
        column: column.to_string(),
    };
    let bytes = ask("swap_columns", Some(input.serialize_wrapper())).await?;
    deserialize_unit(&bytes)
}

pub async fn create_index(table_name: &str, column_name: &str) -> Result<(), JsValue> {
    let input = CreateIndexIn {
        table_name: table_name.to_string(),
        column_name: column_name.to_string(),
    };
    let bytes = ask("create_index", Some(input.serialize_wrapper())).await?;
    deserialize_unit(&bytes)
}

pub async fn check_index(table_name: &str, column_name: &str) -> Result<CheckIndexOut, JsValue> {
    let input = CheckIndexIn {
        table_name: table_name.to_string(),
        column_name: column_name.to_string(),
    };
    let bytes = ask("check_index", Some(input.serialize_wrapper())).await?;
    CheckIndexOut::deserialize_wrapper(&bytes).map_err(|e| JsValue::from_str(&format!("{e:?}")))
}

pub async fn add_column(table_name: &str, column: ColumnDef) -> Result<(), JsValue> {
    let input = AddColumnIn {
        table_name: table_name.to_string(),
        column,
    };
    let bytes = ask("add_column", Some(input.serialize_wrapper())).await?;
    deserialize_unit(&bytes)
}

pub async fn remove_column(table_name: &str, column_name: &str) -> Result<(), JsValue> {
    let input = RemoveColumnIn {
        table_name: table_name.to_string(),
        column_name: column_name.to_string(),
    };
    let bytes = ask("remove_column", Some(input.serialize_wrapper())).await?;
    deserialize_unit(&bytes)
}

pub async fn export_database() -> Result<ExportDatabaseOut, JsValue> {
    let bytes = ask("export_database", Some(Vec::new())).await?;
    ExportDatabaseOut::deserialize_wrapper(&bytes).map_err(|e| JsValue::from_str(&format!("{e:?}")))
}

pub async fn export_tables(table_names: Vec<String>) -> Result<ExportTablesOut, JsValue> {
    let input = ExportTablesIn { table_names };
    let bytes = ask("export_tables", Some(input.serialize_wrapper())).await?;
    ExportTablesOut::deserialize_wrapper(&bytes).map_err(|e| JsValue::from_str(&format!("{e:?}")))
}

pub async fn create_table_from_export(table_name: &str, table: TableExport) -> Result<(), JsValue> {
    let input = CreateTableFromExportIn {
        table_name: table_name.to_string(),
        table,
    };
    let bytes = ask("create_table_from_export", Some(input.serialize_wrapper())).await?;
    deserialize_unit(&bytes)
}

pub async fn copy_table(source_table_name: &str, new_table_name: &str) -> Result<(), JsValue> {
    let input = CopyTableIn {
        source_table_name: source_table_name.to_string(),
        new_table_name: new_table_name.to_string(),
    };
    let bytes = ask("copy_table", Some(input.serialize_wrapper())).await?;
    deserialize_unit(&bytes)
}

fn deserialize_unit(bytes: &[u8]) -> Result<(), JsValue> {
    let result = <Result<(), DbError>>::deserialize_wrapper(bytes)?;
    result.map_err(|e| JsValue::from_str(&format!("{e:?}")))
}
