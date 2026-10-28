#[derive(Clone, Debug)]
pub struct BlockRow {
    pub block_id: String,
    pub page_id: String,
    pub content: String,
    pub is_title: bool,
}

#[derive(Clone)]
pub struct LocalPages {
    pub db_id: usize,
    pub title: String,
    pub block_id: String,
    pub page_id: String,
    pub is_disabled: bool,
}
