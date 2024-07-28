use std::sync::Arc;
use std::vec;

use actix::fut::wrap_future;
use actix::{Context, ContextFutureSpawner};
use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{RwLock, RwLockWriteGuard};

use crate::messages::cancel_payment::CancelPayment;
use crate::messages::finish_payment::FinishPayment;
use crate::messages::message::Message;
use crate::messages::new_order::NewOrder;
use crate::messages::payment_accepted::PaymentAccepted;
use crate::messages::payment_declined::PaymentDeclined;
use crate::robots::robot::{ActiveRobots, OrdersByRobot, PaymentStream, Port, Robot};
use crate::utils::log::{error, info, warn};

pub trait Payment {
    /// Handle the payment accepted message.
    /// If the robot is the leader, it will send the order to the next robot and update the pending orders.
    fn handle_payment_accepted(&self, msg: &PaymentAccepted, ctx: &mut Context<Robot>);
    /// Handle the payment declined message. Does nothing with the order.
    fn handle_payment_declined(&self, _msg: &PaymentDeclined, _ctx: &mut Context<Robot>) {}
    /// Handle the cancel payment message. Sends payment result to all robots
    fn handle_cancel_payment(&self, _msg: &CancelPayment, _ctx: &mut Context<Robot>);
    /// Handle the finish payment message. Sends payment result to all robots
    fn handle_finish_payment(&self, _msg: &FinishPayment, _ctx: &mut Context<Robot>);
}

impl Payment for Robot {
    fn handle_payment_accepted(&self, msg: &PaymentAccepted, ctx: &mut Context<Robot>) {
        let arc_is_leader = self.is_leader.clone();
        let arc_pending_orders = self.pending_orders.clone();
        let arc_other_robots = self.robots.clone();
        let arc_next_robot = self.next_robot.clone();
        let self_id = self.id;
        let msg_c = msg.clone();

        wrap_future::<_, Self>(async move {
            let is_leader = arc_is_leader.read().await;
            if !*is_leader {
                warn("Non-leader robot received payment accepted. Ignoring.");
                return;
            }
            let mut pending_orders = arc_pending_orders.write().await;
            let mut other_robots = arc_other_robots.write().await;
            let mut next_robot = arc_next_robot.write().await;
            let mut chosen_robot = *next_robot;
            let mut sent = false;
            let mut dead_robots: Vec<Port> = vec![];

            while !sent {
                if let Some(Some(ref mut stream)) = other_robots.get_mut(&*next_robot) {
                    Self::send_order(&msg_c, *next_robot, &mut chosen_robot, &mut sent, stream)
                        .await;
                }
                if !sent {
                    error(&format!(
                        "Lost connection to {} while sending NewOrder.",
                        *next_robot
                    ));
                    dead_robots.push(*next_robot);
                } else {
                    pending_orders
                        .entry(*next_robot)
                        .or_insert_with(Vec::new)
                        .push(msg_c.order.clone());
                }
                *next_robot = Robot::find_next_robot(&mut other_robots, *next_robot, self_id).await;
            }

            Robot::send_new_pending_order_to_robots(
                &mut other_robots,
                chosen_robot,
                msg_c.order,
                &mut dead_robots,
            )
            .await;

            Robot::handle_dead_robots(
                &mut other_robots,
                dead_robots,
                &mut pending_orders,
                &mut next_robot,
                self_id,
            )
            .await
        })
        .spawn(ctx);
    }

    fn handle_cancel_payment(&self, msg: &CancelPayment, ctx: &mut Context<Robot>) {
        self.handle_payment(msg.to_bytes(), msg.order_id, msg.port, ctx);
    }

    fn handle_finish_payment(&self, msg: &FinishPayment, ctx: &mut Context<Robot>) {
        self.handle_payment(msg.to_bytes(), msg.order_id, msg.port, ctx);
    }
}

impl Robot {
    fn handle_payment(&self, msg: Vec<u8>, order_id: u16, port: Port, ctx: &mut Context<Robot>) {
        let arc_is_leader = self.is_leader.clone();
        let arc_robots = self.robots.clone();
        let arc_pending_orders = self.pending_orders.clone();
        let arc_payments = self.payments.clone();
        let arc_next_robot = self.next_robot.clone();
        let self_id = self.id;

        wrap_future::<_, Self>(async move {
            let mut pending_orders_map = arc_pending_orders.write().await;
            Self::update_pending_orders(order_id, port, &mut pending_orders_map);
            let is_leader = arc_is_leader.read().await;
            if !*is_leader {
                return;
            }

            let mut robots = arc_robots.write().await;
            let mut dead_robots: Vec<Port> = vec![];

            Self::send_payment_result_to_all(&msg, &mut robots, &mut dead_robots).await;

            Robot::handle_dead_robots(
                &mut robots,
                dead_robots,
                &mut pending_orders_map,
                &mut *arc_next_robot.write().await,
                self_id,
            )
            .await;

            Self::send_payment_result_to_payments(&msg, order_id, arc_payments).await;
        })
        .spawn(ctx);
    }

    /// Sends finished/cancelled order to payments app.
    async fn send_payment_result_to_payments(
        msg: &[u8],
        order_id: u16,
        arc_payments: Arc<RwLock<PaymentStream>>,
    ) {
        let mut sent = false;
        if let Some(stream) = arc_payments.write().await.as_mut() {
            if stream.write(msg).await.is_ok() {
                info(&format!(
                    "Successfully communicated finished/cancelled order {} to payments app.",
                    order_id
                ));
                sent = true;
            }
        }

        if !sent {
            error(&format!(
                "Failed to communicate finished/canceled order {} to payments app.",
                order_id
            ));
        }
    }

    /// Sends cancelled or finished to all robots
    async fn send_payment_result_to_all(
        msg: &[u8],
        robots: &mut ActiveRobots,
        dead_robots: &mut Vec<Port>,
    ) {
        for (port, stream_opt) in &mut *robots {
            if let Some(stream) = stream_opt.as_mut() {
                if stream.write(msg).await.is_err() {
                    dead_robots.push(*port);
                }
            }
        }
    }

    /// Updates pending orders retaining only the ones that are not cancelled or finished.
    fn update_pending_orders(
        order_id: u16,
        port: Port,
        pending_orders_map: &mut RwLockWriteGuard<OrdersByRobot>,
    ) {
        pending_orders_map.entry(port).and_modify(|orders| {
            orders.retain(|order| order.id != order_id);
        });
    }

    /// Sends the new order to next robot
    async fn send_order(
        msg_c: &PaymentAccepted,
        next_robot: Port,
        chosen_robot: &mut Port,
        sent: &mut bool,
        stream: &mut WriteHalf<TcpStream>,
    ) {
        if stream
            .write(&NewOrder::new(msg_c.order.clone()).to_bytes())
            .await
            .is_ok()
        {
            info(&format!(
                "Sent message: NewOrder {} for Robot {}.",
                msg_c.order.clone().id,
                next_robot
            ));
            *chosen_robot = next_robot;
            *sent = true;
        }
    }
}
