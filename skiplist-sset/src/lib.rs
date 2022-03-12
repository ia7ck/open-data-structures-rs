use std::{
    alloc,
    fmt::{self, Formatter},
    ptr,
};

use rand::{rngs::SmallRng, Rng, SeedableRng};

use interface::SSet;

struct Node<T>
where
    T: PartialOrd,
{
    x: Option<T>,
    next: Vec<*mut Node<T>>,
}

impl<T> Node<T>
where
    T: PartialOrd,
{
    fn new(x: Option<T>, height: usize) -> Self {
        Self {
            x,
            next: vec![ptr::null_mut(); height + 1],
        }
    }

    fn height(&self) -> usize {
        self.next.len()
    }
}

pub struct SkipListSSet<T>
where
    T: PartialOrd,
{
    sentinel: *mut Node<T>,
    height: usize, // 「i <= height iff. sentinel.next[i] が non null」となるようにする
    n: usize,
}

impl<T> SkipListSSet<T>
where
    T: PartialOrd,
{
    pub fn new() -> Self {
        let sentinel = Node::new(None, 32);
        let sentinel = Box::into_raw(Box::new(sentinel));
        Self {
            sentinel,
            height: 0,
            n: 0,
        }
    }

    fn pick_height() -> usize {
        // 毎回 rng を生成しているためパフォーマンスが悪そう
        let mut small_rng = SmallRng::from_entropy();
        // 返り値 : 確率
        // 0 : 1/2
        // 1 : 1/4
        // 2 : 1/8
        // 3 : 1/16
        // ...
        small_rng.gen_range(0..u32::MAX).trailing_ones() as usize
    }

    // expected O(log(n)) time
    fn find_pred_node(&self, x: &T) -> *mut Node<T> {
        let mut u = self.sentinel;
        for r in (0..=self.height).rev() {
            loop {
                let next = unsafe { &*u }.next[r];
                if next.is_null() {
                    break;
                }
                let y = unsafe { &*next }.x.as_ref().unwrap();
                if y.lt(x) {
                    u = next;
                } else {
                    break;
                }
            }
        }
        u
    }
}

impl<T> SSet<T> for SkipListSSet<T>
where
    T: PartialOrd,
{
    fn size(&self) -> usize {
        self.n
    }

    // expected O(log(n)) time
    fn add(&mut self, x: T) -> bool {
        let mut u = self.sentinel;
        let h = Self::pick_height(); // 新しく追加するノードの高さ
        let mut stack = Vec::new(); // 固定長の配列 self.buf: [*mut Node<T>; 32] を使い回すほうが速くなりそう
        for r in (0..=self.height.max(h)).rev() {
            let exist = loop {
                let next = unsafe { &*u }.next[r];
                if next.is_null() {
                    // x が skip list 内のどの要素よりも大きい
                    break false;
                }
                let y = unsafe { &*next }.x.as_ref().unwrap();
                if x.gt(y) {
                    u = next;
                } else {
                    break x.eq(y);
                }
            };
            if exist {
                // x と等しい要素があった場合ノードを追加しない
                return false;
            }
            stack.push(u);
        }
        stack.reverse();

        let w = Box::into_raw(Box::new(Node::new(Some(x), h)));
        for i in 0..=h {
            unsafe {
                (*w).next[i] = (*stack[i]).next[i];
            }
            unsafe {
                (*stack[i]).next[i] = w;
            }
        }

        self.height = self.height.max(h);
        self.n += 1;

        true
    }

    // expected O(log(n)) time
    fn remove(&mut self, x: &T) -> bool {
        let mut removed = false;
        let mut del = ptr::null_mut();
        let mut u = self.sentinel;
        for r in (0..=self.height).rev() {
            // add のときとまったく同じ
            let delete_next_node = loop {
                let next = unsafe { &*u }.next[r];
                if next.is_null() {
                    break false;
                }
                let y = unsafe { &*next }.x.as_ref().unwrap();
                if x.gt(y) {
                    u = next;
                } else {
                    break x.eq(y);
                }
            };
            if delete_next_node {
                removed = true;
                del = unsafe { &*u }.next[r];
                unsafe { (*u).next[r] = (*(*u).next[r]).next[r] };
                if u == self.sentinel && unsafe { &*u }.next[r].is_null() {
                    if self.height == 0 {
                        // x を消すと要素数が 0 になるケースでここに来るはず
                        debug_assert_eq!(r, 0);
                        debug_assert_eq!(self.n, 1);
                    } else {
                        self.height -= 1;
                    }
                }
            }
        }
        if removed {
            debug_assert!(!del.is_null());
            unsafe { ptr::drop_in_place(del) };
            unsafe { alloc::dealloc(del as *mut u8, alloc::Layout::new::<Node<T>>()) };
            self.n -= 1;
        }
        removed
    }

    // expected O(log(n)) time
    fn find(&self, x: &T) -> Option<&T> {
        let u = self.find_pred_node(x);
        let next = unsafe { &*u }.next[0];
        if next.is_null() {
            None
        } else {
            let x = unsafe { &*next }.x.as_ref();
            debug_assert!(x.is_some());
            x
        }
    }
}

impl<T> fmt::Debug for SkipListSSet<T>
where
    T: PartialOrd + fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut u = self.sentinel;
        let h = unsafe { &*u }
            .next
            .iter()
            .take_while(|v| !v.is_null())
            .count();
        write!(f, "~\t{}", "#".repeat(h))?;
        writeln!(f)?;
        u = unsafe { &*u }.next[0];
        while !u.is_null() {
            let h = unsafe { &*u }.height();
            let x = unsafe { &*u }.x.as_ref().unwrap();
            write!(f, "{:?}\t{}", x, "#".repeat(h))?;
            writeln!(f)?;
            u = unsafe { &*u }.next[0];
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SkipListSSet;
    use interface::SSet;

    #[test]
    fn test_add_twice() {
        let mut set = SkipListSSet::new();
        let added = set.add('a');
        assert!(added);
        assert_eq!(set.size(), 1);
        let added = set.add('a');
        assert!(!added);
        assert_eq!(set.size(), 1);
    }

    #[test]
    fn test_remove_twice() {
        let mut set = SkipListSSet::new();
        set.add('a');
        let removed = set.remove(&'a');
        assert!(removed);
        let removed = set.remove(&'a');
        assert!(!removed);
    }

    #[test]
    fn test_find() {
        let mut set = SkipListSSet::new();
        assert_eq!(set.find(&'a'), None);
        set.add('a');
        set.add('p');
        set.add('b');
        set.add('q');
        set.add('j');
        set.add('i');
        // a b i j p q
        assert_eq!(set.find(&'a'), Some(&'a'));
        assert_eq!(set.find(&'b'), Some(&'b'));
        assert_eq!(set.find(&'c'), Some(&'i'));
        assert_eq!(set.find(&'i'), Some(&'i'));
        assert_eq!(set.find(&'j'), Some(&'j'));
        assert_eq!(set.find(&'k'), Some(&'p'));
        assert_eq!(set.find(&'p'), Some(&'p'));
        assert_eq!(set.find(&'q'), Some(&'q'));
        assert_eq!(set.find(&'r'), None);
    }
}
