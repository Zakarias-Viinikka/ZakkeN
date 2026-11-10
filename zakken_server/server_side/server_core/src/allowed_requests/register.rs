use db_core::db::user_registration;
use db_wrapper::android_mascot::LiveForever;
use error_stuff::ServErr;
use protocol::error::DbError;
use protocol::serialization::{Convert, ok_serialized};
use server_protocol::RegisterPayload;

pub fn register_user(msg_content: Vec<u8>, state: &LiveForever) -> Vec<u8> {
    let payload = match RegisterPayload::un_payloadify(&msg_content) {
        Ok(p) => p,
        Err(e) => {
            let wrapped: Result<(), ServErr> =
                Err(ServErr::BadCode(format!("couldn't read register payload: {e:?}")));
            return wrapped.to_payload();
        }
    };

    match user_registration::create_user(state, &payload.user_name) {
        Ok(()) => ok_serialized(),
        Err(DbError::IllegalInput(msg)) => {
            let wrapped: Result<(), ServErr> = Err(ServErr::RejectedBecause(msg));
            wrapped.to_payload()
        }
        Err(e) => {
            let wrapped: Result<(), ServErr> = Err(ServErr::DbErrorContainer(e));
            wrapped.to_payload()
        }
    }
}