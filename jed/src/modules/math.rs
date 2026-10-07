use crate::{
    error::ProgramErrorKind,
    object::{Object, ObjectKind},
    operation::{Operation, Response},
};

fn sin(args: &[&Object]) -> Response {
    let obj = args[0];
    match obj.kind {
        ObjectKind::Integer => {
            Response::FunctionReturn(Some(((obj.integer() as f64).sin() as i64).into()))
        }
        ObjectKind::Float => Response::FunctionReturn(Some(obj.float().sin().into())),
        _ => Response::Error(ProgramErrorKind::TypeError(
            ObjectKind::Integer,
            args[0].kind,
        )),
    }
}

fn cos(args: &[&Object]) -> Response {
    let obj = args[0];
    match obj.kind {
        ObjectKind::Integer => {
            Response::FunctionReturn(Some(((obj.integer() as f64).cos() as i64).into()))
        }
        ObjectKind::Float => Response::FunctionReturn(Some(obj.float().cos().into())),
        _ => Response::Error(ProgramErrorKind::TypeError(
            ObjectKind::Integer,
            args[0].kind,
        )),
    }
}

pub const INTERNAL: bool = false;
pub const MODULE: &[u8] = b"math";
pub const FUNCTIONS: &[Operation] = &[
    Operation::ExternFunc(b"sin", 1, sin),
    Operation::ExternFunc(b"cos", 1, cos),
];
