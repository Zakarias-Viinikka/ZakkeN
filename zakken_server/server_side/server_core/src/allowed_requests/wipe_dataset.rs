use db_core::db::{operations_dataset, operations_drift_version};
use db_wrapper::android_mascot::LiveForever;
use protocol::error::DbError;
use protocol::serialization::Convert;
use server_protocol::LoginReply;

pub fn wipe_dataset(_msg_content: Vec<u8>, state: &LiveForever) -> Vec<u8> {
    let result: Result<LoginReply, DbError> = (|| {
        operations_dataset::wipe_dataset_tables(state)?;
        let (version, reason) = operations_drift_version::bump(state, "another client wiped")?;
        Ok(LoginReply {
            drift_version: version,
            drift_reason: reason,
        })
    })();
    result.to_payload()
}