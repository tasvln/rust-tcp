use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let mut socket = TcpStream::connect("127.0.0.1:7777").await.unwrap();
    println!("connected to server");

    socket.write_all(b"hello server").await.unwrap();
}
