use std::{
    alloc,
    fmt::{self, Formatter},
    ptr,
};

use rand::{rngs::SmallRng, Rng, SeedableRng};

use interface::List;

struct Node<T> {
    x: Option<T>,
    length: Vec<usize>,
    next: Vec<*mut Node<T>>,
}

impl<T> Node<T> {
    fn new(x: Option<T>, height: usize) -> Self {
        Self {
            x,
            length: vec![0; height + 1],
            next: vec![ptr::null_mut(); height + 1],
        }
    }
}

pub struct SkipListList<T> {
    sentinel: *mut Node<T>,
    height: usize,
    n: usize,
}

impl<T> SkipListList<T> {
    pub fn new() -> Self {
        let sentinel = Node::new(None, 32);
        let sentinel = Box::into_raw(Box::new(sentinel));
        Self {
            sentinel,
            height: 0,
            n: 0,
        }
    }

    // copy of SkipListSSet::pick_height
    fn pick_height() -> usize {
        let mut small_rng = SmallRng::from_entropy();
        small_rng.gen_range(0..u32::MAX).trailing_ones() as usize
    }

    fn find_pred(&self, i: usize) -> *mut Node<T> {
        let mut u = self.sentinel;
        let mut u_index = 0_usize.wrapping_sub(1);
        for r in (0..=self.height).rev() {
            loop {
                let next = unsafe { &*u }.next[r];
                if next.is_null() {
                    break;
                }
                let length_to_next = unsafe { &*u }.length[r];
                let next_index = u_index.wrapping_add(length_to_next);
                if next_index >= i {
                    break;
                }
                u = next;
                u_index = next_index;
            }
        }
        u
    }
}

impl<T> List<T> for SkipListList<T> {
    fn size(&self) -> usize {
        self.n
    }

    fn get(&self, i: usize) -> Option<&T> {
        let pred = self.find_pred(i);
        let u = unsafe { &*pred }.next[0];
        if u.is_null() {
            None
        } else {
            // u = sentinel のケースがある
            unsafe { &*u }.x.as_ref()
        }
    }

    fn set(&self, i: usize, x: T) -> T {
        assert!(i < self.size());
        let pred = self.find_pred(i);
        let u = unsafe { &*pred }.next[0];
        debug_assert!(!u.is_null());
        let y = unsafe { (*u).x.replace(x) };
        y.unwrap()
    }

    fn add(&mut self, i: usize, x: T) {
        assert!(i <= self.size());
        let w_height = Self::pick_height();
        let w = Box::into_raw(Box::new(Node::new(Some(x), w_height)));
        self.height = self.height.max(w_height);
        let mut u = self.sentinel;
        let mut u_index = 0_usize.wrapping_sub(1); // -1
        for r in (0..=self.height).rev() {
            loop {
                let next = unsafe { &*u }.next[r];
                if next.is_null() {
                    if r <= w_height {
                        // u --(i-u_index)--> w --(0)--> tail
                        unsafe { (*u).next[r] = w };
                        // u_index = -1 のときがある
                        unsafe { (*u).length[r] = i.wrapping_sub(u_index) };
                    }
                    break;
                }
                let length_to_next = unsafe { &*u }.length[r];
                // オーバーフローするのは最初の一回だけ
                let next_index = u_index.wrapping_add(length_to_next);
                if next_index >= i {
                    if r <= w_height {
                        // u --(i-u_index)--> w ------> next
                        // |                             ^
                        // |                             |
                        // +-------(u.length[r]+1)-------+
                        unsafe { (*w).next[r] = next };
                        unsafe { (*u).next[r] = w };
                        let length_u_w = i.wrapping_sub(u_index);
                        unsafe { (*w).length[r] = ((*u).length[r] + 1) - length_u_w };
                        unsafe { (*u).length[r] = length_u_w };
                    } else {
                        unsafe { (*u).length[r] += 1 };
                    }
                    break;
                }
                u = next;
                u_index = next_index;
            }
        }
        self.n += 1;
    }

    fn remove(&mut self, i: usize) -> T {
        assert!(i < self.size());
        let mut x = None;
        let mut del = ptr::null_mut();
        let mut u = self.sentinel;
        let mut u_index = 0_usize.wrapping_sub(1);
        for r in (0..=self.height).rev() {
            loop {
                let next = unsafe { &*u }.next[r];
                if next.is_null() {
                    debug_assert_eq!(unsafe { &*u }.length[r], 0);
                    break;
                }
                let length_to_next = unsafe { &*u }.length[r];
                let next_index = u_index.wrapping_add(length_to_next);
                if next_index >= i {
                    unsafe { (*u).length[r] -= 1 };
                    if next_index == i {
                        x = x.or_else(|| unsafe { (*next).x.take() });
                        del = next;
                        unsafe { (*u).length[r] += (*next).length[r] };
                        unsafe { (*u).next[r] = (*next).next[r] };
                        if u == self.sentinel && unsafe { (*u).next[r] }.is_null() {
                            if self.height == 0 {
                                debug_assert_eq!(self.n, 1);
                                debug_assert_eq!(r, 0);
                            } else {
                                self.height -= 1;
                            }
                        }
                    }
                    break;
                }
                u = next;
                u_index = next_index;
            }
        }
        self.n -= 1;
        let x = x.unwrap();
        unsafe { ptr::drop_in_place(del) };
        unsafe { alloc::dealloc(del as *mut u8, alloc::Layout::new::<Node<T>>()) };
        x
    }
}

impl<T> fmt::Debug for SkipListList<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let h = unsafe { &*self.sentinel }
            .length
            .iter()
            .take_while(|&&l| l != 0)
            .count();
        writeln!(f, "sentinel")?;
        writeln!(f, "length = {:?}", &unsafe { &*self.sentinel }.length[..h])?;
        writeln!(f, "next = {:?}", &unsafe { &*self.sentinel }.next[..h])?;
        let mut u = unsafe { &*self.sentinel }.next[0];
        let mut i = 0;
        while !u.is_null() {
            writeln!(f)?;
            writeln!(f, "node {}: {:p}", i, u)?;
            writeln!(f, "x = {:?}", unsafe { &*u }.x)?;
            writeln!(f, "next = {:?}", unsafe { &*u }.next)?;
            writeln!(f, "length = {:?}", unsafe { &*u }.length)?;
            u = unsafe { &*u }.next[0];
            i += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::SkipListList;
    use interface::List;

    #[test]
    fn test_get_none() {
        let mut list = SkipListList::new();

        let nil = list.get(0);
        assert_eq!(nil, None);

        list.add(0, 'a');
        assert_eq!(list.get(1), None);
        assert_eq!(list.get(2), None);
    }

    #[test]
    fn test_set_get() {
        let mut list = SkipListList::new();
        list.add(0, 'a');
        list.add(1, 'b');
        list.add(2, 'c');

        // a b c
        let old = list.set(0, 'x');
        assert_eq!(old, 'a');
        assert_eq!(list.get(0), Some(&'x'));

        // x b c
        let old = list.set(2, 'z');
        assert_eq!(old, 'c');
        assert_eq!(list.get(2), Some(&'z'));
    }

    #[test]
    fn test_add_remove() {
        let mut list = SkipListList::new();
        list.add(0, 'a');
        list.add(1, 'b');
        list.add(2, 'c');

        // a b c
        let b = list.remove(1);
        assert_eq!(b, 'b');

        // a c
        let c = list.remove(1);
        assert_eq!(c, 'c');

        // a
        let a = list.remove(0);
        assert_eq!(a, 'a');

        assert_eq!(list.size(), 0);
    }
}
