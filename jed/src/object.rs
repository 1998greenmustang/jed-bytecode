use crate::{
    RustObject,
    error::ProgramErrorKind,
    memory::list::{List, ListIter},
    utils,
};
use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    mem::ManuallyDrop,
    rc::Rc,
};

pub type MutableObject = &'static mut Object;
pub type RegObject = &'static Object;

#[repr(u8)]
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone, PartialOrd, Ord)]
pub enum ObjectKind {
    Integer,
    Float,
    String,
    Bool,
    Func,
    Pointer,
    Nil,
    List,
    Iterator,
    BigInteger,
    UnsignedInt,
    Data,
    RustObject,
}

impl From<ObjectData> for ObjectKind {
    fn from(value: ObjectData) -> Self {
        match value {
            ObjectData::Integer(_) => ObjectKind::Integer,
            ObjectData::Float(_) => ObjectKind::Float,
            ObjectData::UnsignedInt(_) => ObjectKind::UnsignedInt,
            ObjectData::String(_) => ObjectKind::String,
            ObjectData::Bool(_) => ObjectKind::Bool,
            ObjectData::Func(_) => ObjectKind::Func,
            ObjectData::List(_) => ObjectKind::List,
            ObjectData::Pointer(_) => ObjectKind::Pointer,
            ObjectData::Iterator(_) => ObjectKind::Iterator,
            // ObjectData::BigInteger(integer) => ObjectKind::BigInteger,
            ObjectData::Nil => ObjectKind::Nil,
            ObjectData::Data(_) => ObjectKind::Data,
            ObjectData::RustObject(_) => ObjectKind::RustObject,
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone, PartialOrd, Ord)]
pub struct Object {
    pub kind: ObjectKind,
    pub data: ObjectData,
    // pub attributes: *mut HashMap<&'static [u8], RegObject>,
}

#[derive(Hash, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub enum ObjectData {
    Integer(isize),
    Float(*mut f64),
    UnsignedInt(usize),
    String(&'static [u8]),
    Bool(bool),
    Func(&'static [u8]),
    List(*mut ManuallyDrop<List<RegObject>>),
    Pointer(*mut RegObject),
    Iterator(*mut ListIter<RegObject>),
    // BigInteger(Integer),
    // TODO generator prolly just use the Iterator data
    Data(*mut HashMap<&'static [u8], RegObject>),
    RustObject(*mut &'static dyn RustObject),
    Nil,
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl Debug for ObjectData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectData::Integer(i) => write!(f, "int ({i})"),
            ObjectData::Float(ft) => write!(f, "float ({})", unsafe { **ft }),
            ObjectData::UnsignedInt(u) => write!(f, "uint ({u})"),
            ObjectData::String(items) => {
                write!(f, "string (\"{}\")", utils::display_bytes(items))
            }
            ObjectData::Bool(b) => write!(f, "bool ({b:?})"),
            ObjectData::Func(items) => write!(f, "func ({})", utils::display_bytes(items)),
            ObjectData::Pointer(pr) => write!(f, "ptr ({pr:p})"),
            ObjectData::Nil => write!(f, "Nil"),
            ObjectData::List(list) => write!(f, "list (@{:?})", list),
            ObjectData::Iterator(iter) => {
                write!(f, "iterate (@{:?})", iter)
            } // ObjectData::BigInteger(i) => write!(f, "bigint ({i})"),
            ObjectData::Data(_) => todo!(),
            ObjectData::RustObject(any) => unsafe { (**any).jed_debug(f) },
        }
    }
}

impl Display for ObjectData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectData::Integer(i) => write!(f, "{i}"),
            ObjectData::Float(ft) => write!(f, "{}", unsafe { **ft }),
            ObjectData::String(s) => write!(f, "{}", utils::display_bytes(s)),
            ObjectData::Bool(b) => write!(f, "{b}"),
            ObjectData::Func(n) => write!(f, "{}", utils::display_bytes(n)),
            ObjectData::Pointer(pr) => write!(f, "{pr:p}"),
            ObjectData::Nil => write!(f, "Nil"),
            ObjectData::UnsignedInt(u) => write!(f, "{u}"),
            ObjectData::List(list) => write!(f, "{}", unsafe { (***list).clone() }),
            ObjectData::Iterator(iter) => write!(f, "<iterator (@{iter:?})>"),
            // ObjectData::BigInteger(i) => write!(f, "{i}"),
            ObjectData::Data(data) => write!(f, "{:?}", unsafe { (**data).clone() }),
            ObjectData::RustObject(any) => unsafe { (**any).jed_display(f) },
        }
    }
}

impl Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectKind::Integer => write!(f, "Integer"),
            ObjectKind::Float => write!(f, "Float"),
            ObjectKind::String => write!(f, "String"),
            ObjectKind::Bool => write!(f, "Bool"),
            ObjectKind::Func => write!(f, "Func"),
            ObjectKind::Pointer => write!(f, "Pointer"),
            ObjectKind::Nil => write!(f, "Nil"),
            ObjectKind::List => write!(f, "List"),
            ObjectKind::Iterator => write!(f, "Iterator"),
            ObjectKind::BigInteger => write!(f, "BigInteger"),
            ObjectKind::UnsignedInt => write!(f, "UnsignedInt"),
            ObjectKind::Data => write!(f, "Data"),
            ObjectKind::RustObject => write!(f, "RustObject"),
        }
    }
}

impl Object {
    #[inline]
    pub fn new(data: ObjectData) -> Self {
        Self {
            kind: data.into(),
            data,
        }
    }
    pub fn nil() -> Self {
        Self::new(ObjectData::Nil)
    }
    pub fn as_tuple(&self) -> (ObjectKind, ObjectData) {
        return (self.kind, self.data);
    }
}

impl From<bool> for Object {
    fn from(value: bool) -> Self {
        Self::new(ObjectData::Bool(value))
    }
}

impl From<isize> for Object {
    fn from(value: isize) -> Self {
        Self::new(ObjectData::Integer(value))
    }
}

impl From<f64> for Object {
    fn from(value: f64) -> Self {
        Self::new(ObjectData::Float(Box::into_raw(Box::new(value))))
    }
}

impl From<&'static dyn RustObject> for Object {
    fn from(value: &'static dyn RustObject) -> Self {
        let bx = Box::new(value);
        Self::new(ObjectData::RustObject(Box::into_raw(bx)))
    }
}
impl From<&'static mut dyn RustObject> for Object {
    fn from(value: &'static mut dyn RustObject) -> Self {
        let bx = Box::new(value as &'static dyn RustObject);
        Self::new(ObjectData::RustObject(Box::into_raw(bx)))
    }
}

impl From<&'static [u8]> for Object {
    fn from(value: &'static [u8]) -> Self {
        Self::new(ObjectData::String(value))
    }
}

impl TryInto<usize> for ObjectData {
    type Error = ProgramErrorKind;

    fn try_into(self) -> Result<usize, Self::Error> {
        match self {
            ObjectData::Integer(i) => Ok(i as usize),
            ObjectData::UnsignedInt(u) => Ok(u),
            _ => Err(ProgramErrorKind::TypeError(
                ObjectKind::Integer,
                self.into(),
            )),
        }
    }
}
