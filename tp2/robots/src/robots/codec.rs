use bytes::BytesMut;
use tokio_util::codec::Decoder;

use crate::robots::errors::RobotError;
const MIN_SIZE: usize = 3;

/// The RobotCodec struct is a custom codec for the robot messages.
pub struct RobotCodec;

impl RobotCodec {
    pub fn new() -> Self {
        RobotCodec {}
    }
}

impl Decoder for RobotCodec {
    type Item = BytesMut;
    type Error = RobotError;

    /// Decodes the bytes into a message.
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
