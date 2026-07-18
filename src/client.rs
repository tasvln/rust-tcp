#[path = "protocol.rs"]
mod protocol;

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
    let mut buf = [0u8; 1024];
    let n = reader.read(&mut buf).await.unwrap();
    let welcome: protocol::ServerMessage = bincode::deserialize(&buf[..n]).unwrap();
    let protocol::ServerMessage::Welcome { player_id } = welcome;

    println!("got: {:?}", welcome);
    let handle = tokio::spawn(async move {
        let mut buf = [0u8; 1024];

        loop {
            let n = reader.read(&mut buf).await.unwrap();
            if n == 0 {
                println!("server closed connection");
                break;
            }

            let msg: protocol::ClientMessage = bincode::deserialize(&buf[..n]).unwrap();
            println!("got: {:?}", msg);
        }
    });

    // read lines you type, send each one
    // let stdin = tokio::io::stdin();
    // let mut lines = BufReader::new(stdin).lines();
    // while let Ok(Some(line)) = lines.next_line().await {
    //     writer.write_all(line.as_bytes()).await.unwrap();
    // }

    let msg = protocol::ClientMessage::Move {
        id: player_id,
        dx: 1.0,
        dy: 0.0,
    };

    let bytes = bincode::serialize(&msg).unwrap();
    writer.write_all(&bytes).await.unwrap();

    handle.await.unwrap();
}
