use interface::{Queue, Stack};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
struct Node<T> {
    x: T,
    next: Option<Rc<RefCell<Node<T>>>>,
}

#[derive(Debug)]
pub struct SLList<T> {
    head: Option<Rc<RefCell<Node<T>>>>,
    tail: Option<Rc<RefCell<Node<T>>>>,
}

impl<T> SLList<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }
}

impl<T> Stack<T> for SLList<T> {
    // O(1) time
    fn push(&mut self, x: T) {
        let u = Node {
            x,
            next: self.head.take(),
        };
        let u = Rc::new(RefCell::new(u));
        // u (head) --> old_head
        self.head = Some(Rc::clone(&u));
        self.tail.get_or_insert(u);
    }

    // O(1) time
    fn pop(&mut self) -> Option<T> {
        let u = self.head.take()?;
        match Rc::try_unwrap(u) {
            Ok(u) => {
                let mut u = RefCell::into_inner(u);
                self.head = u.next.take();
                debug_assert_eq!(self.head.is_some(), true);
                Some(u.x)
            }
            Err(u) => {
                // u = head = tail
                debug_assert_eq!(Rc::strong_count(&u), 2);
                let v = self.tail.take().unwrap();
                debug_assert_eq!(Rc::ptr_eq(&u, &v), true);
                drop(u);
                let v = Rc::try_unwrap(v).ok().unwrap();
                let v = RefCell::into_inner(v);
                Some(v.x)
            }
        }
    }
}

impl<T> Queue<T> for SLList<T> {
    // O(1) time
    fn add(&mut self, x: T) {
        let u = Node { x, next: None };
        let u = Rc::new(RefCell::new(u));
        if let Some(t) = self.tail.take() {
            // head = tail = t, or
            // t and <second last node>.next point to the same allocation
            RefCell::borrow_mut(&t).next = Some(Rc::clone(&u));
            self.tail = Some(u);
        } else {
            // head = tail = None
            self.head = Some(Rc::clone(&u));
            self.tail = Some(u);
        }
    }

    fn remove(&mut self) -> Option<T> {
        self.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::SLList;
    use interface::{Queue, Stack};

    #[test]
    fn test_pop() {
        let mut stack = SLList::<()>::new();
        let nil = stack.pop();
        assert_eq!(nil, None);
    }

    #[test]
    fn test_push_3_pop_4() {
        let mut stack = SLList::<char>::new();
        stack.push('a');
        stack.push('b');
        stack.push('c');
        let c = stack.pop();
        let b = stack.pop();
        let a = stack.pop();
        let nil = stack.pop();
        assert_eq!(c, Some('c'));
        assert_eq!(b, Some('b'));
        assert_eq!(a, Some('a'));
        assert_eq!(nil, None);
    }

    #[test]
    fn test_push_2_pop_1_push_1_pop_1() {
        let mut stack = SLList::<char>::new();
        stack.push('a');
        stack.push('b');
        let b = stack.pop();
        assert_eq!(b, Some('b'));
        stack.push('x');
        let x = stack.pop();
        assert_eq!(x, Some('x'));
    }

    #[test]
    fn test_remove() {
        let mut queue = SLList::<()>::new();
        let nil = queue.remove();
        assert_eq!(nil, None);
    }

    #[test]
    fn test_add_3_remove_4() {
        let mut queue = SLList::<char>::new();
        queue.add('a');
        queue.add('b');
        queue.add('c');
        let a = queue.remove();
        let b = queue.remove();
        let c = queue.remove();
        let nil = queue.remove();
        assert_eq!(a, Some('a'));
        assert_eq!(b, Some('b'));
        assert_eq!(c, Some('c'));
        assert_eq!(nil, None);
    }

    #[test]
    fn test_add_2_remove_1_add_1_remove_1() {
        let mut queue = SLList::<char>::new();
        queue.add('a');
        queue.add('b');
        let a = queue.remove();
        assert_eq!(a, Some('a'));
        queue.add('x');
        let b = queue.remove();
        assert_eq!(b, Some('b'));
    }
}
