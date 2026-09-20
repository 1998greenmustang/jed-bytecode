use crate::{
    object::Object,
    operation::{Operation, Response},
};

fn println(args: &[&Object]) -> Response {
    println!("{}", args[0]);
    Response::ExternReturn(None)
}

pub const MODULE: &[u8] = b"io";
pub const FUNCTIONS: &[Operation] = &[Operation::ExternFunc(b"println", 1, println)];
