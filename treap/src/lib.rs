use rand::{rngs::SmallRng, RngCore, SeedableRng};
use std::{
    alloc,
    cmp::{self, Ordering},
    fmt::{self, Formatter},
    ptr,
};

use interface::SSet;

struct Node<T> {
    x: T,
    priority: u64, // 小さいほうが根側に来るようにする
    parent: *mut Node<T>,
    left: *mut Node<T>,
    right: *mut Node<T>,
}

pub struct Treap<T> {
    n: usize,
    root: *mut Node<T>,
    rng: SmallRng,
}

impl<T> Treap<T> {
    pub fn new() -> Self {
        Self {
            n: 0,
            root: ptr::null_mut(),
            rng: SmallRng::seed_from_u64(122333),
        }
    }

    fn gen_priority(&mut self) -> u64 {
        self.rng.next_u64()
    }

    fn rotate_right(&mut self, u: *mut Node<T>) {
        //         u                      w
        //         |                      |
        //     +---+---+              +---+---+
        //     |       |              |       |
        //     w       c      ->      a       u
        //     |                              |
        // +---+---+                      +---+---+
        // |       |                      |       |
        // a       b                      b       c
        let w = unsafe { &*u }.left;
        debug_assert_ne!(w, ptr::null_mut());
        let p = unsafe { (*u).parent };
        if p == ptr::null_mut() {
            debug_assert_eq!(self.root, u);
            self.root = w;
            unsafe { (*w).parent = ptr::null_mut() };
        } else {
            unsafe { (*w).parent = p };
            if unsafe { (*p).left } == u {
                unsafe { (*p).left = w };
            } else {
                debug_assert_eq!(unsafe { (*p).right }, u);
                unsafe { (*p).right = w };
            }
        }
        unsafe { (*u).parent = w };
        let b = unsafe { (*w).right };
        if b == ptr::null_mut() {
            unsafe { (*u).left = ptr::null_mut() };
        } else {
            unsafe { (*b).parent = u };
            unsafe { (*u).left = b };
        }
        unsafe { (*w).right = u };
    }

    fn rotate_left(&mut self, u: *mut Node<T>) {
        //      u                         w
        //      |                         |
        //  +---+---+                 +---+---+
        //  |       |                 |       |
        //  a       w        ->       u       c
        //          |                 |
        //      +---+---+         +---+---+
        //      |       |         |       |
        //      b       c         a       b
        let w = unsafe { &*u }.right;
        debug_assert_ne!(w, ptr::null_mut());
        let p = unsafe { (*u).parent };
        if p == ptr::null_mut() {
            debug_assert_eq!(self.root, u);
            self.root = w;
            unsafe { (*w).parent = ptr::null_mut() };
        } else {
            unsafe { (*w).parent = p };
            if unsafe { (*p).left } == u {
                unsafe { (*p).left = w };
            } else {
                debug_assert_eq!(unsafe { (*p).right }, u);
                unsafe { (*p).right = w };
            }
        }
        unsafe { (*u).parent = w };
        let b = unsafe { (*w).left };
        if b == ptr::null_mut() {
            unsafe { (*u).right = ptr::null_mut() };
        } else {
            unsafe { (*b).parent = u };
            unsafe { (*u).right = b };
        }
        unsafe { (*w).left = u };
    }
}

