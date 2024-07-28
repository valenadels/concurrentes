use crate::leader_election::ring::RingLeaderElection;
use crate::messages::coordinator::Coordinator;
use crate::messages::message::{Message, ELECTION_ID};
use crate::robots::robot::{Port, Robot};
use actix::Context;
use bytes::{Buf, BytesMut};

pub struct Election {
    pub ids: Vec<Port>,
}

impl Election {
    pub fn add_id(&mut self, id: Port) {
        self.ids.push(id);
    }

    pub fn max_id(&self) -> Port {
        if let Some(max) = self.ids.iter().max() {
            *max
        } else {
            self.ids[0]
        }
    }

    fn contains(&self, id: &Port) -> bool {
        self.ids.contains(id)
    }
}

impl Message for Election {
    /// Creates an Election from a BytesMut.
    /// Bytes should be in the format:
    /// [amount of ids (u16), id1 (u16), id2 (u16), ...]
    fn from_bytes(bytes: &mut BytesMut) -> Self {
        let ids_amount = bytes.get_u16() / 2;
        let mut ids: Vec<Port> = vec![];
        for _ in 0..ids_amount {
            ids.push(bytes.get_u16() as Port);
        }
        Election { ids }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut message = vec![ELECTION_ID];
        message.extend((self.ids.len() as u16 * 2u16).to_be_bytes());
        for id in &self.ids {
            message.extend(id.to_be_bytes());
        }
        message
    }

    fn be_handled_by(&mut self, robot: &mut Robot, ctx: &mut Context<Robot>) {
        if self.contains(&robot.id) {
            robot.send_next_election(
                Coordinator {
                    max_id: self.max_id(),
                }
                .to_bytes(),
                ctx,
            );
        } else {
            self.add_id(robot.id);
            robot.send_next_election(self.to_bytes(), ctx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, BytesMut};

    #[test]
    fn from_bytes_creates_election_with_correct_ids() {
        let mut bytes = BytesMut::new();
        bytes.put_u16(4);
        bytes.put_u16(8000);
        bytes.put_u16(8001);

        let election = Election::from_bytes(&mut bytes);

        assert_eq!(election.ids, vec![8000, 8001]);
    }

    #[test]
    fn to_bytes_creates_correct_byte_representation() {
        let election = Election {
            ids: vec![8000, 8001],
        };

        let bytes = election.to_bytes();
        assert_eq!(bytes, vec![ELECTION_ID, 0, 4, 31, 64, 31, 65]);
    }

    #[test]
    fn from_bytes_handles_empty_bytes() {
        let mut bytes = BytesMut::new();
        bytes.put_u16(0);

        let election = Election::from_bytes(&mut bytes);
        assert_eq!(election.ids, vec![]);
    }

    #[test]
    fn to_bytes_handles_empty_ids() {
        let election = Election { ids: vec![] };

        let bytes = election.to_bytes();
        assert_eq!(bytes, vec![ELECTION_ID, 0, 0]);
    }
}
