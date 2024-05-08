use crate::{
    cmd::Command,
    resp::{Resp, RespDeserializeError, Serialize},
};
use bytes::BytesMut;
use std::io;
use tokio_util::codec::{Decoder, Encoder};
use tracing::info;

pub struct Codec;

impl Decoder for Codec {
    type Item = Command;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        info!("Decoding buffer {:?}", String::from_utf8(buf.to_vec()));
        let resp = Resp::try_from(buf);
        match resp {
            Ok(resp) => {
                let cmd = Command::try_from(resp);
                match cmd {
                    Ok(cmd) => Ok(Some(cmd)),
                    Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
                }
            }
            Err(e) => match e {
                RespDeserializeError::NotComplete => Ok(None),
                _ => Err(io::Error::new(io::ErrorKind::Other, e)),
            },
        }
    }
}

impl Encoder<Resp> for Codec {
    type Error = io::Error;

    fn encode(&mut self, item: Resp, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.extend(item.serialize());
        Ok(())
    }
}
