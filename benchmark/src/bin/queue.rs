use std::collections::VecDeque;
use std::time::{Duration, Instant};

use rand::{rngs::SmallRng, Rng, SeedableRng};

use interface::Queue;
use sllist::SLList;

struct VecAsQueue<T>(Vec<T>);
impl<T> Queue<T> for VecAsQueue<T> {
    fn add(&mut self, x: T) {
        self.0.push(x);
    }
    fn remove(&mut self) -> Option<T> {
        if self.0.is_empty() {
            None
        } else {
            let result = self.0.remove(0);
            Some(result)
        }
    }
}

struct VecDequeAsQueue<T>(VecDeque<T>);
impl<T> Queue<T> for VecDequeAsQueue<T> {
    fn add(&mut self, x: T) {
        self.0.push_back(x);
    }
    fn remove(&mut self) -> Option<T> {
        self.0.pop_front()
    }
}

fn run<T>(mut queue: impl Queue<T>, a: Vec<T>) -> Duration {
    let now = Instant::now();
    let n = a.len();
    for a in a {
        queue.add(a);
    }
    for _ in 0..n {
        queue.remove();
    }
    now.elapsed()
}

fn main() {
    let mut rng = SmallRng::seed_from_u64(122333);
    let n = 500_000;
    let mut a = vec![0_i64; n];
    rng.fill(&mut a[..]);

    let elapsed = run(VecAsQueue(Vec::new()), a.clone());
    println!("std::vec::Vec {} ms", elapsed.as_millis());

    let elapsed = run(VecDequeAsQueue(VecDeque::new()), a.clone());
    println!("std::collections::VecDeque {} ms", elapsed.as_millis());

    let elapsed = run(SLList::new(), a.clone());
    println!("SLList {} ms", elapsed.as_millis());

    // std::vec::Vec 74067 ms
    // std::collections::VecDeque 3 ms
    // SLList 38 ms
}
