use bytes::{Bytes, BytesMut};
use libp2p::futures::SinkExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

struct Connection {
    framed_write: FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>,
    framed_read: FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
}

impl Connection {
    #[allow(dead_code)]
    //waits for an incoming connection on address
    pub async fn listen<T: ToSocketAddrs>(address: T) -> Result<Connection, Box<dyn std::error::Error>> {
        eprintln!("before bind");
        let listener = TcpListener::bind(address).await?;
        eprintln!("after bind");
        // The second item contains the IP and port of the new connection.
        let (stream, _) = listener.accept().await?;
        eprintln!("after accept");
        Ok(Connection::connection_from_stream(stream))
    }

    #[allow(dead_code)]
    //establishes a connection by connecting to address
    pub async fn connect<T: ToSocketAddrs>(address: T) -> Result<Connection, Box<dyn std::error::Error>> {
        let stream = TcpStream::connect(address).await?;
        Ok(Connection::connection_from_stream(stream))
    }

    //creates a Connection from the TcpStream,
    fn connection_from_stream(stream: TcpStream) -> Connection {
        let (read_half, write_half) = stream.into_split();
        Connection {
            framed_read: FramedRead::new(read_half, LengthDelimitedCodec::new()),
            framed_write: FramedWrite::new(write_half, LengthDelimitedCodec::new()),
        }
    }

    #[allow(dead_code)]
    //processes bytes into Sink and flushes.
    //use feed/ send all batch requests more efficient into the Sink
    pub async fn write(&mut self, bytes: Bytes) -> Result<(), Box<dyn std::error::Error>> {
        self.framed_write.send(bytes).await?;
        Ok(())
    }

    #[allow(dead_code)]
    //waits for connection to be able to read one frame
    pub async fn read(&mut self) -> BytesMut {
        let mut buffer: BytesMut = Default::default(); //fix this
        while let Some(Ok(bytes)) = self.framed_read.next().await {
            //decide on how to actually do this
            buffer = bytes
        }
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::thread;
    use std::time::Duration;

    #[tokio::test]
    #[serial]
    //test passes when TcpStream::connect yields a TcpStream value and therefore succeeds
    async fn listen_for_connection() {
        let address = "127.0.0.1:7899";
        let handle1 = tokio::spawn(async move {
            eprintln!("before listen");
            Connection::listen(address).await.expect("failed listening for the connection");
            eprintln!("after listen");
        });

        let handle2 = tokio::spawn(async move {
            eprintln!("before connect");
            thread::sleep(Duration::from_millis(5000));
            eprintln!("after wait");
            TcpStream::connect(address).await.expect("failed connecting");
            eprintln!("after connect");
        });

        handle1.await.unwrap();
        handle2.await.unwrap();
    }

    #[tokio::test]
    #[serial]
    //test fails when TcpStream::connect yields a TcpStream value and therefore did connect
    async fn listen_for_wrong_connection() {
        let address = "127.0.0.1:7890";
        let other_address = "127.0.0.1:7891";
        tokio::spawn(async move {
            Connection::listen(address).await.expect("failed listening for the connection");
        });
        let result = TcpStream::connect(other_address).await; //.expect("failed connecting");
        assert!(result.is_err())
    }

    #[tokio::test]
    #[serial]
    async fn read_frame() {
        let address = "127.0.0.1:7890";
        let buffer: Bytes = Bytes::from_static(b"testdata");
        let expected = buffer.clone();

        let read_handle = tokio::spawn(async move {
            let mut connection = Connection::listen(address).await.unwrap();
            connection.read().await
        });

        let write_handle = tokio::spawn(async move {
            let stream = TcpStream::connect(address).await.unwrap();
            let mut framed = FramedWrite::new(stream, LengthDelimitedCodec::new());
            framed.send(buffer.clone()).await.unwrap();
            framed.flush().await.unwrap(); //this flush is probably unnecessary
        });

        let result = read_handle.await.unwrap();
        write_handle.await.unwrap();

        assert_eq!(expected, result.to_owned())
    }

    //#[tokio::test]
    //async fn write_frame() {
    //    let address = "127.0.0.1:7890";
    //    let mut buffer: Bytes = Bytes::from_static(b"testdata");
    //    let mut expected = buffer.clone();
    //
    //    let read_handle = tokio::spawn(async move {
    //        let mut connection = Connection::listen(address).await.unwrap();
    //        let result = connection.write().await;
    //        result
    //    });
    //
    //    let write_handle = tokio::spawn(async move {
    //        let mut stream = TcpStream::connect(address).await.unwrap();
    //        let mut framed = FramedWrite::new(stream, LengthDelimitedCodec::new());
    //        let x = framed.send(buffer).await.unwrap();
    //        framed.flush().await.unwrap(); //this flush is probably unnecessary
    //    });
    //
    //    let result = read_handle.await.unwrap();
    //    write_handle.await.unwrap();
    //
    //    assert_eq!(buffer, result.to_owned())
    //}
}
