use rand::{rngs::SmallRng, RngCore, SeedableRng};

use interface::SSet;
use treap::Treap;

fn main() {
    let mut rng = SmallRng::seed_from_u64(122333); // Treap 内部で指定した seed と同じ値
    let n = 10_000;
    let mut a = Vec::new();
    for _ in 0..n {
        a.push(rng.next_u64());
    }

    let mut treap = Treap::new();
    for a in a {
        treap.add(a);
    }
    println!("height = {}, size = {}", treap.height(), treap.size());

    // height = 9999, size = 10000
}
