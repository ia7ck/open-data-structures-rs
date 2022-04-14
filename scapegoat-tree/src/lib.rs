use std::{
    alloc,
    cmp::{self, Ordering},
    ptr,
};

use interface::SSet;

struct Node<T> {
    x: T,
    parent: *mut Node<T>,
    left: *mut Node<T>,
    right: *mut Node<T>,
}

pub struct ScapegoatTree<T> {
    n: usize,
    root: *mut Node<T>,
    q: usize, // n/2 <= q <= n
}

impl<T> ScapegoatTree<T> {
    pub fn new() -> Self {
        Self {
            n: 0,
            root: ptr::null_mut(),
            q: 0,
        }
    }

    // O(n) time
    fn size_u(u: *mut Node<T>) -> usize {
        if u == ptr::null_mut() {
            0
        } else {
            1 + Self::size_u(unsafe { &*u }.left) + Self::size_u(unsafe { &*u }.right)
        }
    }

    // u を根とする部分木を完全二分木にする
    // O(n) time
    fn rebuild(&mut self, u: *mut Node<T>) {
        let p = unsafe { &*u }.parent;
        // u を根とする部分木のサイズで Vec の capacity を確保すると速くなるかも
        let nodes = Self::collect_descendants(u);
        if p == ptr::null_mut() {
            self.root = Self::build_balanced(&nodes);
            debug_assert_ne!(self.root, ptr::null_mut());
            unsafe { (*self.root).parent = ptr::null_mut() };
            return;
        }
        if unsafe { &*p }.right == u {
            unsafe { (*p).right = Self::build_balanced(&nodes) };
            unsafe { (*(*p).right).parent = p };
        } else if unsafe { &*p }.left == u {
            unsafe { (*p).left = Self::build_balanced(&nodes) };
            unsafe { (*(*p).left).parent = p };
        } else {
            unreachable!();
        }
    }

    // u を根とする部分木のすべてのノードをキーの昇順に返す
    // O(n) time
    fn collect_descendants(u: *mut Node<T>) -> Vec<*mut Node<T>> {
        if u == ptr::null_mut() {
            return Vec::new();
        }
        let mut result = Self::collect_descendants(unsafe { &*u }.left);
        result.push(u);
        let right = Self::collect_descendants(unsafe { &*u }.right);
        result.extend(right);
        result
    }

    // キーの昇順に並んだノードの列を完全二分木になるようにポインタを張る
    // 作られた完全二分木の根を返す
    // O(n) time
    fn build_balanced(nodes: &[*mut Node<T>]) -> *mut Node<T> {
        if nodes.is_empty() {
            return ptr::null_mut();
        }
        let m = nodes.len() / 2;
        let left = Self::build_balanced(&nodes[..m]);
        unsafe { (*nodes[m]).left = left };
        if left != ptr::null_mut() {
            unsafe { (*left).parent = nodes[m] };
        }
        let right = Self::build_balanced(&nodes[(m + 1)..]);
        unsafe { (*nodes[m]).right = right };
        if right != ptr::null_mut() {
            unsafe { (*right).parent = nodes[m] };
        }
        nodes[m]
    }
}

