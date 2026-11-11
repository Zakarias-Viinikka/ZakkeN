use db_core::db::operations_key_value;
use db_wrapper::android_mascot::LiveForever;
use protocol::serialization::Convert;
use server_errors::ServErr;
use server_protocol::GetReconcileVersionPayload;

/// Return the current reconcile version. Exempt from the drift check.
/// Bumped only by ReconcileLetters on success.
pub fn get_reconcile_version(_msg_content: Vec<u8>, db_wrapper: &LiveForever) -> Vec<u8> {
    let result: Result<GetReconcileVersionPayload, ServErr> = (|| {
        let reconcile_version = operations_key_value::get_reconcile_version(db_wrapper)?;
        Ok(GetReconcileVersionPayload { reconcile_version })
    })();
    result.to_payload()
}
