#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub column_type: String,
    pub primary_key: bool,
    pub not_null: bool,
    pub unique: bool,
    pub default_value: String,
    pub autoincrement: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignKey {
    pub column: String,
    pub ref_table: String,
    pub ref_column: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub foreign_keys: Vec<ForeignKey>,
}
