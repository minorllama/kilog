use std::io::{Read, Write};
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
//use std::io::Cursor;
use std::io::prelude::*;
use base64::{engine::general_purpose, Engine as _};


pub trait Encoder {
    fn encode(&self, data: &mut Vec<u8>);
    fn decode(&self, data: &mut Vec<u8>);
}

use std::fmt;

#[derive(Debug)]
pub enum EncoderParseError {
    UnknownEncoder(String),
}

impl fmt::Display for EncoderParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncoderParseError::UnknownEncoder(name) => write!(f, "Unknown encoder identifier: '{name}'"),
        }
    }
}

impl std::error::Error for EncoderParseError {}

pub struct GzBytes {
    log:bool
}

impl GzBytes {
    pub fn new() -> GzBytes {
        GzBytes {log:true}
    }

    // https://github.com/rust-lang/flate2-rs
    fn gzdecode(&self, bytes: &Vec<u8>) -> std::io::Result<Vec<u8>> {
        let mut decoder = GzDecoder::new(bytes.as_slice());
        let mut buffer = Vec::new();
        decoder.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn gzencode(&self, bytes: &Vec<u8>) -> std::io::Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes)?;
        let encoded = encoder.finish()?;
        Ok(encoded)
    }
}

impl Encoder for GzBytes {
    fn encode(&self, data: &mut Vec<u8>) {
        if let Ok(bytes) = self.gzencode(data) {
            *data = bytes;
        } else {
            if self.log {
                eprintln!("[gzencode_fail]");
            }
            let empty = Vec::new();
            *data = empty
        }
    }
    fn decode(&self, data: &mut Vec<u8>) {
        let decoded = self.gzdecode(data);
        if let Ok(bytes) = decoded {
            *data = bytes;
        } else {
            if self.log {
                eprintln!("[gzdecode_fail] {:?} {:?}", decoded, data);
            }
            let empty = Vec::new();
            *data = empty
        }
    }
}

pub struct Base64Encoder {
    log: bool,
}

impl Base64Encoder {
    pub fn new() -> Self {
        Base64Encoder { log: true }
    }

    pub fn with_log(log: bool) -> Self {
        Base64Encoder { log }
    }
}

impl Encoder for Base64Encoder {
    fn encode(&self, data: &mut Vec<u8>) {
        let encoded_string = general_purpose::STANDARD.encode(&data);
        *data = encoded_string.into_bytes();
    }

    fn decode(&self, data: &mut Vec<u8>) {
        match general_purpose::STANDARD.decode(&data) {
            Ok(decoded_bytes) => *data = decoded_bytes,
            Err(err) => {
                if self.log {
                    eprintln!("[base64decode_fail] {:?}", err);
                }
                data.clear();
            }
        }
    }
}

pub struct StackedEncoder {
    encoders: Vec<Box<dyn Encoder>>,
}


impl StackedEncoder {
    pub fn new(encoders: Vec<Box<dyn Encoder>>) -> Self {
        StackedEncoder { encoders }
    }

    pub fn from_str_spec(spec: &str) -> Result<Self, EncoderParseError> {
        let mut builder = StackedEncoder::new(Vec::new());
        let tokens = spec.split('.');
        for token in tokens {
            let tag = token.trim().to_lowercase();
            if !tag.is_empty() {
                match tag.as_str() {
                    "z" => {
                        builder.encoders.push(Box::new(GzBytes::new()));
                    }
                    "b64" => {
                        builder.encoders.push(Box::new(Base64Encoder::new()));
                    }
                    _ => return Err(EncoderParseError::UnknownEncoder(tag.to_string())),
                }
            }

        }
        Ok(builder)
    }
}

impl Encoder for StackedEncoder {
    fn encode(&self, data: &mut Vec<u8>) {
        for encoder in &self.encoders {
            encoder.encode(data);
        }
    }

    fn decode(&self, data: &mut Vec<u8>) {
        for encoder in self.encoders.iter().rev() {
            encoder.decode(data);
        }
    }
}
