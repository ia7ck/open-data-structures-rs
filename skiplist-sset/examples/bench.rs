use std::collections::BTreeSet;
use std::time::Instant;

use rand::{rngs::SmallRng, Rng, SeedableRng};

use interface::SSet;
use skiplist_sset::SkipListSSet;

fn main() {
    let mut rng = SmallRng::seed_from_u64(122333);
    let n = 200_000;
    let mut a = Vec::new();
    let mut b = Vec::new();
    for _ in 0..n {
        a.push(rng.gen_range(0..n));
        b.push(rng.gen_range(0..n));
    }

    let mut set = BTreeSet::new();
    let now = Instant::now();
    for &a in &a {
        set.insert(a);
    }
    for b in &b {
        set.remove(b);
    }
    println!(
        "std::collections::BTreeSet {} ms",
        now.elapsed().as_millis()
    );

    let mut set = SkipListSSet::new();
    let now = Instant::now();
    for &a in &a {
        set.add(a);
    }
    for b in &b {
        set.remove(b);
    }
    println!("SkipListSSet {} ms", now.elapsed().as_millis());

    // std::collections::BTreeSet 33 ms
    // SkipListSSet 5853 ms
}
