use my_yrs_lib::yrs_wrapper::*;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{ReadTxn, StateVector, Transact};

pub fn create_bookmark_of_synced_state(boss: &BossOfYrs) -> Vec<u8> {
    boss.doc
        .read()
        .unwrap()
        .transact()
        .state_vector()
        .encode_v1()
}

pub fn deserialize_bookmark(bytes: &[u8]) -> StateVector {
    StateVector::decode_v1(bytes).unwrap()
}

pub fn generate_diff_snapshot(boss: &BossOfYrs, remote_sv_bytes: &[u8]) -> Vec<u8> {
    let remote_sv = deserialize_bookmark(remote_sv_bytes);
    boss.doc
        .read()
        .unwrap()
        .transact()
        .encode_diff_v1(&remote_sv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_server_diff_sync() {
        let server = BossOfYrs::new();
        server.insert_new_block("server text".to_string(), "".to_string());
        let block_id = server.get_entire_page().unwrap()[0].id_in_yrs.clone();

        let client = BossOfYrs::new();
        let snapshot = server.snapshot();
        client.merge_with_snapshot(snapshot).unwrap();

        let sync_point_sv_bytes = create_bookmark_of_synced_state(&client);

        client
            .edit_text_block_insert(
                block_id.clone(),
                TextEdit::Insert {
                    text: " hello".to_string(),
                    position: 11,
                },
                EditTarget::Text,
            )
            .unwrap();

        let client_diff = generate_diff_snapshot(&client, &sync_point_sv_bytes);

        server.merge_with_snapshot(client_diff).unwrap();

        let page = server.get_entire_page().unwrap();
        assert_eq!(page[0].text, "server text hello");
    }
}
