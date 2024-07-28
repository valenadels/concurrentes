use actix::Context;
use bytes::BytesMut;

use crate::messages::message::Message;
use crate::robots::robot::Robot;
use crate::utils::log::info;

use super::message::PING_ID;

const PING_BYTES: u16 = 0;

/// Message sent by the leader to check if the robot is still alive.
#[derive(Debug, PartialEq)]
pub struct Ping;

impl Message for Ping {
    fn to_bytes(&self) -> Vec<u8> {
        let id: u8 = PING_ID;
        let mut bytes = vec![id];
        bytes.extend_from_slice(&PING_BYTES.to_be_bytes());
        bytes
    }

    fn from_bytes(_bytes: &mut BytesMut) -> Self {
        Ping
    }

    fn be_handled_by(&mut self, _robot: &mut Robot, _ctx: &mut Context<Robot>) {
        info("Received message: Ping");
    }
}
