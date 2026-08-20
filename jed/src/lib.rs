extern crate jed_macros;
mod binops;
mod builtin;
mod error;
mod frame;
pub mod memory;
mod modules;
mod object;
pub mod operation;
pub mod program;
mod span;
mod unops;
mod utils;
pub mod vm;
const MAGIC_NUMBER: &[u8] = "jed".as_bytes();

pub type BinOpKind = binops::BinOpKind;
pub type BuiltIn = builtin::BuiltIn;
