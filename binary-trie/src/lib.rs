use std::{alloc, ptr};

use interface::SSet;

struct Node<T> {
    // 葉 ⇒ x = Some(.), child = [NULL, NULL]
    // 葉以外 ⇒ x = None, prev = next = NULL
    x: Option<T>,
    child: [*mut Node<T>; 2], // left, right
    parent: *mut Node<T>,
    prev: *mut Node<T>,
    next: *mut Node<T>,
    jump: *mut Node<T>,
}

pub struct BinaryTrie<T> {
    n: usize,
    root: *mut Node<T>,
    dummy: *mut Node<T>,
}

pub trait IntValue {
    fn int_value(&self) -> u64;
}

macro_rules! impl_int_value {
     ($($t:ty),+) => {
         $(
            impl IntValue for $t {
                fn int_value(&self) -> u64 {
                    u64::from(*self)
                }
            }
         )+
     };
}

impl_int_value!(bool, char, u8, u16, u32, u64);

impl<T> BinaryTrie<T> {
    pub fn new() -> Self {
        let dummy = Box::into_raw(Box::new(Node {
            x: None,
            child: [ptr::null_mut(), ptr::null_mut()],
            parent: ptr::null_mut(),
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
            jump: ptr::null_mut(),
        }));
        // prev, next だけ使う
        unsafe { (*dummy).prev = dummy };
        unsafe { (*dummy).next = dummy };
        Self {
            n: 0,
            root: Box::into_raw(Box::new(Node {
                x: None,
                child: [ptr::null_mut(), ptr::null_mut()],
                parent: ptr::null_mut(),
                prev: ptr::null_mut(),
                next: ptr::null_mut(),
                jump: dummy,
            })),
            dummy,
        }
    }
}

