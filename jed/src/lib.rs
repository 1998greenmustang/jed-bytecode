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
    fn jed_add(&self, _rhs: &dyn RustObject) -> Option<Box<dyn RustObject>> {
        None
    }
    fn jed_sub(&self, _rhs: &dyn RustObject) -> Option<Box<dyn RustObject>> {
        None
    }
    fn jed_mul(&self, _rhs: &dyn RustObject) -> Option<Box<dyn RustObject>> {
        None
    }
    fn jed_div(&self, _rhs: &dyn RustObject) -> Option<Box<dyn RustObject>> {
        None
    }
    fn jed_mod(&self, _rhs: &dyn RustObject) -> Option<Box<dyn RustObject>> {
        None
    }
    fn jed_pow(&self, _rhs: &dyn RustObject) -> Option<Box<dyn RustObject>> {
        None
    }
    fn jed_sqrt(&self) -> Option<Box<dyn RustObject>> {
        None
    }
    fn jed_root(&self, _rhs: &dyn RustObject) -> Option<Box<dyn RustObject>> {
        None
    }
    fn jed_eq(&self, _rhs: &dyn RustObject) -> Option<bool> {
        None
    }
    fn jed_lt(&self, _rhs: &dyn RustObject) -> Option<bool> {
        None
    }
    fn jed_gt(&self, _rhs: &dyn RustObject) -> Option<bool> {
        None
    }
    fn jed_le(&self, _rhs: &dyn RustObject) -> Option<bool> {
        None
    }
    fn jed_ge(&self, _rhs: &dyn RustObject) -> Option<bool> {
        None
    }

    fn jed_and(&self, _rhs: &dyn RustObject) -> Option<bool> {
        None
    }
    fn jed_or(&self, _rhs: &dyn RustObject) -> Option<bool> {
        None
    }
}
