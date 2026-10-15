use std::io::{BufWriter, Stderr, Stdout, Write};

use crate::{
    object::Object,
    operation::{Operation, Response},
};

pub struct Io {
    out: BufWriter<Stdout>,
    err: BufWriter<Stderr>,
}

impl Io {
    pub fn new() -> Self {
        Self {
            out: BufWriter::new(std::io::stdout()),
            err: BufWriter::new(std::io::stderr()),
        }
    }

    pub fn write_out(
        &mut self,
        args: &dyn std::fmt::Display,
    ) -> std::result::Result<(), std::io::Error> {
        write!(self.out, "{}", args)
    }

    pub fn write_err(
        &mut self,
        args: &dyn std::fmt::Display,
    ) -> std::result::Result<(), std::io::Error> {
        write!(self.err, "{}", args)
    }
}

fn println(args: &[Object]) -> Response {
    let stdout = std::io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    let _ = write!(writer, "{}\n", args[0]);
    Response::ExternReturn(None)
}

fn print(args: &[Object]) -> Response {
    let stdout = std::io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    let _ = write!(writer, "{}", args[0]);
    Response::ExternReturn(None)
}

pub const INTERNAL: bool = true;
pub const MODULE: &[u8] = b"io";
pub const FUNCTIONS: &[Operation] = &[
    Operation::ExternFunc(b"println", 1, println),
    Operation::ExternFunc(b"print", 1, print),
];
