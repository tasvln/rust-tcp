#[path = "protocol.rs"]
mod protocol;

use std::print;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() {
    let socket = tokio::net::TcpStream::connect("127.0.0.1:7777")
        .await
        .unwrap();
    println!("connected to server");

    let (mut reader, mut writer) = socket.into_split();

    // keep listening for anything the server sends, print it
    // read Welcome in main() so we can use player_id below
    // let mut buf = [0u8; 1024];
    // let n = reader.read(&mut buf).await.unwrap();
    let bytes = protocol::framing::read_msg(&mut reader).await.unwrap();
    let welcome_msg: protocol::ServerMessage = bincode::deserialize(&bytes).unwrap();

    let player_id = match welcome_msg {
        protocol::ServerMessage::Welcome { player_id } => player_id,
        _ => panic!("expected Welcome as first message"),
    };
    println!("got: {:?}", welcome_msg);

    let handle = tokio::spawn(async move {
        loop {
            let bytes = match protocol::framing::read_msg(&mut reader).await {
                Ok(b) => b,
                Err(_) => break, // connection closed or errored — treat as disconnect
            };

            // each iteration
            let msg: protocol::ServerMessage = bincode::deserialize(&bytes).unwrap();
            match msg {
                protocol::ServerMessage::Welcome { .. } => {}
                protocol::ServerMessage::PlayerEvent(client_msg) => {
                    println!("event: {:?}", client_msg);
                }
                protocol::ServerMessage::StateUpdate { players } => {
                    println!("state: {:?}", players);
                }
                protocol::ServerMessage::PlayerLeft { player_id } => {
                    println!("player left: {:?}", player_id);
                }
            }
        }
    });

    let msg = protocol::ClientMessage::Move {
        id: player_id,
        dx: 1.0,
        dy: 0.0,
    };

    let bytes = bincode::serialize(&msg).unwrap();
    protocol::framing::write_msg(&mut writer, &bytes)
        .await
        .unwrap();

    let msg = protocol::ClientMessage::CompleteTask {
        id: player_id,
        task_id: 1,
    };

    let bytes = bincode::serialize(&msg).unwrap();
    protocol::framing::write_msg(&mut writer, &bytes)
        .await
        .unwrap();

    handle.await.unwrap();
}
