use crate::operation::Operation;

pub mod io;
pub mod jed;
pub mod math;
pub mod socket;
pub mod time;

pub const MODULES: [(&[u8], &[Operation]); 3] = [
    (math::MODULE, math::FUNCTIONS),
    (io::MODULE, io::FUNCTIONS),
    (time::MODULE, time::FUNCTIONS),
];

// Module have to contain names that are Operation::Function

// math
// import math
// [Operation::ExternFunction(b"sin", usize, func)]
