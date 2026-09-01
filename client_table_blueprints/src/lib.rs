uniffi::setup_scaffolding!();

pub mod tbl_backlinks;
pub mod tbl_every_block_in_existence;
pub mod tbl_pages;
pub mod tbl_uncommitted_diffs;

/*#[uniffi::export]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[derive(uniffi::Record)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<String>,
}*/
