use std::{
    convert::TryFrom,
    fmt::{Debug, Display},
    u8,
};

use crate::utils;

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
    Float(i32, u32),
    UnsignedInt(usize),
    String(&'static [u8]),
    Bool(bool),
    Func(&'static [u8]),
    List(*mut usize, *mut usize), // *mut usize), // pointer address to the starting object, end, allocated TODO
    Pointer(*mut &'static Object),
    Iterator(*const ObjectData, *mut usize), // start, next
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
            ObjectData::List(start, len) => unsafe {
                write!(f, "list (@{:p}, {})", **start as *const Object, **len)
            },
            ObjectData::Iterator(list, next) => unsafe {
                write!(f, "iterate (@{:?}, next: {:?})", list, next)
            },
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
                let start = **start as *const Object;
                write!(f, "[")?;
                for idx in 0..**len {
                    let addr = start.add(idx);
                    write!(f, "{}", (addr as *const Object).read())?;

                    if idx < (**len) - 1 {
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

impl From<(i32, u32)> for Object {
    fn from(value: (i32, u32)) -> Self {
        Object {
            kind: ObjectKind::Float,
            data: ObjectData::Float(value.0, value.1),
        }
    }
}
