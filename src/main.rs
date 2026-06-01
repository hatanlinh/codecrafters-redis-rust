use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

async fn handle_connection(mut connection: TcpStream) -> tokio::io::Result<()> {
    let mut buf = [0u8; 512];
    loop {
        match connection.read(&mut buf).await {
            Ok(0) => return Ok(()),
            Ok(_bytes_read) => {
                connection.write_all("+PONG\r\n".as_bytes()).await?;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let host = "127.0.0.1";
    let port = "6379";
    let listener = TcpListener::bind(format!("{}:{}", host, port)).await?;

    loop {
        let (connection, _) = listener.accept().await?;

        tokio::spawn(async move {
            handle_connection(connection).await.unwrap();
        });
    }
}
