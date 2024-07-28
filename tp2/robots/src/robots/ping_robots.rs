use crate::{
    messages::{flavour_released::FlavourReleased, message::Message, ping::Ping},
    orders::flavours::Flavour,
    utils::log::{error, info},
};

use super::robot::{ActiveRobots, FlavoursStock, OrdersByRobot, Port, Robot};
use actix_rt::task;
use std::{sync::Arc, time::Duration};
use tokio::{
    io::AsyncWriteExt,
    sync::{
        mpsc::{channel, Receiver, Sender},
        RwLock,
    },
};

use std::thread;

impl Robot {
    pub async fn ping_robots(
        mut rx: Receiver<FlavoursStock>,
        robots: Arc<RwLock<ActiveRobots>>,
        pending_orders: Arc<RwLock<OrdersByRobot>>,
        arc_next_robot: Arc<RwLock<Port>>,
        id: Port,
    ) {
        let mut max_tries = 5;
        let mut current_tries = 0;
        let mut last_flavours = Flavour::initial_flavours();
        let (tx2, mut rx2): (Sender<FlavoursStock>, Receiver<FlavoursStock>) = channel(1);

        task::spawn_blocking(move || loop {
            if let Ok(flavours) = rx.try_recv() {
                last_flavours = flavours;
            } else {
                current_tries += 1;
                if current_tries >= max_tries {
                    let _ = tx2.try_send(last_flavours.clone());
                    current_tries = 0;
                    max_tries += 5;
                }
            }

            thread::sleep(Duration::from_secs(1));
        });

        tokio::spawn(async move {
            while let Some(flavours) = rx2.recv().await {
                let mut robots = robots.write().await;
                let mut dead_robots = vec![];
                for (port, stream) in robots.iter_mut() {
                    if let Some(ref mut stream) = stream {
                        let ping = Ping;
                        let bytes = ping.to_bytes();
                        if stream.write_all(&bytes).await.is_err() {
                            dead_robots.push(*port);
                            error(&format!("Lost connection with robot {}", port));
                        }
                    }
                }

                let mut next_robot = arc_next_robot.write().await;
                let mut pending_orders = pending_orders.write().await;
                if !dead_robots.is_empty() {
                    info("Handling dead robots by asingning their orders to other robots.");
                    Robot::handle_dead_robots(
                        &mut robots,
                        dead_robots,
                        &mut pending_orders,
                        &mut next_robot,
                        id,
                    )
                    .await;
                }
                if let Some(Some(ref mut stream)) = robots.get_mut(&next_robot) {
                    let ping = FlavourReleased { flavours };
                    let bytes = ping.to_bytes();
                    if stream.write_all(&bytes).await.is_err() {
                        error(&format!("Lost connection with robot {}", next_robot));
                        Robot::handle_dead_robot(
                            &mut robots,
                            *next_robot,
                            &mut pending_orders,
                            &mut next_robot,
                            id,
                        )
                        .await;
                    }
                }
                drop(pending_orders);
                drop(robots);
                drop(next_robot);
            }
        });
    }
}
