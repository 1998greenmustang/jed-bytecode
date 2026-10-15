use std::fmt::Display;

use crate::{
    error::ProgramErrorKind,
    object::{Object, ObjectKind},
    operation::{Operation, Response},
};

fn sqrt(operands: &[Object]) -> Response {
    let obj = operands[0];
    let result = match obj.kind {
        ObjectKind::Integer => Some(obj.integer().isqrt().into()),
        ObjectKind::Float => Some(obj.float().sqrt().into()),
        ObjectKind::UnsignedInt => todo!(),
        ObjectKind::RustObject => unsafe {
            let res = (*obj.rust_object()).jed_sqrt();
            return Response::ExternReturn(res.map(|o| Box::leak(o).into()));
        },
        _ => return Response::Error(ProgramErrorKind::TodoError),
    };
    Response::ExternReturn(result)
}

pub fn add(operands: &[Object]) -> Response {
    let (left, right) = (operands[0], operands[1]);
    let result = match (left.kind, right.kind) {
        (ObjectKind::Integer, ObjectKind::Integer) => {
            Some((left.integer() + right.integer()).into())
        }
        (ObjectKind::Float, ObjectKind::Float) => Some((left.float() + right.float()).into()),
        (ObjectKind::Float, ObjectKind::Integer) => {
            Some((left.float() + right.integer() as f64).into())
        }
        (ObjectKind::Integer, ObjectKind::Float) => {
            Some((left.integer() as f64 + right.float()).into())
        }
        (ObjectKind::String, ObjectKind::String) => todo!(),
        (ObjectKind::RustObject, ObjectKind::RustObject) => unsafe {
            let res = (*left.rust_object()).jed_add(*right.rust_object());
            return Response::FunctionReturn(res.map(|o| Box::leak(o).into()));
        },
        _ => return Response::Error(ProgramErrorKind::TodoError),
    };
    Response::FunctionReturn(result)
}

pub fn sub(operands: &[Object]) -> Response {
    let (left, right) = (operands[0], operands[1]);
    let result = match (left.kind, right.kind) {
        (ObjectKind::Integer, ObjectKind::Integer) => {
            match left.integer().checked_sub(right.integer()) {
                Some(v) => Some(v.into()),
                None => {
                    return Response::Error(ProgramErrorKind::Overflow(
                        left.integer(),
                        right.integer(),
                    ));
                }
            }
        }
        // (ObjectKind::UnsignedInt, ObjectKind::UnsignedInt) => {
        //     match left.unsigned().checked_sub(right.unsigned()) {
        //         Some(v) => Some(v.into()),
        //         None => {
        //             return Response::Error(
        //                 ProgramErrorKind::TodoError, // left.unsigned(),
        //                                              // right.unsigned(),
        //                                              // )
        //             );
        //         }
        //     }
        // }
        (ObjectKind::Float, ObjectKind::Float) => Some((left.float() - right.float()).into()),
        (ObjectKind::RustObject, ObjectKind::RustObject) => unsafe {
            let res = (*left.rust_object()).jed_sub(*right.rust_object());
            return Response::FunctionReturn(res.map(|o| Box::leak(o).into()));
        },
        _ => return Response::Error(ProgramErrorKind::TypeError(left.kind, right.kind)),
    };
    Response::FunctionReturn(result)
}

pub fn mul(operands: &[Object]) -> Response {
    let (left, right) = (operands[0], operands[1]);
    let result = match (left.kind, right.kind) {
        (ObjectKind::Integer, ObjectKind::Integer) => {
            match left.integer().checked_mul(right.integer()) {
                Some(v) => Some(v.into()),
                None => {
                    return Response::Error(ProgramErrorKind::Overflow(
                        left.integer(),
                        right.integer(),
                    ));
                }
            }
        }
        (ObjectKind::Float, ObjectKind::Float) => Some((left.float() * right.float()).into()),
        (ObjectKind::RustObject, ObjectKind::RustObject) => unsafe {
            let res = (*left.rust_object()).jed_mul(*right.rust_object());
            return Response::FunctionReturn(res.map(|o| Box::leak(o).into()));
        },
        _ => return Response::Error(ProgramErrorKind::TodoError),
    };
    Response::FunctionReturn(result)
}

pub fn div(operands: &[Object]) -> Response {
    let (left, right) = (operands[0], operands[1]);
    let result = match (left.kind, right.kind) {
        (ObjectKind::Integer, ObjectKind::Integer) => {
            if right.integer() == 0 {
                return Response::Error(ProgramErrorKind::DivByZero);
            }
            Some((left.integer() / right.integer()).into())
        }
        (ObjectKind::Float, ObjectKind::Float) => Some((left.float() / right.float()).into()),
        (ObjectKind::RustObject, ObjectKind::RustObject) => unsafe {
            let res = (*left.rust_object()).jed_div(*right.rust_object());
            return Response::FunctionReturn(res.map(|o| Box::leak(o).into()));
        },
        _ => return Response::Error(ProgramErrorKind::TodoError),
    };
    Response::FunctionReturn(result)
}

