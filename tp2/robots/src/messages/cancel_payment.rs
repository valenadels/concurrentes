use crate::messages::message::{Message, CANCEL_PAYMENT_ID};
use crate::robots::payment::Payment;
use crate::robots::robot::{Port, Robot};
use crate::utils::log::info;
use actix::Context;
use bytes::{Buf, BytesMut};

const CANCEL_PAYMENT_BYTES: u16 = 4;

#[derive(Debug, PartialEq)]
pub struct CancelPayment {
    pub order_id: u16,
    pub port: Port,
}

impl Message for CancelPayment {
    fn from_bytes(bytes: &mut BytesMut) -> Self {
        CancelPayment {
            order_id: bytes.get_u16(),
            port: bytes.get_u16(),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut message = vec![CANCEL_PAYMENT_ID];
        message.extend(CANCEL_PAYMENT_BYTES.to_be_bytes());
        message.extend(self.order_id.to_be_bytes());
        message.extend(self.port.to_be_bytes());
        message
    }

    fn be_handled_by(&mut self, robot: &mut Robot, ctx: &mut Context<Robot>) {
        info(&format!(
            "Received message: CancelPayment for Order {}",
            self.order_id
        ));

        robot.handle_cancel_payment(self, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_from_bytes() {
        let exp_cancel_payment = CancelPayment {
            order_id: 8,
            port: 3000,
        };
        // First byte is read by the actor to identify the message type.
        // The next two bytes are the length of the message.
        let mut bytes = BytesMut::from(&exp_cancel_payment.to_bytes()[3..]);
        let cancel_payment = CancelPayment::from_bytes(&mut bytes);

        assert_eq!(cancel_payment, exp_cancel_payment);
    }
}
