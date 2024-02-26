use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server started on port 8080");

    for stream in listener.incoming() {
        // println!("for");
        handle_client(stream?);
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    println!("Connection established!");
    stream.write_all(b"Connection established!");

    let mut buf = vec![0; 1024];
    for result in stream.read(&mut buf) {
        println!("{:?}", result);
        print!("Read: ");
        io::stdout().write_all(&buf[0..result]).unwrap();
    }
}
