use crate::messages::message::{Message, NEW_ORDER_ID};
use crate::orders::order::Order;
use crate::robots::order::OrderHandler;
use crate::robots::robot::Robot;
use crate::utils::log::info;
use actix::Context;
use bytes::BytesMut;

#[derive(Debug, PartialEq)]
pub struct NewOrder {
    pub order: Order,
}

impl NewOrder {
    pub fn new(order: Order) -> Self {
        NewOrder { order }
    }
}

impl Message for NewOrder {
    fn from_bytes(bytes: &mut BytesMut) -> Self {
        NewOrder {
            order: Order::from_bytes(bytes),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut message = vec![NEW_ORDER_ID];
        let bytes = self.order.to_bytes();
        message.extend((bytes.len() as u16).to_be_bytes());
        message.extend(bytes);
        message
    }

    fn be_handled_by(&mut self, robot: &mut Robot, ctx: &mut Context<Robot>) {
        info(&format!("Received message: NewOrder {:?}", self.order.id));
        robot.handle_new_order(self, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orders::container::Container;
    use bytes::BytesMut;

    #[test]
    fn test_from_bytes() {
        let containers = vec![
            Container::new(500, [0, 1, 3].to_vec()),
            Container::new(1000, [0, 4, 1, 3].to_vec()),
            Container::new(250, [0].to_vec()),
        ];
        let exp_new_order = NewOrder {
            order: Order::new(5, containers),
        };
        // First byte is read by the actor to identify the message type.
        // The next two bytes are the length of the message.
        let mut bytes = BytesMut::from(&exp_new_order.to_bytes()[3..]);
        let new_order = NewOrder::from_bytes(&mut bytes);

        assert_eq!(new_order, exp_new_order);
    }
}
