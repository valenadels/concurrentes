use actix::Message;

/// Message to stop the gateway actor.
pub struct StopGateway;

impl Message for StopGateway {
    /// The result of the message is an empty tuple.
    type Result = ();
}
