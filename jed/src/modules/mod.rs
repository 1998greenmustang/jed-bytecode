use crate::operation::Operation;

pub mod io;
pub mod jed;
pub mod math;
pub mod socket;
pub mod time;

pub const MODULES: [(bool, &[u8], &[Operation]); 3] = [
    (math::INTERNAL, math::MODULE, math::FUNCTIONS),
    (io::INTERNAL, io::MODULE, io::FUNCTIONS),
    (time::INTERNAL, time::MODULE, time::FUNCTIONS),
];

// Module have to contain names that are Operation::Function

// math
// import math
// [Operation::ExternFunction(b"sin", usize, func)]
