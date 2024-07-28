use crate::messages::cancel_payment::CancelPayment;
use crate::messages::coordinator::Coordinator;
use crate::messages::election::Election;
use crate::messages::finish_payment::FinishPayment;
use crate::messages::flavour_released::FlavourReleased;
use crate::messages::new_leader::NewLeader;
use crate::messages::new_order::NewOrder;
use crate::messages::new_pending_order::NewPendingOrder;
use crate::messages::order_done::OrderDone;
use crate::messages::payment_accepted::PaymentAccepted;
use crate::messages::payment_declined::PaymentDeclined;
use crate::robots::errors::RobotError;
use crate::robots::robot::Robot;
use actix::Context;
use bytes::{Buf, BytesMut};

use super::ping::Ping;

pub const NEW_ORDER_ID: u8 = 0;
pub const CAPTURE_PAYMENT_ID: u8 = 1;
pub const FINISH_PAYMENT_ID: u8 = 2;
pub const CANCEL_PAYMENT_ID: u8 = 3;
pub const PAYMENT_ACCEPTED_ID: u8 = 4;
pub const PAYMENT_DECLINED_ID: u8 = 5;
pub const FLAVOUR_RELEASED_ID: u8 = 6;
pub const ELECTION_ID: u8 = 7;
pub const COORDINATOR_ID: u8 = 8;
pub const NEW_LEADER_ID: u8 = 9;
pub const NEW_PENDING_ORDER_ID: u8 = 10;
pub const PING_ID: u8 = 12;

pub trait Message {
    fn from_bytes(bytes: &mut BytesMut) -> Self
    where
        Self: Sized;
    fn to_bytes(&self) -> Vec<u8> {
        vec![]
    }
    fn be_handled_by(&mut self, robot: &mut Robot, ctx: &mut Context<Robot>);
}

pub fn from_bytes(bytes: &mut BytesMut) -> Result<Box<dyn Message>, RobotError> {
    let id_byte = bytes.get_u8();
    let len = bytes.get_u16();
    match id_byte {
        0 => Ok(Box::new(NewOrder::from_bytes(bytes))),
        2 => Ok(Box::new(FinishPayment::from_bytes(bytes))),
        3 => Ok(Box::new(CancelPayment::from_bytes(bytes))),
        4 => Ok(Box::new(PaymentAccepted::from_bytes(bytes))),
        5 => Ok(Box::new(PaymentDeclined::from_bytes(bytes))),
        6 => Ok(Box::new(FlavourReleased::from_bytes(bytes))),
        7 => {
            let mut len_plus_bytes = BytesMut::new();
            len_plus_bytes.extend_from_slice(&len.to_be_bytes());
            len_plus_bytes.extend_from_slice(&bytes[..len as usize]);
            Ok(Box::new(Election::from_bytes(&mut len_plus_bytes)))
        }
        8 => Ok(Box::new(Coordinator::from_bytes(bytes))),
        9 => Ok(Box::new(NewLeader::from_bytes(bytes))),
        10 => Ok(Box::new(NewPendingOrder::from_bytes(bytes))),
        11 => Ok(Box::new(OrderDone::from_bytes(bytes))),
        12 => Ok(Box::new(Ping::from_bytes(bytes))),
        _ => Err(RobotError::ParseError(format!(
            "Unknown message received with id {}.",
            bytes[0]
        ))),
    }
}
