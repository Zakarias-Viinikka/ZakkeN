use my_yrs_lib::yrs_wrapper::*;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_server_diff_sync() {
        let server = Arc::new(BossOfYrs::new("test_user".to_string()));
        Arc::clone(&server)
            .insert_new_block(
                "server text".to_string(),
                "".to_string(),
                PositionToInsert::AtEnd,
            )
            .unwrap();

        let block_id = Arc::clone(&server).get_entire_page().unwrap()[0]
            .id_in_yrs
            .clone();

        let client = Arc::new(BossOfYrs::new("test_user".to_string()));
        let snapshot = Arc::clone(&server).snapshot().unwrap();
        Arc::clone(&client).merge_with_snapshot(snapshot).unwrap();

        let sync_point_sv_bytes = create_bookmark_of_synced_state(Arc::clone(&client)).unwrap();

        Arc::clone(&client)
            .edit_text_block_insert(
                block_id.clone(),
                TextEdit::Insert {
                    text: " hello".to_string(),
                    position: 11,
                },
                EditTarget::Text,
            )
            .unwrap();

        let client_diff = generate_diff_snapshot(Arc::clone(&client), sync_point_sv_bytes).unwrap();

        Arc::clone(&server)
            .merge_with_snapshot(client_diff)
            .unwrap();

        let page = Arc::clone(&server).get_entire_page().unwrap();
        assert_eq!(page[0].text, "server text hello");
    }
}
