use crdt_test::yrs_wrapper::*;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{ReadTxn, StateVector, Transact};

pub fn get_sync_point(boss: &BossOfYrs) -> Vec<u8> {
    boss.doc.transact().state_vector().encode_v1()
}
pub fn read_sync_point(bytes: &[u8]) -> StateVector {
    StateVector::decode_v1(bytes).unwrap()
}

pub fn generate_diff_snapshot(boss: &BossOfYrs, remote_sv_bytes: &[u8]) -> Vec<u8> {
    let remote_sv = read_sync_point(remote_sv_bytes);
    boss.doc.transact().encode_diff_v1(&remote_sv)
}
//
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_server_diff_sync() {
        // server starts with content
        let mut server = BossOfYrs::new();
        server.insert_new_block("server text".to_string(), "".to_string());
        let block_id = server.get_entire_page().unwrap()[0].id_in_yrs.clone();

        // client starts empty, just pulls a full snapshot to get in sync
        let mut client = BossOfYrs::new();
        let snapshot = server.snapshot();
        client.merge_with_snapshot(&snapshot).unwrap();

        // client is synced now -- save this point
        let sync_point_sv_bytes = get_sync_point(&client);

        // client makes a local edit
        client
            .edit_text_block_insert(
                &block_id,
                TextEdit::Insert(" hello".to_string(), 11),
                EditTarget::Text,
            )
            .unwrap();

        // diff since the sync point -- only the new edit, not the whole doc
        let client_diff = generate_diff_snapshot(&client, &sync_point_sv_bytes);

        // server applies just that diff
        server.merge_with_snapshot(&client_diff).unwrap();

        let page = server.get_entire_page().unwrap();
        assert_eq!(page[0].text, "server text hello");
    }
}
