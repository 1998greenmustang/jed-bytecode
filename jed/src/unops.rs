use std::{convert::TryInto, fmt::Display};

use crate::{
    error::ProgramErrorKind,
    object::{Object, ObjectData},
};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum UnOpKind {
    Increment,
    Decrement,
    Plus,
    Minus,
    Not,
    BitNot,
}

impl Display for UnOpKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOpKind::Increment => write!(f, "++"),
            UnOpKind::Decrement => write!(f, "--"),
            UnOpKind::Plus => write!(f, "+"),
            UnOpKind::Minus => write!(f, "-"),
            UnOpKind::Not => write!(f, "!"),
            UnOpKind::BitNot => write!(f, "~"),
        }
    }
}

impl From<&str> for UnOpKind {
    fn from(value: &str) -> Self {
        match value {
            "++" => UnOpKind::Increment,
            "--" => UnOpKind::Decrement,
            "+" => UnOpKind::Plus,
            "-" => UnOpKind::Minus,
            "!" => UnOpKind::Not,
            "~" => UnOpKind::BitNot,
            _ => panic!("Unary operator not implemented: '{}'", value),
        }
    }
}

impl From<u8> for UnOpKind {
    fn from(value: u8) -> Self {
        assert!(value <= 5);
        unsafe { std::mem::transmute(value) }
    }
}

impl UnOpKind {
    pub fn call(&self, operand: Object) -> Object {
        match self {
            UnOpKind::Increment => todo!("UnOpKind::Increment"),
            UnOpKind::Decrement => todo!("UnOpKind::Decrement"),
            UnOpKind::Plus => todo!("UnOpKind::Plus"),
            UnOpKind::Minus => todo!("UnOpKind::Minus"),
            UnOpKind::Not => todo!("UnOpKind::Not"),
            UnOpKind::BitNot => todo!("UnOpKind::BitNot"),
        }
    }
}
