use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};

use interface::SSet;
use skiplist_sset::SkipListSSet;

fn main() {
    let mut rng = SmallRng::from_entropy();
    let mut chars: Vec<char> = "abcde".chars().collect();
    chars.shuffle(&mut rng);

    let mut set = SkipListSSet::new();
    for ch in chars {
        set.add(ch);
    }

    println!("{:?}", set);

    set.remove(&'c');
    println!("{:?}", set);

    set.remove(&'d');
    println!("{:?}", set);

    // ~	#######
    // 'a'	#
    // 'b'	#####
    // 'c'	######
    // 'd'	#######
    // 'e'	###
    //
    // ~	#######
    // 'a'	#
    // 'b'	#####
    // 'd'	#######
    // 'e'	###
    //
    // ~	#####
    // 'a'	#
    // 'b'	#####
    // 'e'	###
}
