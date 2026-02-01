use std::fmt::Display;

use crate::{
    binops::BinOpKind,
    object::{ObjectData, ObjectKind},
    span::Span,
    utils,
};

#[derive(Debug, Clone)]
pub enum ProgramErrorKind {
    StackError(usize),
    BinopError(BinOpKind, ObjectData, ObjectData),
    FunctionExists(&'static [u8]),
    VariableExists(&'static [u8]),
    TempPush,
    TypeError(ObjectKind, ObjectKind), // wanted, given
    ParsingError(String),
    Overflow(BinOpKind, isize, isize),
    IntegerToUnsigned,
    ListIndexError(usize, usize), // index, length
    ConstantExists(&'static [u8]),
    IterNext(usize), // index, length
    IterPrevious,    // index, length
}

impl Display for ProgramErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramErrorKind::StackError(n) => write!(f, "expected {n} objects on the stack"),
            ProgramErrorKind::BinopError(kind, left, right) => write!(
                f,
                "{kind} not implemented for \n   left: {left:?}\n   right: {right:?}"
            ),
            ProgramErrorKind::FunctionExists(items) => write!(
                f,
                "function '{}' does not exist",
                utils::bytes_to_string(items)
            ),
            ProgramErrorKind::VariableExists(items) => write!(
                f,
                "variable '{}' does not exist",
                utils::bytes_to_string(items)
            ),
            ProgramErrorKind::TempPush => write!(f, "no item found in temp register"),
            ProgramErrorKind::TypeError(wanted, given) => {
                write!(f, "wanted a type '{}', was given {}", wanted, given)
            }
            ProgramErrorKind::ParsingError(string) => {
                write!(f, "could not parse as literal \n >>>\t{string}\t<<<")
            }
            ProgramErrorKind::Overflow(kind, left, right) => write!(
                f,
                "{kind} attempt with overflow \n   left: {left:?}\n   right: {right:?}"
            ),
            ProgramErrorKind::IntegerToUnsigned => {
                write!(f, "attempt to use an integer as unsigned")
            }
            ProgramErrorKind::ListIndexError(idx, len) => write!(
                f,
                "index '{}' does not appear in a list of {} length",
                idx, len
            ),
            ProgramErrorKind::IterNext(len) => {
                write!(f, "can not get next in a list of {} length", len)
            }
            ProgramErrorKind::IterPrevious => write!(f, "can not get previous",),
            ProgramErrorKind::ConstantExists(bytes) => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProgramError(pub ProgramErrorKind, pub Span);

impl Display for ProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\ndetail:\n {}", self.1, self.0)
    }
}