pub fn modulus(operands: &[Object]) -> Response {
    let (left, right) = (operands[0], operands[1]);
    let result = match (left.kind, right.kind) {
        (ObjectKind::Integer, ObjectKind::Integer) => {
            if right.integer() == 0 {
                return Response::Error(ProgramErrorKind::DivByZero);
            }
            Some((left.integer() % right.integer()).into())
        }
        (ObjectKind::Float, ObjectKind::Float) => Some((left.float() % right.float()).into()),
        (ObjectKind::RustObject, ObjectKind::RustObject) => unsafe {
            let res = (*left.rust_object()).jed_mod(*right.rust_object());
            return Response::FunctionReturn(res.map(|o| Box::leak(o).into()));
        },
        _ => return Response::Error(ProgramErrorKind::TodoError),
    };
    Response::FunctionReturn(result)
}

pub fn eq(operands: &[Object]) -> Response {
    let (left, right) = (operands[0], operands[1]);
    let result = match (left.kind, right.kind) {
        (ObjectKind::Integer, ObjectKind::Integer) => {
            Some((left.integer() == right.integer()).into())
        }
        (ObjectKind::Float, ObjectKind::Float) => Some((left.float() == right.float()).into()),
        (ObjectKind::RustObject, ObjectKind::RustObject) => unsafe {
            let res = (*left.rust_object()).jed_eq(*right.rust_object());
            return Response::FunctionReturn(res.map(|o| o.into()));
        },
        _ => return Response::Error(ProgramErrorKind::TodoError),
    };
    Response::FunctionReturn(result)
}

pub fn lesser(operands: &[Object]) -> Response {
    let (left, right) = (operands[0], operands[1]);
    let result = match (left.kind, right.kind) {
        (ObjectKind::Integer, ObjectKind::Integer) => {
            Some((left.integer() < right.integer()).into())
        }
        (ObjectKind::Float, ObjectKind::Float) => Some((left.float() < right.float()).into()),
        (ObjectKind::RustObject, ObjectKind::RustObject) => unsafe {
            let res = (*left.rust_object()).jed_lt(*right.rust_object());
            return Response::FunctionReturn(res.map(|o| o.into()));
        },
        _ => return Response::Error(ProgramErrorKind::TodoError),
    };
    Response::FunctionReturn(result)
}

pub fn greater(operands: &[Object]) -> Response {
    let (left, right) = (operands[0], operands[1]);
    let result = match (left.kind, right.kind) {
        (ObjectKind::Integer, ObjectKind::Integer) => {
            Some((left.integer() > right.integer()).into())
        }
        (ObjectKind::Float, ObjectKind::Float) => Some((left.float() > right.float()).into()),
        (ObjectKind::RustObject, ObjectKind::RustObject) => unsafe {
            let res = (*left.rust_object()).jed_gt(*right.rust_object());
            return Response::FunctionReturn(res.map(|o| o.into()));
        },
        _ => return Response::Error(ProgramErrorKind::TodoError),
    };
    Response::FunctionReturn(result)
}

pub fn lesseq(operands: &[Object]) -> Response {
    let (left, right) = (operands[0], operands[1]);
    let result = match (left.kind, right.kind) {
        (ObjectKind::Integer, ObjectKind::Integer) => {
            Some((left.integer() <= right.integer()).into())
        }
        (ObjectKind::Float, ObjectKind::Float) => Some((left.float() <= right.float()).into()),
        (ObjectKind::RustObject, ObjectKind::RustObject) => unsafe {
            let res = (*left.rust_object()).jed_le(*right.rust_object());
            return Response::FunctionReturn(res.map(|o| o.into()));
        },
        _ => return Response::Error(ProgramErrorKind::TodoError),
    };
    Response::FunctionReturn(result)
}

pub fn greateq(operands: &[Object]) -> Response {
    let (left, right) = (operands[0], operands[1]);
    let result = match (left.kind, right.kind) {
        (ObjectKind::Integer, ObjectKind::Integer) => {
            Some((left.integer() >= right.integer()).into())
        }
        (ObjectKind::Float, ObjectKind::Float) => Some((left.float() >= right.float()).into()),
        (ObjectKind::RustObject, ObjectKind::RustObject) => unsafe {
            let res = (*left.rust_object()).jed_ge(*right.rust_object());
            return Response::FunctionReturn(res.map(|o| o.into()));
        },
        _ => return Response::Error(ProgramErrorKind::TodoError),
    };
    Response::FunctionReturn(result)
}

pub fn and(operands: &[Object]) -> Response {
    let (left, right) = (operands[0], operands[1]);
    let result = match (left.kind, right.kind) {
        (ObjectKind::Bool, ObjectKind::Bool) => Some((left.bool() && right.bool()).into()),
        (ObjectKind::RustObject, ObjectKind::RustObject) => unsafe {
            let res = (*left.rust_object()).jed_and(*right.rust_object());
            return Response::FunctionReturn(res.map(|o| o.into()));
        },
        _ => return Response::Error(ProgramErrorKind::TodoError),
    };
    Response::FunctionReturn(result)
}