impl<T> Treap<T>
where
    T: cmp::Ord,
{
    // - x に等しい要素を持つノードがあればそのノードを返す
    // - そうでなければ、x を探索する経路で最後に通ったノードを返す
    //   - ノードがもつ要素は x より小さいこともあれば大きいこともある
    // expected O(log(n)) time
    fn find_last(&self, x: &T) -> *mut Node<T> {
        let mut w = self.root;
        let mut prev = ptr::null_mut();
        while w != ptr::null_mut() {
            prev = w;
            match x.cmp(&unsafe { &*w }.x) {
                Ordering::Less => {
                    w = unsafe { &*w }.left;
                }
                Ordering::Greater => {
                    w = unsafe { &*w }.right;
                }
                Ordering::Equal => {
                    return w;
                }
            }
        }
        prev
    }

    // x を要素に持つノードを p の子として追加する
    // O(1) time
    fn add_child(&mut self, p: *mut Node<T>, x: T) -> *mut Node<T> {
        let u = if p == ptr::null_mut() {
            debug_assert_eq!(self.root, ptr::null_mut());
            self.root = Box::into_raw(Box::new(Node {
                x,
                priority: self.gen_priority(),
                parent: ptr::null_mut(),
                left: ptr::null_mut(),
                right: ptr::null_mut(),
            }));
            self.root
        } else {
            let y = &unsafe { &*p }.x;
            let ord = x.cmp(y);

            let u = Box::into_raw(Box::new(Node {
                x,
                priority: self.gen_priority(),
                parent: p,
                left: ptr::null_mut(),
                right: ptr::null_mut(),
            }));

            match ord {
                Ordering::Less => {
                    debug_assert_eq!(unsafe { &*p }.left, ptr::null_mut());
                    unsafe { (*p).left = u };
                    u
                }
                Ordering::Greater => {
                    debug_assert_eq!(unsafe { &*p }.right, ptr::null_mut());
                    unsafe { (*p).right = u };
                    u
                }
                Ordering::Equal => {
                    unreachable!();
                }
            }
        };

        self.n += 1;
        u
    }
}

impl<T> SSet<T> for Treap<T>
where
    T: cmp::Ord,
{
    // O(1) time
    fn size(&self) -> usize {
        self.n
    }

    // expected O(log(n)) time
    fn add(&mut self, x: T) -> bool {
        let p = self.find_last(&x);
        if p != ptr::null_mut() && unsafe { &*p }.x.eq(&x) {
            return false;
        }

        let u = self.add_child(p, x);
        // bubble up
        loop {
            let p = unsafe { &*u }.parent;
            if p == ptr::null_mut() {
                break;
            }
            if unsafe { &*p }.priority < unsafe { &*u }.priority {
                break;
            }
            if unsafe { &*p }.right == u {
                self.rotate_left(p);
            } else if unsafe { &*p }.left == u {
                self.rotate_right(p);
            } else {
                unreachable!();
            }
        }
        if unsafe { &*u }.parent == ptr::null_mut() {
            self.root = u;
        }
        true
    }

    // expected O(log(n)) time
    fn remove(&mut self, x: &T) -> bool {
        let u = self.find_last(x);
        if u == ptr::null_mut() {
            // 空の状態から削除しようとしたとき
            return false;
        }
        if !unsafe { &*u }.x.eq(x) {
            return false;
        }

        // trickle down
        loop {
            let left = unsafe { &*u }.left;
            let right = unsafe { &*u }.right;
            if left == ptr::null_mut() && right == ptr::null_mut() {
                if self.root == u {
                    self.root = ptr::null_mut();
                } else {
                    let p = unsafe { &*u }.parent;
                    debug_assert_ne!(p, ptr::null_mut());
                    if unsafe { &*p }.left == u {
                        unsafe { (*p).left = ptr::null_mut() };
                    } else if unsafe { &*p }.right == u {
                        unsafe { (*p).right = ptr::null_mut() };
                    } else {
                        unreachable!();
                    }
                }
                unsafe { ptr::drop_in_place(u) };
                unsafe { alloc::dealloc(u as *mut u8, alloc::Layout::new::<Node<T>>()) };
                break;
            }
            if left == ptr::null_mut() {
                self.rotate_left(u);
            } else if right == ptr::null_mut() {
                self.rotate_right(u);
            } else if unsafe { &*left }.priority < unsafe { &*right }.priority {
                self.rotate_right(u);
            } else {
                self.rotate_left(u);
            }
        }
        self.n -= 1;
        true
    }

    // ほとんど find_last と同じ
    // 最後に左に降りたときのノードを覚えておく
    // expected O(log(n))
    fn find(&self, x: &T) -> Option<&T> {
        let mut w = self.root;
        let mut z = ptr::null_mut();
        loop {
            if w == ptr::null_mut() {
                break;
            }
            let y = &unsafe { &*w }.x;
            match x.cmp(y) {
                Ordering::Less => {
                    z = w;
                    w = unsafe { &*w }.left;
                }
                Ordering::Greater => {
                    w = unsafe { &*w }.right;
                }
                Ordering::Equal => {
                    return Some(y);
                }
            }
        }

        if z == ptr::null_mut() {
            None
        } else {
            Some(&unsafe { &*z }.x)
        }
    }
}

