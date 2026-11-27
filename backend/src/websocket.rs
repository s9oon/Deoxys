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
    loop {
        match socket.recv().await {
            Some(Ok(Message::Text(text))) => {

                // Deserialize client message
                let Ok(incoming) = serde_json::from_str::<ClientMsg>(text.as_str()) else {
                    continue; // not a valid ClientMsg
                };

                let reply = handle_message(incoming);

                // Serialize "reply"
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
        // Echo the client message
        ClientMsg::Chat { text } => ServerMsg::Chat { text },
    }
}