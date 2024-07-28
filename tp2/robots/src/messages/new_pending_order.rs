use crate::messages::message::{Message, NEW_PENDING_ORDER_ID};
use crate::orders::order::Order;
use crate::robots::order::OrderHandler;
use crate::robots::robot::{Port, Robot};
use crate::utils::log::info;
use actix::Context;
use bytes::{Buf, BytesMut};

#[derive(Debug, PartialEq)]
pub struct NewPendingOrder {
    pub port: Port,
    pub order: Order,
}

impl NewPendingOrder {
    pub fn new(port: Port, order: Order) -> Self {
        NewPendingOrder { port, order }
    }
}

impl Message for NewPendingOrder {
    fn from_bytes(bytes: &mut BytesMut) -> Self {
        let port = bytes.get_u16();
        let order = Order::from_bytes(bytes);
        NewPendingOrder { port, order }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut message = vec![NEW_PENDING_ORDER_ID];
        let bytes = self.order.to_bytes();
        let size = 2 + (bytes.len() as u16);
        message.extend(size.to_be_bytes());
        message.extend(self.port.to_be_bytes());
        message.extend(bytes);
        message
    }

    fn be_handled_by(&mut self, robot: &mut Robot, ctx: &mut Context<Robot>) {
        info(&format!(
            "Received message: NewPendingOrder {:?} for port {:?}",
            self.order.id, self.port
        ));

        robot.handle_new_pending_order(self, ctx)
    }
}
