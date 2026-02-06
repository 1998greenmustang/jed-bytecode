use std::{convert::TryInto, fmt::Display};

use crate::{
    error::ProgramErrorKind,
    object::{Object, ObjectData},
};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    Eq,
    LessEq,
    GreatEq,
    Lesser,
    Greater,
    And,
    Or,

    Power,
    Root,
}

impl Display for BinOpKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOpKind::Add => write!(f, "+"),
            BinOpKind::Sub => write!(f, "-"),
            BinOpKind::Mul => write!(f, "*"),
            BinOpKind::Div => write!(f, "/"),
            BinOpKind::Mod => write!(f, "%"),
            BinOpKind::Eq => write!(f, "=="),
            BinOpKind::LessEq => write!(f, "<="),
            BinOpKind::GreatEq => write!(f, ">="),
            BinOpKind::Lesser => write!(f, "<"),
            BinOpKind::Greater => write!(f, ">"),
            BinOpKind::And => write!(f, "&&"),
            BinOpKind::Or => write!(f, "||"),
            BinOpKind::Power => write!(f, "pow"),
            BinOpKind::Root => write!(f, "root"),
        }
    }
}

impl From<&str> for BinOpKind {
    fn from(value: &str) -> Self {
        match value {
            "+" => BinOpKind::Add,
            "-" => BinOpKind::Sub,
            "*" => BinOpKind::Mul,
            "/" => BinOpKind::Div,
            "==" => BinOpKind::Eq,
            "<=" => BinOpKind::LessEq,
            ">=" => BinOpKind::GreatEq,
            "<" => BinOpKind::Lesser,
            ">" => BinOpKind::Greater,
            "%" => BinOpKind::Mod,
            "&&" => BinOpKind::And,
            "||" => BinOpKind::Or,
            "pow" => BinOpKind::Power,
            "root" => BinOpKind::Root,
            _ => panic!("Binary operator not implemented: '{}'", value),
        }
    }
}

impl From<u8> for BinOpKind {
    fn from(value: u8) -> Self {
        assert!(value <= 12);
        unsafe { std::mem::transmute(value) }
    }
}

pub fn add(lhs: ObjectData, rhs: ObjectData) -> Result<Object, ProgramErrorKind> {
    match (lhs, rhs) {
        (ObjectData::Integer(left), ObjectData::Integer(right)) => match left.checked_add(right) {
            Some(v) => Ok(v.into()),
            None => Err(ProgramErrorKind::Overflow(BinOpKind::Add, left, right)),
        },
        (ObjectData::Float(_, _), ObjectData::Float(_, _)) => todo!(),
        (ObjectData::Float(_, _), ObjectData::Integer(_)) => return add(rhs, lhs),
        (ObjectData::Integer(left), ObjectData::Float(rightw, leftp)) => {
            let rightw: isize = rightw.try_into().unwrap();
            let whole: isize = match left.checked_add(rightw) {
                Some(w) => w,
                None => return Err(ProgramErrorKind::Overflow(BinOpKind::Add, left, rightw)),
            };
            let prec: u32 = leftp;
            return Ok((whole.try_into().unwrap(), prec).into());
        }
        (ObjectData::String(_), ObjectData::String(_)) => todo!(),
        _ => Err(ProgramErrorKind::BinopError(BinOpKind::Add, lhs, rhs)),
    }
}

