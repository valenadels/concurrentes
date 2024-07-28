use crate::messages::message::{Message, FINISH_PAYMENT_ID};
use crate::robots::payment::Payment;
use crate::robots::robot::{Port, Robot};
use crate::utils::log::info;
use actix::Context;
use bytes::{Buf, BytesMut};

const FINISH_PAYMENT_BYTES: u16 = 4;

#[derive(Debug, PartialEq)]
pub struct FinishPayment {
    pub order_id: u16,
    pub port: Port,
}

impl Message for FinishPayment {
    fn from_bytes(bytes: &mut BytesMut) -> Self {
        FinishPayment {
            order_id: bytes.get_u16(),
            port: bytes.get_u16(),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut message = vec![FINISH_PAYMENT_ID];
        message.extend(FINISH_PAYMENT_BYTES.to_be_bytes());
        message.extend(self.order_id.to_be_bytes());
        message.extend(self.port.to_be_bytes());
        message
    }

    fn be_handled_by(&mut self, robot: &mut Robot, ctx: &mut Context<Robot>) {
        info(&format!(
            "Received message: FinishPayment for Order {}",
            self.order_id
        ));

        robot.handle_finish_payment(self, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_from_bytes() {
        let exp_finish_payment = FinishPayment {
            order_id: 8,
            port: 3000,
        };
        let mut bytes = BytesMut::from(&exp_finish_payment.to_bytes()[3..]); // First byte is read by the actor to identify the message type.

        let finish_payment = FinishPayment::from_bytes(&mut bytes);

        assert_eq!(finish_payment, exp_finish_payment);
    }
}
