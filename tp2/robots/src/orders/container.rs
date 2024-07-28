use tokio_util::bytes::{Buf, BytesMut};

#[derive(Debug, PartialEq, Clone)]
pub struct Container {
    pub size: u16,
    pub flavours: Vec<u8>,
}

impl Container {
    pub fn new(size: u16, flavours: Vec<u8>) -> Self {
        Container { size, flavours }
    }

    pub fn from_bytes(bytes: &mut BytesMut) -> Self {
        let size = bytes.get_u16();
        let flavours_amount: u8 = bytes.get_u8();
        let mut flavours = vec![];

        for _ in 0..flavours_amount {
            flavours.push(bytes.get_u8())
        }

        Container { size, flavours }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut container = Vec::new();

        container.extend(self.size.to_be_bytes());
        container.extend((self.flavours.len() as u8).to_be_bytes());

        for flavour in &self.flavours {
            container.push(*flavour);
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_from_bytes() {
        let exp_container = Container::new(500, [0, 1, 3].to_vec());
        let mut bytes = BytesMut::from(exp_container.to_bytes().as_slice());

        let container = Container::from_bytes(&mut bytes);

        assert_eq!(container, exp_container);
    }
}
