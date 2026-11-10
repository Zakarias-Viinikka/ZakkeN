use crate::telephone::Telephone;
use db_core::db::{operations_dataset, db_actions_for_drift};
use db_wrapper::android_mascot::LiveForever;
use server_errors::ServErr;
use protocol::error::DbError;
use protocol::serialization::Convert;
use server_protocol::{LoginReply, WipeDatasetPayload};
use std::sync::{Arc, Mutex};

use super::core::err_bytes;

pub async fn wipe_dataset(
    msg_content: Vec<u8>,
    state: &Arc<Mutex<LiveForever>>,
    telephone: &Telephone,
    socket_id: u64,
) -> Vec<u8> {
    let payload = match WipeDatasetPayload::un_payloadify(&msg_content) {
        Ok(p) => p,
        Err(e) => {
            return err_bytes(ServErr::BadCode(format!(
                "couldn't read wipe payload: {e:?}"
            )));
        }
    };

    let result: Result<LoginReply, ServErr> = {
        let db_wrapper = state.lock().unwrap();
        let inner: Result<LoginReply, DbError> = (|| {
            operations_dataset::wipe_dataset_tables(&db_wrapper)?;
            let dv = db_actions_for_drift::enter_new_drift_version(
                &db_wrapper,
                "another client wiped",
                payload.drift_solution.clone(),
            )?;
            Ok(LoginReply {
                drift_version: dv.version,
                drift_reason: dv.reason,
                drift_solution: dv.drift_solution,
            })
        })();
        inner.map_err(ServErr::DbErrorContainer)
    };
    if result.is_ok() {
        telephone.notify_drift(Some(socket_id)).await;
    }
    result.to_payload()
}