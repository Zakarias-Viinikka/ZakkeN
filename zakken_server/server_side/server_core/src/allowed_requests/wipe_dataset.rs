use db_core::db::operations_dataset;
use db_wrapper::android_mascot::LiveForever;
use protocol::error::DbError;
use protocol::serialization::Convert;

pub fn wipe_dataset(_msg_content: Vec<u8>, state: &LiveForever) -> Vec<u8> {
    let result: Result<(), DbError> = operations_dataset::wipe_dataset_tables(state);
    result.to_payload()
}
