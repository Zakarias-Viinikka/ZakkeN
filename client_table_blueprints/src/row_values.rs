use protocol::payload::ColumnValue;
use protocol::row_col::Col;
use protocol::schema_helper::SchemaColumn;

pub(crate) fn text_value(column: &SchemaColumn<String>, value: String) -> ColumnValue {
    ColumnValue {
        column_name: column.name.to_string(),
        value: Col::Text(value),
    }
}

pub(crate) fn blob_value(column: &SchemaColumn<Vec<u8>>, value: Vec<u8>) -> ColumnValue {
    ColumnValue {
        column_name: column.name.to_string(),
        value: Col::Blob(value),
    }
}
