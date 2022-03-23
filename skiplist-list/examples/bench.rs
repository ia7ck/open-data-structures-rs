use std::time::Instant;

use rand::{rngs::SmallRng, Rng, SeedableRng};

use dllist::DLList;
use interface::List;
use skiplist_list::SkipListList;

fn main() {
    let mut rng = SmallRng::seed_from_u64(122333);
    let n = 500_00;
    let mut a = Vec::new();
    for i in 0..n {
        a.push(rng.gen_range(0..=i));
    }

    let mut dl_list = DLList::new();
    let now = Instant::now();
    for &a in &a {
        dl_list.add(a, a);
    }
    for &a in a.iter().rev() {
        dl_list.remove(a);
    }
    println!("DLList {} ms", now.elapsed().as_millis());

    let mut v = Vec::new();
    let now = Instant::now();
    for &a in &a {
        v.insert(a, a);
    }
    for &a in a.iter().rev() {
        v.remove(a);
    }
    println!("std::vec::Vec {} ms", now.elapsed().as_millis());

    let mut skip_list = SkipListList::new();
    let now = Instant::now();
    for &a in &a {
        skip_list.add(a, a);
    }
    for &a in a.iter().rev() {
        skip_list.remove(a);
    }
    println!("SkipListList {} ms", now.elapsed().as_millis());

    // DLList 5038 ms
    // std::vec::Vec 160 ms
    // SkipListList 48 ms
}
