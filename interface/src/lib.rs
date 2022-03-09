pub trait Queue<T> {
    fn add(&mut self, x: T);
    fn remove(&mut self) -> Option<T>;
}

pub trait Stack<T> {
    fn push(&mut self, x: T);
    fn pop(&mut self) -> Option<T>;
}

pub trait List<T> {
    fn size(&self) -> usize;
    fn get(&self, i: usize) -> Option<&T>;
    fn set(&self, i: usize, x: T) -> T;
    fn add(&mut self, i: usize, x: T);
    fn remove(&mut self, i: usize) -> T;
}
