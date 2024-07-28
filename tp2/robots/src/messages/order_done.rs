use actix::Context;
use bytes::{Buf, BytesMut};

use crate::messages::message::Message;
use crate::robots::robot::Robot;
use crate::utils::log::info;

/// Message sent by Payments when finish or cancel payment is done for an order.
#[derive(Debug, PartialEq)]
pub struct OrderDone {
    /// Order ID.
    id: u16,
}

impl Message for OrderDone {
    fn from_bytes(bytes: &mut BytesMut) -> Self {
        OrderDone {
            id: bytes.get_u16(),
        }
    }

    fn be_handled_by(&mut self, _robot: &mut Robot, _ctx: &mut Context<Robot>) {
        info(&format!(
            "Received message: OrderDone for order {:?}",
            self.id
        ));
    }
}
