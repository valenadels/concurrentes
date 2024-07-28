use std::{collections::HashMap, sync::Arc};

use actix::{Actor, Context, ContextFutureSpawner, Handler, StreamHandler};
use actix::fut::wrap_future;
use bytes::BytesMut;
use tokio::{
    io::{AsyncWriteExt, split, WriteHalf},
    net::{TcpListener, TcpStream},
    sync::RwLock,
};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_util::codec::FramedRead;

use crate::messages::flavour_released::FlavourReleased;
use crate::messages::message::from_bytes;
use crate::messages::message::Message;
use crate::messages::new_order::NewOrder;
use crate::messages::new_pending_order::NewPendingOrder;
use crate::messages::token_ring::StartTokenRing;
use crate::orders::flavours::Flavour;
use crate::orders::order::Order;
use crate::robots::codec::RobotCodec;
use crate::robots::errors::RobotError;
use crate::robots::init::{
    connect_to_leader, find_next, initialize_depending_on_role, start_token_ring,
};
use crate::utils::log::{error, info};
use crate::utils::util::addr;

const LEADER_PORT_IDX: usize = 0;
pub type Port = u16;
pub type OrdersByRobot = HashMap<Port, Vec<Order>>;
pub type ActiveRobots = HashMap<Port, Option<WriteHalf<TcpStream>>>;
pub type RobotStream = Option<WriteHalf<TcpStream>>;
pub type PaymentStream = Option<WriteHalf<TcpStream>>;
pub type LeaderStream = Option<WriteHalf<TcpStream>>;
pub type FlavoursStock = HashMap<Flavour, u32>;

pub struct Robot {
    /// Identifier by listener port.
    pub id: Port,
    /// Whether robot should act as a leader or not.
    pub is_leader: Arc<RwLock<bool>>,
    /// Orders awaiting to be processed by robot's app.
    pub pending_orders: Arc<RwLock<OrdersByRobot>>,
    /// Amounts of flavour left by ID.
    pub flavours: Arc<RwLock<FlavoursStock>>,
    /// Stream to payments' app. Should hold None for non-leader robots.
    pub payments: Arc<RwLock<PaymentStream>>,

    pub flavours_ping_sender: Sender<FlavoursStock>,
    /// Robot's leader write stream. Should be None for the leader.
    pub leader: Arc<RwLock<LeaderStream>>,
    /// Whether leader election has started or not.
    /// Used to avoid multiple leader elections.
    pub leader_election_started: Arc<RwLock<bool>>,
    /// Next robot in the ring.
    /// Used by leader to do round-robin assignments of orders.
    /// Used by non-leaders to identify what robot is next in ring.
    /// If there's only one non-leader robot, next_robot is itself.
    pub next_robot: Arc<RwLock<Port>>,
    /// Non-leader robots by port.
    /// If is_leader = false -> must be Some just for next_robot
    /// If is_leader -> must be Some for all robots
    pub robots: Arc<RwLock<ActiveRobots>>,
    /// Ports of the screens.
    pub screen_ports: Vec<Port>,
}

impl Actor for Robot {
    type Context = Context<Self>;
}

impl StreamHandler<Result<BytesMut, RobotError>> for Robot {
    /// Handle the message received from the stream.
    fn handle(&mut self, read: Result<BytesMut, RobotError>, ctx: &mut Self::Context) {
        let mut bytes = read.expect("");
        let mut message = from_bytes(&mut bytes).expect("");
        (*message).be_handled_by(self, ctx);
    }

    /// Finish processing the message.
    /// Does not close the connection. Instead, continues to wait for new messages.
    fn finished(&mut self, _: &mut Self::Context) {}
}

impl Handler<StartTokenRing> for Robot {
    type Result = ();

