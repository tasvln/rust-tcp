use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Welcome { player_id: Uuid },
    StateUpdate { players: HashMap<Uuid, PlayerState> },
    PlayerLeft { player_id: Uuid },
    PlayerEvent(ClientMessage),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: Uuid,
    pub x: f32,
    pub y: f32,
    pub completed_tasks: HashSet<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Move { id: Uuid, dx: f32, dy: f32 },
    Chat { id: Uuid, text: String },
    CompleteTask { id: Uuid, task_id: u32 },
}

pub mod framing {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    pub async fn write_msg<W: AsyncWriteExt + Unpin>(
        writer: &mut W,
        bytes: &[u8],
    ) -> std::io::Result<()> {
        writer.write_u32(bytes.len() as u32).await?; // 4-byte length header
        writer.write_all(bytes).await?; // then the actual payload
        Ok(())
    }

    pub async fn read_msg<R: AsyncReadExt + Unpin>(reader: &mut R) -> std::io::Result<Vec<u8>> {
        let len = reader.read_u32().await?; // read the length first
        let mut buf = vec![0u8; len as usize]; // allocate exactly that much
        reader.read_exact(&mut buf).await?; // read exactly that many bytes, no more/less
        Ok(buf.to_vec())
    }
}
