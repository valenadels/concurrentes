use crate::messages::message::{Message, NEW_LEADER_ID};
use crate::robots::election::Election;
use crate::robots::robot::{Port, Robot};
use crate::utils::log::info;
use actix::Context;
use bytes::{Buf, BytesMut};

const NEW_LEADER_BYTES: u16 = 4;

pub struct NewLeader {
    pub id: Port,
    pub leader_next: Port,
}

impl Message for NewLeader {
    fn from_bytes(bytes: &mut BytesMut) -> Self {
        NewLeader {
            id: bytes.get_u16(),
            leader_next: bytes.get_u16(),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut message = vec![NEW_LEADER_ID];
        message.extend(NEW_LEADER_BYTES.to_be_bytes());
        message.extend(self.id.to_be_bytes());
        message.extend(self.leader_next.to_be_bytes());
        message
    }

    fn be_handled_by(&mut self, robot: &mut Robot, ctx: &mut Context<Robot>) {
        info(&format!(
            "Received message: NewLeader with port {:?} and next {:?}",
            self.id, self.leader_next
        ));

        robot.handle_new_leader(self, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, BytesMut};

    #[test]
    fn new_leader_from_bytes_creates_correct_message() {
        let mut bytes = BytesMut::new();
        bytes.put_u16(3000);
        bytes.put_u16(3001);

        let new_leader = NewLeader::from_bytes(&mut bytes);

        assert_eq!(new_leader.id, 3000);
        assert_eq!(new_leader.leader_next, 3001);
    }

    #[test]
    fn new_leader_to_bytes_creates_correct_byte_representation() {
        let id = 3000u16;
        let leader_next = 3001u16;
        let new_leader = NewLeader { id, leader_next };
        let expected_bytes = Vec::from([NEW_LEADER_ID, 0, 4, 11, 184, 11, 185]);
        let bytes = new_leader.to_bytes();

        assert_eq!(bytes, expected_bytes);
    }
}
