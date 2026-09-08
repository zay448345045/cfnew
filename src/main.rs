use ring::digest::{digest, SHA512};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

const PREFIX: &str = "2c04f018-";
const SUFFIX: &str = "cf75fef9";
static COUNTER: AtomicU64 = AtomicU64::new(0);

fn check(candidate: &str) -> bool {
    let hash = digest(&SHA512, candidate.as_bytes());
    let b = hash.as_ref();
    // 前32bit全部为0
// -------- 测试模式：只要求哈希第一个字节最高5bit=0
    // 0b00000xxx 最高5位是0
  //  return (b[0] & 0xF8) == 0;

    if b[0] != 0 || b[1] != 0 || b[2] != 0 || b[3] != 0 {
        return false;
    }
    // 第33bit为0
    (b[4] & 0x80) == 0

}

fn worker(shard_seed: u64) {
    let mut rng = StdRng::seed_from_u64(shard_seed);
    loop {
        let p1: u16 = rng.gen();
        let p2: u16 = rng.gen();
        let p3: u16 = rng.gen_range(0..0x1000);
        let mid = format!("{p1:04x}-{p2:04x}-{p3:03x}");
        let candidate = format!("{PREFIX}{mid}{SUFFIX}");

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

    let num_threads = num_cpus::get();
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