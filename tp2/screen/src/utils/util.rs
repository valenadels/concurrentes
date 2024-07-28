use crate::screens::errors::ScreenError;
use std::collections::HashMap;
use std::env::current_dir;
use std::fs::File;
use std::io::{BufRead, BufReader};

const PROPERTIES_PATH: &str = "conf/screen.properties";
const DESIRED_PORTS: usize = 2;

/// Helper enum to store different types of values in a HashMap.
#[derive(Debug)]
pub enum Value {
    U16(u16),
    U8(u8),
    Text(String),
}

/// Reads the properties file and returns the ports associated with the screen id.
/// # Arguments
/// - screen_id: The screen id to search for.
/// # Returns
/// A vector with the ports associated with the screen id.
/// # Errors
/// Returns a ScreenError if the file cannot be read or the ports are not found.
fn ports_from_id(screen_id: u8) -> Result<Vec<u16>, ScreenError> {
    let properties_file = File::open(PROPERTIES_PATH)?;
    let reader = BufReader::new(properties_file);
    let mut ports = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let mut parts = line.split('=');
        if let (Some(id_part), Some(port_part)) = (parts.next(), parts.next()) {
            let mut add_value = false;
            if id_part.eq("controller-port") {
                add_value = true
            } else if let Ok(id) = id_part.parse::<u8>() {
                if id.eq(&screen_id) {
                    add_value = true;
                }
            }
            if add_value {
                let port = port_part.parse::<u16>()?;
                ports.push(port);
            }
        }
    }

    if ports.len() < DESIRED_PORTS {
        Err(ScreenError::PortNotFound)
    } else {
        Ok(ports)
    }
}

/// Formats the arguments passed to the program and returns a HashMap with:
/// - screen_id: The screen id.
/// - orders_path: The path to the orders file.
/// - screen_port: The port to listen for messages.
/// - controller_port: The port of the initial controller.
/// # Arguments
/// - args: The arguments passed to the program.
/// # Returns
/// A HashMap with the values passed as arguments.
/// # Errors
/// Returns a ScreenError if the arguments are not passed correctly or the parse fails.
fn format_params(args: Vec<String>) -> Result<HashMap<String, Value>, ScreenError> {
    let mut map: HashMap<String, Value> = HashMap::new();
    let orders_path: String = args[1].parse::<String>()?;
    let screen_id: u8 = args[2].parse::<u8>()?;
    let ports = ports_from_id(screen_id)?;
    map.insert("screen_id".to_string(), Value::U8(screen_id));
    map.insert("orders_path".to_string(), Value::Text(orders_path));
    map.insert("screen_port".to_string(), Value::U16(ports[0]));
    map.insert("controller_port".to_string(), Value::U16(ports[1]));
    Ok(map)
}

/// Reads the arguments passed to the program and returns a HashMap with the values.
/// # Returns
/// A HashMap with the values passed as arguments.
/// # Errors
/// Returns a ScreenError if the arguments are not passed correctly or the parse fails.
pub fn retrieve_args_data() -> Result<HashMap<String, Value>, ScreenError> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 2 {
        format_params(args)
    } else {
        Err(ScreenError::ParseError(
            "Failed to read params.".to_string(),
        ))
    }
}

/// Returns the screen id obtained from the passed parameters.
pub fn current_screen_id(map: &HashMap<String, Value>) -> Result<u8, ScreenError> {
    match map.get("screen_id") {
        Some(Value::U8(v)) => Ok(*v),
        _ => Err(ScreenError::ParseError(
            "Failed to get screen_id".to_string(),
        )),
    }
}

/// Returns the orders path obtained from the passed parameters
pub fn current_orders_path(map: &HashMap<String, Value>) -> Result<String, ScreenError> {
    let cur_dir = current_dir()?;
    let cur_dir_str = cur_dir.to_str().expect("Failed to get current directory.");

    match map.get("orders_path") {
        Some(Value::Text(v)) => Ok(format!("{}{}", cur_dir_str, v.clone())),
        _ => Err(ScreenError::ParseError(
            "Failed to get orders_path".to_string(),
        )),
    }
}

/// Returns the screen port obtained from the passed parameters
pub fn current_screen_port(map: &HashMap<String, Value>) -> Result<u16, ScreenError> {
    match map.get("screen_port") {
        Some(Value::U16(v)) => Ok(*v),
        _ => Err(ScreenError::ParseError(
            "Failed to get screen_port".to_string(),
        )),
    }
}

/// Returns the controller port obtained from the passed parameters
pub fn current_controller_port(map: &HashMap<String, Value>) -> Result<u16, ScreenError> {
    match map.get("controller_port") {
        Some(Value::U16(v)) => Ok(*v),
        _ => Err(ScreenError::ParseError(
            "Failed to get controller_port".to_string(),
        )),
    }
}
