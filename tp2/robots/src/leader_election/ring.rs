use actix::Context;
use std::future::Future;

use crate::messages::message::Message;
use crate::robots::robot::{ActiveRobots, Port, Robot};

pub trait RingLeaderElection {
    /// This function starts the ring election process. It is started
    /// by the robot that detects that the leader is dead.
    /// The robot sends an election message to the next robot in the ring.
    /// If the next robot is dead, the robot sends the election message to the next robot of
    /// the next robot, and so on.
    /// # Arguments
    /// * `arc_other_robots` - The other robots in the ring.
    /// * `port` - The port of the robot that detected the leader is dead.
    /// * `next_robot` - The next robot in the ring of the robot that detected the leader is dead.
    fn find_new_leader(
        robots: &mut ActiveRobots,
        port: Port,
        next_robot: Port,
    ) -> impl Future<Output = ()> + Send;
    // Receive an election message and handle it.
    // election_msg: Election message to be handled. Should be Election or Coordinator.
    fn receive_election<M: Message>(&mut self, election_msg: M, ctx: &mut Context<Robot>);
    /// Send the next election message to the next robot in the ring.
    /// The election message can be Election or Coordinator
    /// If the message is not sent, the next robot is considered dead and the next robot is updated.
    /// # Arguments
    /// * `election_message` - Election message to be sent in bytes
    /// * `ctx` - The context of the actor
    fn send_next_election(&mut self, election_message: Vec<u8>, ctx: &mut Context<Robot>);
    /// The current robot assumes leadership. This means:
    /// - The current robot is the leader.
    /// - The leader socket is set to None.
    /// - The current robot connects to all other robots.
    /// Each of these steps is done concurrently. We don't need to wait for one to finish to start the next one.
    fn assume_leadership(&mut self);
    /// Remove the dead robot from the election message.
    /// If the message is a Coordinator, nothing is done.
    fn remove_dead_robot_from_election(msg_bytes: Vec<u8>, dead_robot: &Port) -> Vec<u8>;
}
