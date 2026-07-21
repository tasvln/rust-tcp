#[path = "protocol.rs"]
mod protocol;
use protocol::ClientMessage;
use protocol::PlayerState;
use protocol::ServerMessage;
use uuid::Uuid;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:7777")
        .await
        .unwrap();
    println!("server listening on: 7777");

    let state: Arc<tokio::sync::Mutex<HashMap<Uuid, PlayerState>>> =
        Arc::new(tokio::sync::Mutex::new(HashMap::new()));

    // tx / ClientMessage — "here's an event that just happened" (useful for things like Chat, where you do want every individual message, not just latest-state)
    // state_tx / ServerMessage::StateUpdate — "here's the complete truth as of right now" (useful for positions, where you only care about current state, not the history of how it got there)

    let (tx, _rx) = tokio::sync::broadcast::channel::<ClientMessage>(16);
    let (state_tx, _state_rx) = tokio::sync::broadcast::channel::<ServerMessage>(16);

    loop {
        {
            let state = state.clone();
            let state_tx = state_tx.clone();

            tokio::spawn(async move {
                let mut tick = tokio::time::interval(tokio::time::Duration::from_millis(500));
                loop {
                    tick.tick().await;
                    let players = state.lock().await.clone();
                    let _ = state_tx.send(ServerMessage::StateUpdate { players });
                }
            });
        }

        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("got a connection from {addr}");

        let tx = tx.clone(); // this client's own handle to the sender
        let state_tx = state_tx.clone();

        let rx = tx.subscribe();
        let mut state_rx = state_tx.subscribe();

        let state = state.clone();

        let player_id = Uuid::new_v4();
        let welcome = ServerMessage::Welcome { player_id };
        let bytes = bincode::serialize(&welcome).unwrap();
        protocol::framing::write_msg(&mut socket, &bytes)
            .await
            .unwrap();
        println!("assigned {player_id} to {addr}");

        tokio::spawn(async move {
            let (mut reader, mut writer) = socket.into_split();

            let writer = Arc::new(tokio::sync::Mutex::new(writer));

            // forward broadcast messages out to this client
            let mut rx2 = rx;
            let writer1 = writer.clone();

            tokio::spawn(async move {
                while let Ok(msg) = rx2.recv().await {
                    let wrapped = ServerMessage::PlayerEvent(msg);
                    let bytes = bincode::serialize(&wrapped).unwrap();
                    if protocol::framing::write_msg(&mut *writer1.lock().await, &bytes)
                        .await
                        .is_err()
                    {
                        break; // client's gone, stop trying to write to them
                    }
                }
            });

            let mut state_rx = state_tx.subscribe();
            let writer2 = writer.clone();
            tokio::spawn(async move {
                while let Ok(msg) = state_rx.recv().await {
                    let bytes = bincode::serialize(&msg).unwrap();
                    if protocol::framing::write_msg(&mut *writer2.lock().await, &bytes)
                        .await
                        .is_err()
                    {
                        break; // client's gone, stop trying to write to them
                    }
                }
            });

            // read from this client, broadcast what they send
            loop {
                let bytes = match protocol::framing::read_msg(&mut reader).await {
                    Ok(b) => b,
                    Err(_) => break, // connection closed or errored — treat as disconnect
                };
                let msg: ClientMessage = bincode::deserialize(&bytes).unwrap();
                println!("received: {:?}", msg);

                match &msg {
                    ClientMessage::Move { id, dx, dy } => {
                        let mut s = state.lock().await;
                        let player = s.entry(*id).or_insert(PlayerState {
                            id: *id,
                            x: 0.0,
                            y: 0.0,
                            completed_tasks: HashSet::new(),
                        });
                        player.x += *dx;
                        player.y += *dy;
                    }
                    ClientMessage::Chat { .. } => {}
                    ClientMessage::CompleteTask { id, task_id } => {
                        let mut s = state.lock().await;
                        if let Some(player) = s.get_mut(id) {
                            player.completed_tasks.insert(*task_id);
                        }
                    }
                }

                let _ = tx.send(msg);
            }

            // loop has ended — client disconnected
            state.lock().await.remove(&player_id);
            let _ = state_tx.send(ServerMessage::PlayerLeft { player_id });
            println!("{player_id} disconnected")
        });
    }
}
