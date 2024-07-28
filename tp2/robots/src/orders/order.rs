use crate::orders::container::Container;
use bytes::{Buf, BytesMut};

#[derive(Debug, PartialEq, Clone)]
pub struct Order {
    pub id: u16,
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

    pub fn from_bytes(bytes: &mut BytesMut) -> Self {
        let id = bytes.get_u16();
        let containers_amount = bytes.get_u8();
        let mut containers = vec![];

        for _ in 0..containers_amount {
            containers.push(Container::from_bytes(bytes))
        }

        Order { id, containers }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_from_bytes() {
        let containers = vec![
            Container::new(500, [0, 1, 3].to_vec()),
            Container::new(1000, [0, 4, 1, 3].to_vec()),
            Container::new(250, [0].to_vec()),
        ];
        let exp_order = Order::new(5, containers);
        let mut bytes = BytesMut::from(exp_order.to_bytes().as_slice());

        let order = Order::from_bytes(&mut bytes);

        assert_eq!(order, exp_order);
    }
}