impl<T> ScapegoatTree<T>
where
    T: cmp::Ord,
{
    // x がキーのノードを挿入して、ノードとその深さを返す
    // すでに scapegoat 木に x が含まれていたら None を返す
    // O(log(n)) time
    fn add_with_depth(&mut self, x: T) -> Option<(*mut Node<T>, usize)> {
        let mut w = self.root;
        let mut depth = 0;
        if w == ptr::null_mut() {
            self.root = Box::into_raw(Box::new(Node {
                x,
                left: ptr::null_mut(),
                right: ptr::null_mut(),
                parent: ptr::null_mut(),
            }));
            self.n += 1;
            self.q += 1;
            return Some((self.root, depth));
        }
        loop {
            debug_assert_ne!(w, ptr::null_mut());
            let y = &unsafe { &*w }.x;
            match x.cmp(y) {
                Ordering::Less => {
                    let left = unsafe { &*w }.left;
                    if left == ptr::null_mut() {
                        let u = Box::into_raw(Box::new(Node {
                            x,
                            left: ptr::null_mut(),
                            right: ptr::null_mut(),
                            parent: w,
                        }));
                        unsafe { (*w).left = u };
                        self.n += 1;
                        self.q += 1;
                        break Some((u, depth));
                    } else {
                        w = left;
                        depth += 1;
                    }
                }
                Ordering::Greater => {
                    let right = unsafe { &*w }.right;
                    if right == ptr::null_mut() {
                        let u = Box::into_raw(Box::new(Node {
                            x,
                            left: ptr::null_mut(),
                            right: ptr::null_mut(),
                            parent: w,
                        }));
                        unsafe { (*w).right = u };
                        self.n += 1;
                        self.q += 1;
                        break Some((u, depth));
                    } else {
                        w = right;
                        depth += 1;
                    }
                }
                Ordering::Equal => {
                    break None;
                }
            }
        }
    }

    // copy of Treap::find_last
    // O(log(n)) time
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

    // u を削除する
    // O(log(n)) time
    fn remove_u(&mut self, u: *mut Node<T>) {
        debug_assert_ne!(u, ptr::null_mut());
        let left_u = unsafe { &*u }.left;
        let right_u = unsafe { &*u }.right;
        let p = unsafe { &*u }.parent;
        if left_u == ptr::null_mut() && right_u == ptr::null_mut() {
            if p == ptr::null_mut() {
                self.root = ptr::null_mut();
            } else if unsafe { &*p }.left == u {
                unsafe { (*p).left = ptr::null_mut() };
            } else {
                debug_assert_eq!(unsafe { &*p }.right, u);
                unsafe { (*p).right = ptr::null_mut() };
            }
        } else if left_u == ptr::null_mut() || right_u == ptr::null_mut() {
            let child = if left_u == ptr::null_mut() {
                right_u
            } else {
                left_u
            };
            debug_assert_ne!(child, ptr::null_mut());
            unsafe { (*child).parent = p };
            if p == ptr::null_mut() {
                self.root = child;
            } else if unsafe { &*p }.left == u {
                unsafe { (*p).left = child };
            } else {
                debug_assert_eq!(unsafe { &*p }.right, u);
                unsafe { (*p).right = child };
            }
        } else {
            debug_assert_ne!(left_u, ptr::null_mut());
            debug_assert_ne!(right_u, ptr::null_mut());
            let mut w = right_u;
            // w.x が u.x より大きい最小の値になるように左の子を辿る
            loop {
                let left_w = unsafe { &*w }.left;
                if left_w == ptr::null_mut() {
                    break;
                }
                w = left_w;
            }
            // w を u の位置に持っていく
            // 関係するポインタを張り替える
            let p_w = unsafe { &*w }.parent;
            if unsafe { &*p_w }.left == w {
                if unsafe { &*w }.right != ptr::null_mut() {
                    unsafe { (*(*w).right).parent = p_w };
                }
                unsafe { (*p_w).left = (*w).right };
                unsafe { (*w).left = left_u };
                unsafe { (*w).right = right_u };
                unsafe { (*w).parent = p };
                unsafe { (*left_u).parent = w };
                unsafe { (*right_u).parent = w };
            } else {
                debug_assert_eq!(unsafe { &*p_w }.right, w);
                debug_assert_eq!(p_w, u);
                debug_assert_eq!(right_u, w);
                unsafe { (*p_w).right = (*w).right };
                unsafe { (*w).left = left_u };
                unsafe { (*w).parent = p };
                unsafe { (*left_u).parent = w };
            }
            if p == ptr::null_mut() {
                self.root = w;
            } else if unsafe { &*p }.left == u {
                unsafe { (*p).left = w };
            } else {
                debug_assert_eq!(unsafe { &*p }.right, u);
                unsafe { (*p).right = w };
            }
        }
        self.n -= 1;
        unsafe { ptr::drop_in_place(u) };
        unsafe { alloc::dealloc(u as *mut u8, alloc::Layout::new::<Node<T>>()) };
    }
}

