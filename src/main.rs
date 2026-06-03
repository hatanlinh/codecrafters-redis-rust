mod engine;
mod resp;
mod storage;

use crate::engine::Engine;
use crate::resp::{RespData, RespParser};

use std::sync::Arc;

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

async fn handle_connection(mut stream: TcpStream, engine: Arc<Mutex<Engine>>) -> Result<()> {
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
                [RespData::BulkString(command), args @ ..] => {
                    let mut engine_guard = engine.lock().await;
                    let response = match command.as_slice() {
                        cmd if cmd.eq_ignore_ascii_case(b"PING") => engine_guard.handle_ping(),
                        cmd if cmd.eq_ignore_ascii_case(b"ECHO") => engine_guard.handle_echo(args),
                        cmd if cmd.eq_ignore_ascii_case(b"SET") => engine_guard.handle_set(args),
                        cmd if cmd.eq_ignore_ascii_case(b"GET") => engine_guard.handle_get(args),
                        _ => RespData::Error(String::from("ERR unknown command received")),
                    };
                    stream.write_all(response.serialize().as_slice()).await?;
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
    let engine = Arc::new(Mutex::new(Engine::new()));

    let host = "127.0.0.1";
    let port = "6379";
    let listener = TcpListener::bind(format!("{}:{}", host, port)).await?;

    loop {
        let connection = listener.accept().await;

        match connection {
            Ok((stream, _)) => {
                let engine_clone = engine.clone();
                tokio::spawn(handle_connection(stream, engine_clone));
            }
            Err(e) => {
                print!("error: {}", e);
            }
        }
    }
}
