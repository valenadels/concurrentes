use std::env::args;

use robot::robots::errors::RobotError;
use robot::robots::robot::{Port, Robot};
use robot::utils::util::read_all_ports_config;

const ARGS_SIZE: usize = 2;

const ROBOT_PORT: usize = 1;

#[actix_rt::main]
async fn main() -> Result<(), RobotError> {
    let args: Vec<String> = args().collect();
    if args.len() < ARGS_SIZE {
        return Err(RobotError::InvalidArguments);
    }
    let port = args[ROBOT_PORT].parse::<Port>()?;
    let all_ports = read_all_ports_config()?;
    let robots_ports = all_ports[0].clone();
    let screen_ports = all_ports[1].clone();
    let payments_port = *all_ports[2].first().ok_or(RobotError::ParseError(
        "Payments port not found".to_string(),
    ))?;
    Robot::start(port, robots_ports, screen_ports, payments_port).await?;
    Ok(())
}
