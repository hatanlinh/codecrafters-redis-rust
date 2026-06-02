mod resp;

use crate::resp::{RespData, RespParser};

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0u8; 1024];
    let mut parser = RespParser::new();

    loop {
        let bytes_read = stream.read(&mut buffer).await?;

        if bytes_read == 0 {
            println!("Client closed the connection.");
            break;
        }

        parser.feed(&buffer[..bytes_read]);

        while let Some(RespData::Array(request)) = parser.parse()? {
            match request.as_slice() {
                [RespData::BulkString(command)] => {
                    if command.eq_ignore_ascii_case(b"PING") {
                        let pong = RespData::SimpleString(String::from("PONG"));
                        stream.write_all(pong.serialize().as_slice()).await?;
                    }
                }

                [RespData::BulkString(command), arg] => {
                    if command.eq_ignore_ascii_case(b"ECHO") {
                        if matches!(arg, RespData::BulkString(_)) {
                            stream.write_all(arg.serialize().as_slice()).await?;
                        }
                    }
                }

                _ => {
                    continue;
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let host = "127.0.0.1";
    let port = "6379";
    let listener = TcpListener::bind(format!("{}:{}", host, port)).await?;

    loop {
        let connection = listener.accept().await;

        match connection {
            Ok((stream, _)) => {
                tokio::spawn(handle_connection(stream));
            }
            Err(e) => {
                print!("error: {}", e);
            }
        }
    }
}
