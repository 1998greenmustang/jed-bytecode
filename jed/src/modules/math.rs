use crate::{
    error::ProgramErrorKind,
    object::{Object, ObjectData, ObjectKind},
    operation::{Operation, Response},
};

fn sin(args: &[&Object]) -> Response {
    match args[0].data {
        ObjectData::Integer(n) => Response::ExternReturn(Some(((n as f64).sin() as isize).into())),
        ObjectData::Float(n) => Response::ExternReturn(Some(unsafe { *n }.sin().into())),
        _ => Response::Error(ProgramErrorKind::TypeError(
            ObjectKind::Integer,
            args[0].kind,
        )),
    }
}

fn cos(args: &[&Object]) -> Response {
    match args[0].data {
        ObjectData::Integer(n) => Response::ExternReturn(Some(((n as f64).cos() as isize).into())),
        ObjectData::Float(n) => Response::ExternReturn(Some(unsafe { *n }.cos().into())),
        _ => Response::Error(ProgramErrorKind::TypeError(
            ObjectKind::Integer,
            args[0].kind,
        )),
    }
}

pub const MODULE: &[u8] = b"math";
pub const FUNCTIONS: &[Operation] = &[
    Operation::ExternFunc(b"sin", 1, sin),
    Operation::ExternFunc(b"cos", 1, cos),
];
