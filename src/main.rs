use ring::digest::{digest, SHA512};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn check(candidate: &str) -> bool {
    let hash = digest(&SHA512, candidate.as_bytes());
    let b = hash.as_ref();
    if b[0] != 0 || b[1] != 0 || b[2] != 0 || b[3] != 0 {
        return false;
    }
    (b[4] & 0x80) == 0
}

fn worker(shard_seed: u64) {
    let mut rng = StdRng::seed_from_u64(shard_seed);
    loop {
        let p1: u16 = rng.gen();
        let p2: u16 = rng.gen();
        let p3: u16 = rng.gen();
        let p4: u16 = rng.gen();

        let seg1 = "2c04f018";
        let seg2 = format!("{p1:04x}");
        let seg3 = format!("{p2:04x}");
        let seg4 = format!("{p3:04x}");
        let seg5 = format!("{p4:04x}2b049b96");

        let candidate = format!("{seg1}-{seg2}-{seg3}-{seg4}-{seg5}");

        if check(&candidate) {
            println!("!!!FOUND!!! /answer {candidate}");
            std::process::exit(0);
        }

        let cnt = COUNTER.fetch_add(1, Ordering::Relaxed);
        if cnt % 500_000 == 0 {
            eprintln!("Tried {} candidates", cnt);
        }
    }
}

fn main() {
    let shard_id: u64 = env::var("MATRIX_SHARD_ID")
        .unwrap_or_else(|_| "0".into())
        .parse()
        .unwrap_or(0);

    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);

    println!("shard_id={shard_id}, spawn {num_threads} worker threads");

    let mut handles = Vec::new();
    for t in 0..num_threads {
        let seed = shard_id * 1000 + t as u64;
        handles.push(thread::spawn(move || worker(seed)));
    }

    for h in handles {
        let _ = h.join();
    }
}