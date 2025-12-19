pub struct Set<V> {
    data: Vec<V>,
}

impl<V: std::cmp::PartialEq> Set<V> {
    pub fn new() -> Self {
        Set { data: vec![] }
    }

    pub fn push(&mut self, item: V) {
        if !self.data.contains(&item) {
            self.data.push(item)
        }
    }

    pub fn index_of(&self, item: &V) -> Option<usize> {
        for (idx, val) in self.data.iter().enumerate() {
            if val == item {
                return Some(idx);
            }
        }
        return None;
    }
}

pub struct Map<K, V> {
    keys: Set<K>,
    values: Vec<V>,
}

impl<K, V> Map<K, V>
where
    K: std::cmp::PartialEq,
{
    pub fn new() -> Self {
        Map {
            keys: Set::new(),
            values: vec![],
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if let Some(exists) = self.keys.index_of(&key) {
            self.values.remove(exists);
            self.values.insert(exists, value);
        } else {
            self.keys.push(key);
            self.values.push(value);
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        if let Some(idx) = self.keys.index_of(key) {
            self.values.get(idx)
        } else {
            None
        }
    }
}
