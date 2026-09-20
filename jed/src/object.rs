use crate::{
    error::ProgramErrorKind,
    memory::list::{List, ListIter},
    utils,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
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
    List(*mut Rc<RefCell<List<RegObject>>>),
    Pointer(*mut RegObject),
    Iterator(*mut Rc<RefCell<ListIter<RegObject>>>),
    // BigInteger(Integer),
    // TODO generator prolly just use the Iterator data
    Data(*mut HashMap<&'static [u8], RegObject>),
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
            ObjectData::List(list) => write!(f, "{}", unsafe { (**list).borrow() }),
            ObjectData::Iterator(iter) => write!(f, "<iterator (@{iter:?})>"),
            // ObjectData::BigInteger(i) => write!(f, "{i}"),
            ObjectData::Data(data) => write!(f, "{:?}", unsafe { (**data).clone() }),
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
            ObjectKind::UnsignedInt => todo!(),
            ObjectKind::Data => todo!(),
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
