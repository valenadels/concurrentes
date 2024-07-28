use crate::screens::container::Container;
use serde::Deserialize;

/// Struct to store the order information.
#[derive(Debug, Deserialize, Clone)]
pub struct Order {
    id: u16,
    containers: Vec<Container>,
}

impl Order {
    /// Serializes the Order into a byte vector.
    /// # Returns
    /// A byte vector with the serialized Order.
    /// # Errors
    /// Returns a ScreenError if the containers cannot be serialized.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut order = vec![];
        order.extend(self.id.to_be_bytes());
        order.extend((self.containers.len() as u8).to_be_bytes());

        for container in &self.containers {
            order.extend_from_slice(&container.to_bytes());
        }

        order
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_order_to_bytes() {
        let order = Order {
            id: 1,
            containers: vec![
                Container::new(250, vec!["vanilla".to_string()]),
                Container::new(500, vec!["chocolate".to_string()]),
            ],
        };

        let c1 = Container::new(250, vec!["vanilla".to_string()]).to_bytes();
        let c2 = Container::new(500, vec!["chocolate".to_string()]).to_bytes();

        let mut expected = Vec::new();
        expected.extend(1u16.to_be_bytes());
        expected.extend(2u8.to_be_bytes());
        expected.extend_from_slice(&c1);
        expected.extend_from_slice(&c2);

        let order_bytes = order.to_bytes();
        assert_eq!(order_bytes[0..2], expected[0..2]);
        assert_eq!(order_bytes[2..3], expected[2..3]);
    }
}
