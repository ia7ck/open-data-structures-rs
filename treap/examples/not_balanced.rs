use rand::{rngs::SmallRng, Rng, SeedableRng};

use interface::SSet;
use treap::Treap;

fn main() {
    let mut rng = SmallRng::seed_from_u64(122333);
    let n = 10_000;
    let mut a = vec![0_i64; n];
    rng.fill(&mut a[..]);

    let mut treap = Treap::new();
    for a in a {
        treap.add(a);
    }
    println!("height = {}, size = {}", treap.height(), treap.size());

    // height = 5054, size = 10000
    // ;_;
}
