use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use rand::{rngs::SmallRng, seq::SliceRandom, Rng, SeedableRng};

use interface::SSet;
use skiplist_sset::SkipListSSet;
use treap::Treap;

struct MyBTreeSet<T>(BTreeSet<T>);
impl<T> SSet<T> for MyBTreeSet<T>
where
    T: Ord,
{
    fn size(&self) -> usize {
        unreachable!()
    }
    fn add(&mut self, x: T) -> bool {
        self.0.insert(x)
    }
    fn remove(&mut self, x: &T) -> bool {
        self.0.remove(x)
    }
    fn find(&self, _: &T) -> Option<&T> {
        unreachable!()
    }
}

fn add_remove<T>(mut set: impl SSet<T>, a: Vec<T>, b: Vec<T>) -> Duration {
    let now = Instant::now();
    for a in a {
        set.add(a);
    }
    for b in b {
        set.remove(&b);
    }
    now.elapsed()
}

fn main() {
    let mut rng = SmallRng::seed_from_u64(1223334);
    let n = 200_000;
    let mut a = vec![0_i64; n];
    rng.fill(&mut a[..]);
    // b: 半分が a の要素、半分がランダム
    let mut b: Vec<i64> = a
        .iter()
        .copied()
        .take(n / 2)
        .chain(std::iter::repeat_with(|| rng.gen()).take(n / 2))
        .collect();
    b.shuffle(&mut rng);

    let elapsed = add_remove(MyBTreeSet(BTreeSet::new()), a.clone(), b.clone());
    println!("std::collections::BTreeSet {} ms", elapsed.as_millis());

    let elapsed = add_remove(SkipListSSet::new(), a.clone(), b.clone());
    println!("SkipListSSet {} ms", elapsed.as_millis());

    let elapsed = add_remove(Treap::new(), a.clone(), b.clone());
    println!("Treap {} ms", elapsed.as_millis());

    // std::collections::BTreeSet 41 ms
    // SkipListSSet 429 ms
    // Treap 164 ms
}
