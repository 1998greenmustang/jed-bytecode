use core::fmt;
use std::any::Any;

extern crate jed_macros;
mod error;
mod frame;
pub mod memory;
mod modules;
mod object;
pub mod operation;
pub mod program;
mod span;
mod utils;
pub mod vm;
const MAGIC_NUMBER: &[u8] = "jed".as_bytes();

pub trait RustObject: Any {
    fn jed_display(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Err(fmt::Error)
    }
    fn jed_debug(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Err(fmt::Error)
    }
    fn jed_sub(&self, _rhs: &dyn RustObject) -> Option<Box<dyn RustObject>> {
        None
    }
}
