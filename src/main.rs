use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

fn handle_connection(mut stream: TcpStream) {
    stream.write("+PONG\r\n".as_bytes()).unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                println!("accepted new connection");
                handle_connection(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
