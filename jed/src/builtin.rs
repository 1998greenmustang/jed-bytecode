use std::fmt::Display;

use crate::object::{Object, ObjectData};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum BuiltIn {
    PrintLn,
    Sqrt,
}

impl BuiltIn {
    pub fn call(&self, arg: Object) -> Option<Object> {
        match self {
            BuiltIn::PrintLn => {
                println!("{arg}");

                None
            }
            BuiltIn::Sqrt => Some(object_sqrt(arg)),
        }
    }
}

impl Display for BuiltIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuiltIn::PrintLn => write!(f, "println"),
            BuiltIn::Sqrt => write!(f, "sqrt"),
        }
    }
}

impl From<&str> for BuiltIn {
    fn from(value: &str) -> Self {
        match value {
            "println" => BuiltIn::PrintLn,
            "sqrt" => BuiltIn::Sqrt,
            _ => panic!("No such builtin '{}'", value),
        }
    }
}

impl From<u8> for BuiltIn {
    fn from(value: u8) -> Self {
        match value {
            0 => BuiltIn::PrintLn,
            1 => BuiltIn::Sqrt,
            _ => panic!(),
        }
    }
}

fn object_sqrt(obj: Object) -> Object {
    match obj.data {
        ObjectData::Integer(i) => i.isqrt().into(),
        ObjectData::Float(i, p) => todo!(),
        ObjectData::UnsignedInt(i) => todo!(),
        _ => panic!(),
    }
}
