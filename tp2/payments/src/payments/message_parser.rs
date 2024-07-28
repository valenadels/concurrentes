use crate::payments::errors::PaymentError;
use crate::payments::messages::Message;
use bytes::{Buf, BytesMut};

/// Parse a payment message from a byte array, received from a leader robot
/// # Arguments
/// * `bytes` - A vector of bytes representing a payment message
/// # Returns
/// A Result containing the parsed message or an error
/// # Errors
/// * PaymentError::ParseError - If the message is too short
/// * PaymentError::InvalidMessageId - If the message id is not recognized
pub fn parse_message(mut bytes: BytesMut) -> Result<Message, PaymentError> {
    Message::new_from_id(bytes.get_u8(), bytes).ok_or(PaymentError::InvalidMessageId)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payments::order::Order;

    #[test]
    fn parse_message_capture_payment() {
        let bytes = vec![1, 0x30, 0x39, 0];
        let result = parse_message(BytesMut::from(bytes.as_slice()));
        assert_eq!(
            result.unwrap(),
            Message::CapturePayment(Order::new(12345, vec![]))
        );
    }

    #[test]
    fn parse_message_finish_payment() {
        let bytes = vec![2, 0x30, 0x39, 0x00, 0x00];
        let result = parse_message(BytesMut::from(bytes.as_slice()));
        assert_eq!(result.unwrap(), Message::FinishPayment(12345, 0));
    }

    #[test]
    fn parse_message_cancel_payment() {
        let bytes = vec![3, 0x30, 0x39];
        let result = parse_message(BytesMut::from(bytes.as_slice()));
        assert_eq!(result.unwrap(), Message::CancelPayment(12345));
    }
}