pub fn or(operands: &[Object]) -> Response {
    let (left, right) = (operands[0], operands[1]);
    let result = match (left.kind, right.kind) {
        (ObjectKind::Bool, ObjectKind::Bool) => Some((left.bool() || right.bool()).into()),
        (ObjectKind::RustObject, ObjectKind::RustObject) => unsafe {
            let res = (*left.rust_object()).jed_or(*right.rust_object());
            return Response::FunctionReturn(res.map(|o| o.into()));
        },
        _ => return Response::Error(ProgramErrorKind::TodoError),
    };
    Response::FunctionReturn(result)
}

pub fn pow(operands: &[Object]) -> Response {
    let (left, right) = (operands[0], operands[1]);
    let result = match (left.kind, right.kind) {
        (ObjectKind::Integer, ObjectKind::Integer) => {
            let exp: u32 = right.integer().try_into().expect("exponent too large");
            Some(left.integer().pow(exp).into())
        }
        (ObjectKind::Float, ObjectKind::Float) => Some((left.float().powf(right.float())).into()),
        (ObjectKind::RustObject, ObjectKind::RustObject) => unsafe {
            let res = (*left.rust_object()).jed_pow(*right.rust_object());
            return Response::FunctionReturn(res.map(|o| Box::leak(o).into()));
        },
        _ => return Response::Error(ProgramErrorKind::TodoError),
    };
    Response::FunctionReturn(result)
}

pub fn root(operands: &[Object]) -> Response {
    let (left, right) = (operands[0], operands[1]);
    let result = match (left.kind, right.kind) {
        (ObjectKind::Integer, ObjectKind::Integer) => {
            // root(x, y) is equivalent to x^(1/y)
            let exp = 1.0 / right.float() as f64;
            Some((left.float() as f64).powf(exp).into())
        }
        (ObjectKind::Float, ObjectKind::Float) => {
            Some((left.float().powf(1.0 / right.float())).into())
        }
        (ObjectKind::RustObject, ObjectKind::RustObject) => unsafe {
            let res = (*left.rust_object()).jed_root(*right.rust_object());
            return Response::FunctionReturn(res.map(|o| Box::leak(o).into()));
        },
        _ => return Response::Error(ProgramErrorKind::TodoError),
    };
    Response::FunctionReturn(result)
}

pub const FUNCTIONS: &[Operation] = &[
    Operation::ExternFunc(b"jed_sqrt", 1, sqrt),
    Operation::ExternFunc(b"jedop_add", 2, add),
    Operation::ExternFunc(b"jedop_sub", 2, sub),
    Operation::ExternFunc(b"jedop_mul", 2, mul),
    Operation::ExternFunc(b"jedop_div", 2, div),
    Operation::ExternFunc(b"jedop_mod", 2, modulus),
    Operation::ExternFunc(b"jedop_eq", 2, eq),
    Operation::ExternFunc(b"jedop_lesser", 2, lesser),
    Operation::ExternFunc(b"jedop_greater", 2, greater),
    Operation::ExternFunc(b"jedop_lesseq", 2, lesseq),
    Operation::ExternFunc(b"jedop_greateq", 2, greateq),
    Operation::ExternFunc(b"jedop_and", 2, and),
    Operation::ExternFunc(b"jedop_or", 2, or),
    Operation::ExternFunc(b"jedop_pow", 2, pow),
    Operation::ExternFunc(b"jedop_root", 2, root),
];
// TODO list of operator functions i want to support
#[derive(Clone, Debug)]
pub enum BinOpKind {
    // Additive
    Add,
    Sub,

    // Multiplicative
    Mul,
    Div,
    Mod,

    // Exponetial
    Power,
    Root,

    // Comparitive
    Eq,
    LessEq,
    GreatEq,
    Lesser,
    Greater,
    And,
    Or,

    // Bitwise
    BitAnd,
    BitOr,
    Xor,
    BitShLeft,
    BitShRight,
}
impl Display for BinOpKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOpKind::Add => write!(f, "+"),
            BinOpKind::Sub => write!(f, "-"),
            BinOpKind::Mul => write!(f, "*"),
            BinOpKind::Div => write!(f, "/"),
            BinOpKind::Mod => write!(f, "%"),
            BinOpKind::Power => write!(f, "**"),
            BinOpKind::Root => write!(f, "root"),
            BinOpKind::Eq => write!(f, "="),
            BinOpKind::LessEq => write!(f, "<="),
            BinOpKind::GreatEq => write!(f, ">="),
            BinOpKind::Lesser => write!(f, "<"),
            BinOpKind::Greater => write!(f, ">"),
            BinOpKind::And => write!(f, "&&"),
            BinOpKind::Or => write!(f, "||"),
            BinOpKind::BitAnd => write!(f, "&"),
            BinOpKind::BitOr => write!(f, "|"),
            BinOpKind::Xor => write!(f, "^"),
            BinOpKind::BitShLeft => write!(f, "<<"),
            BinOpKind::BitShRight => write!(f, ">>"),
        }
    }
}
