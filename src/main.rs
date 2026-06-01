use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buf = [0u8; 1024];

    loop {
        let bytes_read = stream.read(&mut buf).await?;

        if bytes_read == 0 {
            println!("Client closed the connection.");
            break;
        }

        if &buf[..bytes_read] == b"*1\r\n$4\r\nPING\r\n" {
            stream.write_all(b"+PONG\r\n").await?;
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
