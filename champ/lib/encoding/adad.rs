//! Synchronous and asynchronous implementations of ADAD decoders and encoders.
//!
//! Following the latest draft of PRC-4: https://pog.network/specification/PIPs/04-ADAD/

use futures::io::{AsyncRead, AsyncReadExt};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ADADError {
    #[error("unknown error")]
    Unknown,
    #[error("failed to read associated data length")]
    VarIntReadError(#[from] unsigned_varint::io::ReadError),
    #[error(transparent)]
    IOReadError(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct Data {
    pub associated_data: Vec<u8>,
    pub associated_data_codec: usize,
    pub authenticated_data: Vec<u8>,
    pub authenticated_data_codec: usize,
}

#[derive(Debug)]
pub struct ADAD {
    pub associated_data_max_size: usize,
    pub authenticated_data_max_size: usize,
}

const ASSOCIATED_DATA_MAX_SIZE: usize = 2_000_000;
const AUTHENTICATED_DATA_MAX_SIZE: usize = 10_000_000;

impl Default for ADAD {
    fn default() -> Self {
        Self {
            associated_data_max_size: ASSOCIATED_DATA_MAX_SIZE,
            authenticated_data_max_size: AUTHENTICATED_DATA_MAX_SIZE,
        }
    }
}

#[allow(non_upper_case_globals)]
pub const default: &ADAD = &ADAD {
    associated_data_max_size: ASSOCIATED_DATA_MAX_SIZE,
    authenticated_data_max_size: AUTHENTICATED_DATA_MAX_SIZE,
};

impl ADAD {
    pub fn new(associated_data_max_size: usize, authenticated_data_max_size: usize) -> Self {
        Self {
            associated_data_max_size,
            authenticated_data_max_size,
        }
    }

    pub fn default() -> Self {
        Default::default()
    }

    /// Encodes data as ADAD bytes
    pub fn encode(&self, data: Data) -> Vec<u8> {
        let aud_length = data.authenticated_data.len();
        let asd_length = data.associated_data.len();

        let asd_varint = usize_to_varint(asd_length);
        let associated_data_codec = usize_to_varint(data.associated_data_codec);
        let authenticated_data_codec = usize_to_varint(data.authenticated_data_codec);

        let mut buf: Vec<u8> = Vec::with_capacity(asd_varint.len() + aud_length + asd_length);

        buf.extend(asd_varint);
        buf.extend(associated_data_codec);
        buf.extend(&data.associated_data);
        buf.extend(authenticated_data_codec);
        buf.extend(&data.authenticated_data);
        buf
    }

    /// Reads ADAD data from a reader and decodes it to associated data and authenticated data
    pub fn read<T: std::io::Read>(&self, mut reader: T) -> Result<Data, ADADError> {
        let (associated_data, associated_data_codec) = self.read_associated_data(&mut reader)?;
        let (authenticated_data, authenticated_data_codec) = self.read_authenticated_data(&mut reader)?;

        Ok(Data {
            associated_data,
            associated_data_codec,
            authenticated_data,
            authenticated_data_codec,
        })
    }

    /// Reads authenticated data from a async reader
    pub fn read_authenticated_data<T: std::io::Read>(&self, mut reader: T) -> Result<(Vec<u8>, usize), ADADError> {
        let codec = unsigned_varint::io::read_usize(&mut reader)?;
        let mut authenticated_data = vec![];
        reader.read_to_end(&mut authenticated_data)?;
        Ok((authenticated_data, codec))
    }

    /// Reads associated data from a reader
    pub fn read_associated_data<T: std::io::Read>(&self, mut reader: T) -> Result<(Vec<u8>, usize), ADADError> {
        let length = unsigned_varint::io::read_usize(&mut reader)?;
        let codec = unsigned_varint::io::read_usize(&mut reader)?;
        let mut associated_data = vec![0u8; length];

        reader.read_exact(&mut associated_data)?;
        Ok((associated_data, codec))
    }

    /// Reads ADAD data from a async reader and decodes it to associated data and authenticated data
    pub async fn async_read<T>(&self, mut reader: T) -> Result<Data, ADADError>
    where
        T: AsyncRead + Unpin + Send,
    {
        let (associated_data, associated_data_codec) = self.async_read_associated_data(&mut reader).await?;
        let (authenticated_data, authenticated_data_codec) = self.async_read_authenticated_data(&mut reader).await?;

        Ok(Data {
            associated_data,
            associated_data_codec,
            authenticated_data,
            authenticated_data_codec,
        })
    }

    /// Reads associated data from a async reader
    pub async fn async_read_associated_data<T>(&self, mut reader: T) -> Result<(Vec<u8>, usize), ADADError>
    where
        T: futures::io::AsyncRead + Unpin + Send,
    {
        let length = unsigned_varint::aio::read_usize(&mut reader).await?;
        let codec = unsigned_varint::aio::read_usize(&mut reader).await?;

        todo!("pls add max size to prevent ddos");
        let mut associated_data = vec![0u8; length];

        reader.read_exact(&mut associated_data).await?;
        Ok((associated_data, codec))
    }

    /// Reads authenticated data from a async reader
    pub async fn async_read_authenticated_data<T>(&self, mut reader: T) -> Result<(Vec<u8>, usize), ADADError>
    where
        T: futures::io::AsyncRead + Unpin + Send,
    {
        let codec = unsigned_varint::aio::read_usize(&mut reader).await?;
        let mut authenticated_data = vec![];

        todo!("pls add max size to prevent ddos");
        reader.read_to_end(&mut authenticated_data).await?;
        Ok((authenticated_data, codec))
    }
}

fn usize_to_varint(n: usize) -> Vec<u8> {
    let buf = &mut unsigned_varint::encode::usize_buffer();
    unsigned_varint::encode::usize(n, buf).into()
}

// full list on https://github.com/multiformats/multicodec/blob/master/table.csv
pub enum Codecs {
    Plaintext = 0x706c61,
    Protobuf = 0x50,
    Messagepack = 0x0201,
    RLP = 0x60,
}

#[cfg(test)]
mod tests {
    use super::default as adad;
    use super::*;

    const CODECS: [(usize, usize); 6] = [
        (Codecs::Plaintext as usize, Codecs::Plaintext as usize),
        (Codecs::RLP as usize, Codecs::Protobuf as usize),
        (Codecs::Messagepack as usize, Codecs::Messagepack as usize),
        (Codecs::Protobuf as usize, Codecs::Protobuf as usize),
        (0, 0),
        (usize::MAX, usize::MAX),
    ];

    const VALID_DATA: [(&[u8], &[u8]); 5] = [
        ("test_header".as_bytes(), "test_data".as_bytes()),
        ("".as_bytes(), "".as_bytes()),
        ("".as_bytes(), "test_data".as_bytes()),
        ("test_header".as_bytes(), "".as_bytes()),
        ("test_header".as_bytes(), "long_test_data".as_bytes()),
    ];

    #[test]
    fn test() {
        for (associated_data, authenticated_data) in VALID_DATA {
            for (codec1, codec2) in CODECS {
                let encoded = adad.encode(Data {
                    associated_data: associated_data.to_vec(),
                    associated_data_codec: codec1,
                    authenticated_data: authenticated_data.to_vec(),
                    authenticated_data_codec: codec2,
                });

                let decoded = adad.read(&mut encoded.as_slice()).expect("should decode");

                assert_eq!(decoded.associated_data, associated_data);
                assert_eq!(decoded.authenticated_data, authenticated_data);
            }
        }
    }

    #[tokio::test]
    async fn test_async() {
        for (associated_data, authenticated_data) in VALID_DATA {
            for (codec1, codec2) in CODECS {
                let encoded = adad.encode(Data {
                    associated_data: associated_data.to_vec(),
                    associated_data_codec: codec1,
                    authenticated_data: authenticated_data.to_vec(),
                    authenticated_data_codec: codec2,
                });

                let decoded = adad.async_read(&mut encoded.as_slice()).await.expect("should decode");

                assert_eq!(decoded.associated_data, associated_data);
                assert_eq!(decoded.authenticated_data, authenticated_data);
            }
        }
    }
}
