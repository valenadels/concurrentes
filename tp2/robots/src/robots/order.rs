use std::collections::HashMap;
use std::sync::Arc;

use actix::fut::wrap_future;
use actix::{Context, ContextFutureSpawner};
use tokio::io::{split, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::time::sleep;

use crate::leader_election::ring::RingLeaderElection;
use crate::messages::cancel_payment::CancelPayment;
use crate::messages::capture_payment::CapturePayment;
use crate::messages::finish_payment::FinishPayment;
use crate::messages::flavour_released::FlavourReleased;
use crate::messages::message::Message;
use crate::messages::new_order::NewOrder;
use crate::messages::new_pending_order::NewPendingOrder;
use crate::orders::container::Container;
use crate::orders::flavours::Flavour;
use crate::orders::order::Order;
use crate::robots::robot::{ActiveRobots, LeaderStream, PaymentStream, Robot};
use crate::utils::log::{error, info, warn};
use crate::utils::util::addr;

use super::robot::{FlavoursStock, OrdersByRobot, Port};

pub trait OrderHandler {
    ///Pushes an order to the pending orders of the robot.
    fn handle_new_pending_order(&self, msg: &NewPendingOrder, ctx: &mut Context<Robot>);
    /// When a new order comes, depending on the robot's role,
    /// it will either send the order to payments or prepare the order.
    fn handle_new_order(&self, msg: &NewOrder, ctx: &mut Context<Robot>);
    /// Releases the flavours according to token ring algorithm.
    fn handle_flavour_released(&mut self, msg: &FlavourReleased, ctx: &mut Context<Robot>);
}

impl OrderHandler for Robot {
    fn handle_new_pending_order(&self, msg: &NewPendingOrder, ctx: &mut Context<Robot>) {
        let arc_pending_orders = self.pending_orders.clone();
        let port = msg.port;
        let order = msg.order.clone();

        wrap_future::<_, Self>(async move {
            let mut pending_orders = arc_pending_orders.write().await;
            remove_order_if_exists(&mut pending_orders, &order);
            match pending_orders.get_mut(&port) {
                Some(robot_orders) => {
                    robot_orders.push(order);
                }
                None => {
                    pending_orders.insert(port, vec![order]);
                }
            }
        })
        .spawn(ctx);
    }

    fn handle_new_order(&self, msg: &NewOrder, ctx: &mut Context<Robot>) {
        let robot_id = self.id;
        let is_leader = self.is_leader.clone();
        let arc_payments = self.payments.clone();
        let order = msg.order.clone();
        let pending_orders = self.pending_orders.clone();

        wrap_future::<_, Self>(async move {
            let is_leader = is_leader.read().await;
            if *is_leader {
                Self::capture_payment(arc_payments, order.clone()).await;
            }

            let mut orders_from_robot = pending_orders.write().await;
            match orders_from_robot.get_mut(&robot_id) {
                Some(orders) => {
                    orders.push(order);
                }
                None => {
                    orders_from_robot.insert(robot_id, vec![order]);
                }
            }
        })
        .spawn(ctx);
    }

    fn handle_flavour_released(&mut self, msg: &FlavourReleased, ctx: &mut Context<Robot>) {
        let robot_id = self.id;
        let arc_is_leader = self.is_leader.clone();
        let arc_flavours = self.flavours.clone();
        let arc_pending_orders = self.pending_orders.clone();
        let arc_next_robot = self.next_robot.clone();
        let arc_robots = self.robots.clone();
        let arc_leader = self.leader.clone();
        let mut flavours_updated = msg.flavours.clone();
        let mut order_can_be_prepared = true;
        let arc_leader_election_started = self.leader_election_started.clone();
        let tx = self.flavours_ping_sender.clone();

        wrap_future::<_, Self>(async move {
            let mut flavours = arc_flavours.write().await;
            let is_leader = arc_is_leader.read().await;

            if *is_leader {
                let _ = tx.try_send(flavours_updated.clone());
                *flavours = flavours_updated;
                return;
            }
            let mut pending_orders = arc_pending_orders.write().await;
            let mut leader = arc_leader.write().await;
            let mut robots = arc_robots.write().await;
            let next_robot_port = arc_next_robot.write().await;
            let mut leader_election_started = arc_leader_election_started.write().await;

            if let Some(orders) = pending_orders.get_mut(&robot_id) {
                if !orders.is_empty() {
                    let first = orders.remove(0);

                    retrieve_updated_flavours(
                        &first.containers,
                        &mut flavours_updated,
                        &mut order_can_be_prepared,
                    );
                    let mut leader_is_up = false;
                    if order_can_be_prepared && !leader_is_up {
                        leader_is_up =
                            Self::prepare_order(&flavours_updated, &first, &mut leader, robot_id)
                                .await;
                    } else {
                        leader_is_up = Self::cancel_payment(robot_id, &mut leader, first).await;
                    }

                    if !leader_is_up && !*leader_election_started {
                        Robot::find_new_leader(&mut robots, robot_id, *next_robot_port).await;
                        *leader_election_started = true;
                    }
                }
            }
            let mut next_robot_port = arc_next_robot.write().await;
            let mut robots = arc_robots.write().await;
            let mut leader_election_started = arc_leader_election_started.write().await;

            Self::send_flavour_released(
                robot_id,
                &mut flavours_updated,
                &mut leader,
                &mut robots,
                &mut next_robot_port,
                &mut leader_election_started,
            )
            .await;
        })
        .spawn(ctx);
    }
}

/// Gets the updated flavours after preparing an order.
/// If there is not enough stock, the order can't be prepared.
/// # Arguments
/// * `containers` - The containers of the order.
/// * `updated_flavours` - The flavours stock.
/// * `order_can_be_prepared` - A flag to indicate if the order can be prepared.
pub fn retrieve_updated_flavours(
    containers: &Vec<Container>,
    updated_flavours: &mut FlavoursStock,
    order_can_be_prepared: &mut bool,
) {
    for c in containers {
        let amount_by_flavour = c.size as u32 / c.flavours.len() as u32;
        let container_flavours = c.flavours.clone();
        for f in container_flavours {
            let curr_flavour = Flavour::from_u8(f);
            match updated_flavours.get(&curr_flavour) {
                Some(stock) => {
                    if stock < &amount_by_flavour {
                        *order_can_be_prepared = false;
                        break;
                    }
                    updated_flavours.insert(curr_flavour, stock - amount_by_flavour);
                }
                None => {
                    error(&format!("Flavour {:?} not found in stock.", curr_flavour));
                    *order_can_be_prepared = false;
                    break;
                }
            }
        }
        if !*order_can_be_prepared {
            break;
        }
    }
}

impl Robot {
    /// Sends the flavours to the next robot in the token ring.
    pub async fn release_to_next_robot(
        bytes: &[u8],
        next_robot_port: &mut Port,
        robots: &mut ActiveRobots,
        id: Port,
    ) {
        loop {
            if let Some(next_robot_stream_op) = robots.get_mut(next_robot_port) {
                match next_robot_stream_op {
                    Some(next_robot_stream) => {
                        if next_robot_stream.write_all(bytes).await.is_err() {
                            // Found stream but lost connection (dead robot).
                            // TODO: Revisar esto
                            robots.remove(next_robot_port);
                            // pending_orders.remove((next_robot_port)); // TODO: revisar esto
                            *next_robot_port =
                                Robot::find_next_robot(robots, *next_robot_port, id).await;
                        } else {
                            break;
                        }
                    }
                    None => {
                        // Found next_robot but no stream has been created yet (first round of flavour releasing).
                        match TcpStream::connect(addr(*next_robot_port)).await {
                            Ok(stream) => {
                                let mut write_half = split(stream).1;

                                match write_half.write_all(bytes).await {
                                    Ok(_) => {
                                        let _ = write_half.flush().await;
                                        robots.insert(*next_robot_port, Some(write_half));
                                        break;
                                    }
                                    Err(_) => {
                                        warn(&format!(
                                            "Lost connection to next {}",
                                            next_robot_port
                                        ));
                                        *next_robot_port =
                                            Robot::find_next_robot(robots, *next_robot_port, id)
                                                .await;
                                    }
                                }
                            }
                            Err(_) => {
                                // Failed to connect to next robot
                                error(&format!("Failed to connect to {}", next_robot_port));
                                *next_robot_port =
                                    Robot::find_next_robot(robots, *next_robot_port, id).await;
                            }
                        }
                    }
                }
            } else {
                // Could not find next_robot to send flavours (must have been deleted by another thread, and it did not update next_robot_port).
                error("Unexpected error. Could not find next_robot at release_to_next_robot.");
            }
        }
    }

    /// Sends the flavours to the leader.
    pub async fn release_to_leader(
        leader: &mut LeaderStream,
        leader_election_started: &mut bool,
        bytes: &[u8],
        port: Port,
        next_robot: &Port,
        robots: &mut ActiveRobots,
    ) {
        if let Some(ref mut stream) = leader {
            if stream.write_all(bytes).await.is_err() {
                if !*leader_election_started {
                    Robot::find_new_leader(robots, port, *next_robot).await;
                    *leader_election_started = true;
                }
            } else {
                let _ = stream.flush().await;
            }
        }
    }

    /// Sends Capture payment to payments. This function should be used by the leader
    async fn capture_payment(arc_payments: Arc<RwLock<PaymentStream>>, order: Order) {
        let mut payments_op = arc_payments.write().await;
        if let Some(ref mut payments) = *payments_op {
            let msg = CapturePayment::new(order).to_bytes();
            if payments.write(&msg).await.is_ok() {
                //Payments app is always up
                info(&format!("Sent Capture payment msg: {:?}", msg));
            }
        }
    }

    /// Simulates the preparation of an order with a sleep depending on the size of the containers.
    /// If the order is prepared, sends FinishPayment to the leader.
    /// Returns true if the message was sent successfully, false otherwise.
    async fn prepare_order(
        flavours_updated: &HashMap<Flavour, u32>,
        order: &Order,
        leader: &mut LeaderStream,
        robot_id: Port,
    ) -> bool {
        for c in &order.containers {
            sleep(core::time::Duration::from_millis(c.size as u64)).await;
        }

        let finish_payment = FinishPayment {
            order_id: order.id,
            port: robot_id,
        };

        if let Some(ref mut leader) = *leader {
            if leader.write(&finish_payment.to_bytes()).await.is_err() {
                error("Failed to send FinishPayment to leader.");
                return false;
            }
        }

        info(&format!("Order {} prepared", order.id));
        info(&format!("Flavours updated: {:?}", flavours_updated));
        true
    }

    /// Sends cancel payment to leader.
    /// Returns true if the message was sent successfully, false otherwise.
    async fn cancel_payment(robot_id: Port, leader: &mut LeaderStream, first: Order) -> bool {
        warn(&format!(
            "Order {} could not be prepared. Not enough stock.",
            first.id
        ));
        let cancel_payment = CancelPayment {
            order_id: first.id,
            port: robot_id,
        };
        if let Some(ref mut leader) = *leader {
            if leader.write(&cancel_payment.to_bytes()).await.is_err() {
                error("Failed to send CancelPayment to leader.");
                return false;
            }
        }
        true
    }

    /// Creates a FlavourReleased with the updated flavours and sends them to leader and next robot
    async fn send_flavour_released(
        robot_id: Port,
        flavours_updated: &mut HashMap<Flavour, u32>,
        leader: &mut LeaderStream,
        robots: &mut ActiveRobots,
        next_robot_port: &mut Port,
        leader_election_started: &mut bool,
    ) {
        let bytes = FlavourReleased::new(flavours_updated).to_bytes();
        Robot::release_to_next_robot(&bytes, next_robot_port, robots, robot_id)
            .await;

        Robot::release_to_leader(
            leader,
            leader_election_started,
            &bytes,
            robot_id,
            next_robot_port,
            robots,
        )
        .await;
    }
}

/// Removes an order from the pending orders if it exists.
fn remove_order_if_exists(pending_orders: &mut OrdersByRobot, order: &Order) {
    for (_, orders) in pending_orders.iter_mut() {
        orders.retain(|o| o.id != order.id);
    }
}
