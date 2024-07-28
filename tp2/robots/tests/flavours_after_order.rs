use std::fs::File;
use std::io::{BufRead, BufReader};

use serde::Deserialize;
use serde_json::from_str;

use robot::orders::container::Container;
use robot::orders::flavours::Flavour;
use robot::orders::order::Order;
use robot::robots::errors::RobotError;
use robot::robots::order::retrieve_updated_flavours;

#[derive(Debug, Deserialize)]
struct ContainerTest {
    pub size: u16,
    pub flavours: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OrderTest {
    pub id: u16,
    pub containers: Vec<ContainerTest>,
}

fn convert_to_order(order_test: OrderTest) -> Order {
    let mut containers: Vec<Container> = vec![];

    for container in order_test.containers {
        let mut flavours: Vec<u8> = vec![];

        for flavour in &container.flavours {
            flavours.push(flavour_to_bytes(flavour));
        }

        containers.push(Container::new(container.size, flavours));
    }

    Order::new(order_test.id, containers)
}

fn flavour_to_bytes(flavour: &str) -> u8 {
    match flavour {
        "vanilla" => 0,
        "chocolate" => 1,
        "strawberry" => 2,
        "cookies" => 3,
        _ => {
            println!("Flavour unknown with ID {}", flavour);
            0
        }
    }
}

fn get_orders_from_path(orders_path: &str) -> Result<Vec<Order>, RobotError> {
    let file = File::open(orders_path)?;
    let reader = BufReader::new(file);
    let mut orders = vec![];

    for l in reader.lines() {
        let line = l?;
        let order: OrderTest = from_str(&line).expect("Error parsing JSON");
        let new_order = convert_to_order(order);
        orders.push(new_order);
    }

    Ok(orders)
}

#[test]
fn test_flavours_order_1() -> Result<(), RobotError> {
    let orders_path = "test-files/order_payments_declined.jsonl";
    let orders = get_orders_from_path(orders_path)?;

    let mut flavours = Flavour::initial_flavours();
    let mut order_can_be_prepared = true;

    for order in orders {
        retrieve_updated_flavours(&order.containers, &mut flavours, &mut order_can_be_prepared);
    }

    // {Strawberry: 7625, Cookies: 4625, Chocolate: 5125, Vanilla: 5625} => Pasada del robot
    assert_eq!(flavours.get(&Flavour::Strawberry), Some(&7625));
    assert_eq!(flavours.get(&Flavour::Cookies), Some(&4625));
    assert_eq!(flavours.get(&Flavour::Chocolate), Some(&5125));
    assert_eq!(flavours.get(&Flavour::Vanilla), Some(&5625));

    Ok(())
}
