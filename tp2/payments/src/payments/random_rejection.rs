use rand::Rng;

/// Probability of declining an order. Should be a number between 0 and 1.
const PROBABILITY: f64 = 0.2;

/// Returns true if the order should be declined
/// based on the probability of declining an order.
pub fn should_decline_order() -> bool {
    let mut rng = rand::thread_rng();
    let random_value: f64 = rng.gen_range(0.0..1.0);
    random_value < PROBABILITY
}
