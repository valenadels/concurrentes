use crate::payments::errors::PaymentError;
use bytes::BytesMut;
use tokio_util::codec::Decoder;
const MIN_SIZE: usize = 3;

/// Codec for the payments server.
pub struct PaymentsCodec;

impl Decoder for PaymentsCodec {
    type Item = BytesMut;
    type Error = PaymentError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let len = src.len();
        if len <= MIN_SIZE {
            return Ok(None);
        }

        let bytes = src.to_vec();
        let msg_len = ((bytes[1] as usize) << 8) | (bytes[2] as usize);

        match len >= msg_len + MIN_SIZE {
            true => Ok(Some(src.split_to(msg_len + MIN_SIZE))),
            false => Ok(None),
        }
    }
}
