use super::order::Order;

const NEW_ORDER_ID: u8 = 0;

#[derive(Clone)]
pub struct NewOrder {
    pub order: Order,
}

impl NewOrder {
    pub fn new(order: Order) -> NewOrder {
        NewOrder { order }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut message = vec![NEW_ORDER_ID];
        let bytes = self.order.to_bytes();
        message.extend((bytes.len() as u16).to_be_bytes());
        message.extend(bytes);
        message
    }
}
