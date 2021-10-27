mod main;

use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_util::codec::{Framed, FramedRead, FramedWrite, LengthDelimitedCodec};

use bytes::{Bytes, BytesMut};
use libp2p::futures::SinkExt;

use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio_stream::StreamExt;

struct Connection {
    framed_write: FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>,
    framed_read: FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
}

impl Connection {
    //waits for a connection on that address
    pub async fn new<T: ToSocketAddrs>(address: T) -> Result<Connection, Box<dyn std::error::Error>> {
        let stream = Connection::connect_lol(address).await?;
        let (mut read_half, mut write_half) = stream.into_split();
        let connection = Connection {
            framed_read: FramedRead::new(read_half, LengthDelimitedCodec::new()),
            framed_write: FramedWrite::new(write_half, LengthDelimitedCodec::new()),
        };
        Ok(connection)
    }

    async fn connect_lol<T: ToSocketAddrs>(address: T) -> Result<TcpStream, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(address).await?;
        // The second item contains the IP and port of the new connection.
        let (stream, _) = listener.accept().await?;
        Ok(stream)
    }

    pub async fn write(&mut self, bytes: Bytes) -> Result<(), Box<dyn std::error::Error + '_>> {
        //idk what this anonymous lifetimes does
        //put Codec in own function / have as shared state?
        self.framed_write.send(bytes).await?; //use feed/ send all batch requests more efficient into the Sink
        Ok(())
    }

    pub async fn read(&mut self) -> BytesMut {
        //
        let mut buffer: BytesMut = Default::default(); //fix this
        while let Some(Ok(bytes)) = self.framed_read.next().await {
            //decide on how to actually do this
            buffer = bytes
        }
        buffer
    }
}
