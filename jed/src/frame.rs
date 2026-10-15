use crate::object::Object;

#[derive(Clone)]
pub struct Frame {
    pub base: usize,
    pub return_address: usize,
    pub internal: Option<Object>,
}

impl Frame {
    pub fn new(base: usize, return_address: usize) -> Self {
        Frame {
            base,
            return_address,
            internal: None,
        }
    }

    pub fn set_internal(&mut self, obj: Object) {
        self.internal = Some(obj)
    }
}
