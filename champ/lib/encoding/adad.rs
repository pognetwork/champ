use thiserror::Error;

#[derive(Error, Debug)]
pub enum ADADError {
    #[error("unknown error")]
    Unknown,
    #[error("failed to read associated data length")]
    VarIntReadError(#[from] unsigned_varint::io::ReadError),
    #[error("failed to read associated data length")]
    IOReadError(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct ADAD {
    pub associated_data: Vec<u8>,
    pub authenticated_data: Vec<u8>,
}

pub fn encode(data: ADAD) -> Vec<u8> {
    let aud_length = data.authenticated_data.len();
    let asd_length = data.associated_data.len();

    let varint_buf = &mut unsigned_varint::encode::usize_buffer();
    let asd_varint = unsigned_varint::encode::usize(asd_length, varint_buf);
    let mut buf: Vec<u8> = Vec::with_capacity(asd_varint.len() + aud_length + asd_length);

    buf.extend(asd_varint);
    buf.extend(&data.associated_data);
    buf.extend(&data.authenticated_data);
    buf
}

pub fn read<T: std::io::Read>(mut reader: T) -> Result<ADAD, ADADError> {
    let associated_data = read_associated_data(&mut reader)?;

    let mut authenticated_data = vec![];
    reader.read_to_end(&mut authenticated_data)?;

    Ok(ADAD {
        associated_data,
        authenticated_data,
    })
}

pub fn read_associated_data<T: std::io::Read>(mut reader: T) -> Result<Vec<u8>, ADADError> {
    let length = unsigned_varint::io::read_usize(&mut reader)?;
    let mut associated_data = vec![0u8; length];

    reader.read_exact(&mut associated_data)?;
    Ok(associated_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adad;

    #[test]
    fn test() {
        let valid_data = vec![
            ("test_header".as_bytes(), "test_data".as_bytes()),
            ("".as_bytes(), "".as_bytes()),
            ("".as_bytes(), "test_data".as_bytes()),
            ("test_header".as_bytes(), "".as_bytes()),
            ("test_header".as_bytes(), "long_test_data".as_bytes()),
        ];

        for (associated_data, authenticated_data) in valid_data {
            let encoded = adad::encode(ADAD {
                associated_data: associated_data.to_vec(),
                authenticated_data: authenticated_data.to_vec(),
            });

            let decoded = adad::read(&mut encoded.as_slice()).expect("should decode");

            assert_eq!(decoded.associated_data, associated_data);
            assert_eq!(decoded.authenticated_data, authenticated_data);
        }
    }
}
