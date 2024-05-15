use std::ops::{Add, Sub};
use std::sync::{Condvar, Mutex};

/*
Ej parcial 2c2023
 */
struct Semaphore {
    value: Mutex<i16>,
    condvar: Condvar,
}

impl Semaphore {
    pub fn new(initial_value: i16) -> Semaphore {
        Semaphore {
            value: Mutex::new(initial_value),
            condvar: Condvar::new(),
        }
    }

    pub fn wait(&self) {
        let mut lock = self
            .condvar
            .wait_while(self.value.lock().expect(""), |v| *v >= 0)
            .expect("");
        lock.sub(1);
    }

    pub fn signal(&self) {
        let lock = self.value.lock().unwrap();
        lock.add(1);
        if lock.eq(&0) || lock.is_positive() {
            self.condvar.notify_one()
        }
    }
}

fn main() {}
