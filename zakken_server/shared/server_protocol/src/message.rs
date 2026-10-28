use crate::request::Request;
use crate::responses::UnsentLetter;
use protocol::serialization::Base64Bytes;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub message_id: usize,
    pub request: Request,
    pub payload: Base64Bytes,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Response {
    /// A reply to a request the client sent. `message_id` matches the
    /// request, so the client knows which of its asks this answers.
    Reply {
        message_id: usize,
        request: Request,
        payload: Base64Bytes,
    },
    /// The server handing over one love letter. Carries no request id;
    /// the client acks it individually.
    LoveLetterFromServer(UnsentLetter),
}