use crate::payments::order::Order;

/// Message to get the orders in progress when controller robot changes
pub struct GetOrdersInProgress;
impl actix::Message for GetOrdersInProgress {
    /// The result of the message is a vector of order IDs
    type Result = Vec<Order>;
}
