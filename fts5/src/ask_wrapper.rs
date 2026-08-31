use crate::ask_js::ask;
use protocol::error::DbError;
use protocol::new_table::ColumnDef;
use protocol::payload::*;
use protocol::row_col::Col;
use protocol::serialization::Convert;

pub async fn list_tables() -> Result<ListTablesOut, String> {
    let bytes = ask("list_tables", None)
        .await
        .map_err(|e| format!("{e:?}"))?;
    ListTablesOut::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))
}

pub async fn create_table(table_name: &str, columns: &[ColumnDef]) -> Result<(), String> {
    let input = CreateTableIn {
        table_name: table_name.to_string(),
        columns: columns.to_vec(),
    };
    let bytes = ask("create_table", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let result: Result<(), DbError> =
        Convert::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))?;
    result.map_err(|e| format!("{e:?}"))
}

pub async fn drop_table(table_name: &str) -> Result<(), String> {
    let input = DropTableIn {
        table_name: table_name.to_string(),
    };
    let bytes = ask("drop_table", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let result: Result<(), DbError> =
        Convert::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))?;
    result.map_err(|e| format!("{e:?}"))
}

pub async fn check_table(table_name: &str) -> Result<CheckTableOut, String> {
    let input = CheckTableIn {
        table_name: table_name.to_string(),
    };
    let bytes = ask("check_table", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    CheckTableOut::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))
}

pub async fn get_data(
    table_name: &str,
    arguments: Vec<SelectArgument>,
    columns_to_read: Vec<String>,
) -> Result<GetDataOut, String> {
    let input = GetDataIn {
        table_name: table_name.to_string(),
        arguments,
        columns_to_read,
    };
    let bytes = ask("get_data", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    GetDataOut::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))
}

pub async fn get_data_ordered(
    table_name: &str,
    arguments: Vec<SelectArgument>,
    columns_to_read: Vec<String>,
    order_by: &str,
) -> Result<GetDataOut, String> {
    let input = GetDataOrderedIn {
        table_name: table_name.to_string(),
        arguments,
        columns_to_read,
        order_by: order_by.to_string(),
    };
    let bytes = ask("get_data_ordered", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    GetDataOut::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))
}

pub async fn insert_data(table_name: &str, values: Vec<ColumnValue>) -> Result<(), String> {
    let input = InsertDataIn {
        table_name: table_name.to_string(),
        values,
    };
    let bytes = ask("insert_data", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let out = InsertDataOut::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))?;
    match out.result {
        Some(e) => Err(format!("{e:?}")),
        None => Ok(()),
    }
}

pub async fn edit_col_in_row(
    table_name: &str,
    row_id: &str,
    column: &str,
    new_value: Col,
) -> Result<(), String> {
    let input = EditColInRowIn {
        table_name: table_name.to_string(),
        row_id: row_id.to_string(),
        column: column.to_string(),
        new_value,
    };
    let bytes = ask("edit_col_in_row", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let result: Result<(), DbError> =
        Convert::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))?;
    result.map_err(|e| format!("{e:?}"))
}

pub async fn delete_row(table_name: &str, row_id: &str) -> Result<(), String> {
    let input = DeleteRowIn {
        table_name: table_name.to_string(),
        row_id: row_id.to_string(),
    };
    let bytes = ask("delete_row", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let result: Result<(), DbError> =
        Convert::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))?;
    result.map_err(|e| format!("{e:?}"))
}

pub async fn swap_columns(
    table_name: &str,
    row_id_1: &str,
    row_id_2: &str,
    column: &str,
) -> Result<(), String> {
    let input = SwapColumnsIn {
        table_name: table_name.to_string(),
        row_id_1: row_id_1.to_string(),
        row_id_2: row_id_2.to_string(),
        column: column.to_string(),
    };
    let bytes = ask("swap_columns", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let result: Result<(), DbError> =
        Convert::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))?;
    result.map_err(|e| format!("{e:?}"))
}

