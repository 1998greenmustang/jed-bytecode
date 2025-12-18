use std::collections::HashMap;

use crate::object::Object;

pub struct Frame {
    locals: HashMap<Object, Object>,
    pub return_address: usize,
}

impl Frame {
    pub fn new(return_address: usize) -> Self {
        Frame {
            locals: Default::default(),
            return_address,
        }
    }

    pub fn add_local(&mut self, name: Object, obj: Object) {
        // if self.locals.contains_key(name) {
        //     panic!("{} has already been declared");
        // }
        self.locals.insert(name, obj);
    }

    pub fn get_local(&self, name: &Object) -> Option<Object> {
        self.locals.get(name).cloned()
    }
}
