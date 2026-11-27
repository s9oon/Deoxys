use shared::{ChatHub, ServerMsg};
use tokio::sync::mpsc;

// Everything that can go wrong inside a hub method.
// process_message (websocket.rs) decides what each one means for the connection.
#[derive(Debug)]
pub enum HubError {
    // The send task has stopped: the connection is finished
    ClientGone,
    // A query failed; details are logged, the client only gets a generic message
    Db(sqlx::Error),
}

// Lets hub methods use `?` on sqlx queries
impl From<sqlx::Error> for HubError {
    fn from(e: sqlx::Error) -> Self {
        HubError::Db(e)
    }
}

// One per connection: holds everything hub methods need
pub struct Hub {
    // This client's send task (SignalR: Clients.Caller)
    caller: mpsc::Sender<ServerMsg>,
}

impl Hub {
    pub fn new(caller: mpsc::Sender<ServerMsg>) -> Self {
        Hub { caller }
    }

    // Sends a message to this client
    pub async fn reply(&self, msg: ServerMsg) -> Result<(), HubError> {
        self.caller.send(msg).await.map_err(|_| HubError::ClientGone)
    }
}

// One method per line in hub! (shared/src/lib.rs)
impl ChatHub for Hub {
    type Error = HubError;

    async fn send_chat(&self, text: String) -> Result<(), HubError> {
        self.reply(ServerMsg::Chat { text }).await // echo for now
    }
}
