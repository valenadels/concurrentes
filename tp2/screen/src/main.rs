use screen::screens::errors::ScreenError;
use screen::screens::recv_screen::RecvScreen;
use screen::screens::send_screen::SendScreen;
use screen::utils::util::{
    current_controller_port, current_orders_path, current_screen_port, retrieve_args_data,
};
use std::sync::Arc;
use tokio::io::split;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// The main function of the program.
/// This function is the entry point of the program. It reads command line arguments,
/// reads the ports from the properties file, initializes a new Screen instance, and starts it.
/// # Returns
/// A Result<(), ScreenError> indicating the success or failure of the function.
/// # Errors
/// This function will return an error if the properties file cannot be read, if the Screen
/// cannot be initialized, or if the Screen cannot be started.
#[actix::main]
async fn main() -> Result<(), ScreenError> {
    let params_data = retrieve_args_data()?;
    let controller_port = current_controller_port(&params_data)?;
    let screen_port = current_screen_port(&params_data)?;
    let orders_path = current_orders_path(&params_data)?;

    let (_, write_half) =
        split(TcpStream::connect(format!("localhost:{}", controller_port)).await?);
    let stream = Arc::new(Mutex::new(write_half));

    RecvScreen::start(screen_port, stream.clone()).await;

    let mut send_screen = SendScreen::new(stream).await;
    send_screen.start(&orders_path).await?;

    Ok(())
}
