use std::time::{Duration, Instant};

use rand::{rngs::SmallRng, Rng, SeedableRng};

use dllist::DLList;
use interface::List;
use skiplist_list::SkipListList;

struct VecAsList<T>(Vec<T>);
impl<T> List<T> for VecAsList<T> {
    fn size(&self) -> usize {
        unreachable!()
    }
    fn get(&self, _: usize) -> Option<&T> {
        unreachable!()
    }
    fn set(&self, _: usize, _: T) -> T {
        unreachable!()
    }
    fn add(&mut self, i: usize, x: T) {
        self.0.insert(i, x);
    }
    fn remove(&mut self, i: usize) -> T {
        self.0.remove(i)
    }
}

fn add_remove<T>(mut list: impl List<T>, a: Vec<usize>, b: Vec<T>) -> Duration {
    let now = Instant::now();
    for (&a, b) in a.iter().zip(b.into_iter()) {
        list.add(a, b);
    }
    for &a in a.iter().rev() {
        list.remove(a);
    }
    now.elapsed()
}

fn main() {
    let mut rng = SmallRng::seed_from_u64(122333);
    let n = 50_000;
    let mut a = Vec::new();
    for i in 0..n {
        a.push(rng.gen_range(0..=i));
    }
    let mut b = vec![0_u64; n];
    rng.fill(&mut b[..]);

    let elapsed = add_remove(DLList::new(), a.clone(), b.clone());
    println!("DLList {} ms", elapsed.as_millis());

    let elapsed = add_remove(VecAsList(Vec::new()), a.clone(), b.clone());
    println!("std::vec::Vec {} ms", elapsed.as_millis());

    let elapsed = add_remove(SkipListList::new(), a.clone(), b.clone());
    println!("SkipListList {} ms", elapsed.as_millis());

    // DLList 5038 ms
    // std::vec::Vec 160 ms
    // SkipListList 48 ms
}
