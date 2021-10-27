mod main;

use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use bytes::{Bytes, BytesMut};
use libp2p::futures::SinkExt;

use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio_stream::StreamExt;

struct Connection {
    framed_write: FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>,
    framed_read: FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
}

impl Connection {
    //waits for an incoming connection on address
    pub async fn listen<T: ToSocketAddrs>(address: T) -> Result<Connection, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(address).await?;
        // The second item contains the IP and port of the new connection.
        let (stream, _) = listener.accept().await?;
        Connection::connection_from_stream(stream)
    }

    //establishes a connection by connecting to address
    pub async fn connect<T: ToSocketAddrs>(address: T) -> Result<Connection, Box<dyn std::error::Error>> {
        let stream = TcpStream::connect(address).await?;
        Connection::connection_from_stream(stream)
    }

    //creates a Connection from the TcpStream,
    fn connection_from_stream(stream: TcpStream) -> Result<Connection, Box<dyn std::error::Error>> {
        let (read_half, write_half) = stream.into_split();
        let connection = Connection {
            framed_read: FramedRead::new(read_half, LengthDelimitedCodec::new()),
            framed_write: FramedWrite::new(write_half, LengthDelimitedCodec::new()),
        };
        Ok(connection)
    }

    //processes bytes into Sink and flushes.
    //use feed/ send all batch requests more efficient into the Sink
    pub async fn write(&mut self, bytes: Bytes) -> Result<(), Box<dyn std::error::Error>> {
        self.framed_write.send(bytes).await?;
        Ok(())
    }

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
