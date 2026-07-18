#[path = "protocol.rs"]
mod protocol;
use protocol::ClientMessage;

use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, broadcast};
use tokio::time::{Duration, interval};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7777").await.unwrap();
    println!("server listening on: 7777");

    let (tx, _rx) = tokio::sync::broadcast::channel::<ClientMessage>(16);

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("got a connection from {addr}");

        let tx = tx.clone(); // this client's own handle to the sender
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (mut reader, mut writer) = socket.into_split();

            // forward broadcast messages out to this client
            let mut rx2 = rx;
            tokio::spawn(async move {
                while let Ok(msg) = rx2.recv().await {
                    let bytes = bincode::serialize(&msg).unwrap();
                    let _ = writer.write_all(&bytes).await;
                }
            });

            // read from this client, broadcast what they send
            loop {
                let mut buf = [0u8; 1024];
                let n = reader.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                } // client disconnected
                let msg: ClientMessage = bincode::deserialize(&buf[..n]).unwrap();
                println!("received: {:?}", msg);
                let _ = tx.send(msg);
            }
        });
    }
}
