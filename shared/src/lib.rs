use serde::{Deserialize, Serialize};

// With `--features ts`, ts-rs exports this type to desktop/src/ipc/Hello.ts
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Hello {
    pub message: String,
}