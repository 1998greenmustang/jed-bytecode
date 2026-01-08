use crate::{mutable::MutableObject, object::Object};

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
    pub fn call_with_slice(&self, slice: &[Object]) {
        match self {
            BuiltIn::PrintLn => {
                let (last, rest) = slice.split_last().unwrap();
                let mut output = "[".to_owned();
                for obj in rest {
                    output += &std::format!("{obj},");
                }
                output += &std::format!("{last}]");
                println!("{output}");
            }
        }
    }
}

impl From<&String> for BuiltIn {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "println" => BuiltIn::PrintLn,
            _ => panic!("No such builtin '{}'", value),
        }
    }
}
