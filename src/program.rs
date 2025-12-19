use std::{array, collections::BTreeMap};

use crate::{
    arena::Dropless,
    frame::Frame,
    map::Map,
    object::{Object, ObjectKind},
    operation::{BinOpKind, Operation},
};

type Index = usize;

pub struct Program {
    pub arena: Dropless,
    pub instructions: Vec<Operation>,
    pub funcs: BTreeMap<Object, Index>,
}

impl Program {
    pub fn get_op(&self, idx: usize) -> &Operation {
        self.instructions.get(idx).unwrap()
    }

    pub fn register_string(&mut self, string: &str) -> Object {
        Object(ObjectKind::String, self.register(string.as_bytes()))
    }

    pub fn register_as_a_number(&mut self, number: &str) -> Object {
        if number.contains('.') {
            let bytes = number.parse::<f64>().ok().unwrap().to_be_bytes();
            Object(ObjectKind::Float, self.register(&bytes))
        } else {
            let bytes = i64::from_str_radix(number, 10).ok().unwrap().to_be_bytes();
            Object(ObjectKind::Integer, self.register(&bytes))
        }
    }

    pub fn register_bool(&mut self, bol: bool) -> Object {
        let bytes = {
            if bol {
                "true"
            } else {
                "false"
            }
        }
        .as_bytes();
        Object(ObjectKind::Bool, self.register(&bytes))
    }

    pub fn register_integer(&mut self, number: i64) -> Object {
        let bytes = number.to_be_bytes();
        Object(ObjectKind::Integer, self.register(&bytes))
    }

    pub fn register_float(&mut self, number: f64) -> Object {
        let bytes = number.to_be_bytes();
        Object(ObjectKind::Float, self.register(&bytes))
    }

    pub fn register(&mut self, byte_str: &[u8]) -> &'static [u8] {
        let byte_str: &[u8] = self.arena.alloc_slice(byte_str);
        let byte_str: &'static [u8] = unsafe { &*(byte_str as *const [u8]) };
        return byte_str;
    }

    pub fn from_string(text: String) -> Self {
        let mut program = Program {
            arena: Default::default(),
            instructions: vec![],
            funcs: BTreeMap::new(),
        };

        for line in text.split('\n') {
            if line.trim().is_empty() {
                continue;
            }
            let line_spl: Vec<&str> = line.split(' ').map(|x| x.trim()).collect();
            let op = line_spl[0];
            assert!(Operation::exists(op), "{} not a valid operation", op);

            let arg = line_spl[1..].join(" ");

            let saved_arg = if !arg.is_empty() {
                if arg.chars().all(|c| c.is_digit(10) || c == '.') {
                    Some(program.register_as_a_number(arg.as_str()))
                } else {
                    Some(program.register_string(arg.as_str()))
                }
            } else {
                None
            };

            if op == "func" {
                let idx = program.instructions.len();
                let mut func = saved_arg.unwrap().clone();
                func.0 = ObjectKind::Func;
                program.funcs.insert(func, idx);
            } else {
                let op_code = Operation::get_opcode(op);
                program
                    .instructions
                    .push(Operation::new(op_code, saved_arg));
            }
        }

        return program;
    }

    pub fn get_func(&self, lbl: &Object) -> Option<&Index> {
        self.funcs.get(lbl)
    }

    pub fn handle_bin_op(&mut self, kind: BinOpKind, lhs: Object, rhs: Object) -> Object {
        match (kind, lhs.0, rhs.0) {
            (BinOpKind::Add, ObjectKind::Integer, ObjectKind::Integer) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = i64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = i64::from_be_bytes(rbytes);
                self.register_integer(left + right)
            }
            (BinOpKind::Add, ObjectKind::Float, ObjectKind::Float) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = f64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = f64::from_be_bytes(rbytes);
                self.register_float(left + right)
            }
            (BinOpKind::Sub, ObjectKind::Integer, ObjectKind::Integer) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = i64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = i64::from_be_bytes(rbytes);
                self.register_integer(left - right)
            }
            (BinOpKind::Sub, ObjectKind::Float, ObjectKind::Float) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = f64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = f64::from_be_bytes(rbytes);
                self.register_float(left - right)
            }
            (BinOpKind::Mul, ObjectKind::Integer, ObjectKind::Integer) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = i64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = i64::from_be_bytes(rbytes);
                self.register_integer(left * right)
            }
            (BinOpKind::Mul, ObjectKind::Float, ObjectKind::Float) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = f64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = f64::from_be_bytes(rbytes);
                self.register_float(left * right)
            }
            (BinOpKind::Div, ObjectKind::Integer, ObjectKind::Integer) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = i64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = i64::from_be_bytes(rbytes);
                self.register_integer(left / right)
            }
            (BinOpKind::Div, ObjectKind::Float, ObjectKind::Float) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = f64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = f64::from_be_bytes(rbytes);
                self.register_float(left / right)
            }
            (BinOpKind::Eq, ObjectKind::Integer, ObjectKind::Integer) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = i64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = i64::from_be_bytes(rbytes);
                self.register_bool(left == right)
            }
            (BinOpKind::Eq, ObjectKind::Float, ObjectKind::Float) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = f64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = f64::from_be_bytes(rbytes);
                self.register_bool(left == right)
            }
            (BinOpKind::LessEq, ObjectKind::Integer, ObjectKind::Integer) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = i64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = i64::from_be_bytes(rbytes);
                self.register_bool(left <= right)
            }
            (BinOpKind::LessEq, ObjectKind::Float, ObjectKind::Float) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = f64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = f64::from_be_bytes(rbytes);
                self.register_bool(left <= right)
            }
            (BinOpKind::Lesser, ObjectKind::Integer, ObjectKind::Integer) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = i64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = i64::from_be_bytes(rbytes);
                self.register_bool(left < right)
            }
            (BinOpKind::Lesser, ObjectKind::Float, ObjectKind::Float) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = f64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = f64::from_be_bytes(rbytes);
                self.register_bool(left < right)
            }
            (BinOpKind::GreatEq, ObjectKind::Integer, ObjectKind::Integer) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = i64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = i64::from_be_bytes(rbytes);
                self.register_bool(left >= right)
            }
            (BinOpKind::GreatEq, ObjectKind::Float, ObjectKind::Float) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = f64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = f64::from_be_bytes(rbytes);
                self.register_bool(left >= right)
            }
            (BinOpKind::Greater, ObjectKind::Integer, ObjectKind::Integer) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = i64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = i64::from_be_bytes(rbytes);
                self.register_bool(left > right)
            }
            (BinOpKind::Greater, ObjectKind::Float, ObjectKind::Float) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = f64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = f64::from_be_bytes(rbytes);
                self.register_bool(left > right)
            }
            (BinOpKind::Mod, ObjectKind::Integer, ObjectKind::Integer) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = i64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = i64::from_be_bytes(rbytes);
                self.register_integer(left % right)
            }
            (BinOpKind::Mod, ObjectKind::Float, ObjectKind::Float) => {
                let lbytes: [u8; 8] = array::from_fn(|i| lhs.1[i]);
                let left = f64::from_be_bytes(lbytes);
                let rbytes: [u8; 8] = array::from_fn(|i| rhs.1[i]);
                let right = f64::from_be_bytes(rbytes);
                self.register_float(left % right)
            }
            _ => todo!("{:?}\n\tleft:{:?}\n\tright:{:?}", kind, lhs.0, rhs.0),
        }
    }
}
