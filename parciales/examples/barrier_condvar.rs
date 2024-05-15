/* Ej recu 2c2023
Variables necesarias
- cant de threads a esperar
- contador de la cant de hilos que hicieron wait -> mutex pq lo van a modificar varios hilos a la vez
- condvar para esperar a que todos los threads lleguen
 */

use std::ops::Add;
use std::sync::{Condvar, Mutex};

pub struct Barrier {
    n: usize,
    count: Mutex<usize>,
    condvar: Condvar,
}

impl Barrier {
    pub fn new(n: usize) -> Barrier {
        Barrier {
            n,
            count: Mutex::new(0),
            condvar: Condvar::new(),
        }
    }

    pub fn wait(&self) {
        let mut lock = self.count.lock().expect("Lock error");
        lock.add(1);
        if lock.eq(&self.n) {
            self.condvar.notify_all();
        } else {
            let _ = self
                .condvar
                .wait_while(lock, |lock| lock.eq(&self.n))
                .expect("Error");
        }
    }
}

fn main() {}
