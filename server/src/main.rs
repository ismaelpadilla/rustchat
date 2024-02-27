use std::io::{self, Read, Write};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server started on port 8080");

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_client(socket).await;
        });
    }
}

async fn handle_client(mut stream: TcpStream) {
    println!("Connection established!");
    stream.write_all(b"Connection established!\n").await;

    let mut buf = vec![0; 1024];
    'outer: loop {
        let result = stream.read(&mut buf).await.unwrap();
        if result == 0 {
            println!("Connection closed");
            break 'outer;
        }
        print!("Read {:?} bytes: ", result);
        io::stdout().write_all(&buf[0..result]).unwrap();
    }
}
