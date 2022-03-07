use interface::Stack;
use std::rc::Rc;

#[derive(Debug)]
struct Node<T> {
    x: T,
    next: Option<Rc<Node<T>>>,
}

#[derive(Debug)]
pub struct SLList<T> {
    head: Option<Rc<Node<T>>>,
    tail: Option<Rc<Node<T>>>,
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
        let u = Rc::new(u);
        // u (head) --> old_head
        self.head = Some(Rc::clone(&u));
        self.tail.get_or_insert(u);
    }

    // O(1) time
    fn pop(&mut self) -> Option<T> {
        let u = self.head.take()?;
        match Rc::try_unwrap(u) {
            Ok(mut u) => {
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
                Some(v.x)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SLList;
    use interface::Stack;

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
}
