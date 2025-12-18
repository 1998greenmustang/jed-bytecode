#[derive(Debug)]
pub struct Stack<T>(Vec<T>);

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack(vec![])
    }

    pub fn append(&mut self, mut v: Vec<T>) {
        self.0.append(&mut v);
    }

    pub fn push(&mut self, v: T) {
        self.0.push(v);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }

    pub fn get(&mut self, idx: usize) -> Option<&T> {
        self.0.get(idx)
    }

    pub fn len(&mut self) -> usize {
        self.0.len()
    }

    pub fn last(&mut self) -> Option<&T> {
        self.0.last()
    }
    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.0.last_mut()
    }
}