impl<T> fmt::Debug for Treap<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.root == ptr::null_mut() {
            return Ok(());
        }

        let mut stack = Vec::new();
        stack.push((self.root, true, 0));
        while let Some((u, first_visit, depth)) = stack.pop() {
            assert_ne!(u, ptr::null_mut());
            if first_visit {
                stack.push((u, false, depth));
                let left = unsafe { &*u }.left;
                let right = unsafe { &*u }.right;
                if left != ptr::null_mut() {
                    stack.push((left, true, depth + 1));
                }
                if right != ptr::null_mut() {
                    stack.push((right, true, depth + 1));
                }
            } else {
                writeln!(
                    f,
                    "[{:p}] parent = {:p}, left = {:p}, right = {:p}, x = {:?}, priority = {}",
                    u,
                    unsafe { &*u }.parent,
                    unsafe { &*u }.left,
                    unsafe { &*u }.right,
                    unsafe { &*u }.x,
                    unsafe { &*u }.priority
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Node, Treap};
    use interface::SSet;
    use rand::{rngs::SmallRng, Rng, SeedableRng};
    use std::collections::BTreeSet;
    use std::ptr;

    #[test]
    fn test_rotate() {
        macro_rules! node {
            ($x: expr, $left: expr, $right: expr) => {
                Box::into_raw(Box::new(Node {
                    x: $x,
                    priority: 0,
                    parent: ptr::null_mut(),
                    left: $left,
                    right: $right,
                }))
            };
        }
        let mut treap = Treap::new();
        let a = node!('a', ptr::null_mut(), ptr::null_mut());
        let b = node!('b', ptr::null_mut(), ptr::null_mut());
        let c = node!('c', ptr::null_mut(), ptr::null_mut());
        let w = node!('w', a, b);
        let u = node!('u', w, c);
        let r = node!('r', u, ptr::null_mut());
        unsafe {
            (*a).parent = w;
            (*b).parent = w;
            (*c).parent = u;
            (*w).parent = u;
            (*u).parent = r;
        };
        treap.root = r;

        treap.rotate_right(u);

        assert_eq!(unsafe { (*a).parent }, w);
        assert_eq!(unsafe { (*b).parent }, u);
        assert_eq!(unsafe { (*c).parent }, u);
        assert_eq!(unsafe { (*w).parent }, r);
        assert_eq!(unsafe { (*w).left }, a);
        assert_eq!(unsafe { (*w).right }, u);
        assert_eq!(unsafe { (*u).parent }, w);
        assert_eq!(unsafe { (*u).left }, b);
        assert_eq!(unsafe { (*u).right }, c);
        assert_eq!(unsafe { (*r).left }, w);
    }

    #[test]
    fn remove_from_empty_set() {
        let mut treap = Treap::new();
        let removed = treap.remove(&42);
        assert!(!removed);
    }

    #[test]
    fn add_same() {
        let mut treap = Treap::new();
        let added = treap.add(42);
        assert!(added);
        let added = treap.add(42);
        assert!(!added);
    }

    #[test]
    fn add_remove() {
        let mut treap = Treap::new();
        treap.add(42);
        assert_eq!(treap.size(), 1);
        let removed = treap.remove(&42);
        assert!(removed);
        assert_eq!(treap.size(), 0);
    }

    #[test]
    fn find_less_equal_greater() {
        let mut treap = Treap::new();
        treap.add(42);
        assert_eq!(treap.find(&41), Some(&42));
        assert_eq!(treap.find(&42), Some(&42));
        assert_eq!(treap.find(&43), None);
    }

    #[test]
    fn test_random() {
        let mut rng = SmallRng::seed_from_u64(0);
        let mut treap = Treap::new();
        let mut btree_set = BTreeSet::new();

        for _ in 0..100 {
            let x = rng.gen_range(0..100_u8);
            let added_1 = treap.add(x);
            let added_2 = btree_set.insert(x);
            assert_eq!(added_1, added_2);
        }

        for _ in 0..100 {
            let x = rng.gen_range(0..100_u8);
            let y1 = treap.find(&x);
            let y2 = btree_set.range(x..).next();
            assert_eq!(y1, y2);
        }

        for _ in 0..100 {
            let x = rng.gen_range(0..100_u8);
            let removed_1 = treap.remove(&x);
            let removed_2 = btree_set.remove(&x);
            assert_eq!(removed_1, removed_2);
        }
    }
}