pub async fn create_index(table_name: &str, column_name: &str) -> Result<(), String> {
    let input = CreateIndexIn {
        table_name: table_name.to_string(),
        column_name: column_name.to_string(),
    };
    let bytes = ask("create_index", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let result: Result<(), DbError> =
        Convert::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))?;
    result.map_err(|e| format!("{e:?}"))
}

pub async fn check_index(table_name: &str, column_name: &str) -> Result<CheckIndexOut, String> {
    let input = CheckIndexIn {
        table_name: table_name.to_string(),
        column_name: column_name.to_string(),
    };
    let bytes = ask("check_index", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    CheckIndexOut::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))
}

pub async fn add_column(table_name: &str, column: ColumnDef) -> Result<(), String> {
    let input = AddColumnIn {
        table_name: table_name.to_string(),
        column,
    };
    let bytes = ask("add_column", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let result: Result<(), DbError> =
        Convert::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))?;
    result.map_err(|e| format!("{e:?}"))
}

pub async fn remove_column(table_name: &str, column_name: &str) -> Result<(), String> {
    let input = RemoveColumnIn {
        table_name: table_name.to_string(),
        column_name: column_name.to_string(),
    };
    let bytes = ask("remove_column", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let result: Result<(), DbError> =
        Convert::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))?;
    result.map_err(|e| format!("{e:?}"))
}

pub async fn export_database() -> Result<ExportDatabaseOut, String> {
    let bytes = ask("export_database", Some(Vec::new()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    ExportDatabaseOut::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))
}

pub async fn export_tables(table_names: Vec<String>) -> Result<ExportTablesOut, String> {
    let input = ExportTablesIn { table_names };
    let bytes = ask("export_tables", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    ExportTablesOut::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))
}

pub async fn create_table_from_export(table_name: &str, table: TableExport) -> Result<(), String> {
    let input = CreateTableFromExportIn {
        table_name: table_name.to_string(),
        table,
    };
    let bytes = ask("create_table_from_export", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let result: Result<(), DbError> =
        Convert::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))?;
    result.map_err(|e| format!("{e:?}"))
}

pub async fn copy_table(source_table_name: &str, new_table_name: &str) -> Result<(), String> {
    let input = CopyTableIn {
        source_table_name: source_table_name.to_string(),
        new_table_name: new_table_name.to_string(),
    };
    let bytes = ask("copy_table", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let result: Result<(), DbError> =
        Convert::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))?;
    result.map_err(|e| format!("{e:?}"))
}

pub async fn create_fts5_table(
    source_table_name: &str,
    columns: Vec<String>,
) -> Result<(), String> {
    let input = CreateFts5TableIn {
        source_table_name: source_table_name.to_string(),
        columns,
    };
    let bytes = ask("create_fts5_table", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let result: Result<(), DbError> =
        Convert::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))?;
    result.map_err(|e| format!("{e:?}"))
}

pub async fn search_fts5(table_name: &str, text_to_lookup: String) -> Result<GetDataOut, String> {
    let input = SearchFts5In {
        table_name: table_name.to_string(),
        text_to_lookup,
    };
    let bytes = ask("search_fts5", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    SearchFts5Out::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))
}

pub async fn rebuild_fts5_index(table_name: &str) -> Result<(), String> {
    let input = RebuildFts5In {
        table_name: table_name.to_string(),
    };
    let bytes = ask("rebuild_fts5_index", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let result: Result<(), DbError> =
        Convert::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))?;
    result.map_err(|e| format!("{e:?}"))
}

pub async fn force_drop_table(table_name: &str) -> Result<(), String> {
    let input = DropTableIn {
        table_name: table_name.to_string(),
    };
    let bytes = ask("force_drop_table", Some(input.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let result: Result<(), DbError> =
        Convert::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))?;
    result.map_err(|e| format!("{e:?}"))
}
