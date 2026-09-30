use std::{
    io::{BufWriter, Write},
    os::fd::AsRawFd,
};

use crate::{
    object::Object,
    operation::{Operation, Response},
};

fn println(args: &[&Object]) -> Response {
    let stdout = std::io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    let _ = write!(writer, "{}\n", args[0]);
    Response::ExternReturn(None)
}

fn print(args: &[&Object]) -> Response {
    let stdout = std::io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    let _ = write!(writer, "{}", args[0]);
    Response::ExternReturn(None)
}

pub const MODULE: &[u8] = b"io";
pub const FUNCTIONS: &[Operation] = &[
    Operation::ExternFunc(b"println", 1, println),
    Operation::ExternFunc(b"print", 1, print),
];