impl<T> SSet<T> for ScapegoatTree<T>
where
    T: cmp::Ord,
{
    fn size(&self) -> usize {
        self.n
    }

    // amortized O(log(n)) time
    fn add(&mut self, x: T) -> bool {
        if let Some((u, depth)) = self.add_with_depth(x) {
            if depth as f64 > (self.q as f64).log(3.0 / 2.0) {
                let mut w = unsafe { &*u }.parent;
                loop {
                    debug_assert_ne!(w, ptr::null_mut());
                    debug_assert_ne!(unsafe { &*w }.parent, ptr::null_mut());
                    let a = Self::size_u(w);
                    let b = Self::size_u(unsafe { &*w }.parent);
                    // a/b > 2/3
                    if a * 3 > b * 2 {
                        // 補題 8.1 より、いつか loop から抜ける
                        break;
                    }
                    w = unsafe { &*w }.parent;
                }
                self.rebuild(unsafe { &*w }.parent);
            }
            true
        } else {
            false
        }
    }

    // amortized O(log(n)) time
    fn remove(&mut self, x: &T) -> bool {
        let u = self.find_last(x);
        if u != ptr::null_mut() && unsafe { &*u }.x.eq(x) {
            self.remove_u(u);
            if self.q > self.n * 2 {
                if self.root != ptr::null_mut() {
                    self.rebuild(self.root);
                }
                self.q = self.n;
            }
            true
        } else {
            false
        }
    }

    // copy of Treap::find
    // O(log(n)) time
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

#[cfg(test)]
mod tests {
    use super::ScapegoatTree;
    use interface::SSet;
    use rand::{rngs::SmallRng, Rng, SeedableRng};
    use std::collections::BTreeSet;

    #[test]
    fn add_same() {
        let mut scapegoat_tree = ScapegoatTree::new();
        let added = scapegoat_tree.add('a');
        assert!(added);
        let added = scapegoat_tree.add('a');
        assert!(!added);
    }

    #[test]
    fn add_remove() {
        let mut scapegoat_tree = ScapegoatTree::new();
        scapegoat_tree.add('a');
        assert_eq!(scapegoat_tree.size(), 1);
        let removed = scapegoat_tree.remove(&'a');
        assert!(removed);
        assert_eq!(scapegoat_tree.size(), 0);
        let removed = scapegoat_tree.remove(&'a');
        assert!(!removed);
    }

    #[test]
    fn find_less_equal_greater() {
        let mut scapegoat_tree = ScapegoatTree::new();
        scapegoat_tree.add('b');
        assert_eq!(scapegoat_tree.find(&'a'), Some(&'b'));
        assert_eq!(scapegoat_tree.find(&'b'), Some(&'b'));
        assert_eq!(scapegoat_tree.find(&'c'), None);
    }

    #[test]
    fn test_large() {
        let mut rng = SmallRng::seed_from_u64(0);
        let mut scapegoat_tree = ScapegoatTree::new();
        let mut btree = BTreeSet::new();
        let n = 100;
        for x in 0..n {
            let added_1 = scapegoat_tree.add(x);
            let added_2 = btree.insert(x);
            assert_eq!(added_1, added_2);
        }
        for _ in 0..n {
            let x = rng.gen_range(0..n);
            let y1 = scapegoat_tree.find(&x);
            let y2 = btree.range(x..).next();
            assert_eq!(y1, y2);
        }
        for _ in 0..n {
            let x = rng.gen_range(0..n);
            let removed_1 = scapegoat_tree.remove(&x);
            let removed_2 = btree.remove(&x);
            assert_eq!(removed_1, removed_2);
        }
    }
}
