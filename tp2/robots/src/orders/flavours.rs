use crate::orders::flavours::Flavour::{Chocolate, Cookies, Strawberry, Vanilla};
use crate::utils::log::warn;
use std::collections::HashMap;

const INITIAL_AMOUNT: u32 = 10000;
#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum Flavour {
    Vanilla,
    Strawberry,
    Chocolate,
    Cookies,
}

const VANILLA_ID: u8 = 0;

const STRAWBERRY_ID: u8 = 2;

const CHOCOLATE_ID: u8 = 1;

const COOKIES_ID: u8 = 3;

const FLAVOURS: usize = 4;

impl Flavour {
    pub fn from_u8(value: u8) -> Flavour {
        match value {
            VANILLA_ID => Vanilla,
            STRAWBERRY_ID => Strawberry,
            CHOCOLATE_ID => Chocolate,
            COOKIES_ID => Cookies,
            _ => {
                warn(&format!(
                    "Invalid flavour value: {}. Returning default",
                    value
                ));
                Vanilla
            }
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Vanilla => VANILLA_ID,
            Strawberry => STRAWBERRY_ID,
            Chocolate => CHOCOLATE_ID,
            Cookies => COOKIES_ID,
        }
    }

    pub fn initial_flavours() -> HashMap<Flavour, u32> {
        let mut flavours = HashMap::with_capacity(FLAVOURS);
        flavours.insert(Vanilla, INITIAL_AMOUNT);
        flavours.insert(Strawberry, INITIAL_AMOUNT);
        flavours.insert(Chocolate, INITIAL_AMOUNT);
        flavours.insert(Cookies, INITIAL_AMOUNT);
        flavours
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flavour_from_u8_returns_correct_flavour() {
        assert_eq!(Flavour::from_u8(VANILLA_ID), Vanilla);
        assert_eq!(Flavour::from_u8(STRAWBERRY_ID), Strawberry);
        assert_eq!(Flavour::from_u8(CHOCOLATE_ID), Chocolate);
        assert_eq!(Flavour::from_u8(COOKIES_ID), Cookies);
    }

    #[test]
    fn flavour_from_u8_returns_vanilla_for_invalid_value() {
        assert_eq!(Flavour::from_u8(100), Vanilla);
    }

    #[test]
    fn flavour_to_u8_returns_correct_id() {
        assert_eq!(Vanilla.to_u8(), VANILLA_ID);
        assert_eq!(Strawberry.to_u8(), STRAWBERRY_ID);
        assert_eq!(Chocolate.to_u8(), CHOCOLATE_ID);
        assert_eq!(Cookies.to_u8(), COOKIES_ID);
    }

    #[test]
    fn initial_flavours_returns_correct_amounts() {
        let flavours = Flavour::initial_flavours();
        assert_eq!(flavours.get(&Vanilla), Some(&INITIAL_AMOUNT));
        assert_eq!(flavours.get(&Strawberry), Some(&INITIAL_AMOUNT));
        assert_eq!(flavours.get(&Chocolate), Some(&INITIAL_AMOUNT));
        assert_eq!(flavours.get(&Cookies), Some(&INITIAL_AMOUNT));
    }
}
