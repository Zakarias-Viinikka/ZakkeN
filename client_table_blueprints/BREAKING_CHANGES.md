## Friday, September 25, 2026

renamed `<table>_columns()` to `new_table_<table>()` in every tbl_* file.

renamed `new_<thing>_row()` row helpers to `new_row_<table>()`.

moved every `new_table_*` and `new_row_*` into its own tbl_* file.

moved the table name consts and column name consts into their tbl_* file.
consumers of the schema now import them from there instead of hand-typing strings.

renamed `key_value_storage.rs` to `tbl_key_value_storage.rs`.
