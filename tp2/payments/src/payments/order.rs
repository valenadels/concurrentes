use crate::payments::container::Container;
use crate::payments::messages::OrderId;
use tokio_util::bytes::{Buf, BytesMut};

/// Order struct that represents an order with an id and containers.
#[derive(Debug, PartialEq, Clone)]
pub struct Order {
    pub id: OrderId,
    pub containers: Vec<Container>,
}

impl Order {
    pub fn new(id: u16, containers: Vec<Container>) -> Self {
        Order { id, containers }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut order = vec![];
        order.extend(self.id.to_be_bytes());
        order.extend((self.containers.len() as u8).to_be_bytes());

        for container in &self.containers {
            order.extend_from_slice(&container.to_bytes());
        }

        order
    }

    pub fn from_bytes(mut bytes: BytesMut) -> Self {
        let id = bytes.get_u16();
        let containers_amount = bytes.get_u8();
        let mut containers = vec![];

        for _ in 0..containers_amount {
            containers.push(Container::from_bytes(&mut bytes))
        }

        Order { id, containers }
    }
}
