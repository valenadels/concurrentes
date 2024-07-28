use crate::robots::errors::RobotError;
use crate::robots::robot::Port;
use std::env::current_dir;
use std::fs::File;
use std::io::{BufRead, BufReader};

const CONFIG_LINES: usize = 3;
const PROPERTIES_FILE_PATH: &str = "/conf/properties.conf";
const PAYMENTS_PORT_INDEX: usize = 2;
const PAYMENTS_DEFAULT_PORT: Port = 4000;

/// Reads all the ports from the config file.
pub fn read_all_ports_config() -> Result<Vec<Vec<Port>>, RobotError> {
    let cur_dir = current_dir()?;
    let cur_dir_str = match cur_dir.to_str() {
        Some(dir) => dir,
        None => return Err(RobotError::ParseError("Invalid directory".to_string())),
    };
    let file = File::open(format!("{}{}", cur_dir_str, PROPERTIES_FILE_PATH))?;
    let mut lines = BufReader::new(file).lines();
    let mut ports = Vec::new();
    while let Some(Ok(line)) = lines.next() {
        let port = line
            .split(',')
            .map(|port| port.parse::<Port>())
            .collect::<Result<Vec<Port>, _>>()?;
        ports.push(port);
    }

    if ports.len() != CONFIG_LINES {
        return Err(RobotError::ParseError(
            "Config file must have 3 lines".to_string(),
        ));
    }
    Ok(ports)
}

/// Returns the localhost address with the given port.
pub fn addr(port: Port) -> String {
    format!("localhost:{}", port)
}

/// Reads payment port from config. if it is not defined, returns default one (4000).
pub fn read_payments_port() -> Port {
    let payments_port = match read_all_ports_config() {
        Ok(ports) => match ports.get(PAYMENTS_PORT_INDEX) {
            Some(port_slice) => {
                if let Some(first_port) = port_slice.first() {
                    *first_port
                } else {
                    PAYMENTS_DEFAULT_PORT
                }
            }
            None => PAYMENTS_DEFAULT_PORT,
        },
        Err(_) => PAYMENTS_DEFAULT_PORT,
    };
    payments_port
}
