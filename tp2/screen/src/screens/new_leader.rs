use tokio_util::bytes::{Buf, BytesMut};

pub const NEW_LEADER_ID: u8 = 9;

pub struct NewLeader {
    pub port: u16,
}

impl NewLeader {
    pub fn from_bytes(bytes: &mut BytesMut) -> Self {
        let _ = bytes.get_u16(); // Temporarily unused.
        let port = bytes.get_u16();
        let _ = bytes.get_u16(); // leader_next field used by robots. Meant to be ignored by screen.
        NewLeader { port }
    }
}
