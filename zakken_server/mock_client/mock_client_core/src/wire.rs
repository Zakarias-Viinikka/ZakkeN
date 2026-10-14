use error_stuff::{ErrorDetails, ServErr};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use protocol::error::DbError;
use protocol::serialization::{Base64Bytes, Convert};
use serde::Serialize;
use serde::de::DeserializeOwned;
use server_protocol::{
    Message, Request, Response, json_str_to_response, message_to_json_str,
};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type WsWrite = SplitSink<WsStream, WsMessage>;
pub type WsRead = SplitStream<WsStream>;

fn client_err(msg: impl Into<String>, method: &str) -> ServErr {
    ServErr::ServerError(ErrorDetails {
        err_msg: msg.into(),
        file: file!().to_string(),
        method: method.to_string(),
    })
}

/// Every response from the server is `Result<T, DbError>` bincode-encoded.
/// `Ok` means the request succeeded and T is the payload. `Err` means the
/// request failed and the DbError says why.
pub fn decode_result<T: Serialize + DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    let outer: Result<T, DbError> = Convert::un_payloadify(bytes)
        .map_err(|e| format!("couldn't decode server response: {e:?}"))?;
    outer.map_err(|e| format!("server returned an error: {e:?}"))
}

pub async fn send_request(
    ws: &mut WsWrite,
    msg_id: usize,
    request: Request,
    payload_bytes: Vec<u8>,
) -> Result<(), ServErr> {
    let msg = Message {
        message_id: msg_id,
        request,
        payload: Base64Bytes(payload_bytes),
    };
    let json = message_to_json_str(&msg)?;
    ws.send(WsMessage::Text(json.into()))
        .await
        .map_err(|e| client_err(format!("send failed: {e}"), "send_request"))
}

pub async fn expect_response(ws: &mut WsRead, msg_id: usize) -> Result<Response, ServErr> {
    while let Some(msg) = ws.next().await {
        let msg = msg.map_err(|e| client_err(format!("recv failed: {e}"), "expect_response"))?;
        let WsMessage::Text(text) = msg else {
            continue;
        };
        let response = json_str_to_response(text.as_str())?;
        if response.message_id == msg_id {
            return Ok(response);
        }
        // else: a push. Ignore it — mock client doesn't receive love letters.
    }
    Err(client_err("socket closed before response", "expect_response"))
}

pub async fn send_get_all_pages(ws: &mut WsWrite, msg_id: usize) -> Result<(), ServErr> {
    send_request(ws, msg_id, Request::GetAllYrsPages, Vec::new()).await
}