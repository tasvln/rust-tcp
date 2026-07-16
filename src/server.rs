use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, broadcast};
use tokio::time::{Duration, interval};

use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7777").await.unwrap();
    println!("server listening on: 7777");

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("got a connection from {addr}");

        let mut buf = [0u8; 1024];
        let n = socket.read(&mut buf).await.unwrap();
        println!("received {n} bytes: {}", String::from_utf8_lossy(&buf[..n]));
    }
}
