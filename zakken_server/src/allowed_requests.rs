use db_wrapper::android_mascot::LiveForever;
use protocol::{messages, serialization};

pub fn do_request(
    msg_request: &messages::Request,
    msg_content: Vec<u8>,
    liver: &LiveForever,
) -> Vec<u8> {
    match msg_request {
        messages::Request::GetData => liver.get_data(msg_content),
        _ => serialization::i_dont_want_to(),
    }
}
