use rand::random;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::thread::sleep;
use std::time::Duration;

/*
Ej parcial 2c2023
 */
fn promedio_latencia(path: String, N: usize) -> u64 {
    let file = File::open(path).expect("error");
    let reader = BufReader::new(file);
    let mut sitios: Vec<String> = Vec::new(); //son 100
    for line in reader.lines() {
        sitios.push(line.unwrap())
    }

    let tp = rayon::ThreadPoolBuilder::new();
    tp.num_threads(N);

    sitios
        .par_iter()
        .map(|_| {
            let t: u64 = random();
            sleep(Duration::from_secs(t));
            t
        })
        .reduce(|| 0, |acc, t| acc + t)
        / 100
}

fn main() {}
