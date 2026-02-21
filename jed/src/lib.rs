extern crate jed_macros;
pub mod arena;
mod binops;
mod builtin;
mod error;
mod frame;
mod modules;
mod object;
pub mod operation;
mod program;
mod span;
mod stack;
mod utils;
pub mod vm;
const MAGIC_NUMBER: &[u8] = "jed".as_bytes();

pub type BinOpKind = binops::BinOpKind;
pub type BuiltIn = builtin::BuiltIn;
