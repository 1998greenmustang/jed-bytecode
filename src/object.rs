use std::{fmt::Display, u8};

#[derive(PartialEq, Debug)]
// pub enum Object<'a> {
//     Literal(&'a Literal),
//     Func(usize), // usize => instruction pointer
// }
#[repr(u8)]
#[derive(Hash, Eq, Copy, Clone, PartialOrd, Ord)]
pub enum ObjectKind {
    Integer,
    Float,
    String,
    Bool,
    Func,
    Nil,
}

#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone, PartialOrd, Ord)]
pub struct Object(pub ObjectKind, pub &'static [u8]);

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ObjectKind::Integer => write!(f, "{}", unsafe {
                assert!(self.0 == ObjectKind::Integer, "Not a integer!");
                let [a, b, c, d, e, f, g, h] = self.1 else {
                    unreachable!("Nice memory!")
                };
                i64::from_be_bytes([*a, *b, *c, *d, *e, *f, *g, *h])
            }),
            ObjectKind::Float => write!(f, "{}", unsafe {
                assert!(self.0 == ObjectKind::Float, "Not a float!");
                let [a, b, c, d, e, f, g, h] = self.1 else {
                    unreachable!("Nice memory!")
                };
                f64::from_be_bytes([*a, *b, *c, *d, *e, *f, *g, *h])
            }),

            ObjectKind::String => write!(f, "{}", unsafe {
                std::mem::transmute::<&'static [u8], &str>(self.1)
            }),

            ObjectKind::Bool => write!(f, "{}", unsafe {
                match std::mem::transmute::<&'static [u8], &str>(self.1) {
                    "true" => true,
                    "false" => false,
                    _ => unreachable!("Nice memory!"),
                }
            }),

            ObjectKind::Func => todo!(),
            ObjectKind::Nil => todo!(),
        }
    }
}
