use actix::Context;
use bytes::BytesMut;

use crate::messages::message::Message;
use crate::orders::order::Order;
use crate::robots::payment::Payment;
use crate::robots::robot::Robot;
use crate::utils::log::info;

#[derive(Debug, PartialEq, Clone)]
pub struct PaymentAccepted {
    pub order: Order,
}

impl Message for PaymentAccepted {
    fn from_bytes(bytes: &mut BytesMut) -> Self {
        PaymentAccepted {
            order: Order::from_bytes(bytes),
        }
    }

    fn be_handled_by(&mut self, robot: &mut Robot, ctx: &mut Context<Robot>) {
        info(&format!(
            "Received message: PaymentAccepted for order {:?}",
            self.order.id
        ));

        robot.handle_payment_accepted(self, ctx);
    }
}
