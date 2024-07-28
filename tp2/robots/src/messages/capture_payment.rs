use crate::messages::message::CAPTURE_PAYMENT_ID;
use crate::orders::order::Order;

pub struct CapturePayment(Order);

impl CapturePayment {
    pub fn new(order: Order) -> Self {
        CapturePayment(order)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut message = vec![CAPTURE_PAYMENT_ID];
        let bytes = self.0.to_bytes();
        message.extend((bytes.len() as u16).to_be_bytes());
        message.extend(bytes);
        message
    }
}
