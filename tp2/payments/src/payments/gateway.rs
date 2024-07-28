use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::payments::errors::PaymentError;
use crate::payments::get_orders::GetOrdersInProgress;
use crate::payments::message_parser::parse_message;
use crate::payments::messages::{Message, OrderId};
use crate::payments::order::Order;
use crate::payments::random_rejection::should_decline_order;
use crate::payments::stop_gateway::StopGateway;
use actix::fut::wrap_future;
use actix::{Actor, ActorContext, Context, ContextFutureSpawner, Handler, StreamHandler};
use tokio::io::{split, AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_util::bytes::BytesMut;

const LEADER_DEFAULT_PORT: u16 = 6000;

/// Payment Gateway actor is responsible for processing payment requests.
pub struct Gateway {
    /// Write half of the TCP stream to send messages to the robot.
    pub leader_robot: Arc<Mutex<Option<WriteHalf<TcpStream>>>>,
    /// Address of the robot.
    pub addr: SocketAddr,
    /// List of order IDs that are currently being processed.
    pub orders_in_progress: HashMap<OrderId, Order>,
}

impl Gateway {
    /// Depending on the message received, the Gateway actor will either capture the payment,
    /// finish the payment, or cancel the payment.
    fn process_message(&mut self, msg: Message, ctx: &mut Context<Gateway>) {
        match msg {
            Message::CapturePayment(order) => {
                self.capture_payment(ctx, order);
            }
            Message::FinishPayment(order_id, _) => {
                self.finish_or_cancel_payment(ctx, order_id, false);
            }
            Message::CancelPayment(order_id) => {
                self.finish_or_cancel_payment(ctx, order_id, true);
            }
            Message::NewLeader(leader_port, _) => {
                self.update_leader(leader_port, ctx);
            }
            _ => {
                println!("Gateway: received unknown message: {:?}", msg);
            }
        }
    }

    /// Capture the payment for the order. If the order is already in progress, do nothing.
    /// If the order should be declined, send a PaymentDeclined message. Otherwise, send a
    /// PaymentAccepted message.
    fn capture_payment(&mut self, ctx: &mut Context<Gateway>, order: Order) {
        let order_id = order.id;
        if self.find_order(order_id).is_some() {
            println!("Gateway: order {} already in progress", order_id);
            self.send_msg(Message::PaymentAccepted(order.clone()), ctx);
            return;
        }

        if should_decline_order() {
            self.send_msg(Message::PaymentDeclined(order), ctx);
        } else {
            self.orders_in_progress.insert(order_id, order.clone());
            self.send_msg(Message::PaymentAccepted(order), ctx);
        }
    }

    /// Finish or cancel the payment for the order. If the order is in progress, remove it from the
    /// list of orders in progress and send a PaymentAccepted message. If the order is not in progress,
    /// do nothing.
    /// If cancel is true, log that the order was cancelled.
    fn finish_or_cancel_payment(
        &mut self,
        ctx: &mut Context<Gateway>,
        order_id: OrderId,
        cancel: bool,
    ) {
        let order = self.find_order(order_id).cloned();
        if order.is_some() {
            if self.delete_order(order_id).is_some() {
                self.send_msg(Message::OrderDone(order_id), ctx);
            }
            if cancel {
                println!("Gateway: order {} cancelled", order_id);
            }
        } else {
            println!("Gateway: order {} not found", order_id);
        }
    }

    fn delete_order(&mut self, order_id: OrderId) -> Option<Order> {
        self.orders_in_progress.remove(&order_id)
    }

    fn find_order(&mut self, order_id: OrderId) -> Option<&Order> {
        self.orders_in_progress.get(&order_id)
    }

    /// Send a message to the robot.
    fn send_msg(&mut self, msg: Message, ctx: &mut Context<Gateway>) {
        let arc = self.leader_robot.clone();
        println!("Gateway: sending message: {:?}", msg);
        wrap_future::<_, Self>(async move {
            let mut write = arc.lock().await;
            if write.is_none() {
                let connection_with_leader = split(
                    TcpStream::connect(format!("localhost:{}", LEADER_DEFAULT_PORT))
                        .await
                        .unwrap(),
                )
                .1;
                *write = Some(connection_with_leader);
            }

            if let Some(stream) = write.as_mut() {
                let res = stream.write(&msg.to_bytes()).await;
                match res {
                    Ok(_) => {
                        println!("Gateway: message {:?} sent successfully", msg);
                    }
                    Err(e) => {
                        println!("Gateway: error sending message {:?}: {:?}", msg, e);
                    }
                }
            }
        })
        .spawn(ctx);
    }

    fn update_leader(&self, port: u16, ctx: &mut Context<Gateway>) {
        let leader_write = self.leader_robot.clone();
        wrap_future::<_, Self>(async move {
            let res = &mut *leader_write.lock().await;

            if let Ok(stream) = TcpStream::connect(format!("localhost:{}", port)).await {
                let connection_with_leader = split(stream).1;
                *res = Some(connection_with_leader);
            }
        })
        .spawn(ctx);
    }
}

impl Actor for Gateway {
    /// Gateway actor's context type is the actor itself.
    type Context = Context<Self>;
}

impl StreamHandler<Result<BytesMut, PaymentError>> for Gateway {
    /// Handle the message received from the robot. Parse the message and process it.
    /// If the message is successfully parsed, print the message and process it.
    /// If the message is not successfully parsed, print an error message.
    fn handle(&mut self, read: Result<BytesMut, PaymentError>, ctx: &mut Self::Context) {
        if let Ok(bytes) = read {
            let message = parse_message(bytes);
            match message {
                Ok(msg) => {
                    println!("Gateway: received message: {:?}", msg);
                    self.process_message(msg, ctx);
                }
                Err(e) => {
                    println!("Gateway: error parsing message: {:?}", e);
                }
            }
        } else {
            println!("Gateway: error reading message: {:?}", read)
        }
    }

    ///Finish processing the message.
    /// Does not close the connection. Instead, continues to wait for new messages.
    fn finished(&mut self, _: &mut Self::Context) {
        println!("Finished processing message. Waiting for new messages...");
    }
}

impl Handler<GetOrdersInProgress> for Gateway {
    type Result = Vec<Order>;

    /// Handle the GetOrdersInProgress message by returning the list of orders in progress.
    fn handle(&mut self, _msg: GetOrdersInProgress, _ctx: &mut Context<Self>) -> Self::Result {
        println!(
            "Gateway: getting orders in progress: {:?}",
            self.orders_in_progress
        );
        self.orders_in_progress.values().cloned().collect()
    }
}

impl Handler<StopGateway> for Gateway {
    type Result = ();

    /// This handle message is used when a new robot leader is announced.
    /// The current Gateway is stopped and a new one starts.
    fn handle(&mut self, _msg: StopGateway, ctx: &mut Context<Self>) -> Self::Result {
        println!("Gateway: stopping previous actor...");
        ctx.stop();
    }
}
