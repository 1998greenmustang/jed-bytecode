// Not like the trait
// But yk its cool

pub struct PeekableIterator<T> {
    data: Vec<T>,
    index: usize,
    history: Vec<usize>, // keeps track of the times it changed; stores the old self.index
}

impl<T: Clone + PartialEq + std::fmt::Debug> PeekableIterator<T> {
    fn new(data: Vec<T>) -> Self {
        return PeekableIterator {
            data,
            index: 0,
            history: vec![],
        };
    }

    pub fn undo(&mut self) {
        self.index = self.history.pop().unwrap_or_default();
    }

    pub fn peek(&self) -> Option<&T> {
        self.data.get(self.index)
    }

    pub fn until(&mut self, predicate: impl Fn(&T) -> bool) -> Option<&[T]> {
        let mut v = vec![];
        for i in self.index..self.data.len() {
            let item = &self.data[i];
            if !predicate(item) {
                v.push(item.clone());
            } else {
                break;
            }
        }
        if !v.is_empty() {
            self.history.push(self.index);
            self.index += v.len();
            return Some(v.leak());
        } else {
            return None;
        }
    }

    pub fn until_inclusive(&mut self, predicate: impl Fn(&T) -> bool) -> Option<&[T]> {
        let mut v = vec![];
        for i in self.index..self.data.len() {
            let item = &self.data[i];
            if !predicate(item) {
                v.push(item.clone());
            } else {
                v.push(item.clone());
                break;
            }
        }
        if !v.is_empty() {
            self.history.push(self.index);
            self.index += v.len();
            return Some(v.leak());
        } else {
            return None;
        }
    }

    #[inline]
    pub fn until_any(&mut self, slice: &[T]) -> Option<&[T]> {
        self.until(|item| slice.contains(item))
    }
    #[inline]
    pub fn until_any_inclusive(&mut self, slice: &[T]) -> Option<&[T]> {
        self.until_inclusive(|item| slice.contains(item))
    }

    pub fn consume(&mut self, n: usize) {
        self.history.push(self.index);
        self.index += n;
    }

    pub fn next(&mut self) -> Option<&T> {
        let tmp = self.index;
        self.index += 1;
        self.history.push(tmp);
        self.data.get(tmp)
    }
}

impl<T> FromIterator<T> for PeekableIterator<T> {
    fn from_iter<A: IntoIterator<Item = T>>(iter: A) -> Self {
        PeekableIterator {
            data: iter.into_iter().collect(),
            index: 0,
            history: vec![],
        }
    }
}
