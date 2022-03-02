use async_trait::async_trait;
use libp2p::{
    core::{
        upgrade::{read_length_prefixed, write_length_prefixed},
        ProtocolName,
    },
    futures::{io, AsyncRead, AsyncWrite, AsyncWriteExt},
    request_response::{ProtocolSupport, RequestResponse, RequestResponseCodec},
};

#[derive(Debug, Clone)]
pub struct PogCodec();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PogRequest(String);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PogResponse(String);

#[derive(Debug, Clone)]
pub struct PogProtocol();

impl ProtocolName for PogProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/pog/1".as_bytes()
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
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        Ok(PogRequest(String::from_utf8(vec).unwrap()))
    }

    async fn read_response<T>(&mut self, _: &PogProtocol, io: &mut T) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        Ok(PogResponse(String::from_utf8(vec).unwrap()))
    }

    async fn write_request<T>(&mut self, _: &PogProtocol, io: &mut T, PogRequest(data): PogRequest) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &PogProtocol,
        io: &mut T,
        PogResponse(data): PogResponse,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }
}

pub type Pog = RequestResponse<PogCodec>;

pub fn new() -> RequestResponse<PogCodec> {
    RequestResponse::new(PogCodec(), std::iter::once((PogProtocol(), ProtocolSupport::Full)), Default::default())
}
