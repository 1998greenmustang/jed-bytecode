use crate::{
    error::ProgramErrorKind,
    object::{Object, ObjectData},
    operation::{Operation, Response},
};

fn sqrt(operands: &[&Object]) -> Response {
    match operands[0].data {
        ObjectData::Integer(i) => Response::FunctionReturn(Some(i.isqrt().into())),
        ObjectData::Float(f) => Response::FunctionReturn(Some(unsafe { *f }.sqrt().into())),
        ObjectData::UnsignedInt(_i) => todo!(),
        _ => Response::Error(ProgramErrorKind::TodoError),
    }
}

pub fn add(operands: &[&Object]) -> Response {
    match (operands[0].data, operands[1].data) {
        (ObjectData::Integer(left), ObjectData::Integer(right)) => match left.checked_add(right) {
            Some(v) => Response::FunctionReturn(Some(v.into())),
            None => Response::Error(ProgramErrorKind::Overflow(left, right)),
        },
        (ObjectData::Float(left), ObjectData::Float(right)) => {
            Response::FunctionReturn(Some(unsafe { *left + *right }.into()))
        }
        (ObjectData::Float(flt), ObjectData::Integer(int))
        | (ObjectData::Integer(int), ObjectData::Float(flt)) => {
            return Response::FunctionReturn(Some((int as f64 + unsafe { *flt }).into()));
        }
        (ObjectData::String(_), ObjectData::String(_)) => todo!(),
        _ => Response::Error(ProgramErrorKind::TodoError),
    }
}

pub fn sub(operands: &[&Object]) -> Response {
    match (operands[0].data, operands[1].data) {
        (ObjectData::Integer(left), ObjectData::Integer(right)) => match left.checked_sub(right) {
            Some(v) => Response::FunctionReturn(Some(v.into())),
            None => Response::Error(ProgramErrorKind::Overflow(left, right)),
        },
        (ObjectData::Float(left), ObjectData::Float(right)) => unsafe {
            let left = *left;
            let right = *right;
            Response::FunctionReturn(Some((left - right).into()))
        },
        _ => Response::Error(ProgramErrorKind::TodoError),
    }
}
pub fn mul(operands: &[&Object]) -> Response {
    match (operands[0].data, operands[1].data) {
        (ObjectData::Integer(left), ObjectData::Integer(right)) => match left.checked_mul(right) {
            Some(v) => Response::FunctionReturn(Some(v.into())),
            None => Response::Error(ProgramErrorKind::Overflow(left, right)),
        },
        (ObjectData::Float(_), ObjectData::Float(_)) => todo!(),
        _ => Response::Error(ProgramErrorKind::TodoError),
    }
}
pub fn div(operands: &[&Object]) -> Response {
    match (operands[0].data, operands[1].data) {
        (ObjectData::Integer(_), ObjectData::Integer(_)) => todo!(),
        (ObjectData::Float(_), ObjectData::Float(_)) => todo!(),
        _ => Response::Error(ProgramErrorKind::TodoError),
    }
}
pub fn modulus(operands: &[&Object]) -> Response {
    match (operands[0].data, operands[1].data) {
        (ObjectData::Integer(left), ObjectData::Integer(right)) => {
            Response::FunctionReturn(Some((left % right).into()))
        }
        (ObjectData::Float(_), ObjectData::Float(_)) => todo!(),
        _ => Response::Error(ProgramErrorKind::TodoError),
    }
}
pub fn eq(operands: &[&Object]) -> Response {
    match (operands[0].data, operands[1].data) {
        (ObjectData::Integer(left), ObjectData::Integer(right)) => {
            Response::FunctionReturn(Some((left == right).into()))
        }
        (ObjectData::Float(_), ObjectData::Float(_)) => todo!(),
        _ => Response::Error(ProgramErrorKind::TodoError),
    }
}
pub fn lesser(operands: &[&Object]) -> Response {
    match (operands[0].data, operands[1].data) {
        (ObjectData::Integer(_), ObjectData::Integer(_)) => todo!(),
        (ObjectData::Float(_), ObjectData::Float(_)) => todo!(),
        _ => Response::Error(ProgramErrorKind::TodoError),
    }
}
pub fn greater(operands: &[&Object]) -> Response {
    match (operands[0].data, operands[1].data) {
        (ObjectData::Integer(_), ObjectData::Integer(_)) => todo!(),
        (ObjectData::Float(_), ObjectData::Float(_)) => todo!(),
        _ => Response::Error(ProgramErrorKind::TodoError),
    }
}
pub fn lesseq(operands: &[&Object]) -> Response {
    match (operands[0].data, operands[1].data) {
        (ObjectData::Integer(left), ObjectData::Integer(right)) => {
            Response::FunctionReturn(Some((left <= right).into()))
        }
        // (ObjectData::Float(lefti, leftp), ObjectData::Float(lefti, leftp)) => left <= right,
        _ => Response::Error(ProgramErrorKind::TodoError),
    }
}
pub fn greateq(operands: &[&Object]) -> Response {
    match (operands[0].data, operands[1].data) {
        (ObjectData::Integer(_), ObjectData::Integer(_)) => todo!(),
        (ObjectData::Float(_), ObjectData::Float(_)) => todo!(),
        _ => Response::Error(ProgramErrorKind::TodoError),
    }
}
pub fn and(operands: &[&Object]) -> Response {
    match (operands[0].data, operands[1].data) {
        (ObjectData::Bool(left), ObjectData::Bool(right)) => {
            Response::FunctionReturn(Some((left && right).into()))
        }
        _ => Response::Error(ProgramErrorKind::TodoError),
    }
}
pub fn or(operands: &[&Object]) -> Response {
    match (operands[0].data, operands[1].data) {
        (ObjectData::Bool(left), ObjectData::Bool(right)) => {
            Response::FunctionReturn(Some((left || right).into()))
        }
        _ => Response::Error(ProgramErrorKind::TodoError),
    }
}

pub fn pow(operands: &[&Object]) -> Response {
    match (operands[0].data, operands[1].data) {
        (ObjectData::Integer(left), ObjectData::Integer(right)) => {
            Response::FunctionReturn(Some(left.pow(right.try_into().expect("no")).into()))
        }
        (ObjectData::Float(_), ObjectData::Float(_)) => todo!(),
        _ => Response::Error(ProgramErrorKind::TodoError),
    }
}

pub fn root(operands: &[&Object]) -> Response {
    match (operands[0].data, operands[1].data) {
        (ObjectData::Integer(left), ObjectData::Integer(right)) => {
            println!("root(int, int): do not use");
            Response::FunctionReturn(Some(left.pow((1 / right).try_into().expect("no")).into()))
        }
        (ObjectData::Float(_), ObjectData::Float(_)) => todo!(),
        _ => Response::Error(ProgramErrorKind::TodoError),
    }
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
// pub enum BinOpKind {
//     // Additive
//     Add,
//     Sub,

//     // Multiplicative
//     Mul,
//     Div,
//     Mod,

//     // Exponetial
//     Power,
//     Root,

//     // Comparitive
//     Eq,
//     LessEq,
//     GreatEq,
//     Lesser,
//     Greater,
//     And,
//     Or,

//     // Bitwise
//     BitAnd,
//     BitOr,
//     Xor,
//     BitShLeft,
//     BitShRight,
