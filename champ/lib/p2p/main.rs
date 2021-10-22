use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::Connection;
use tokio::net::TcpStream;
use tokio_util::codec::{LengthDelimitedCodec, FramedWrite};
use libp2p::futures::SinkExt;
use bytes::Bytes;

#[cfg(test)]
    #[tokio::test]
    async fn read_write_frame() {
        let address = "127.0.0.1:7890";
        let mut buffer:&'static [u8] = b"sdsd";

        let read_handle = tokio::spawn(async move {
            let mut connection = Connection::new(address).await.unwrap();
            let result = connection.read().await;
            result
        });

        let write_handle = tokio::spawn(async move {
            let mut stream = TcpStream::connect(address).await.unwrap();
            let mut framed = FramedWrite::new(stream, LengthDelimitedCodec::new());
            let x = framed.send(Bytes::from(buffer)).await.unwrap();
            framed.flush().await.unwrap(); //this flush is probably unnecessary
        });

        let result = read_handle.await.unwrap();
        write_handle.await.unwrap();

        assert_eq!(buffer, result.to_owned())
    }

