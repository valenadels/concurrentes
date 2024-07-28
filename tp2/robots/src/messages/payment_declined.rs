use crate::messages::message::{Message, PAYMENT_DECLINED_ID};
use crate::orders::order::Order;
use crate::robots::payment::Payment;
use crate::robots::robot::Robot;
use crate::utils::log::info;
use actix::Context;
use bytes::BytesMut;

#[derive(Debug, PartialEq)]
pub struct PaymentDeclined {
    pub order: Order,
}

impl Message for PaymentDeclined {
    fn from_bytes(bytes: &mut BytesMut) -> Self {
        PaymentDeclined {
            order: Order::from_bytes(bytes),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut message = vec![PAYMENT_DECLINED_ID];
        let bytes = self.order.to_bytes();
        message.extend((bytes.len() as u16).to_be_bytes());
        message.extend(bytes);
        message
    }

    fn be_handled_by(&mut self, robot: &mut Robot, ctx: &mut Context<Robot>) {
        info(&format!(
            "Received message: PaymentDeclined for order {:?}",
            self.order.id
        ));

        robot.handle_payment_declined(self, ctx);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_from_bytes() {
        /*let exp_payment_declined = PaymentDeclined {order: }
        let mut bytes = BytesMut::from(&exp_payment_declined.to_bytes()[1..]); // First byte is read by the actor to identify the message type.

        let payment_declined = PaymentDeclined::from_bytes(&mut bytes);

        assert_eq!(payment_declined, exp_payment_declined);*/
    }
}
