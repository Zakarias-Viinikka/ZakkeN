pub struct BlockInputEvent {
    pub block_id: String,
    pub old_text: String,
    pub new_text: String,
    pub was_enter: bool,
    pub cursor_pos: usize,
}
