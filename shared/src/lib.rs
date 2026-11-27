use serde::{Deserialize, Serialize};

// With `--features ts`, ts-rs exports this type to desktop/src/ipc/Hello.ts
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Hello {
    pub message: String,
}

// Declares what the client can call on the server, like a SignalR hub.
// Each line `Variant => method { fields }` generates:
//   - a `ClientMsg::Variant { fields }` (the wire message, exported to TypeScript)
//   - an `async fn method(&self, fields)` in `ChatHub` (the backend writes the body)
//   - the arm in `ChatHub::handle` that routes the message to that method
macro_rules! hub {
    ( $( $variant:ident => $method:ident { $( $field:ident : $ty:ty ),* } ),* $(,)? ) => {
        #[derive(Serialize, Deserialize, Debug)]
        #[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
        #[serde(tag = "type")]
        pub enum ClientMsg {
            $( $variant { $( $field: $ty ),* } ),*
        }

        // Only implemented on the backend's concrete Hub, so plain `async fn` is fine
        #[allow(async_fn_in_trait)]
        pub trait ChatHub {
            // The backend picks the error type, so this crate doesn't depend on sqlx
            type Error;

            $( async fn $method(&self, $( $field: $ty ),*) -> Result<(), Self::Error>; )*

            // Already written: routes a ClientMsg to the matching method
            async fn handle(&self, msg: ClientMsg) -> Result<(), Self::Error> {
                match msg {
                    $( ClientMsg::$variant { $( $field ),* } => self.$method($( $field ),*).await, )*
                }
            }
        }
    };
}

hub! {
    Chat => send_chat { text: String },
}

// What the server can send to the client
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum ServerMsg {
    Chat { text: String },
    Error { message: String },
}
