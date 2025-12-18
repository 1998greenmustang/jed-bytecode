// use std::{fmt::Display, ops};

// #[derive(PartialEq, PartialOrd, Debug)]
// pub struct Literal(pub LitKind);

// // pub enum Integer {}
// // pub enum Float {}
// // // pub enum Bool {}
// // pub enum List {}

// #[derive(PartialEq, PartialOrd, Debug)]
// pub enum LitKind {
//     Str(&'static [u8]),
//     Int(i64),
//     Flt(f64),
//     Bol(bool),
//     // Lst(List),
// }

// impl Literal {
//     pub fn from_bytes(arg: &'static [u8]) -> Literal {
//         let s = unsafe { &String::from_utf8_unchecked(arg.to_vec()) };
//         // bool
//         if s == "true" || s == "false" {
//             return Literal(LitKind::Bol(s == "true"));
//         }
//         // string
//         if s.starts_with('"') && s.ends_with('"') {
//             return Literal(LitKind::Str(&arg.split_last().unwrap().1[1..]));
//         }
//         // number
//         let mut is_number_or_even_int = (true, true);
//         for c in s.chars() {
//             if c == '.' {
//                 is_number_or_even_int = (is_number_or_even_int.0, false);
//             }
//             if !c.is_numeric() {
//                 is_number_or_even_int = (false, is_number_or_even_int.1)
//             }
//         }
//         if is_number_or_even_int.0 {
//             match is_number_or_even_int.1 {
//                 true => return Literal(LitKind::Int(s.parse().unwrap())),
//                 false => return Literal(LitKind::Flt(s.parse().unwrap())),
//             }
//         }
//         panic!("Not a literal: {}", s);
//         // list
//         // if s.starts_with(pat)
//     }
// }

// impl Display for Literal {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self.0 {
//             LitKind::Str(v) => write!(f, "{v:?}"),
//             LitKind::Int(v) => write!(f, "{v}"),
//             LitKind::Flt(v) => write!(f, "{v}"),
//             LitKind::Bol(v) => write!(f, "{v}"),
//         }
//     }
// }

// impl From<bool> for Literal {
//     fn from(value: bool) -> Self {
//         Literal(LitKind::Bol(value))
//     }
// }

// impl From<f64> for Literal {
//     fn from(value: f64) -> Self {
//         Literal(LitKind::Flt(value))
//     }
// }

// impl From<i64> for Literal {
//     fn from(value: i64) -> Self {
//         Literal(LitKind::Int(value))
//     }
// }

// impl ops::Add for Literal {
//     type Output = Literal;

//     fn add(self, rhs: Literal) -> Literal {
//         match (self.0, rhs.0) {
//             (LitKind::Int(l), LitKind::Int(r)) => Literal::from(l + r),
//             (LitKind::Flt(l), LitKind::Flt(r)) => Literal::from(l + r),
//             _ => panic!(),
//         }
//     }
// }
// impl ops::Sub for Literal {
//     type Output = Literal;

//     fn sub(self, rhs: Literal) -> Literal {
//         match (self.0, rhs.0) {
//             (LitKind::Int(l), LitKind::Int(r)) => Literal::from(l - r),
//             (LitKind::Flt(l), LitKind::Flt(r)) => Literal::from(l - r),
//             _ => panic!(),
//         }
//     }
// }
// impl ops::Mul for Literal {
//     type Output = Literal;

//     fn mul(self, rhs: Literal) -> Literal {
//         match (self.0, rhs.0) {
//             (LitKind::Int(l), LitKind::Int(r)) => Literal::from(l * r),
//             (LitKind::Flt(l), LitKind::Flt(r)) => Literal::from(l * r),
//             _ => panic!(),
//         }
//     }
// }
// impl ops::Div for Literal {
//     type Output = Literal;

//     fn div(self, rhs: Literal) -> Literal {
//         match (self.0, rhs.0) {
//             (LitKind::Int(l), LitKind::Int(r)) => Literal::from(l / r),
//             (LitKind::Flt(l), LitKind::Flt(r)) => Literal::from(l / r),
//             _ => panic!(),
//         }
//     }
// }
