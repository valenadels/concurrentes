use std::io::Error;
use std::sync::Arc;

use actix::fut::wrap_future;
use actix::{Actor, Context, ContextFutureSpawner, StreamHandler};
use tokio::io::{split, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task;
use tokio_util::bytes::{Buf, BytesMut};
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::screens::new_leader::{NewLeader, NEW_LEADER_ID};

/// Struct to receive messages from the controller and with a tx channel to send
/// messages to the SendScreen.
pub struct RecvScreen {
    stream: Arc<Mutex<WriteHalf<TcpStream>>>,
}

/// Implementation of the Actor trait for the RecvScreen struct.
impl Actor for RecvScreen {
    type Context = Context<Self>;
}

/// Implementation of the StreamHandler trait for the RecvScreen struct.
/// This trait is used to handle the messages received from the controller.
/// If the message is a Coordinator message, it is sent the new coordinator port to the SendScreen.
/// # Arguments
/// * `Result<BytesMut, Error>` - The result of the message received.
/// * `Self::Context` - The context of the RecvScreen.
impl StreamHandler<Result<BytesMut, Error>> for RecvScreen {
    fn handle(&mut self, read: Result<BytesMut, Error>, ctx: &mut Self::Context) {
        if let Ok(mut bytes) = read {
            match bytes.get_u8() {
                NEW_LEADER_ID => {
                    let arc_stream = self.stream.clone();
                    let msg = NewLeader::from_bytes(&mut bytes);

                    wrap_future::<_, Self>(async move {
                        let mut stream = arc_stream.lock().await;
                        let (_, write_half) = split(
                            TcpStream::connect(format!("localhost:{}", msg.port))
                                .await
                                .expect("Failed to connect to new leader."),
                        );
                        *stream = write_half;
                        println!(
                            "Successfully updated leader's port. Resuming order processing..."
                        );
                    })
                    .spawn(ctx);
                }
                id => {
                    println!("Received unexpected message with ID {}", id)
                }
            }
        }
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {}
}

impl RecvScreen {
    /// Creates a new RecvScreen instance. Start listening from the port (controlelr port) and
    /// listens for new messages.
    /// # Arguments
    /// * `port` - The port to listen to.
    /// * `tx` - The Sender channel to send messages to the SendScreen.
    pub async fn start(port: u16, write_half: Arc<Mutex<WriteHalf<TcpStream>>>) {
        let listener = TcpListener::bind(format!("localhost:{}", port))
            .await
            .expect("Failed to establish listener.");

        task::spawn_local(async move {
            while let Ok((stream, _)) = listener.accept().await {
                RecvScreen::create(|ctx| {
                    println!("Received new connection!");
                    let (read_half, _) = split(stream);
                    RecvScreen::add_stream(FramedRead::new(read_half, BytesCodec::new()), ctx);
                    RecvScreen {
                        stream: write_half.clone(),
                    }
                });
            }
        });
    }
}
