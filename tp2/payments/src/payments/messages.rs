use crate::payments::messages::Message::{
    CancelPayment, CapturePayment, FinishPayment, NewLeader, OrderDone, PaymentAccepted,
    PaymentDeclined,
};
use crate::payments::order::Order;
use bytes::{Buf, BytesMut};

/// Message IDs
const CAPTURE_PAYMENT_ID: u8 = 1;
const FINISH_PAYMENT_ID: u8 = 2;
const CANCEL_PAYMENT_ID: u8 = 3;
const PAYMENT_ACCEPTED_ID: u8 = 4;
const PAYMENT_DECLINED_ID: u8 = 5;
const NEW_LEADER_ID: u8 = 9;
const ORDER_DONE_ID: u8 = 11;

/// Order ID type
pub type OrderId = u16;

/// Payment messages
#[derive(PartialEq, Debug)]
pub enum Message {
    /// Capture payment message.
    /// It is sent by the robot.
    CapturePayment(Order),
    /// Finish payment message
    /// It contains order id and robot port
    /// It is sent by the robot.
    FinishPayment(OrderId, u16),
    /// Payment accepted message
    /// It is sent by the payment gateway.
    PaymentAccepted(Order),
    /// Payment declined message
    /// It is sent by the payment gateway.
    PaymentDeclined(Order),
    /// Cancel payment message
    /// It is sent by the robot.
    CancelPayment(OrderId),
    /// New leader message
    /// Has the new leader port and its next port (not used here)
    NewLeader(u16, u16),
    OrderDone(OrderId),
}

impl Message {
    /// Create a new message from the message ID and order ID.
    /// Returns None if the message ID is invalid.
    /// # Arguments
    /// * `msg_id` - Message ID (1 byte)
    /// * `order_id` - Order ID (2 bytes)
    pub fn new_from_id(msg_id: u8, mut bytes: BytesMut) -> Option<Message> {
        bytes.get_u16(); // Skip len
        match msg_id {
            CAPTURE_PAYMENT_ID => Some(CapturePayment(Order::from_bytes(bytes))),
            FINISH_PAYMENT_ID => Some(FinishPayment(bytes.get_u16(), bytes.get_u16())),
            CANCEL_PAYMENT_ID => Some(CancelPayment(bytes.get_u16())),
            PAYMENT_ACCEPTED_ID => Some(PaymentAccepted(Order::from_bytes(bytes))),
            PAYMENT_DECLINED_ID => Some(PaymentDeclined(Order::from_bytes(bytes))),
            NEW_LEADER_ID => Some(NewLeader(bytes.get_u16(), bytes.get_u16())),
            _ => None,
        }
    }

    /// Convert the message to a byte sequence
    /// Messages that are not sent by Payments are not considered here.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        match self {
            PaymentAccepted(order) | PaymentDeclined(order) => {
                bytes.push(match self {
                    PaymentAccepted(_) => PAYMENT_ACCEPTED_ID,
                    _ => PAYMENT_DECLINED_ID,
                });
                let order_bytes = order.to_bytes();
                bytes.extend((order_bytes.len() as u16).to_be_bytes());
                bytes.extend_from_slice(&order_bytes)
            }
            OrderDone(order_id) => {
                bytes.push(ORDER_DONE_ID);
                let bytes_id = order_id.to_be_bytes();
                bytes.extend((bytes_id.len() as u16).to_be_bytes());
                bytes.extend(bytes_id);
            }
            _ => {}
        }
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_from_id_creates_correct_message() {
        let order_id: u16 = 123;
        let port: u16 = 3000;
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&order_id.to_be_bytes());
        bytes.extend_from_slice(&port.to_be_bytes());
        let message = Message::new_from_id(FINISH_PAYMENT_ID, BytesMut::from(bytes.as_slice()));
        assert_eq!(message, Some(FinishPayment(order_id, port)));
    }

    #[test]
    fn new_from_id_returns_none_for_invalid_id() {
        let order_id: u16 = 123;
        let message = Message::new_from_id(0, BytesMut::from(order_id.to_be_bytes().as_slice()));
        assert_eq!(message, None);
    }

    #[test]
    fn new_to_bytes_creates_correct_byte_sequence() {
        let order_id = 12345;
        let order = Order::new(order_id, vec![]);
        let message = CapturePayment(order);
        let bytes = message.to_bytes();
        let expected = vec![1, 0x30, 0x39, 0];
        assert_eq!(bytes, expected);
    }
}