impl<T> SSet<T> for BinaryTrie<T>
where
    T: IntValue,
{
    fn size(&self) -> usize {
        self.n
    }

    fn add(&mut self, x: T) -> bool {
        let w = u64::BITS;
        let ix = x.int_value();
        let mut u = self.root;
        for i in 0..w {
            let b = (ix >> (w - i - 1) & 1) as usize;
            let child = unsafe { &*u }.child[b];
            if child != ptr::null_mut() {
                u = child;
            } else {
                // pred = (x より小さい最大の要素)
                let pred = if b == 1 {
                    // 1: right
                    unsafe { (*u).jump }
                } else {
                    // 0: left
                    debug_assert_eq!(b, 0);
                    // u.jump は x より大きい最小の要素を指すので prev でひとつ戻す
                    unsafe { (*(*u).jump).prev }
                };

                // ix への経路をつくる
                for j in i..w {
                    let b = (ix >> (w - j - 1) & 1) as usize;
                    let child = Box::into_raw(Box::new(Node {
                        x: None,
                        child: [ptr::null_mut(), ptr::null_mut()],
                        parent: u,
                        prev: ptr::null_mut(),
                        next: ptr::null_mut(),
                        jump: ptr::null_mut(),
                    }));
                    unsafe { (*u).child[b] = child };
                    u = child;
                }

                // 深さ w のノードが葉なので x を保持
                unsafe { (*u).x = Some(x) };

                // u を葉の連結リストに入れる
                unsafe { (*u).prev = pred };
                unsafe { (*u).next = (*pred).next };
                unsafe { (*(*u).prev).next = u };
                unsafe { (*(*u).next).prev = u };

                let mut v = unsafe { &*u }.parent;
                while v != ptr::null_mut() {
                    let left = unsafe { &*v }.child[0];
                    let right = unsafe { &*v }.child[1];
                    let jump = unsafe { &*v }.jump;
                    let left_jump = left == ptr::null_mut() // 左の子がいない
                        && (jump == ptr::null_mut() || jump == self.dummy // jump ポインタがない
                            // ix が v の部分木内で最小
                            || unsafe { &*jump }.x.as_ref().unwrap().int_value() > ix);
                    let right_jump = right == ptr::null_mut()
                        && (jump == ptr::null_mut() || jump == self.dummy
                            // ix が v の部分木内で最大
                            || unsafe { &*jump }.x.as_ref().unwrap().int_value() < ix);

                    if left != ptr::null_mut() && right != ptr::null_mut() {
                        unsafe { (*v).jump = ptr::null_mut() };
                    } else if left_jump || right_jump {
                        unsafe { (*v).jump = u };
                    }

                    v = unsafe { &*v }.parent;
                }

                self.n += 1;
                return true;
            }
        }

        // 葉まで辿りついたのですでに x が BinaryTrie に含まれていた
        false
    }

    fn remove(&mut self, x: &T) -> bool {
        let w = u64::BITS;
        let ix = x.int_value();
        let mut u = self.root;

        for i in 0..w {
            let b = (ix >> (w - i - 1) & 1) as usize;
            let child = unsafe { &*u }.child[b];
            if child == ptr::null_mut() {
                return false;
            }
            u = child;
        }

        debug_assert!(unsafe { &*u }.x.is_some());
        // debug_assert_eq!(unsafe {&*u}.x.as_ref(), Some(x));

        // u を葉の連結リストから除く
        unsafe { (*(*u).prev).next = (*u).next };
        unsafe { (*(*u).next).prev = (*u).prev };

        let mut v = u;
        for i in (0..w).rev() {
            v = unsafe { &*v }.parent;
            let b = (ix >> (w - i - 1) & 1) as usize;
            unsafe { ptr::drop_in_place((*v).child[b]) };
            unsafe { alloc::dealloc((*v).child[b] as *mut u8, alloc::Layout::new::<Node<T>>()) };
            unsafe { (*v).child[b] = ptr::null_mut() };

            // 左 or 右の子があるので v は消さない
            if unsafe { &*v }.child[1 - b] != ptr::null_mut() {
                let prev = unsafe { &*u }.prev;
                let next = unsafe { &*u }.next;
                debug_assert_eq!(unsafe { &*v }.jump, ptr::null_mut());
                unsafe { (*v).jump = if b == 0 { next } else { prev } };
                v = unsafe { &*v }.parent;
                for j in (0..i).rev() {
                    let b = (ix >> (w - j - 1) & 1) as usize;
                    if unsafe { &*v }.jump == u {
                        unsafe { (*v).jump = if b == 0 { next } else { prev } };
                    }
                    v = unsafe { &*v }.parent;
                }
                debug_assert_eq!(v, ptr::null_mut());
                break;
            }
        }

        true
    }

    fn find(&self, x: &T) -> Option<&T> {
        if self.n == 0 {
            return None;
        }

        let w = u64::BITS;
        let ix = x.int_value();
        let mut u = self.root;
        for i in 0..w {
            let b = (ix >> (w - i - 1) & 1) as usize;
            let child = unsafe { &*u }.child[b];
            if child == ptr::null_mut() {
                break;
            }
            u = child;
        }
        let left = unsafe { &*u }.child[0];
        let right = unsafe { &*u }.child[1];
        if left == ptr::null_mut() && right == ptr::null_mut() {
            // 葉
            let x = unsafe { &*u }.x.as_ref();
            debug_assert!(x.is_some());
            x
        } else {
            let v = unsafe { &*u }.jump;
            if left == ptr::null_mut() {
                let x = unsafe { &*v }.x.as_ref();
                if v == self.dummy {
                    debug_assert!(x.is_none());
                } else {
                    debug_assert!(x.is_some());
                }
                x
            } else if right == ptr::null_mut() {
                let w = unsafe { &*v }.next;
                let x = unsafe { &*w }.x.as_ref();
                if w == self.dummy {
                    debug_assert!(x.is_none());
                } else {
                    debug_assert!(x.is_some());
                }
                x
            } else {
                unreachable!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BinaryTrie;
    use interface::SSet;
    use rand::{rngs::SmallRng, Rng, SeedableRng};
    use std::collections::BTreeSet;

    #[test]
    fn test_find() {
        let mut binary_trie = BinaryTrie::<u8>::new();
        assert_eq!(binary_trie.find(&0), None);
        binary_trie.add(0);
        binary_trie.add(10);
        binary_trie.add(100);
        assert_eq!(binary_trie.find(&0), Some(&0));
        assert_eq!(binary_trie.find(&1), Some(&10));
        assert_eq!(binary_trie.find(&10), Some(&10));
        assert_eq!(binary_trie.find(&11), Some(&100));
        assert_eq!(binary_trie.find(&101), None);
    }

    #[test]
    fn test_remove() {
        let mut binary_trie = BinaryTrie::<u8>::new();
        binary_trie.add(42);
        let removed = binary_trie.remove(&42);
        assert!(removed);
        let removed = binary_trie.remove(&42);
        assert!(!removed);
    }

    #[test]
    fn test_random() {
        let mut rng = SmallRng::seed_from_u64(0);
        let mut binary_trie = BinaryTrie::<u32>::new();
        let mut btree_set = BTreeSet::new();
        let n = 1000;
        for _ in 0..n {
            let x = rng.gen_range(0..n);
            assert_eq!(binary_trie.add(x), btree_set.insert(x));
        }
        for _ in 0..n {
            let x = rng.gen_range(0..n);
            assert_eq!(binary_trie.find(&x), btree_set.range(&x..).next());
        }
        for _ in 0..n {
            let x = rng.gen_range(0..n);
            assert_eq!(binary_trie.remove(&x), btree_set.remove(&x));
        }
    }
}
