use crate::robots::robot::{ActiveRobots, Port, Robot};
use crate::utils::log::error;
use crate::utils::util::addr;
use tokio::io::split;
use tokio::net::TcpStream;

impl Robot {
    /// Find the next robot in the list of robot ports.
    /// If the robot is the last one, return the first one.
    /// If the robot is not in the list, return the first one.
    /// # Arguments
    /// - robots: all non-dead, non-leader robots.
    /// - cur_next_robot: current next robot's port.
    /// - id: caller's port.
    /// # Returns
    /// The next robot port.
    pub async fn find_next_robot(
        robots: &mut ActiveRobots,
        cur_next_robot: Port,
        id: Port,
    ) -> Port {
        let mut robot_ports: Vec<Port> = Self::robot_ports(robots);
        robot_ports.sort();
        let robot_ports_len = robot_ports.len();

        loop {
            // Si solo queda un robot entre los conocidos, significa que el siguiente robot es sí mismo.
            // robot_ports[0] debería ser igual a id.
            if robot_ports_len == 1 {
                if let Some(value) = Self::self_is_next_robot(robots, &mut robot_ports).await {
                    return value;
                }
            }

            if let Some(port) = Self::self_is_not_next_robot(
                robots,
                cur_next_robot,
                id,
                &mut robot_ports,
                robot_ports_len,
            )
            .await
            {
                return port;
            }
        }
    }

    async fn self_is_not_next_robot(
        robots: &mut ActiveRobots,
        cur_next_robot: Port,
        id: Port,
        robot_ports: &mut Vec<Port>,
        robot_ports_len: usize,
    ) -> Option<Port> {
        return match robot_ports.iter().position(|&x| x == cur_next_robot) {
            Some(index) => {
                let mut next_robot_idx = index + 1;

                // Si se excede del largo, se reinicia en 0 el idx.
                if next_robot_idx >= robot_ports_len {
                    next_robot_idx = 0;
                }
                // Si el nuevo idx coincide con el ID, se lo saltea.
                if robot_ports[next_robot_idx] == id {
                    next_robot_idx += 1;
                }
                // Si saltear el ID causó que se vaya de rango el idx, se lo reinicia en 0.
                if next_robot_idx >= robot_ports_len {
                    next_robot_idx = 0;
                }
                println!("{:}", next_robot_idx);
                let new_port = robot_ports[next_robot_idx];
                Self::found_next_port(robots, robot_ports, next_robot_idx, new_port).await
            }
            None => {
                error(&format!(
                    "Unexpected error. Failed to find next robot for {} with current one {}.",
                    id, cur_next_robot
                ));
                Some(robot_ports[0])
            }
        };
    }

    async fn found_next_port(
        robots: &mut ActiveRobots,
        robot_ports: &mut Vec<Port>,
        next_robot_idx: usize,
        new_port: Port,
    ) -> Option<Port> {
        if let Some(active_robot) = robots.get(&new_port) {
            match active_robot {
                Some(_) => return Some(new_port),
                None => {
                    // new_port is not necessarily dead, but its connection has not been established.
                    match TcpStream::connect(addr(new_port)).await {
                        Ok(stream) => {
                            robots.insert(new_port, Some(split(stream).1));
                            return Some(new_port);
                        }
                        Err(_) => {
                            robot_ports.remove(next_robot_idx);
                            robots.remove(&new_port);
                        }
                    }
                }
            }
        } else {
            error("Unexpected error. Failed to find next_robot - found_next_port.");
        }
        None
    }

    ///Sets next robot to self if it is the only robot left.
    async fn self_is_next_robot(
        robots: &mut ActiveRobots,
        robot_ports: &mut [Port],
    ) -> Option<Port> {
        if let Some(active_robot) = robots.get(&robot_ports[0]) {
            return Some(match active_robot {
                Some(_) => robot_ports[0],
                None => {
                    if let Ok(stream) = TcpStream::connect(addr(robot_ports[0])).await {
                        robots.insert(robot_ports[0], Some(split(stream).1));
                        robot_ports[0]
                    } else {
                        error("No healthy robots left :(");
                        return None;
                    }
                }
            });
        } else {
            error("Unexpected error. Failed to find next_robot.");
        }
        None
    }
}
