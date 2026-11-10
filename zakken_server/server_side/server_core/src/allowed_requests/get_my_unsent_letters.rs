use super::core::unsent_letters_for_user;
use db_core::db::db_actions_for_drift;
use db_wrapper::android_mascot::LiveForever;
use error_stuff::ServErr;
use protocol::serialization::Convert;
use server_protocol::GetMyUnsentLettersPayload;

/// A client is asking for every letter it hasn't received yet. Used as a
/// manual pull; the server also pushes letters on login and on new
/// letters, so this path is a fallback that still works.
pub fn send_unsent_letters(
    _msg_content: Vec<u8>,
    state: &LiveForever,
    sender: Option<&str>,
) -> Vec<u8> {
    let Some(user_name) = sender else {
        let wrapped: Result<GetMyUnsentLettersPayload, ServErr> = Err(
            ServErr::RejectedBecause("GetMyUnsentLetters without a logged-in user".to_string()),
        );
        return wrapped.to_payload();
    };

    let letters = match unsent_letters_for_user(state, user_name) {
        Ok(letters) => letters,
        Err(e) => {
            let wrapped: Result<GetMyUnsentLettersPayload, ServErr> =
                Err(ServErr::DbErrorContainer(e));
            return wrapped.to_payload();
        }
    };

    let drift_version = match db_actions_for_drift::get_current(state) {
        Ok(Some(dv)) => dv.version,
        Ok(None) => {
            let wrapped: Result<GetMyUnsentLettersPayload, ServErr> =
                Err(ServErr::BadCode("server has no drift_version row".to_string()));
            return wrapped.to_payload();
        }
        Err(e) => {
            let wrapped: Result<GetMyUnsentLettersPayload, ServErr> =
                Err(ServErr::DbErrorContainer(e));
            return wrapped.to_payload();
        }
    };

    let wrapped: Result<GetMyUnsentLettersPayload, ServErr> = Ok(GetMyUnsentLettersPayload {
        letters,
        drift_version,
    });
    wrapped.to_payload()
}