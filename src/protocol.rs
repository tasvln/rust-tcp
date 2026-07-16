use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Move { dx: f32, dy: f32 },
    Chat { text: String },
}
