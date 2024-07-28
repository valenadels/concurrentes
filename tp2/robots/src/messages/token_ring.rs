use actix::Message;

pub struct StartTokenRing {}

impl Message for StartTokenRing {
    type Result = ();
}
