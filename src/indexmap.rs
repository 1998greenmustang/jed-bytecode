use std::collections::HashMap;

use crate::arena::Dropless;

type Index = usize;

pub struct IndexMap<V> {
    arena: Dropless,
    innards: HashMap<Index, V>,
    top: Index,
}

impl<V> IndexMap<V> {
    pub fn new() -> Self {
        Self {
            arena: Default::default(),
            innards: HashMap::new(),
            top: 0,
        }
    }

    pub fn insert(&mut self, idx: usize, element: V) -> Option<V> {
        // TODO add to self.top
        self.innards.insert(idx, element)
    }

    pub fn get(&self, idx: &usize) -> Option<&V> {
        self.innards.get(idx)
    }

    pub fn get_mut(&mut self, idx: &usize) -> Option<&mut V> {
        self.innards.get_mut(idx)
    }

    pub fn push(&mut self, element: V) -> Index {
        let _ = self.innards.insert(self.top, element);
        self.top += 1;
        return self.top - 1;
    }

    pub fn elements(&self) -> std::collections::hash_map::Iter<'_, Index, V> {
        self.innards.iter()
    }
}

pub struct IndexSet<V>
where
    V: Eq,
{
    innards: IndexMap<V>,
}

impl<V> IndexSet<V>
where
    V: Eq,
{
    pub fn new() -> Self {
        Self {
            innards: IndexMap::new(),
        }
    }

    pub fn add(&mut self, element: V) -> Index {
        let filtered: Vec<(&Index, &V)> = self
            .innards
            .elements()
            .filter(|kv| element == *kv.1)
            .collect();

        if filtered.len() == 1 {
            return *filtered.get(0).unwrap().0;
        } else {
            self.innards.push(element)
        }
    }

    pub fn get(&self, idx: &usize) -> Option<&V> {
        self.innards.get(idx)
    }
}
