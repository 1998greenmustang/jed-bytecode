use std::{
    any::Any,
    fmt,
    time::{Duration, Instant},
};

use crate::{
    RustObject,
    object::Object,
    operation::{Operation, Response},
};

pub fn now(_: &[&Object]) -> Response {
    let data = Instant::now();
    let refr: &'static dyn RustObject = Box::leak(Box::new(data));
    Response::ExternReturn(Some(refr.into()))
}

impl RustObject for Instant {
    fn jed_sub(&self, rhs: &dyn RustObject) -> Option<Box<dyn RustObject>> {
        let rhs: &dyn Any = rhs;
        Some(Box::new(*self - *rhs.downcast_ref::<Self>().unwrap()))
    }

    fn jed_display(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }

    fn jed_debug(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl RustObject for Duration {
    fn jed_display(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }

    fn jed_debug(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

pub const FUNCTIONS: &[Operation] = &[Operation::ExternFunc(b"now", 0, now)];
pub const MODULE: &[u8] = b"time";
