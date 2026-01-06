use crate::object::Object;

#[derive(Debug, Clone, Copy)]
pub enum MutableObject {
    List(&'static [Object]), // a literal list of MutablePtr Objects
    Object(Object),
}

impl From<Object> for MutableObject {
    fn from(value: Object) -> Self {
        MutableObject::Object(value)
    }
}
