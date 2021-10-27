use crate::Connection;
use bytes::Bytes;
use libp2p::futures::SinkExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

#[cfg(test)]
#[tokio::test]
//test passes when TcpStream::connect yields a TcpStream value and therefore succeeds
async fn listen_for_connection() {
    let address = "127.0.0.1:7890";
    let handle = tokio::spawn(async move {
        Connection::listen(address).await.expect("failed listening for the connection");
    });
    let stream = TcpStream::connect(address).await.expect("failed connecting");;
}

#[cfg(test)]
#[tokio::test]
#[should_panic]
//test fails when TcpStream::connect yields a TcpStream value and therefore did connect
async fn listen_for_wrong_connection() {
    let address = "127.0.0.1:7890";
    let other_address = "127.0.0.1:7891";
    let handle = tokio::spawn(async move {
        Connection::listen(address).await.expect("failed listening for the connection");
    });
    let stream = TcpStream::connect(other_address).await.expect("failed connecting");;
}

#[cfg(test)]
#[tokio::test]
async fn read_frame() {
    let address = "127.0.0.1:7890";
    let mut buffer: Bytes = Bytes::from_static(b"testdata");
    let mut expected = buffer.clone();

    let read_handle = tokio::spawn(async move {
        let mut connection = Connection::new(address).await.unwrap();
        let result = connection.read().await;
        result
    });


    let write_handle = tokio::spawn(async move {
        let mut stream = TcpStream::connect(address).await.unwrap();
        let mut framed = FramedWrite::new(stream, LengthDelimitedCodec::new());
        let x = framed.send(buffer.clone()).await.unwrap();
        framed.flush().await.unwrap(); //this flush is probably unnecessary
    });

    let result = read_handle.await.unwrap();
    write_handle.await.unwrap();

    assert_eq!(expected, result.to_owned())
}

//#[cfg(test)]
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