use std::future::Future;
use std::sync::Arc;

use actix::fut::wrap_future;
use actix::{Context, ContextFutureSpawner};
use bytes::BytesMut;
use futures::join;
use tokio::io::{split, AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::RwLock;

use crate::leader_election::ring::RingLeaderElection;
use crate::messages::election::Election as ElectionMessage;
use crate::messages::message::{Message, ELECTION_ID};
use crate::messages::new_leader::NewLeader;
use crate::robots::robot::{ActiveRobots, LeaderStream, OrdersByRobot, PaymentStream, Port, Robot};
use crate::utils::log::{error, info, warn};
use crate::utils::util::{addr, read_payments_port};

pub trait Election {
    /// Connects leader to all other robots.
    /// Sends NewLeader message to all other robots.
    /// Asynchronously notifies screen.
    /// Sets leader stream to None if there is more than one robot.
    fn connect_to_all(
        robots: &mut ActiveRobots,
        leader_id: Port,
        next: Arc<RwLock<Port>>,
        screens: Vec<Port>,
        arc_pending_orders: Arc<RwLock<OrdersByRobot>>,
    ) -> impl Future<Output = ()> + Send;

    /// Sets the leader socket to None because the leader is the current robot.
    fn set_leader_to_none(
        arc_leader_socket: Arc<RwLock<Option<WriteHalf<TcpStream>>>>,
    ) -> impl Future<Output = ()> + Send;

    /// Sets the current robot as the leader.
    fn set_as_leader(is_leader: &mut bool) -> impl Future<Output = ()> + Send;

    /// NewLeader message is processed by the robot.
    /// If the leader is my current next robot, update leader and next robot.
    /// If the leader is not my next robot, create a connection to the new leader.
    fn handle_new_leader(&self, msg: &mut NewLeader, ctx: &mut Context<Robot>);

    /// Notifies the screen that the leader has changed.
    fn notify_leader_to_screen(screens: Vec<Port>, new_leader_msg: Vec<u8>);
    /// Connects to the new leader.
    /// Updates the leader socket to the new leader.
    fn connect_to_new_leader(
        leader: &mut LeaderStream,
        leader_port: Port,
        other_robots_locked: &mut ActiveRobots,
    ) -> impl Future<Output = ()> + Send;

    /// Sets leader to the new one. In this case, robot's next is the leader, so
    /// it also changes the next robot to the leader's next and opens a connection.
    fn update_leader_and_next(
        robot_id: Port,
        leader_id: Port,
        leader_next: Port,
        other_robots: &mut ActiveRobots,
        leader: &mut LeaderStream,
        next_robot_port: &mut Port,
        pending_orders: &mut OrdersByRobot,
    ) -> impl Future<Output = ()> + Send;
    fn send_port_to_payments(
        leader_port: Port,
        payments_arc: Arc<RwLock<PaymentStream>>,
    ) -> impl Future<Output = ()> + Send;
}

impl RingLeaderElection for Robot {
    async fn find_new_leader(robots: &mut ActiveRobots, port: Port, mut next_robot: Port) {
        error(&format!(
            "Leader is dead. Robot {} is starting the election process...",
            port
        ));
        let mut sent = false;
        let mut robot_stream = robots.get_mut(&next_robot);
        while !sent {
            if let Some(Some(ref mut stream)) = robot_stream {
                if stream
                    .write(ElectionMessage { ids: vec![port] }.to_bytes().as_slice())
                    .await
                    .is_err()
                {
                    error(&format!(
                        "Failed to send election msg. Robot {} is dead. Retrying...",
                        next_robot
                    ));
                    next_robot = Robot::find_next_robot(robots, next_robot, port).await;
                    robot_stream = robots.get_mut(&next_robot);
                } else {
                    sent = true;
                }
            } else {
                error(&format!(
                    "Unexpected error. Failed to find robot {} in other_robots.",
                    next_robot
                ));
                next_robot = Robot::find_next_robot(robots, next_robot, port).await;
                robot_stream = robots.get_mut(&next_robot);
            }
        }
    }

    fn receive_election<M: Message>(&mut self, mut election_msg: M, ctx: &mut Context<Robot>) {
        election_msg.be_handled_by(self, ctx);
    }

    fn send_next_election(&mut self, mut election_message: Vec<u8>, ctx: &mut Context<Robot>) {
        let arc_next_robot = self.next_robot.clone();
        let arc_other_robots = self.robots.clone();
        let self_id = self.id;

        wrap_future::<_, Self>(async move {
            let mut next_robot_locked = arc_next_robot.write().await;
            let mut other_robots_locked = arc_other_robots.write().await;
            let mut sent = false;
            while !sent {
                if let Some(Some(ref mut stream)) = other_robots_locked.get_mut(&next_robot_locked)
                {
                    if stream.write(&election_message).await.is_err() {
                        error(&format!(
                            "Failed to send election msg. Robot {} is dead. Retrying...",
                            next_robot_locked
                        ));
                        election_message = Self::remove_dead_robot_from_election(
                            election_message,
                            &next_robot_locked,
                        );
                        *next_robot_locked = Robot::find_next_robot(
                            &mut other_robots_locked,
                            *next_robot_locked,
                            self_id,
                        )
                        .await;
                        other_robots_locked.remove(&next_robot_locked);
                    }
                    sent = true;
                } else {
                    error(&format!(
                        "Unexpected error. Failed to find robot {} in other_robots.",
                        next_robot_locked
                    ));
                    *next_robot_locked = Robot::find_next_robot(
                        &mut other_robots_locked,
                        *next_robot_locked,
                        self_id,
                    )
                    .await;
                }
            }
        })
        .spawn(ctx);
    }

    fn assume_leadership(&mut self) {
        let arc_is_leader = self.is_leader.clone();
        let arc_other_robots = self.robots.clone();
        let arc_leader_socket = self.leader.clone();
        let arc_next = self.next_robot.clone();
        let leader_id = self.id;
        let screens = self.screen_ports.clone();
        let arc_pending_orders = self.pending_orders.clone();
        let arc_payments = self.payments.clone();

        tokio::spawn(async move {
            let mut is_leader = arc_is_leader.write().await;
            if *is_leader {
                //Robot is already leader
                return;
            }
            info(&format!("Robot {} is the new leader!", leader_id));

            let mut robots = arc_other_robots.write().await;
            join!(
                Robot::set_as_leader(&mut is_leader),
                Robot::set_leader_to_none(arc_leader_socket),
                Robot::connect_to_all(
                    &mut robots,
                    leader_id,
                    arc_next,
                    screens.clone(),
                    arc_pending_orders,
                ),
                Robot::send_port_to_payments(leader_id, arc_payments)
            );
        });
    }

    /// Remove the dead robot from the election message.
    /// If the message is a Coordinator, nothing is done.
    fn remove_dead_robot_from_election(msg_bytes: Vec<u8>, dead_robot: &Port) -> Vec<u8> {
        if msg_bytes[0] != ELECTION_ID {
            //could be Coordinator
            return msg_bytes;
        }

        let msg = ElectionMessage::from_bytes(&mut BytesMut::from(msg_bytes.as_slice()));
        let ids = msg
            .ids
            .iter()
            .filter(|id| **id != *dead_robot)
            .copied()
            .collect();

        ElectionMessage { ids }.to_bytes()
    }
}

impl Election for Robot {
    /// Connects leader to all other robots.
    /// Sends NewLeader message to all other robots.
    /// Asynchronously notifies screen.
    async fn connect_to_all(
        robots: &mut ActiveRobots,
        leader_id: Port,
        next: Arc<RwLock<Port>>,
        screens: Vec<Port>,
        arc_pending_orders: Arc<RwLock<OrdersByRobot>>,
    ) {
        robots.remove(&leader_id);
        let robot_ports = Robot::robot_ports(robots);
        let mut next = next.write().await;
        let mut pending_orders = arc_pending_orders.write().await;
        let new_leader_msg = NewLeader {
            id: leader_id,
            leader_next: *next,
        }
        .to_bytes();
        let msg_cloned = new_leader_msg.clone();
        Self::notify_leader_to_screen(screens, msg_cloned);

        let mut dead_robots: Vec<Port> = vec![];
        Self::connect_tcp_streams(robots, robot_ports, &new_leader_msg, &mut dead_robots).await;

        Robot::handle_dead_robots(
            robots,
            dead_robots,
            &mut pending_orders,
            &mut next,
            leader_id,
        )
        .await;
    }

    /// Sets the leader socket to None because the leader is the current robot.
    /// In case there is one robot, the leader socket is not set to None.
    async fn set_leader_to_none(arc_leader_socket: Arc<RwLock<LeaderStream>>) {
        let mut leader_socket_locked = arc_leader_socket.write().await;
        *leader_socket_locked = None;
    }

    /// Sets the current robot as the leader.
    async fn set_as_leader(is_leader: &mut bool) {
        *is_leader = true;
    }

    /// NewLeader message is processed by the robot.
    /// If the leader is my current next robot, update leader and next robot.
    /// If the leader is not my next robot, create a connection to the new leader.
    fn handle_new_leader(&self, msg: &mut NewLeader, ctx: &mut Context<Robot>) {
        let robot_id = self.id;
        let arc_next_robot = self.next_robot.clone();
        let arc_other_robots = self.robots.clone();
        let arc_leader = self.leader.clone();
        let arc_is_new_leader = self.is_leader.clone();
        let leader_port = msg.id;
        let msg_next_robot = msg.leader_next;
        let arc_pending_orders = self.pending_orders.clone();
        let arc_leader_election_started = self.leader_election_started.clone();
        wrap_future::<_, Self>(async move {
            let is_new_leader = arc_is_new_leader.read().await;
            if *is_new_leader {
                warn("Leader received NewLeader message. Ignoring.");
                return;
            }

            let mut leader_stream = arc_leader.write().await;
            let mut next_robot_locked = arc_next_robot.write().await;
            let mut other_robots_locked = arc_other_robots.write().await;
            let mut pending_orders = arc_pending_orders.write().await;
            if *next_robot_locked == leader_port {
                Self::update_leader_and_next(
                    robot_id,
                    leader_port,
                    msg_next_robot,
                    &mut other_robots_locked,
                    &mut leader_stream,
                    &mut next_robot_locked,
                    &mut pending_orders,
                )
                .await;
            } else {
                Self::connect_to_new_leader(
                    &mut leader_stream,
                    leader_port,
                    &mut other_robots_locked,
                )
                .await;
            }

            info(&format!(
                "Pending orders after new leader election: {:?}",
                pending_orders
            ));
            *arc_leader_election_started.write().await = false;
        })
        .spawn(ctx);
    }

    /// Notifies the screen that the leader has changed.
    /// This function is useful when the leader changes while a screen is sending orders.
    fn notify_leader_to_screen(screens: Vec<Port>, new_leader_msg: Vec<u8>) {
        tokio::spawn(async move {
            for s in screens {
                let stream_screen = TcpStream::connect(addr(s)).await;
                if let Ok(stream) = stream_screen {
                    let mut connection = split(stream).1;
                    let _ = connection.write(&new_leader_msg).await;
                    info(&format!("Connected to screen: {}", s));
                }
                //Else: Robot doesn´t connect to screen because it is not running
            }
        });
    }

    /// Connects to the new leader.
    /// Removes the leader from other robots in case it is there.
    /// Updates the leader socket to the new leader.
    async fn connect_to_new_leader(
        leader: &mut LeaderStream,
        leader_port: Port,
        other_robots_locked: &mut ActiveRobots,
    ) {
        if other_robots_locked.remove(&leader_port).is_none() {
            warn("Leader msg repeated. Already connected!")
        } else if let Ok(stream) = TcpStream::connect(addr(leader_port)).await {
            *leader = Some(split(stream).1);
            info(&format!("Connected to new leader: {}", leader_port));
        } //else if connection fails, will be detected soon
    }

    /// Sets leader to the new one. In this case, robot's next is the leader, so
    /// it also changes the next robot to the leader's next and opens a connection.
    async fn update_leader_and_next(
        robot_id: Port,
        leader_id: Port,
        leader_next: Port,
        other_robots: &mut ActiveRobots,
        leader: &mut LeaderStream,
        next_robot_port: &mut Port,
        pending_orders: &mut OrdersByRobot,
    ) {
        match other_robots.remove(&leader_id) {
            None => warn("Leader msg repeated. Already connected!"),
            Some(new_leader) => {
                *leader = new_leader;
                *next_robot_port = leader_next;

                match TcpStream::connect(addr(leader_next)).await {
                    Err(_) => {
                        error(&format!(
                            "Failed to connect to next robot after leader election: {}",
                            leader_next
                        ));
                        Robot::handle_dead_robot(
                            other_robots,
                            leader_next,
                            pending_orders,
                            next_robot_port,
                            robot_id,
                        )
                        .await;
                    }
                    Ok(stream) => {
                        other_robots.insert(leader_next, Some(split(stream).1));
                        info(&format!("Connected to leader: {}", leader_id));
                    }
                }
            }
        }
    }

    /// Sends new leader port to payments and opens a connection.
    async fn send_port_to_payments(leader_port: Port, payments_arc: Arc<RwLock<PaymentStream>>) {
        let write_payments = &mut *payments_arc.write().await;
        let new_leader = NewLeader::to_bytes(&NewLeader {
            id: leader_port,
            leader_next: 0,
        });
        if write_payments.is_none() {
            if let Ok(stream) = TcpStream::connect(addr(read_payments_port())).await {
                let payments_connection = Some(split(stream).1);
                *write_payments = payments_connection;
            }
        }

        if let Err(send) = write_payments
            .as_mut()
            .unwrap()
            .write_all(&new_leader)
            .await
        {
            //is always Some, don´t need to check unwrap
            error(&format!(
                "Failed to send leader port to payments: {:?}",
                send
            ))
        }
    }
}

impl Robot {
    /// Connects leader to all robots in the ring and sends new leader msg
    async fn connect_tcp_streams(
        robots: &mut ActiveRobots,
        robot_ports: Vec<Port>,
        new_leader_msg: &[u8],
        dead_robots: &mut Vec<Port>,
    ) {
        for port in robot_ports.iter() {
            let port_deref = *port;
            if let Ok(mut stream) = TcpStream::connect(addr(port_deref)).await {
                if stream.write(new_leader_msg).await.is_err() {
                    dead_robots.push(port_deref);
                    continue;
                }
                robots.insert(port_deref, Some(split(stream).1));
            } else {
                dead_robots.push(port_deref);
            }
        }
    }
}
