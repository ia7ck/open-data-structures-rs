use std::time::{Duration, Instant};

use rand::{rngs::SmallRng, Rng, SeedableRng};

use interface::Stack;
use sllist::SLList;

struct VecAsStack<T>(Vec<T>);
impl<T> Stack<T> for VecAsStack<T> {
    fn push(&mut self, x: T) {
        self.0.push(x);
    }
    fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }
}

fn run<T>(mut stack: impl Stack<T>, a: Vec<T>) -> Duration {
    let now = Instant::now();
    let n = a.len();
    for a in a {
        stack.push(a);
    }
    for _ in 0..n {
        stack.pop();
    }
    now.elapsed()
}

fn main() {
    let mut rng = SmallRng::seed_from_u64(122333);
    let n = 5_000_000;
    let mut a = vec![0_i64; n];
    rng.fill(&mut a[..]);

    let elapsed = run(VecAsStack(Vec::new()), a.clone());
    println!("std::vec::Vec {} ms", elapsed.as_millis());

    let elapsed = run(SLList::new(), a.clone());
    println!("SLList {} ms", elapsed.as_millis());

    // std::vec::Vec 26 ms
    // SLList 381 ms
}
