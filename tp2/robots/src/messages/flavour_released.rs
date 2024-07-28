use crate::messages::message::{Message, FLAVOUR_RELEASED_ID};
use crate::orders::flavours::Flavour;
use crate::robots::order::OrderHandler;
use crate::robots::robot::Robot;
use actix::Context;
use bytes::{Buf, BytesMut};
use std::collections::HashMap;
//use crate::utils::log::info;

#[derive(PartialEq, Debug)]
pub struct FlavourReleased {
    // Flavor ID and remaining amount.
    pub flavours: HashMap<Flavour, u32>,
}

impl FlavourReleased {
    pub fn new(flavours: &HashMap<Flavour, u32>) -> Self {
        FlavourReleased {
            flavours: flavours.clone(),
        }
    }
}

impl Message for FlavourReleased {
    fn from_bytes(bytes: &mut BytesMut) -> Self {
        let mut flavours = HashMap::new();
        let mut i = 0;
        while i < 4 {
            let flavour = Flavour::from_u8(bytes.get_u8());
            let amount = bytes.get_u32();
            flavours.insert(flavour, amount);
            i += 1;
        }
        FlavourReleased { flavours }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut message = vec![FLAVOUR_RELEASED_ID];
        let mut flavours: Vec<u8> = vec![];
        for (flavour, amount) in &self.flavours {
            flavours.push(flavour.to_u8());
            flavours.extend(&amount.to_be_bytes());
        }

        message.extend((flavours.len() as u16).to_be_bytes());
        message.extend(flavours);
        message
    }

    fn be_handled_by(&mut self, robot: &mut Robot, ctx: &mut Context<Robot>) {
        // info(&format!(
        //     "Received message: FlavourReleased with amounts {:?}",
        //     self.flavours)
        // );

        robot.handle_flavour_released(self, ctx);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use bytes::{BufMut, BytesMut};

    #[test]
    fn from_bytes_creates_flavour_released_with_correct_flavours() {
        let mut bytes = BytesMut::new();
        bytes.put_u8(Flavour::Vanilla.to_u8());
        bytes.put_u32(10);
        bytes.put_u8(Flavour::Strawberry.to_u8());
        bytes.put_u32(20);
        bytes.put_u8(Flavour::Chocolate.to_u8());
        bytes.put_u32(10);
        bytes.put_u8(Flavour::Cookies.to_u8());
        bytes.put_u32(20);

        let flavour_released = FlavourReleased::from_bytes(&mut bytes);

        assert_eq!(flavour_released.flavours[&Flavour::Vanilla], 10);
        assert_eq!(flavour_released.flavours[&Flavour::Strawberry], 20);
        assert_eq!(flavour_released.flavours[&Flavour::Chocolate], 10);
        assert_eq!(flavour_released.flavours[&Flavour::Cookies], 20);
    }

    #[test]
    fn to_bytes_creates_correct_byte_representation() {
        let mut flavours = HashMap::new();
        flavours.insert(Flavour::Vanilla, 10);
        flavours.insert(Flavour::Strawberry, 20);
        let flavour_released = FlavourReleased { flavours };

        let mut bytes = flavour_released.to_bytes();

        let vanilla_bytes = vec![Flavour::Vanilla.to_u8(), 0, 0, 0, 10];
        let strawberry_bytes = vec![Flavour::Strawberry.to_u8(), 0, 0, 0, 20];
        assert_eq!(bytes.remove(0), FLAVOUR_RELEASED_ID);
        let a = bytes.remove(0);
        let b = bytes.remove(0);
        let size = u16::from_be_bytes([a, b]);
        assert_eq!(size, 10); // 2 flavours * 5 bytes (1 for flavour and 4 for u32)
        let vys = bytes.split_at(5);
        assert!(vys.0 == vanilla_bytes || vys.0 == strawberry_bytes);
        assert!(vys.1 == vanilla_bytes || vys.1 == strawberry_bytes);
    }

    #[test]
    fn to_bytes_handles_empty_flavours() {
        let flavours = HashMap::new();
        let flavour_released = FlavourReleased { flavours };

        let bytes = flavour_released.to_bytes();

        assert_eq!(bytes, vec![FLAVOUR_RELEASED_ID, 0, 0]); // 0 as u16 bytes for 0 flavours
    }
}
