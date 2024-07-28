use serde_json::from_str;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::screens::errors::ScreenError;
use crate::screens::new_order::NewOrder;
use crate::screens::order::Order;

/// Struct to send orders to the controller.
#[derive(Clone)]
pub struct SendScreen {
    stream: Arc<Mutex<WriteHalf<TcpStream>>>,
}

impl SendScreen {
    /// Creates a new SendScreen instance.
    /// # Arguments
    /// * `controller_port` - The port of the controller.
    /// * `tx` - The Sender channel to send messages to the RecvScreen.
    /// # Returns
    /// A Result<SendScreen, ScreenError> indicating the success or failure of the function.
    pub async fn new(stream: Arc<Mutex<WriteHalf<TcpStream>>>) -> Self {
        SendScreen { stream }
    }

    /// Starts the SendScreen instance.
    /// # Arguments
    /// * `rx` - The Receiver channel to receive messages from the RecvScreen.
    /// * `orders_path` - The path to the orders file.
    /// # Returns
    /// A Result<(), ScreenError> indicating the success or failure of the function.
    pub async fn start(&mut self, orders_path: &str) -> Result<(), ScreenError> {
        self.process_orders(orders_path).await?;
        println!("Sent orders successfully");
        Ok(())
    }

    /// Processes the orders from the orders file.
    /// # Arguments
    /// * `orders_path` - The path to the orders file.
    /// # Returns
    /// A Result<(), ScreenError> indicating the success or failure of the function.
    async fn process_orders(&mut self, orders_path: &str) -> Result<(), ScreenError> {
        let file = File::open(orders_path)?;
        let reader = BufReader::new(file);

        for l in reader.lines() {
            let line = l?;
            let order: Order = from_str(&line)?;
            sleep(Duration::from_secs(1)).await;
            println!("Sending order: {:?}", order);
            self.process_order(order).await?;
        }

        Ok(())
    }

    /// Processes an order and sends it to the controller. Each order is retried up to 3 times.
    /// If the order cannot be sent after 3 tries, a ControllerConnectionError is returned.
    /// # Arguments
    /// * `order` - The order to process.
    /// # Returns
    /// A Result<(), ScreenError> indicating the success or failure of the function.
    async fn process_order(&mut self, order: Order) -> Result<(), ScreenError> {
        let mut tries = 0;

        while tries < 5 {
            let mut guard = self.stream.lock().await;
            let new_order = NewOrder::new(order.clone());
            match guard.write(&new_order.to_bytes()).await {
                Ok(_) => return Ok(()),
                Err(_) => {
                    println!("Failed to send order. Retrying...");
                    tries += 1;
                }
            }
        }

        Err(ScreenError::ControllerConnectionError)
    }
}
