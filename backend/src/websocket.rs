use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket, Message},
    response::Response,
};

use shared::{ClientMsg, ServerMsg};

// GET /ws : upgrades the HTTP request to a WebSocket, then runs handle_socket for it
pub async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

// Runs for one client's connection until it closes
async fn handle_socket(mut socket: WebSocket) {
    // Ends the loop on a receive error or when the client closes the connection
    while let Some(Ok(msg)) = socket.recv().await {
        let text = match msg {
            Message::Text(text) => text,
            _ => continue, // not a text message: ignore
        };

        let incoming = match serde_json::from_str::<ClientMsg>(text.as_str()) {
            Ok(incoming) => incoming,
            Err(_) => continue, // not a valid ClientMsg: skip it
        };

        let reply = handle_message(incoming);

        // Serialize the reply and send it back
        let json = serde_json::to_string(&reply).unwrap();
        if socket.send(Message::Text(json.into())).await.is_err() {
            return; // failed to send to client
        }
    }
}

// Every message type is listed here; the compiler errors if one is missing
fn handle_message(msg: ClientMsg) -> ServerMsg {
    match msg {
        // Echo the client message
        ClientMsg::Chat { text } => ServerMsg::Chat { text },
    }
}