    /// This handle message is used upon creation of the first robot leader.
    /// It sends the initial flavours to the first robot in the ring.
    fn handle(&mut self, _msg: StartTokenRing, ctx: &mut Context<Self>) -> Self::Result {
        info("Leader: Releasing flavours to first robot in the ring...");
        let next_robot = self.next_robot.clone();
        let robots = self.robots.clone();
        wrap_future::<_, Self>(async move {
            let initial_flavour_released = FlavourReleased {
                flavours: Flavour::initial_flavours(),
            }
            .to_bytes();
            let mut robots = robots.write().await;
            let next_robot = next_robot.write().await;
            let stream = robots.get_mut(&next_robot).unwrap().as_mut().unwrap();
            let _ = stream.write(&initial_flavour_released).await; //we assume that nothing is going to go down before screen starts
            let _ = stream.flush().await;
        })
        .spawn(ctx);
    }
}

impl Robot {
    /// Starts the robot actor.
    /// Initializes the robot actor with the given ports, depending on whether the robot should be the leader or not.
    /// Binds the robot to the given port and listens for incoming connections.
    /// # Arguments
    /// * `port` - The port to bind the robot to.
    /// * `robots_ports` - The ports of all other robots. First port is the leader's port.
    /// * `screens` - The ports of the screens.
    /// * `payments` - The port of the payments app.
    pub async fn start(
        port: Port,
        mut robots_ports: Vec<Port>,
        screens: Vec<Port>,
        payments: Port,
    ) -> Result<(), RobotError> {
        let leader_port = robots_ports.remove(LEADER_PORT_IDX);
        let should_be_leader = port == leader_port;
        let arc_is_leader = Arc::new(RwLock::new(should_be_leader));
        let pending_orders: Arc<RwLock<OrdersByRobot>> = Arc::new(RwLock::new(HashMap::new()));
        let initial_flavours = Arc::new(RwLock::new(Flavour::initial_flavours()));
        let next_robot_port = find_next(&robots_ports, port);
        let next_robot_arc = Arc::new(RwLock::new(next_robot_port));
        let leader_election_started = Arc::new(RwLock::new(false));

        let (payments_arc, robots_arc) =
            initialize_depending_on_role(&robots_ports, payments, should_be_leader).await?;

        let mut flavours_sent = false;
        let leader: Arc<RwLock<LeaderStream>> = Arc::new(RwLock::new(None));

        let listener = TcpListener::bind(addr(port)).await?;
        let (tx, rx): (Sender<FlavoursStock>, Receiver<FlavoursStock>) = channel(1);
        let is_leader = arc_is_leader.read().await;
        if *is_leader {
            Robot::ping_robots(
                rx,
                robots_arc.clone(),
                pending_orders.clone(),
                next_robot_arc.clone(),
                port,
            )
            .await;
        }

        while let Ok((stream, _)) = listener.accept().await {
            if connect_to_leader(leader_port, should_be_leader, &leader)
                .await
                .is_err()
            {
                info("Discard this msg if leader election has run. Error connecting to leader.");
            }

            let robot_actor = Robot::create(|ctx| {
                let (read, _) = split(stream);
                Robot::add_stream(FramedRead::new(read, RobotCodec::new()), ctx);
                Robot {
                    id: port,
                    is_leader: arc_is_leader.clone(),
                    pending_orders: pending_orders.clone(),
                    flavours: initial_flavours.clone(),
                    payments: payments_arc.clone(),
                    leader: leader.clone(),
                    leader_election_started: leader_election_started.clone(),
                    next_robot: next_robot_arc.clone(),
                    robots: robots_arc.clone(),
                    screen_ports: screens.clone(),
                    flavours_ping_sender: tx.clone(),
                }
            });
            start_token_ring(*is_leader, &mut flavours_sent, robot_actor)
                .await
                .map_err(|e| {
                    error("Error starting token ring");
                    e
                })
                .unwrap();
        }

        Ok(())
    }

    /// Returns the ports of all other robots.
    pub fn robot_ports(other_robots: &ActiveRobots) -> Vec<Port> {
        other_robots.keys().copied().collect()
    }

