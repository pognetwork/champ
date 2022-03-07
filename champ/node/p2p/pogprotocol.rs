use async_trait::async_trait;
use encoding::adad;
use libp2p::{
    core::{upgrade::write_length_prefixed, ProtocolName},
    futures::{io, AsyncRead, AsyncWrite, AsyncWriteExt},
    request_response::{ProtocolSupport, RequestResponse, RequestResponseCodec},
};
use pog_proto::{p2p, Message};

#[derive(Debug, Clone)]
pub struct PogCodec();

#[derive(Debug, Clone)]
pub struct PogRequest {
    pub header: p2p::RequestHeader,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PogResponse {
    pub header: p2p::ResponseHeader,
    pub data: p2p::ResponseData,
    pub data_raw: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PogProtocol();

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
        let data = adad::async_read(io).await.map_err(|_| invalid_data_error("invalid data 😔"))?;

        is_proto(data.authenticated_data_codec)?;
        is_proto(data.associated_data_codec)?;

        let header = p2p::RequestHeader::decode(&*data.associated_data)?;

        Ok(PogRequest {
            data: data.authenticated_data,
            header,
        })
    }

    async fn read_response<T>(&mut self, _: &PogProtocol, io: &mut T) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let raw_data = adad::async_read(io).await.map_err(|_| invalid_data_error("invalid data 😔"))?;

        is_proto(raw_data.authenticated_data_codec)?;
        is_proto(raw_data.associated_data_codec)?;

        let header = p2p::ResponseHeader::decode(&*raw_data.associated_data)?;
        let data = p2p::ResponseData::decode(&*raw_data.authenticated_data)?;

        Ok(PogResponse {
            data,
            data_raw: raw_data.authenticated_data,
            header,
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
        let associated_data = header.encode_to_vec();
        let authenticated_data = data.encode_to_vec();

        let buf = adad::encode(adad::ADAD {
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
            data_raw,
            data: _,
        }: PogResponse,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let associated_data = header.encode_to_vec();
        let authenticated_data = data_raw;

        let buf = adad::encode(adad::ADAD {
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

pub type Pog = RequestResponse<PogCodec>;

pub fn new() -> Pog {
    RequestResponse::new(PogCodec(), std::iter::once((PogProtocol(), ProtocolSupport::Full)), Default::default())
}
