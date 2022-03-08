use std::ptr;

use interface::List;

#[derive(Debug)]
struct Node<T> {
    // dummy ノードだけ x が None
    // ぐぬぬ
    x: Option<T>,
    next: *mut Node<T>,
    prev: *mut Node<T>,
}

#[derive(Debug)]
pub struct DLList<T> {
    dummy: *mut Node<T>,
    n: usize,
}

impl<T> DLList<T> {
    pub fn new() -> Self {
        let dummy = Node::<T> {
            x: None,
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
        };
        let dummy = Box::into_raw(Box::new(dummy));
        unsafe {
            (*dummy).next = dummy;
            (*dummy).prev = dummy;
        }
        Self { dummy, n: 0 }
    }

    // i = n のとき dummy を返す
    // O(min(i, n-i)) time
    fn get_node(&self, i: usize) -> Option<*mut Node<T>> {
        if i <= self.n / 2 {
            let mut u = unsafe { (*self.dummy).next };
            for _ in 0..i {
                let next = unsafe { &*u }.next;
                u = next;
            }
            Some(u)
        } else if i <= self.n {
            let mut u = self.dummy;
            for _ in 0..(self.n - i) {
                let prev = unsafe { &*u }.prev;
                u = prev;
            }
            Some(u)
        } else {
            None
        }
    }

    // w = dummy の場合がありうる
    // O(1) time
    fn add_before(&mut self, w: *mut Node<T>, x: T) {
        let u = Node::<T> {
            x: Some(x),
            next: w,
            prev: unsafe { (*w).prev },
        };
        let u = Box::into_raw(Box::new(u));
        // p --> u
        unsafe { (*(*u).prev).next = u };
        //       u <-- w
        unsafe { (*w).prev = u };

        self.n += 1;
    }

    // w = dummy の場合何もしない
    // O(1) time
    fn remove_node(&mut self, w: *mut Node<T>) -> Option<T> {
        let x = unsafe { (*w).x.take() }?;
        debug_assert_ne!(w, self.dummy);

        // prev <--> w <--> next
        let prev_w = unsafe { (*w).prev };
        let next_w = unsafe { (*w).next };
        unsafe { (*prev_w).next = next_w }; // prev --> next
        unsafe { (*next_w).prev = prev_w }; // prev <-- next

        unsafe { ptr::drop_in_place(w) };
        self.n -= 1;
        Some(x)
    }
}

impl<T> List<T> for DLList<T> {
    fn size(&self) -> usize {
        self.n
    }

    // O(min(i, n-i)) time
    fn get(&self, i: usize) -> Option<&T> {
        let u = self.get_node(i)?;
        let u = unsafe { &*u };
        u.x.as_ref()
    }

    // O(min(i, n-i)) time
    fn set(&self, i: usize, x: T) -> T {
        let u = self
            .get_node(i)
            .unwrap_or_else(|| panic!("expect `i` < DLList::size()"));
        let y = unsafe { (*u).x.replace(x) };
        y.unwrap()
    }

    // O(min(i, n-i)) time
    fn add(&mut self, i: usize, x: T) {
        let w = self
            .get_node(i)
            .unwrap_or_else(|| panic!("expect `i` <= DLList::size()"));
        self.add_before(w, x);
    }

    // O(min(i, n-i)) time
    fn remove(&mut self, i: usize) -> T {
        self.get_node(i)
            .and_then(|w| self.remove_node(w))
            .unwrap_or_else(|| panic!("expect `i` < DLList::size()"))
    }
}

#[cfg(test)]
mod tests {
    use super::DLList;
    use interface::List;

    #[test]
    fn test_get_none() {
        let mut list = DLList::new();

        let nil = list.get(0);
        assert_eq!(nil, None);

        list.add(0, 'a');
        assert_eq!(list.get(1), None);
        assert_eq!(list.get(2), None);
    }

    #[test]
    fn test_abc() {
        let mut list = DLList::new();
        list.add(0, 'a');
        list.add(1, 'b');
        list.add(2, 'c');
        // a -> b -> c
        let a = list.get(0);
        let b = list.get(1);
        let c = list.get(2);
        assert_eq!(a, Some(&'a'));
        assert_eq!(b, Some(&'b'));
        assert_eq!(c, Some(&'c'));
    }

    #[test]
    fn test_cba() {
        let mut list = DLList::new();
        list.add(0, 'a');
        list.add(0, 'b');
        list.add(0, 'c');
        // c -> b - > a
        let c = list.get(0);
        let b = list.get(1);
        let a = list.get(2);
        assert_eq!(c, Some(&'c'));
        assert_eq!(b, Some(&'b'));
        assert_eq!(a, Some(&'a'));
    }

    #[test]
    fn test_set_get() {
        let mut list = DLList::new();
        list.add(0, 'a');
        list.add(1, 'b');
        list.add(2, 'c');

        // a -> b -> c
        let old = list.set(0, 'x');
        assert_eq!(old, 'a');
        assert_eq!(list.get(0), Some(&'x'));

        // x -> b -> c
        let old = list.set(2, 'z');
        assert_eq!(old, 'c');
        assert_eq!(list.get(2), Some(&'z'));
    }

    #[test]
    fn test_add_remove() {
        let mut list = DLList::new();
        list.add(0, 'a');
        list.add(1, 'b');
        list.add(2, 'c');

        // a -> b -> c
        let b = list.remove(1);
        assert_eq!(b, 'b');

        // a -> c
        let c = list.remove(1);
        assert_eq!(c, 'c');

        // a
        let a = list.remove(0);
        assert_eq!(a, 'a');

        assert_eq!(list.size(), 0);
    }
}