    /// If a robot is dead, it is removed from the list of active robots.
    /// Its orders are reassigned to the next robot in the ring.
    pub async fn handle_dead_robot(
        robots: &mut ActiveRobots,
        dead_robot: Port,
        pending_orders: &mut OrdersByRobot,
        next_robot: &mut Port,
        id: Port,
    ) {
        if dead_robot == *next_robot {
            *next_robot = Robot::find_next_robot(robots, *next_robot, id).await;
        }
        robots.remove(&dead_robot);
        if let Some(dead_robot_orders) = pending_orders.remove(&dead_robot) {
            Box::pin(Robot::assign_orders_to_robots(
                dead_robot_orders,
                next_robot,
                pending_orders,
                robots,
                id,
            ))
            .await;
        }
    }

    /// If a robot is dead, it is removed from the list of active robots.
    /// Its orders are reassigned to the next robot in the ring.
    pub async fn handle_dead_robots(
        other_robots: &mut ActiveRobots,
        dead_robots: Vec<Port>,
        pending_orders: &mut OrdersByRobot,
        next_robot: &mut Port,
        id: Port,
    ) {
        for dead_robot in dead_robots {
            Robot::handle_dead_robot(other_robots, dead_robot, pending_orders, next_robot, id)
                .await;
        }
    }

    /// Sends a new order to the next alive robot in the ring.
    async fn send_new_order_to_next_alive_robot(
        robots: &mut ActiveRobots,
        next_robot: &mut Port,
        order: &Order,
        pending_orders: &mut OrdersByRobot,
        id: Port,
    ) {
        let mut sent = false;
        while !sent {
            if let Some(Some(ref mut stream)) = robots.get_mut(&*next_robot) {
                if stream
                    .write(&NewOrder::new(order.clone()).to_bytes())
                    .await
                    .is_err()
                {
                    Robot::handle_dead_robot(robots, *next_robot, pending_orders, next_robot, id)
                        .await;
                    *next_robot = Robot::find_next_robot(robots, *next_robot, id).await;
                } else {
                    sent = true;
                }
            }
        }
    }

    /// Sends a new pending order to all robots except the chosen robot.
    pub async fn send_new_pending_order_to_robots(
        robots: &mut ActiveRobots,
        chosen_robot: Port,
        order: Order,
        dead_robots: &mut Vec<Port>,
    ) {
        for (port, stream_opt) in robots {
            if chosen_robot == *port {
                continue;
            }

            match stream_opt {
                Some(ref mut stream) => {
                    match stream
                        .write(&NewPendingOrder::new(chosen_robot, order.clone()).to_bytes())
                        .await
                    {
                        Ok(_) => {
                            info(&format!(
                                "Sent message: NewPendingOrder {} for Robot {}.",
                                order.clone().id,
                                *port
                            ));
                        }
                        Err(_) => {
                            dead_robots.push(*port);
                            error(&format!(
                                "Lost connection to {} while sending NewPendingOrder A",
                                *port
                            ));
                        }
                    }
                }
                None => {
                    error(&format!(
                        "Lost connection to {} while sending NewPendingOrder B.",
                        *port
                    ));
                    dead_robots.push(*port);
                }
            }
        }
    }

    /// Assigns orders from dead robots to the next robot in the ring.
    pub async fn assign_orders_to_robots(
        orders_from_dead_robots: Vec<Order>,
        next_robot: &mut Port,
        pending_orders: &mut OrdersByRobot,
        robots: &mut ActiveRobots,
        id: Port,
    ) {
        for order in orders_from_dead_robots {
            while !pending_orders.contains_key(next_robot) {
                *next_robot = Robot::find_next_robot(robots, *next_robot, id).await;
            }
            Robot::send_new_order_to_next_alive_robot(
                robots,
                next_robot,
                &order,
                pending_orders,
                id,
            )
            .await;
            let mut dead_robots: Vec<Port> = vec![];
            Robot::send_new_pending_order_to_robots(
                robots,
                *next_robot,
                order.clone(),
                &mut dead_robots,
            )
            .await;
            Robot::handle_dead_robots(robots, dead_robots, pending_orders, next_robot, id).await;
            pending_orders.entry(*next_robot).or_default().push(order);
            *next_robot = Robot::find_next_robot(robots, *next_robot, id).await;
        }
    }
}
