use std::{
    convert::TryFrom,
    fmt::{Debug, Display},
    u8,
};

use crate::utils;

#[derive(PartialEq, Debug)]
// pub enum Object<'a> {
//     Literal(&'a Literal),
//     Func(usize), // usize => instruction pointer
// }
#[repr(u8)]
#[derive(Hash, Eq, Copy, Clone, PartialOrd, Ord)]
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
}

impl TryFrom<&u8> for ObjectKind {
    type Error = &'static str;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        if value > &7 {
            return Err("not a valid kind");
        } else {
            return Ok(unsafe { std::mem::transmute(*value) });
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
    Float(isize, usize),
    UnsignedInt(usize),
    String(&'static [u8]),
    Bool(bool),
    Func(&'static [u8]),
    List(*const Object, usize), // start, end
    Pointer(*mut &'static Object),
    Iterator(*const Object, *mut usize), // start, next
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
                write!(f, "string (\"{}\")", utils::bytes_to_string(items))
            }
            ObjectData::Bool(b) => write!(f, "bool ({b:?})"),
            ObjectData::Func(items) => write!(f, "func ({})", utils::bytes_to_string(items)),
            ObjectData::Pointer(pr) => write!(f, "ptr ({pr:p})"),
            ObjectData::Nil => write!(f, "Nil"),
            ObjectData::List(start, len) => write!(f, "list (@{start:p}, {len})"),
            ObjectData::Iterator(list, next) => todo!(),
        }
    }
}

impl Display for ObjectData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectData::Integer(i) => write!(f, "{i}"),
            ObjectData::Float(i, p) => write!(f, "{i}.{p}"),
            ObjectData::String(s) => write!(f, "{}", utils::bytes_to_string(s)),
            ObjectData::Bool(b) => write!(f, "{b}"),
            ObjectData::Func(n) => write!(f, "{}", utils::bytes_to_string(n)),
            ObjectData::Pointer(pr) => write!(f, "{pr:p}"),
            ObjectData::Nil => write!(f, "Nil"),
            ObjectData::UnsignedInt(_) => todo!(),
            ObjectData::List(start, len) => unsafe {
                write!(f, "[")?;
                for idx in 0..*len {
                    let addr = start.add(idx);
                    write!(f, "{}", (addr as *const Object).read())?;

                    if idx < len - 1 {
                        write!(f, ",")?
                    }
                }
                write!(f, "]")
            },
            ObjectData::Iterator(list_ptr, next) => unsafe { write!(f, "<iterator>",) },
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

impl From<(isize, usize)> for Object {
    fn from(value: (isize, usize)) -> Self {
        Object {
            kind: ObjectKind::Float,
            data: ObjectData::Float(value.0, value.1),
        }
    }
}
