use async_trait::async_trait;
use encoding::adad;
pub use libp2p::request_response::RequestResponseEvent;
pub use libp2p::request_response::RequestResponseMessage::{Request as RequestMessage, Response as ResponseMessage};

use libp2p::{
    core::{upgrade::write_length_prefixed, ProtocolName},
    futures::{io, AsyncRead, AsyncWrite, AsyncWriteExt},
    request_response::{ProtocolSupport, RequestResponse, RequestResponseCodec, RequestResponseMessage},
};

#[derive(Debug, Clone)]
pub struct PogCodec();

#[derive(Debug, Clone)]
pub struct PogRequest {
    pub header: Vec<u8>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PogResponse {
    pub header: Vec<u8>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PogProtocol {}

impl ProtocolName for PogProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/pog/1".as_bytes()
    }
}

fn invalid_data_error(msg: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
}

const PROTOBUF_CODEC: u32 = adad::Codecs::Protobuf as u32;
fn is_proto(codec: usize) -> Result<(), io::Error> {
    match codec as u32 {
        PROTOBUF_CODEC => Ok(()),
        _ => Err(invalid_data_error("invalid coded")),
    }
}

#[async_trait]
impl RequestResponseCodec for PogCodec {
    type Protocol = PogProtocol;
    type Request = PogRequest;
    type Response = PogResponse;

    async fn read_request<T>(&mut self, _: &PogProtocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let data = adad::default.async_read(io).await.map_err(|_| invalid_data_error("invalid data 😔"))?;

        is_proto(data.authenticated_data_codec)?;
        is_proto(data.associated_data_codec)?;

        Ok(PogRequest {
            data: data.authenticated_data,
            header: data.associated_data,
        })
    }

    async fn read_response<T>(&mut self, _: &PogProtocol, io: &mut T) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let raw_data = adad::default.async_read(io).await.map_err(|_| invalid_data_error("invalid data 😔"))?;

        is_proto(raw_data.authenticated_data_codec)?;
        is_proto(raw_data.associated_data_codec)?;

        Ok(PogResponse {
            data: raw_data.authenticated_data,
            header: raw_data.associated_data,
        })
    }

    async fn write_request<T>(
        &mut self,
        _: &PogProtocol,
        io: &mut T,
        PogRequest {
            data,
            header,
        }: PogRequest,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let associated_data = header;
        let authenticated_data = data;

        let buf = adad::default.encode(adad::Data {
            associated_data,
            associated_data_codec: adad::Codecs::Protobuf as usize,
            authenticated_data,
            authenticated_data_codec: adad::Codecs::Protobuf as usize,
        });

        write_length_prefixed(io, buf).await?;
        io.close().await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &PogProtocol,
        io: &mut T,
        PogResponse {
            header,
            data,
        }: PogResponse,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let associated_data = header;
        let authenticated_data = data;

        let buf = adad::default.encode(adad::Data {
            associated_data,
            associated_data_codec: adad::Codecs::Protobuf as usize,
            authenticated_data,
            authenticated_data_codec: adad::Codecs::Protobuf as usize,
        });

        write_length_prefixed(io, buf).await?;
        io.close().await?;

        Ok(())
    }
}

impl PogProtocol {
    pub fn new() -> Self {
        PogProtocol {}
    }

    pub fn behavior(self) -> PogBehavior {
        RequestResponse::new(PogCodec(), std::iter::once((self, ProtocolSupport::Full)), Default::default())
    }
}

pub type PogBehavior = RequestResponse<PogCodec>;
pub type PogMessage = RequestResponseMessage<PogRequest, PogResponse>;
