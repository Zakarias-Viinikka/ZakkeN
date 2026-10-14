use protocol::serialization::Base64Bytes;
use server_protocol::{Request, Response, response_to_json_str};

pub fn craft_response(data: Vec<u8>, message_id: usize, request: Request) -> String {
    let response = Response {
        message_id,
        request,
        payload: Base64Bytes(data),
    };
    response_to_json_str(&response)
        .unwrap_or_else(|e| format!(r#"{{"error":"failed to serialize response: {}"}}"#, e))
}

use axum::extract::ws::{Message, WebSocket};

pub async fn send_message(socket: &mut WebSocket, response: String) -> Result<(), String> {
    socket
        .send(Message::Text(response.into()))
        .await
        .map_err(|_| "failed to send message".to_string())
}