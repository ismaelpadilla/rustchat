pub(crate) use std::io::{self, Write};
use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::{tcp::OwnedWriteHalf, TcpListener};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

enum Command {
    Open {
        key: String,
        val: OwnedWriteHalf,
        addr: SocketAddr,
        responder: Responder,
    },
    Close {
        key: usize,
    },
    Message {
        key: usize,
        val: String,
    },
}

struct Client {
    stream: OwnedWriteHalf,
    addr: SocketAddr,
}
type Responder = oneshot::Sender<usize>;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server started on port 8080");

    let (tx, mut rx) = mpsc::channel(32);

    let _manager = tokio::spawn(async move {
        let mut clients = vec![];
        while let Some(message) = rx.recv().await {
            match message {
                Command::Open {
                    key,
                    val,
                    addr,
                    responder,
                } => {
                    clients.push(Client { stream: val, addr });
                    responder.send(addr.port().into()).unwrap();
                    println!("Open {}", key);
                }
                Command::Close { key } => {
                    println!("Close {}", key);
                    clients.retain(|client| usize::from(client.addr.port()) != key);
                }
                Command::Message { key, val } => {
                    for client in &mut clients {
                        client.stream.write_all(val.as_bytes()).await.unwrap();
                    }
                    println!("Message {} {}", key, val);
                }
            }
        }
    });
    loop {
        let (socket, addr) = listener.accept().await.unwrap();

        let (read_half, write_half) = socket.into_split();
        let (resp_tx, resp_rx) = oneshot::channel();

        tx.send(Command::Open {
            key: String::from("key"),
            addr,
            val: write_half,
            responder: resp_tx,
        })
        .await
        .unwrap();
        let key = resp_rx.await.unwrap();

        let tx2 = tx.clone();
        tokio::spawn(async move {
            handle_client(key, read_half, tx2).await;
        });
    }
}

async fn handle_client(id: usize, mut stream: OwnedReadHalf, sender: Sender<Command>) {
    println!("Connection established!");

    let mut buf = vec![0; 1024];
    'outer: loop {
        let result = stream.read(&mut buf).await.unwrap();
        if result == 0 {
            println!("Connection closed");
            sender.send(Command::Close { key: id }).await.unwrap();
            break 'outer;
        }
        sender
            .send(Command::Message {
                key: id,
                val: String::from_utf8(buf.clone()).unwrap(),
            })
            .await
            .unwrap();
        print!("Read {:?} bytes: ", result);
        io::stdout().write_all(&buf[0..result]).unwrap();
    }
}
