use core::slice;

#[derive(Hash, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub enum NumberFlags {
    Zero,
    Positive,
    Negative,
}

// Numbers are stored in an arena in VM

#[derive(Hash, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub struct Number {
    flags: NumberFlags,
    ptr: *const u8,
    bytes: usize,
}

impl TryFrom<Number> for usize {
    type Error = &'static str;

    fn try_from(value: Number) -> Result<Self, Self::Error> {
        match <&[u8] as TryInto<[u8; 8]>>::try_into(value.bytes()) {
            Ok(v) => Ok(usize::from_be_bytes(v)),
            Err(_e) => Err("number too big"),
        }
    }
}

impl Number {
    pub fn new(flags: NumberFlags, ptr: *const u8, bytes: usize) -> Self {
        Number { flags, ptr, bytes }
    }
    fn bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr, self.bytes) }
    }
}
