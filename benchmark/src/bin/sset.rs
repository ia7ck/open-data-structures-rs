use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use binary_trie::{BinaryTrie, IntValue};
use rand::{rngs::SmallRng, Rng, SeedableRng};

use interface::SSet;
use scapegoat_tree::ScapegoatTree;
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

fn run<T>(label: &str, a: Vec<T>, b: Vec<T>)
where
    T: Clone + Ord + IntValue,
{
    let elapsed = add_remove(MyBTreeSet(BTreeSet::new()), a.clone(), b.clone());
    println!(
        "[{}] std::collections::BTreeSet {} ms",
        label,
        elapsed.as_millis()
    );

    let elapsed = add_remove(SkipListSSet::new(), a.clone(), b.clone());
    println!("[{}] SkipListSSet {} ms", label, elapsed.as_millis());

    let elapsed = add_remove(Treap::new(), a.clone(), b.clone());
    println!("[{}] Treap {} ms", label, elapsed.as_millis());

    let elapsed = add_remove(ScapegoatTree::new(), a.clone(), b.clone());
    println!("[{}] ScapegoatTree {} ms", label, elapsed.as_millis());

    let elapsed = add_remove(BinaryTrie::new(), a.clone(), b.clone());
    println!("[{}] BinaryTrie {} ms", label, elapsed.as_millis());
}

fn main() {
    let mut rng = SmallRng::seed_from_u64(1223334);

    let n = 200_000;
    let m = 200_000_u32;

    let mut a = vec![0; n];
    let mut b = vec![0; n];
    for i in 0..n {
        a[i] = rng.gen_range(0..m);
        b[i] = rng.gen_range(0..m);
    }

    run("random", a, b);
    run("sorted", (0..m).collect(), (0..m).collect());

    // メモリ確保・解放の時間が多くを占めている気がする……

    // [random] std::collections::BTreeSet 60 ms
    // [random] SkipListSSet 1908 ms
    // [random] Treap 309 ms
    // [random] ScapegoatTree 443 ms
    // [random] BinaryTrie 508 ms

    // [sorted] std::collections::BTreeSet 78 ms
    // [sorted] SkipListSSet 1945 ms
    // [sorted] Treap 302 ms
    // [sorted] ScapegoatTree 8309 ms
    // [sorted] BinaryTrie 700 ms
}
