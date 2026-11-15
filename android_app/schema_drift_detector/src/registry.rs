use crate::model::{Column, ForeignKey, Table};
use protocol::new_table::{ColumnDef, ForeignKeyDef};

pub struct Entry {
    pub table_name: &'static str,
    pub fn_name: &'static str,
    pub columns_fn: fn() -> Vec<ColumnDef>,
    pub fk_fn_name: Option<&'static str>,
    pub fk_fn: Option<fn() -> Vec<ForeignKeyDef>>,
}

impl Entry {
    /// Every exported fn name this entry accounts for. Used by the
    /// source-scan coverage check so it doesn't flag `get_foreign_def_*`
    /// as missing from the registry.
    pub fn all_fn_names(&self) -> Vec<&'static str> {
        let mut v = vec![self.fn_name];
        if let Some(fk) = self.fk_fn_name {
            v.push(fk);
        }
        v
    }
}

/// Order matches V3.bootstrap's actual table-creation order: FK-bearing
/// tables come after their referenced tables.
pub const REGISTRY: &[Entry] = &[
    Entry {
        table_name: "key_value_storage",
        fn_name: "new_table_key_value_storage",
        columns_fn: client_table_blueprints::tbl_key_value_storage::new_table_key_value_storage,
        fk_fn_name: None,
        fk_fn: None,
    },
    Entry {
        table_name: "pages",
        fn_name: "new_table_pages",
        columns_fn: client_table_blueprints::tbl_pages::new_table_pages,
        fk_fn_name: None,
        fk_fn: None,
    },
    Entry {
        table_name: "client_love_letter",
        fn_name: "new_table_client_love_letter",
        columns_fn: client_table_blueprints::tbl_client_love_letter::new_table_client_love_letter,
        fk_fn_name: None,
        fk_fn: None,
    },
    Entry {
        table_name: "logs",
        fn_name: "new_table_logs",
        columns_fn: client_table_blueprints::tbl_logs::new_table_logs,
        fk_fn_name: None,
        fk_fn: None,
    },
    Entry {
        table_name: "incoming_love_letters",
        fn_name: "new_table_incoming_love_letters",
        columns_fn: client_table_blueprints::tbl_incoming_love_letters::new_table_incoming_love_letters,
        fk_fn_name: None,
        fk_fn: None,
    },
    Entry {
        table_name: "backlinks",
        fn_name: "new_table_backlinks",
        columns_fn: client_table_blueprints::tbl_backlinks::new_table_backlinks,
        fk_fn_name: Some("get_foreign_def_backlinks"),
        fk_fn: Some(client_table_blueprints::tbl_backlinks::get_foreign_def_backlinks),
    },
    Entry {
        table_name: "every_block_in_existence",
        fn_name: "new_table_every_block_in_existence",
        columns_fn: client_table_blueprints::tbl_every_block_in_existence::new_table_every_block_in_existence,
        fk_fn_name: Some("get_foreign_def_every_block_in_existence"),
        fk_fn: Some(client_table_blueprints::tbl_every_block_in_existence::get_foreign_def_every_block_in_existence),
    },
    Entry {
        table_name: "seen_server_letters",
        fn_name: "new_table_seen_server_letters",
        columns_fn: client_table_blueprints::tbl_seen_server_letters::new_table_seen_server_letters,
        fk_fn_name: None,
        fk_fn: None,
    },
];

pub fn live_tables() -> Vec<Table> {
    REGISTRY
        .iter()
        .map(|e| {
            let cols = (e.columns_fn)();
            let fks = e.fk_fn.map(|f| f()).unwrap_or_default();
            Table {
                name: e.table_name.to_string(),
                columns: cols.into_iter().map(convert_col).collect(),
                foreign_keys: fks.into_iter().map(convert_fk).collect(),
            }
        })
        .collect()
}

fn convert_col(c: ColumnDef) -> Column {
    Column {
        name: c.name,
        column_type: c.column_type,
        primary_key: c.primary_key,
        not_null: c.not_null,
        unique: c.unique,
        default_value: c.default_value,
        autoincrement: c.autoincrement,
    }
}

fn convert_fk(f: ForeignKeyDef) -> ForeignKey {
    ForeignKey {
        column: f.column_name,
        ref_table: f.referenced_table_name,
        ref_column: f.referenced_column_name,
    }
}
