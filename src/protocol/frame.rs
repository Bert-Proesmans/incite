use bytes::{BigEndian, BufMut, ByteOrder, Bytes, BytesMut};
use tokio_codec::{Decoder, Encoder};

use incite_gen::prost::Message;
use incite_gen::proto::bnet::protocol::Header;

// 2 bytes long preamble, interpreted as U16BE
const HEADER_PREAMBLE_LENGTH: usize = 2;

mod error {
    use incite_gen::prost;
    use std::io;

    error_chain!{
        foreign_links {
            Io(io::Error) #[doc = "Error during IO"];
            ProtoEncode(prost::EncodeError) #[doc = "Error during Protobuffer encoding"];
            ProtoDecode(prost::DecodeError) #[doc = "Error during Protobuffer decoding"];
        }

        errors {
            TLSEnabled {
                description("The client tries to perform a TLS handshake")
                display("The client is using TLS")
            }

            MissingData(field_name: String) {
                description("Missing required data")
                display("Data for the field '{}' was not provided", field_name)
            }
        }
    }
}

pub use self::error::*;
// Overwrite scope with std's Result because Encoder and Decoder traits need it!
use std::result::Result;

#[derive(Debug)]
pub struct BNetPacket {
    pub header: Header,
    pub body: Bytes,
}

impl BNetPacket {
    fn new(header: Header, body: Bytes) -> Self {
        Self { header, body }
    }
}

pub struct BNetCodec {
    header: Option<Header>,
    header_length: Option<u16>,
    body_size: Option<u32>,
}

impl BNetCodec {
    pub fn new() -> Self {
        BNetCodec {
            header: None,
            header_length: None,
            body_size: None,
        }
    }
}

impl Encoder for BNetCodec {
    type Item = BNetPacket;
    type Error = Error;

    fn encode(&mut self, item: BNetPacket, destination: &mut BytesMut) -> Result<(), Self::Error> {
        let BNetPacket { header, body } = item;
        let data_length = HEADER_PREAMBLE_LENGTH + header.encoded_len() + body.len();
        destination.reserve(data_length);

        let header_preamble: u16 = header.encoded_len() as _;
        BigEndian::write_u16(destination, header_preamble);
        header.encode(destination)?;
        destination.put(body);

        Ok(())
    }
}

impl Decoder for BNetCodec {
    type Item = BNetPacket;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if self.header_length == None {
            if src.len() < 2 {
                return Ok(None);
            }

            let length_buf = src.split_to(HEADER_PREAMBLE_LENGTH);
            let header_length: u16 = BigEndian::read_u16(&length_buf);
            self.header_length = Some(header_length);
        }

        if self.header == None {
            let total_length = self.header_length.unwrap() as usize;
            if src.len() < total_length {
                return Ok(None);
            }

            let header_buf = src.split_to(total_length);
            let header = Header::decode(header_buf.freeze())?;

            let body_size = header
                .size
                .ok_or_else(|| ErrorKind::MissingData("Size".into()))?;
            self.body_size = Some(body_size);
            self.header = Some(header);
        }

        if let Some(body_size) = self.body_size {
            let body_size = body_size as usize;
            if src.len() >= body_size {
                let header = self.header.take().unwrap();
                let body = src.split_to(body_size);
                self.header_length.take();

                let packet = BNetPacket::new(header, body.freeze());
                return Ok(Some(packet));
            }
        }

        Ok(None)
    }
}
