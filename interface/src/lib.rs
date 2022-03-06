pub trait Queue<T> {
    fn add(&mut self, x: T);
    fn remove(&mut self) -> Option<T>;
}

pub trait Stack<T> {
    fn push(&mut self, x: T);
    fn pop(&mut self) -> Option<T>;
}
