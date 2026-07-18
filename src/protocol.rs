use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Welcome { player_id: Uuid },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Move { id: Uuid, dx: f32, dy: f32 },
    Chat { id: Uuid, text: String },
}
