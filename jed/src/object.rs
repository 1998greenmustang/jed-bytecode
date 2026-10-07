use crate::{
    RustObject,
    memory::list::{List, ListIter},
    object::ObjectKind::*,
    utils,
};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    mem::ManuallyDrop,
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

#[derive(Copy, Clone)]
pub struct Object {
    pub kind: ObjectKind,
    pub data: ObjectData,
    // pub attributes: *mut HashMap<&'static [u8], RegObject>,
}

#[derive(Copy, Clone)]
pub union ObjectData {
    pub integer: i64,
    pub float: f64,
    pub unsigned: u64,
    pub bytes: &'static [u8],
    pub boolean: bool,
    pub list: *mut ManuallyDrop<List<RegObject>>,
    pub object: *mut RegObject,
    pub listiter: *mut ListIter<RegObject>,
    pub hashmap: *mut HashMap<&'static [u8], RegObject>,
    pub rustobj: *mut &'static dyn RustObject,
    nil: u64,
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            match self.kind {
                ObjectKind::Integer => write!(f, "{}", self.data.integer),
                ObjectKind::Float => write!(f, "{}", self.data.float),
                ObjectKind::String => write!(f, "{}", utils::display_bytes(self.data.bytes)),
                ObjectKind::Bool => write!(f, "{}", self.data.boolean),
                ObjectKind::Func => write!(f, "{}", utils::display_bytes(self.data.bytes)),
                ObjectKind::Pointer => write!(f, "<ptr {:p}>", self.data.object),
                ObjectKind::Nil => write!(f, "{}", "Nil"),
                ObjectKind::List => write!(f, "{}", (**self.data.list).clone()),
                ObjectKind::Iterator => write!(f, "<iterator {:p}>", self.data.listiter),
                ObjectKind::UnsignedInt => write!(f, "{}", self.data.unsigned),
                // ObjectKind::Data => write!(f, "{:?}", unsafe { *self.data.hashmap }),
                ObjectKind::RustObject => (*self.data.rustobj).jed_display(f),
                _ => todo!("{:?}", self.kind),
            }
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
    pub fn new_integer(data: i64) -> Self {
        Self {
            kind: Integer,
            data: ObjectData { integer: data },
        }
    }
    #[inline]
    pub fn new_float(data: f64) -> Self {
        Self {
            kind: Float,
            data: ObjectData { float: data },
        }
    }
    #[inline]
    pub fn new_string(data: &'static [u8]) -> Self {
        Self {
            kind: String,
            data: ObjectData { bytes: data },
        }
    }
    #[inline]
    pub fn new_bool(data: bool) -> Self {
        Self {
            kind: Bool,
            data: ObjectData { boolean: data },
        }
    }
    #[inline]
    pub fn new_func(data: &'static [u8]) -> Self {
        Self {
            kind: Func,
            data: ObjectData { bytes: data },
        }
    }
    #[inline]
    pub fn new_pointer(data: *mut RegObject) -> Self {
        Self {
            kind: Pointer,
            data: ObjectData { object: data },
        }
    }
    #[inline]
    pub fn new_nil() -> Self {
        Self {
            kind: Nil,
            data: ObjectData { nil: 0 },
        }
    }
    #[inline]
    pub fn new_list(data: *mut ManuallyDrop<List<RegObject>>) -> Self {
        Self {
            kind: List,
            data: ObjectData { list: data },
        }
    }
    #[inline]
    pub fn new_iterator(data: *mut ListIter<RegObject>) -> Self {
        Self {
            kind: Iterator,
            data: ObjectData { listiter: data },
        }
    }
    #[inline]
    // pub fn new_bigInteger (data: ) -> Self { Self {kind: BigInteger, data: ObjectData { : data }}}
    // #[inline]
    pub fn new_unsigned(data: u64) -> Self {
        Self {
            kind: UnsignedInt,
            data: ObjectData { unsigned: data },
        }
    }
    #[inline]
    pub fn new_data(data: *mut HashMap<&'static [u8], RegObject>) -> Self {
        Self {
            kind: Data,
            data: ObjectData { hashmap: data },
        }
    }
    #[inline]
    pub fn new_rust_object(data: *mut &'static (dyn RustObject + 'static)) -> Self {
        Self {
            kind: RustObject,
            data: ObjectData { rustobj: data },
        }
    }
    #[inline]
    pub fn integer(&self) -> i64 {
        unsafe { self.data.integer }
    }
    #[inline]
    pub fn float(&self) -> f64 {
        unsafe { self.data.float }
    }
    #[inline]
    pub fn string(&self) -> &'static [u8] {
        unsafe { self.data.bytes }
    }
    #[inline]
    pub fn bool(&self) -> bool {
        unsafe { self.data.boolean }
    }
    #[inline]
    pub fn func(&self) -> &'static [u8] {
        unsafe { self.data.bytes }
    }
    #[inline]
    pub fn pointer(&self) -> *mut RegObject {
        unsafe { self.data.object }
    }
    #[inline]
    pub fn list(&self) -> *mut ManuallyDrop<List<RegObject>> {
        unsafe { self.data.list }
    }
    #[inline]
    pub fn iterator(&self) -> *mut ListIter<RegObject> {
        unsafe { self.data.listiter }
    }
    #[inline]
    // pub fn bigInteger (data: ) -> Self { Self {kind: BigInteger, data: ObjectData { : data }}}
    // #[inline]
    pub fn unsigned(&self) -> u64 {
        unsafe { self.data.unsigned }
    }
    #[inline]
    pub fn data(&self) -> *mut HashMap<&'static [u8], RegObject> {
        unsafe { self.data.hashmap }
    }
    #[inline]
    pub fn rust_object(&self) -> *mut &'static (dyn RustObject + 'static) {
        unsafe { self.data.rustobj }
    }

    #[inline]
    pub fn is(&self, kind: ObjectKind) -> bool {
        self.kind == kind
    }

    #[inline]
    pub fn is_usizeable(&self) -> bool {
        self.is(Integer) || self.is(UnsignedInt)
    }

    #[inline]
    pub fn usize(&self) -> usize {
        match self.kind {
            ObjectKind::Integer => self.integer().try_into().expect("TODO integer into usize"),
            ObjectKind::UnsignedInt => self.usize().try_into().expect("TODO unsigned into usize"),
            _ => unreachable!("you shouldve checked if this is usizeable..."),
        }
    }
}

impl From<bool> for Object {
    fn from(value: bool) -> Self {
        Self::new_bool(value)
    }
}

impl From<i64> for Object {
    fn from(value: i64) -> Self {
        Self::new_integer(value)
    }
}

impl From<u64> for Object {
    fn from(value: u64) -> Self {
        Self::new_unsigned(value)
    }
}

impl From<f64> for Object {
    fn from(value: f64) -> Self {
        Self::new_float(value)
    }
}

impl From<&'static dyn RustObject> for Object {
    fn from(value: &'static dyn RustObject) -> Self {
        let bx = Box::new(value);
        Self::new_rust_object(Box::into_raw(bx))
    }
}
impl From<&'static mut dyn RustObject> for Object {
    fn from(value: &'static mut dyn RustObject) -> Self {
        let bx = Box::new(value as &'static dyn RustObject);
        Self::new_rust_object(Box::into_raw(bx))
    }
}

impl From<&'static [u8]> for Object {
    fn from(value: &'static [u8]) -> Self {
        Self::new_string(value)
    }
}
