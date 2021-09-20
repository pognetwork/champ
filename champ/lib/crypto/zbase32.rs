// This file is based on https://github.com/matusf/z-base-32mit
// Copyright (c) 2021 The Pog Network Contributors
// Copyright (c) 2021 Matúš Ferech
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::string::FromUtf8Error;
use thiserror::Error;

const ZBASE_CHARS_LOWER: &[u8; 32] = b"ybndrfg8ejkmcpqxot1uwisza345h769";
const ZBASE_CHARS_UPPER: &[u8; 32] = b"YBNDRFG8EJKMCPQXOT1UWISZA345H769";

const ZBASE_INVERSE: [i8; 123] = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 18, -1, 25, 26, 27, 30, 29, 7, 31,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 24, 1, 12, 3, 8, 5, 6, 28, 21, 9, 10, -1, 11, 2, 16, 13, 14, 4, 22, 17, 19,
    -1, 20, 15, 0, 23,
];

#[derive(Error, Debug, PartialEq)]
pub enum ZbaseError {
    #[error("unknown error")]
    Unknown,
    #[error("decoding error")]
    DecodeError,
    #[error("invalid digit")]
    InvalidDigit,
    #[error("utf8 error")]
    Utf8Err { err: FromUtf8Error },
}

pub trait ToZbase {
    fn encode_zbase(&self) -> Result<String, ZbaseError>;
    fn encode_zbase_upper(&self) -> Result<String, ZbaseError>;
}

impl<T: AsRef<[u8]>> ToZbase for T {
    fn encode_zbase(&self) -> Result<String, ZbaseError> {
        encode(self)
    }

    fn encode_zbase_upper(&self) -> Result<String, ZbaseError> {
        encode_upper(self)
    }
}

pub trait FromZbase: Sized {
    fn from_zbase<T: AsRef<[u8]>>(zbase: T) -> Result<Self, ZbaseError>;
}

impl FromZbase for Vec<u8> {
    fn from_zbase<T: AsRef<[u8]>>(zbase: T) -> Result<Self, ZbaseError> {
        decode(zbase)
    }
}

fn encode_internal<T: AsRef<[u8]>>(input: T, alphabet: &[u8; 32]) -> Result<String, ZbaseError> {
    let data = input.as_ref();
    let mut result = Vec::new();
    let chunks = data.chunks(5);

    for chunk in chunks {
        let buf = {
            let mut buf = [0u8; 5];
            for (i, &b) in chunk.iter().enumerate() {
                buf[i] = b;
            }
            buf
        };
        result.push(alphabet[((buf[0] & 0xF8) >> 3) as usize]);
        result.push(alphabet[((buf[0] & 0x07) << 2 | (buf[1] & 0xC0) >> 6) as usize]);
        result.push(alphabet[((buf[1] & 0x3E) >> 1) as usize]);
        result.push(alphabet[((buf[1] & 0x01) << 4 | (buf[2] & 0xF0) >> 4) as usize]);
        result.push(alphabet[((buf[2] & 0x0F) << 1 | (buf[3] & 0x80) >> 7) as usize]);
        result.push(alphabet[((buf[3] & 0x7C) >> 2) as usize]);
        result.push(alphabet[((buf[3] & 0x03) << 3 | (buf[4] & 0xE0) >> 5) as usize]);
        result.push(alphabet[(buf[4] & 0x1F) as usize]);
    }

    let expected_len = (data.len() as f32 * 8.0 / 5.0).ceil() as usize;
    for _ in 0..(result.len() - expected_len) {
        result.pop();
    }
    String::from_utf8(result).map_err(|err| ZbaseError::Utf8Err { err })
}

pub fn encode<T: AsRef<[u8]>>(input: T) -> Result<String, ZbaseError> {
    encode_internal(input.as_ref(), ZBASE_CHARS_LOWER)
}

pub fn encode_upper<T: AsRef<[u8]>>(input: T) -> Result<String, ZbaseError> {
    encode_internal(input.as_ref(), ZBASE_CHARS_UPPER)
}

pub fn decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, ZbaseError> {
    let data = input.as_ref();
    let mut result = Vec::new();
    for chunk in data.chunks(8) {
        let buf = {
            let mut buf = [0u8; 8];
            for (i, &ch) in chunk.iter().enumerate() {
                match ZBASE_INVERSE.get(ch as usize) {
                    Some(-1) => return Err(ZbaseError::DecodeError),
                    Some(x) => buf[i] = *x as u8,
                    None => return Err(ZbaseError::DecodeError),
                };
            }
            buf
        };
        result.push((buf[0] << 3) | (buf[1] >> 2));
        result.push((buf[1] << 6) | (buf[2] << 1) | (buf[3] >> 4));
        result.push((buf[3] << 4) | (buf[4] >> 1));
        result.push((buf[4] << 7) | (buf[5] << 2) | (buf[6] >> 3));
        result.push((buf[6] << 5) | buf[7]);
    }

    for _ in 0..(result.len() - data.len() * 5 / 8) {
        result.pop();
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple_encode() {
        assert_eq!(encode(b"asdasd").unwrap(), "cf3seamuco".to_string());
    }

    #[test]
    fn simple_decode() {
        assert_eq!(decode("cf3seamu").unwrap(), b"asdas".to_vec())
    }

    #[test]
    fn encode_decode() {
        assert_eq!(decode(&encode(b"foo").unwrap()).unwrap(), b"foo")
    }

    #[test]
    fn invalid_decode() {
        assert_eq!(decode("bar#").unwrap_err(), ZbaseError::DecodeError)
    }
}
