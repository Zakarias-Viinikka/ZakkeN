use db_core::db::operations_drift_version;
use db_wrapper::android_mascot::LiveForever;
use error_stuff::ServErr;
use protocol::serialization::Convert;
use server_protocol::LoginReply;

/// Anyone can ask what the current drift version is, without holding a
/// matching one. It's how a client finds out it was stale, so refusing
/// the ask would be circular. Same reply shape as login, because the
/// two answers are the same answer.
pub fn get_current_drift(_msg_content: Vec<u8>, state: &LiveForever) -> Vec<u8> {
    let result: Result<LoginReply, ServErr> = (|| {
        match operations_drift_version::get_current(state)? {
            Some(dv) => Ok(LoginReply {
                drift_version: dv.version,
                drift_reason: dv.reason,
            }),
            None => Err(ServErr::BadCode(
                "server has no drift_version row".to_string(),
            )),
        }
    })();
    result.to_payload()
}