pub fn sub(lhs: ObjectData, rhs: ObjectData) -> Result<Object, ProgramErrorKind> {
    match (lhs, rhs) {
        (ObjectData::Integer(left), ObjectData::Integer(right)) => match left.checked_sub(right) {
            Some(v) => Ok(v.into()),
            None => Err(ProgramErrorKind::Overflow(BinOpKind::Sub, left, right)),
        },
        (ObjectData::Float(_, _), ObjectData::Float(_, _)) => todo!(),
        _ => Err(ProgramErrorKind::BinopError(BinOpKind::Sub, lhs, rhs)),
    }
}
pub fn mul(lhs: ObjectData, rhs: ObjectData) -> Result<Object, ProgramErrorKind> {
    match (lhs, rhs) {
        (ObjectData::Integer(left), ObjectData::Integer(right)) => match left.checked_mul(right) {
            Some(v) => Ok(v.into()),
            None => Err(ProgramErrorKind::Overflow(BinOpKind::Mul, left, right)),
        },
        (ObjectData::Float(_, _), ObjectData::Float(_, _)) => todo!(),
        _ => Err(ProgramErrorKind::BinopError(BinOpKind::Mul, lhs, rhs)),
    }
}
pub fn div(lhs: ObjectData, rhs: ObjectData) -> Result<Object, ProgramErrorKind> {
    match (lhs, rhs) {
        (ObjectData::Integer(_), ObjectData::Integer(_)) => todo!(),
        (ObjectData::Float(_, _), ObjectData::Float(_, _)) => todo!(),
        _ => Err(ProgramErrorKind::BinopError(BinOpKind::Div, lhs, rhs)),
    }
}
pub fn modulus(lhs: ObjectData, rhs: ObjectData) -> Result<Object, ProgramErrorKind> {
    match (lhs, rhs) {
        (ObjectData::Integer(left), ObjectData::Integer(right)) => Ok((left % right).into()),
        (ObjectData::Float(_, _), ObjectData::Float(_, _)) => todo!(),
        _ => Err(ProgramErrorKind::BinopError(BinOpKind::Mod, lhs, rhs)),
    }
}
pub fn eq(lhs: ObjectData, rhs: ObjectData) -> Result<Object, ProgramErrorKind> {
    match (lhs, rhs) {
        (ObjectData::Integer(left), ObjectData::Integer(right)) => Ok((left == right).into()),
        (ObjectData::Float(_, _), ObjectData::Float(_, _)) => todo!(),
        _ => Err(ProgramErrorKind::BinopError(BinOpKind::Eq, lhs, rhs)),
    }
}
pub fn lesser(lhs: ObjectData, rhs: ObjectData) -> Result<Object, ProgramErrorKind> {
    match (lhs, rhs) {
        (ObjectData::Integer(_), ObjectData::Integer(_)) => todo!(),
        (ObjectData::Float(_, _), ObjectData::Float(_, _)) => todo!(),
        _ => Err(ProgramErrorKind::BinopError(BinOpKind::Lesser, lhs, rhs)),
    }
}
pub fn greater(lhs: ObjectData, rhs: ObjectData) -> Result<Object, ProgramErrorKind> {
    match (lhs, rhs) {
        (ObjectData::Integer(_), ObjectData::Integer(_)) => todo!(),
        (ObjectData::Float(_, _), ObjectData::Float(_, _)) => todo!(),
        _ => Err(ProgramErrorKind::BinopError(BinOpKind::Greater, lhs, rhs)),
    }
}
pub fn lesseq(lhs: ObjectData, rhs: ObjectData) -> Result<Object, ProgramErrorKind> {
    match (lhs, rhs) {
        (ObjectData::Integer(left), ObjectData::Integer(right)) => Ok((left <= right).into()),
        // (ObjectData::Float(lefti, leftp), ObjectData::Float(lefti, leftp)) => left <= right,
        _ => Err(ProgramErrorKind::BinopError(BinOpKind::LessEq, lhs, rhs)),
    }
}
pub fn greateq(lhs: ObjectData, rhs: ObjectData) -> Result<Object, ProgramErrorKind> {
    match (lhs, rhs) {
        (ObjectData::Integer(_), ObjectData::Integer(_)) => todo!(),
        (ObjectData::Float(_, _), ObjectData::Float(_, _)) => todo!(),
        _ => Err(ProgramErrorKind::BinopError(BinOpKind::GreatEq, lhs, rhs)),
    }
}
pub fn and(lhs: ObjectData, rhs: ObjectData) -> Result<Object, ProgramErrorKind> {
    match (lhs, rhs) {
        (ObjectData::Bool(left), ObjectData::Bool(right)) => Ok((left && right).into()),
        _ => Err(ProgramErrorKind::BinopError(BinOpKind::And, lhs, rhs)),
    }
}
pub fn or(lhs: ObjectData, rhs: ObjectData) -> Result<Object, ProgramErrorKind> {
    match (lhs, rhs) {
        (ObjectData::Bool(left), ObjectData::Bool(right)) => Ok((left || right).into()),
        _ => Err(ProgramErrorKind::BinopError(BinOpKind::Or, lhs, rhs)),
    }
}

pub fn pow(lhs: ObjectData, rhs: ObjectData) -> Result<Object, ProgramErrorKind> {
    match (lhs, rhs) {
        (ObjectData::Integer(left), ObjectData::Integer(right)) => {
            Ok(left.pow(right.try_into().expect("no")).into())
        }
        (ObjectData::Float(_, _), ObjectData::Float(_, _)) => todo!(),
        _ => Err(ProgramErrorKind::BinopError(BinOpKind::Or, lhs, rhs)),
    }
}

pub fn root(lhs: ObjectData, rhs: ObjectData) -> Result<Object, ProgramErrorKind> {
    match (lhs, rhs) {
        (ObjectData::Integer(left), ObjectData::Integer(right)) => {
            println!("root(int, int): do not use");
            Ok(left.pow((1 / right).try_into().expect("no")).into())
        }
        (ObjectData::Float(_, _), ObjectData::Float(_, _)) => todo!(),
        _ => Err(ProgramErrorKind::BinopError(BinOpKind::Or, lhs, rhs)),
    }
}
