#[derive(Clone)]
pub struct LocalPages {
    // the id field in every_block table
    pub db_id: usize,
    // the title text
    pub title: String,
    // yrs id for the block
    pub block_id: String,
    // yrs id for the page the block belongs to
    pub page_id: String,
    // whether a page is disabled or not
    pub is_disabled: bool,
}
