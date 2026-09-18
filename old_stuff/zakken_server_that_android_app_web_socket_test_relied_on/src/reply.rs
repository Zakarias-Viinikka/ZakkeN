use protocol::{messages::*, serialization::*};

pub fn craft_response(data: Vec<u8>, message_id: usize, request: Request) -> String {
    let response = Response {
        message_id,
        request,
        payload: Base64Bytes(data),
    };
    let prepared_response = response_to_json_str(&response)
        .map_err(|e| serialization_error_as_jsonstr(e.to_string()))
        .unwrap_or_else(|e| e);

    prepared_response
}

fn serialization_error_as_jsonstr(err: String) -> String {
    format!(r#"{{"error":"failed to serialize response: {}"}}"#, err)
}

use axum::extract::ws::{Message, WebSocket};

pub async fn send_message(socket: &mut WebSocket, response: String) -> Result<(), String> {
    socket
        .send(Message::Text(response.into()))
        .await
        .map_err(|_| "failed to send message".to_string())
}
