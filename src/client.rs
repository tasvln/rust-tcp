#[path = "protocol.rs"]
mod protocol;
use protocol::ClientMessage;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() {
    let socket = tokio::net::TcpStream::connect("127.0.0.1:7777")
        .await
        .unwrap();
    println!("connected to server");

    let (mut reader, mut writer) = socket.into_split();

    // Task: keep listening for anything the server sends, print it
    let handle = tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            let n = reader.read(&mut buf).await.unwrap();
            if n == 0 {
                println!("server closed connection");
                break;
            }

            let msg: ClientMessage = bincode::deserialize(&buf[..n]).unwrap();
            println!("got: {:?}", msg);
        }
    });

    // read lines you type, send each one
    // let stdin = tokio::io::stdin();
    // let mut lines = BufReader::new(stdin).lines();
    // while let Ok(Some(line)) = lines.next_line().await {
    //     writer.write_all(line.as_bytes()).await.unwrap();
    // }

    let msg = ClientMessage::Move { dx: 1.0, dy: 0.0 };
    let bytes = bincode::serialize(&msg).unwrap();
    writer.write_all(&bytes).await.unwrap();

    handle.await.unwrap();
}
