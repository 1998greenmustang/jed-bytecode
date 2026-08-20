use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    rc::Rc,
    u8,
};

use rug::Integer;

use crate::{error::ProgramErrorKind, memory::list::List, utils};

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
}

impl From<ObjectData> for ObjectKind {
    fn from(value: ObjectData) -> Self {
        match value {
            ObjectData::Integer(_) => ObjectKind::Integer,
            ObjectData::Float(_, _) => ObjectKind::Float,
            ObjectData::UnsignedInt(_) => ObjectKind::UnsignedInt,
            ObjectData::String(items) => ObjectKind::String,
            ObjectData::Bool(_) => ObjectKind::Bool,
            ObjectData::Func(items) => ObjectKind::Func,
            ObjectData::List(_) => ObjectKind::List,
            ObjectData::Pointer(_) => ObjectKind::Pointer,
            ObjectData::Iterator(_, _) => ObjectKind::Iterator,
            ObjectData::BigInteger(integer) => ObjectKind::BigInteger,
            ObjectData::Nil => ObjectKind::Nil,
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone, PartialOrd, Ord)]
pub struct Object {
    pub kind: ObjectKind,
    pub data: ObjectData,
}

#[derive(Hash, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub enum ObjectData {
    Integer(isize),
    Float(i32, u32),
    UnsignedInt(usize),
    String(&'static [u8]),
    Bool(bool),
    Func(&'static [u8]),
    List(*mut Rc<RefCell<List<RegObject>>>),
    Pointer(*mut RegObject),
    Iterator(*const ObjectData, *mut usize), // start, next
    BigInteger(Integer),
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
            ObjectData::Float(i, p) => write!(f, "float ({i}.{p})"),
            ObjectData::UnsignedInt(u) => write!(f, "uint ({u})"),
            ObjectData::String(items) => {
                write!(f, "string (\"{}\")", utils::display_bytes(items))
            }
            ObjectData::Bool(b) => write!(f, "bool ({b:?})"),
            ObjectData::Func(items) => write!(f, "func ({})", utils::display_bytes(items)),
            ObjectData::Pointer(pr) => write!(f, "ptr ({pr:p})"),
            ObjectData::Nil => write!(f, "Nil"),
            ObjectData::List(list) => unsafe { write!(f, "list (@{:?})", list) },
            ObjectData::Iterator(list, next) => {
                write!(f, "iterate (@{:?}, next: {:?})", list, next)
            }
            ObjectData::BigInteger(i) => write!(f, "bigint ({i})"),
        }
    }
}

impl Display for ObjectData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectData::Integer(i) => write!(f, "{i}"),
            ObjectData::Float(i, p) => write!(f, "{i}.{p}"),
            ObjectData::String(s) => write!(f, "{}", utils::display_bytes(s)),
            ObjectData::Bool(b) => write!(f, "{b}"),
            ObjectData::Func(n) => write!(f, "{}", utils::display_bytes(n)),
            ObjectData::Pointer(pr) => write!(f, "{pr:p}"),
            ObjectData::Nil => write!(f, "Nil"),
            ObjectData::UnsignedInt(_) => todo!(),
            ObjectData::List(list) => write!(f, "{}", unsafe { (**list).borrow() }),
            ObjectData::Iterator(_list_ptr, _next) => write!(f, "<iterator>",),
            ObjectData::BigInteger(i) => write!(f, "{i}"),
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
        }
    }
}

impl Object {
    pub fn nil() -> Self {
        Self {
            kind: ObjectKind::Nil,
            data: ObjectData::Nil,
        }
    }
    pub fn as_tuple(&self) -> (ObjectKind, ObjectData) {
        return (self.kind, self.data);
    }
    pub fn as_ptr_mut(&mut self) -> *mut Object {
        &mut *self as *mut Object
    }
    pub fn as_ptr(&self) -> *const Object {
        &*self as *const Object
    }
}

impl From<bool> for Object {
    fn from(value: bool) -> Self {
        Object {
            kind: ObjectKind::Bool,
            data: ObjectData::Bool(value),
        }
    }
}

impl From<isize> for Object {
    fn from(value: isize) -> Self {
        Object {
            kind: ObjectKind::Integer,
            data: ObjectData::Integer(value),
        }
    }
}

impl From<(i32, u32)> for Object {
    fn from(value: (i32, u32)) -> Self {
        Object {
            kind: ObjectKind::Float,
            data: ObjectData::Float(value.0, value.1),
        }
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
