use crate::message::{Message, Response};
use error_stuff::{ErrorDetails, ServErr};

pub fn json_str_to_message(json: &str) -> Result<Message, ServErr> {
    serde_json::from_str(json).map_err(|e| protocol_err("json_str_to_message", e.to_string()))
}

pub fn json_str_to_response(json: &str) -> Result<Response, ServErr> {
    serde_json::from_str(json).map_err(|e| protocol_err("json_str_to_response", e.to_string()))
}

pub fn message_to_json_str(msg: &Message) -> Result<String, ServErr> {
    serde_json::to_string(msg).map_err(|e| protocol_err("message_to_json_str", e.to_string()))
}

pub fn response_to_json_str(resp: &Response) -> Result<String, ServErr> {
    serde_json::to_string(resp).map_err(|e| protocol_err("response_to_json_str", e.to_string()))
}

fn protocol_err(method: &str, err_msg: String) -> ServErr {
    ServErr::ServerError(ErrorDetails {
        err_msg,
        file: file!().to_string(),
        method: method.to_string(),
    })
}
