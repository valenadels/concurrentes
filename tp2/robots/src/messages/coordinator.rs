use crate::leader_election::ring::RingLeaderElection;
use crate::messages::message::{Message, COORDINATOR_ID};
use crate::robots::robot::{Port, Robot};
use actix::Context;
use bytes::{Buf, BytesMut};

const COORDINATOR_BYTES: u16 = 2;

pub struct Coordinator {
    pub max_id: Port,
}

impl Message for Coordinator {
    fn from_bytes(bytes: &mut BytesMut) -> Self {
        Coordinator {
            max_id: bytes.get_u16(),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut message = vec![COORDINATOR_ID];
        message.extend(COORDINATOR_BYTES.to_be_bytes());
        message.extend(self.max_id.to_be_bytes());
        message
    }

    fn be_handled_by(&mut self, robot: &mut Robot, ctx: &mut Context<Robot>) {
        if robot.id == self.max_id {
            robot.assume_leadership();
        } else {
            robot.send_next_election(self.to_bytes(), ctx);
        }
    }
}
