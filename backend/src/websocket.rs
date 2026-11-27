use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket, Message},
    response::Response,
};

use shared::{ClientMsg, ServerMsg};

pub async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    loop {
        match socket.recv().await {
            Some(Ok(Message::Text(text))) => {
                let Ok(incoming) = serde_json::from_str::<ClientMsg>(text.as_str()) else {
                    continue; // not valid JSON: skip it
                };

                let reply = handle_message(incoming);

                let json = serde_json::to_string(&reply).unwrap();
                if socket.send(Message::Text(json.into())).await.is_err() {
                    return; // failed to send to client
                }
            }
            Some(Ok(_)) => {}       // not a text message: ignore
            Some(Err(_)) => return, // failed to receive from client
            None => return,         // client closed the connection
        }
    }
}

// Every message type is listed here; the compiler errors if one is missing
fn handle_message(msg: ClientMsg) -> ServerMsg {
    match msg {
        ClientMsg::Chat { text } => ServerMsg::Chat { text },
    }
}