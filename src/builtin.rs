use std::fmt::Display;

use crate::object::Object;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum BuiltIn {
    PrintLn,
}

impl BuiltIn {
    pub fn call(&self, arg: Object) {
        match self {
            BuiltIn::PrintLn => println!("{arg}"),
        }
    }
}

impl Display for BuiltIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuiltIn::PrintLn => write!(f, "println"),
        }
    }
}

impl From<&str> for BuiltIn {
    fn from(value: &str) -> Self {
        match value {
            "println" => BuiltIn::PrintLn,
            _ => panic!("No such builtin '{}'", value),
        }
    }
}
