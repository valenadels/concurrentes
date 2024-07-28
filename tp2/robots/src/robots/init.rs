use crate::messages::token_ring::StartTokenRing;
use crate::robots::errors::RobotError;
use crate::robots::robot::{ActiveRobots, LeaderStream, PaymentStream, Port, Robot};
use crate::utils::util::addr;
use actix::Addr;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::split;
use tokio::net::TcpStream;
use tokio::sync::RwLock;

/// Initialize the robot depending on its role.
/// # Arguments
/// * `robots_ports` - The list of robot ports.
/// * `payments` - The port of the payments server.
/// * `should_be_leader` - Whether the robot should be the leader.
/// # Returns
/// A tuple with the payments stream and the robot connections.
pub async fn initialize_depending_on_role(
    robots_ports: &[Port],
    payments: Port,
    should_be_leader: bool,
) -> Result<(Arc<RwLock<PaymentStream>>, Arc<RwLock<ActiveRobots>>), RobotError> {
    let mut payments_stream = None;
    let mut robot_connections: ActiveRobots = HashMap::new();
    if should_be_leader {
        initialize_leader_fields(
            robots_ports,
            payments,
            &mut payments_stream,
            &mut robot_connections,
        )
        .await?;
    } else {
        initialize_non_leader_fields(robots_ports, &mut robot_connections).await?;
    }

    let payments_arc = Arc::new(RwLock::new(payments_stream));
    let robots_arc = Arc::new(RwLock::new(robot_connections));
    Ok((payments_arc, robots_arc))
}

/// Initialize the fields of the robot that is not the leader.
/// # Arguments
/// * `robots` - The list of robot ports
/// * `robot_connections` - The connections to the robots.
pub async fn initialize_non_leader_fields(
    robots: &[Port],
    robot_connections: &mut ActiveRobots,
) -> Result<(), RobotError> {
    for port in robots {
        robot_connections.insert(*port, None);
    }
    Ok(())
}

/// Initialize the fields of the leader robot.
/// # Arguments
/// * `robots` - The list of robot ports not including the leader.
/// * `payments` - The port of the payments server.
/// * `payments_stream` - The stream to the payments server.
/// * `robot_connections` - The connections to the robots.
pub async fn initialize_leader_fields(
    robots: &[Port],
    payments: Port,
    payments_stream: &mut PaymentStream,
    robot_connections: &mut ActiveRobots,
) -> Result<(), RobotError> {
    *payments_stream = Some(split(TcpStream::connect(addr(payments)).await?).1);
    for port in robots {
        let port_deref = *port;
        robot_connections.insert(
            port_deref,
            Some(split(TcpStream::connect(addr(port_deref)).await?).1),
        );
    }
    Ok(())
}

/// Find the next robot in the list of robot ports.
/// If the robot is the last one, return the first one.
/// If the robot is not in the list, return the first one.
/// # Arguments
/// * `robot_ports` - The list of robot ports.
/// * `next_robot` - The current next robot port.
/// # Returns
/// The next robot port.
pub fn find_next(robot_ports: &[Port], next_robot: Port) -> Port {
    match robot_ports.iter().position(|&x| x == next_robot) {
        Some(index) => {
            if index + 1 < robot_ports.len() {
                return robot_ports[index + 1];
            }
            robot_ports[0]
        }
        None => robot_ports[0],
    }
}

/// Connects a robot to the leader. This function should only be called when intializing the robots at the start of the program.
pub async fn connect_to_leader(
    leader_port: Port,
    should_be_leader: bool,
    leader: &Arc<RwLock<LeaderStream>>,
) -> Result<(), RobotError> {
    if !should_be_leader {
        let mut leader_val = leader.write().await;
        if leader_val.is_none() {
            *leader_val = Some(split(TcpStream::connect(addr(leader_port)).await?).1);
        }
    }
    Ok(())
}

/// Starts the token ring if the robot should be the leader and the flavours have not been sent yet.
pub async fn start_token_ring(
    should_be_leader: bool,
    flavours_sent: &mut bool,
    robot_actor: Addr<Robot>,
) -> Result<(), RobotError> {
    if should_be_leader && !*flavours_sent {
        robot_actor
            .send(StartTokenRing {})
            .await
            .map_err(|_| RobotError::ReleaseFlavourError)?;
        *flavours_sent = true;
    }
    Ok(())
}
