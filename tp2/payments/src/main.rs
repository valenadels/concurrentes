use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::sync::Arc;

use actix::{Actor, Addr, StreamHandler};
use tokio::io::split;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_util::codec::FramedRead;

use payments::payments::codec::PaymentsCodec;
use payments::payments::errors::PaymentError;
use payments::payments::gateway::Gateway;
use payments::payments::get_orders::GetOrdersInProgress;
use payments::payments::messages::OrderId;
use payments::payments::order::Order;
use payments::payments::stop_gateway::StopGateway;

/// The path of the properties file that contains the port.
const PROPERTIES_FILE: &str = "conf/payments.properties";
/// The address of the local host.
const LOCALHOST: &str = "localhost";

/// The main function of the program.
/// This function is the entry point of the program. It reads the port from the properties file,
/// initializes a new TcpListener, and starts a Gateway actor for each new connection.
#[actix_rt::main]
async fn main() -> Result<(), PaymentError> {
    let ports = obtain_payments_config()?;
    let payments_port = *ports.first().ok_or(PaymentError::ParseError(
        "Port payments not found".to_string(),
    ))?;
    let listener = TcpListener::bind(address(payments_port)).await?;
    let mut orders_in_progress = Vec::new();
    let mut gateway: Option<Addr<Gateway>> = None;

    while let Ok((stream, addr)) = listener.accept().await {
        println!("[{:?}] New leader robot", addr);
        if let Some(gateway) = &gateway {
            orders_in_progress = gateway.send(GetOrdersInProgress).await.map_err(|_| {
                PaymentError::ActorError("Gateway: Could not get orders".to_string())
            })?;
            gateway.do_send(StopGateway)
        }

        gateway = Some(Gateway::create(|ctx| {
            let (read_half, _) = split(stream);
            Gateway::add_stream(FramedRead::new(read_half, PaymentsCodec), ctx);
            Gateway {
                leader_robot: Arc::new(Mutex::new(None)),
                addr,
                orders_in_progress: orders_to_hashmap(&orders_in_progress),
            }
        }));
    }

    Ok(())
}

/// Converts a vector of orders to a hashmap.
fn orders_to_hashmap(orders_vec: &Vec<Order>) -> HashMap<OrderId, Order> {
    let mut orders_hashmap = HashMap::new();
    for order in orders_vec {
        orders_hashmap.insert(order.id, order.clone());
    }
    orders_hashmap
}

/// Returns the address of the local host with the given port.
fn address(port: u16) -> String {
    format!("{}:{}", LOCALHOST, port)
}

/// Reads the port from the properties file.
/// # Returns
/// A Result<u16, PaymentError> indicating the success or failure of the function.
/// # Errors
/// This function will return an error if the properties file cannot be read or if the port cannot be parsed.
fn obtain_payments_config() -> Result<Vec<u16>, PaymentError> {
    let file = File::open(PROPERTIES_FILE)?;
    let reader = std::io::BufReader::new(file);
    let mut ports = Vec::new();
    for line in reader.lines() {
        ports.push(
            line?
                .split('=')
                .last()
                .ok_or(PaymentError::ParseError("Port not found".to_string()))?
                .parse::<u16>()?,
        );
    }
    Ok(ports)
}
