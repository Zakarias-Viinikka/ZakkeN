use super::core::unsent_letters_for_user;
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

    match unsent_letters_for_user(state, user_name) {
        Ok(letters) => {
            let wrapped: Result<GetMyUnsentLettersPayload, ServErr> =
                Ok(GetMyUnsentLettersPayload { letters });
            wrapped.to_payload()
        }
        Err(e) => {
            let wrapped: Result<GetMyUnsentLettersPayload, ServErr> =
                Err(ServErr::DbErrorContainer(e));
            wrapped.to_payload()
        }
    }
}