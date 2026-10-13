#[derive(Clone)]
pub struct PageHolder {
    pub db_id: usize,
    pub page_id: String,
}

#[derive(Clone)]
pub struct BlockHolder {
    pub db_id: usize,
    pub block_id: String,
    pub page_id: String,
    pub content: String,
}
