use std::{convert::TryFrom, fmt::Display, u8};

use crate::mutable::MutableObject;

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
    MutablePtr,
    Nil,
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
pub struct Object(pub ObjectKind, pub &'static [u8]);

impl Object {
    pub fn dummy() -> Object {
        Object(ObjectKind::Nil, &[])
    }

    pub fn pointer(&self) -> usize {
        if self.0 == ObjectKind::MutablePtr || self.0 == ObjectKind::Integer {
            usize::from_be_bytes(std::array::from_fn(|i| self.1[i]))
        } else {
            panic!("Not a pointer.")
        }
    }

    pub fn string(&self) -> String {
        if self.0 == ObjectKind::String {
            unsafe { String::from_utf8_unchecked(self.1.to_vec()) }
        } else {
            panic!("Not a string.");
        }
    }

    pub fn integer(&self) -> i64 {
        if self.0 == ObjectKind::Integer {
            i64::from_be_bytes(std::array::from_fn(|i| self.1[i]))
        } else {
            panic!("Not a integer.");
        }
    }

    pub fn float(&self) -> f64 {
        if self.0 == ObjectKind::Float {
            f64::from_be_bytes(std::array::from_fn(|i| self.1[i]))
        } else {
            panic!("Not a float.");
        }
    }

    pub fn bool(&self) -> bool {
        if self.0 == ObjectKind::Bool {
            match unsafe { String::from_utf8_unchecked(self.1.to_vec()) }.as_str() {
                "true" => true,
                "false" => false,
                _ => unreachable!(),
            }
        } else {
            panic!("Not a bool.");
        }
    }

    pub fn as_bytes(self) -> Vec<u8> {
        let key: u8 = self.0 as u8;
        let mut data = self.1.to_vec();
        data.insert(0, key);
        return data;
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ObjectKind::Integer => write!(f, "{}", {
                assert!(self.0 == ObjectKind::Integer, "Not a integer!");
                let bytes: [u8; 8] = std::array::from_fn(|i| self.1[i]);
                i64::from_be_bytes(bytes)
            }),
            ObjectKind::Float => write!(f, "{}", {
                assert!(self.0 == ObjectKind::Float, "Not a float!");
                let bytes: [u8; 8] = std::array::from_fn(|i| self.1[i]);
                f64::from_be_bytes(bytes)
            }),
            ObjectKind::String => write!(f, "{}", unsafe {
                std::mem::transmute::<&'static [u8], &str>(self.1)
            }),
            ObjectKind::Bool => write!(f, "{}", unsafe {
                match std::mem::transmute::<&'static [u8], &str>(self.1) {
                    "true" => true,
                    "false" => false,
                    _ => unreachable!("Nice memory!"),
                }
            }),
            ObjectKind::Func => write!(f, "<func {}>", self.string()),
            ObjectKind::Nil => write!(f, "Nil"),
            ObjectKind::MutablePtr => write!(f, "<ptr @{}>", self.pointer()),

            _ => todo!(),
        }
    }
}

impl From<&MutableObject> for Object {
    fn from(value: &MutableObject) -> Self {
        match value {
            MutableObject::Object(object) => *object,
            _ => unreachable!(),
        }
    }
}
