use error_stuff::{ErrorDetails, ServErr};
use protocol::serialization::Base64Bytes;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub message_id: usize,
    pub request: Request,
    pub payload: Base64Bytes,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub message_id: usize,
    pub request: Request,
    pub payload: Base64Bytes,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Register,
    SendLoveLetter,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RegisterPayload {
    pub user_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SendLoveLetterPayload {
    pub love_letter: Vec<u8>,
}

pub fn json_str_to_message(json: &str) -> Result<Message, ServErr> {
    serde_json::from_str(json).map_err(|e| {
        ServErr::ServerError(ErrorDetails {
            err_msg: e.to_string(),
            file: file!().to_string(),
            method: "json_str_to_message".to_string(),
        })
    })
}

pub fn json_str_to_response(json: &str) -> Result<Response, ServErr> {
    serde_json::from_str(json).map_err(|e| {
        ServErr::ServerError(ErrorDetails {
            err_msg: e.to_string(),
            file: file!().to_string(),
            method: "json_str_to_response".to_string(),
        })
    })
}

pub fn message_to_json_str(msg: &Message) -> Result<String, ServErr> {
    serde_json::to_string(msg).map_err(|e| {
        ServErr::ServerError(ErrorDetails {
            err_msg: e.to_string(),
            file: file!().to_string(),
            method: "message_to_json_str".to_string(),
        })
    })
}

pub fn response_to_json_str(resp: &Response) -> Result<String, ServErr> {
    serde_json::to_string(resp).map_err(|e| {
        ServErr::ServerError(ErrorDetails {
            err_msg: e.to_string(),
            file: file!().to_string(),
            method: "response_to_json_str".to_string(),
        })
    })
}