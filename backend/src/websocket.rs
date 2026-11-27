// Structure adapted from the axum websockets example (MIT):
// https://github.com/tokio-rs/axum/blob/main/examples/websockets/src/main.rs

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};

use std::net::SocketAddr;
use std::ops::ControlFlow;
use tokio::sync::mpsc;

// Allows extracting the IP of the connecting user
use axum::extract::connect_info::ConnectInfo;

// Allows splitting the websocket stream into separate TX and RX branches
use futures_util::{sink::SinkExt, stream::StreamExt};

use crate::hub::{Hub, HubError};
use shared::{ChatHub, ClientMsg, ServerMsg};

/// The handler for the HTTP request (this gets called when the HTTP request lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client.
pub async fn handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    println!("{addr} connected");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(socket: WebSocket, who: SocketAddr) {
    // By splitting socket we can send and receive at the same time.
    let (mut sender, mut receiver) = socket.split();

    // Anything that wants to send to this client pushes a ServerMsg into `tx`;
    // the send task is the only thing that writes to the socket
    let (tx, mut rx) = mpsc::channel::<ServerMsg>(32);

    // This connection's hub; replies go through `tx` to the send task
    let hub = Hub::new(tx);

    // Send task: ServerMsg from the channel -> JSON -> socket
    let mut send_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(msg) = rx.recv().await {
            // Serializing our own enum can't fail
            let json = serde_json::to_string(&msg).unwrap();
            // In case of any websocket error, we exit.
            if sender.send(Message::Text(json.into())).await.is_err() {
                break;
            }
            cnt += 1;
        }
        cnt
    });

    // Receive task: socket -> process_message, until Close or an error
    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;
            // handle message and break if instructed to do so
            if process_message(msg, who, &hub).await.is_break() {
                break;
            }
        }
        cnt
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        rv_a = (&mut send_task) => {
            match rv_a {
                Ok(a) => println!("{a} messages sent to {who}"),
                Err(a) => println!("Error sending messages {a:?}")
            }
            recv_task.abort();
        },
        rv_b = (&mut recv_task) => {
            match rv_b {
                Ok(b) => println!("Received {b} messages"),
                Err(b) => println!("Error receiving messages {b:?}")
            }
            send_task.abort();
        }
    }

    // returning from the handler closes the websocket connection
    println!("Websocket context {who} destroyed");
}

/// Handles one frame: Text is decoded as a ClientMsg and dispatched to the hub,
/// Close ends the connection, the rest are logged.
async fn process_message(msg: Message, who: SocketAddr, hub: &Hub) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            let Ok(incoming) = serde_json::from_str::<ClientMsg>(t.as_str()) else {
                println!(">>> {who} sent invalid message: {t:?}");
                return ControlFlow::Continue(());
            };

            // Each error decides what happens to the connection
            match hub.handle(incoming).await {
                Ok(()) => {}
                Err(HubError::ClientGone) => return ControlFlow::Break(()),
                Err(HubError::Db(e)) => {
                    println!(">>> {who} db error: {e}"); // full detail stays on the server
                    let msg = ServerMsg::Error { message: "server error".into() };
                    if hub.reply(msg).await.is_err() {
                        return ControlFlow::Break(());
                    }
                }
            }
        }
        Message::Binary(d) => {
            println!(">>> {who} sent {} bytes: {d:?}", d.len());
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {who} sent close with code {} and reason `{}`",
                    cf.code, cf.reason
                );
            } else {
                println!(">>> {who} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            println!(">>> {who} sent pong with {v:?}");
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            println!(">>> {who} sent ping with {v:?}");
        }
    }
    ControlFlow::Continue(())
}
