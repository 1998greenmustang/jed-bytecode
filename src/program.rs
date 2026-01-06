use std::collections::{BTreeMap, HashMap};

use crate::{
    arena::Dropless,
    binops::BinOpKind,
    object::{Object, ObjectKind},
    operation::Operation,
};

type Arity = usize;
type Index = usize;

pub type MemoKey = (Index, &'static [Object]);
type MemoTable = HashMap<MemoKey, Object>;

pub struct Program {
    pub arena: Dropless,
    pub instructions: Vec<Operation>,
    pub funcs: BTreeMap<Object, (Index, Arity)>,
    pub memos: MemoTable,
}

impl Program {
    pub fn get_main(&self) -> Index {
        let lbl = &Object(ObjectKind::Func, "main".as_bytes());
        let (idx, _) = self
            .get_func(lbl)
            .unwrap_or_else(|| panic!("No main func!"));
        return *idx;
    }
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
            let bytes = number.parse::<i64>().ok().unwrap().to_be_bytes();
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
    pub fn register_objects(&mut self, objects: &[Object]) -> &'static [Object] {
        let saved_bytes = self.arena.alloc_slice(objects);
        let saved_bytes: &'static [Object] = unsafe { &*(saved_bytes as *const [Object]) };
        saved_bytes
    }

    pub fn register_objects_as_u8(&mut self, objects: &[Object]) -> &'static [u8] {
        unsafe {
            let objs: &[u8] = std::mem::transmute(objects);
            let mem = self
                .arena
                .alloc_raw(std::alloc::Layout::for_value::<[u8]>(objs))
                as *mut &[u8];
            mem.copy_from_nonoverlapping(&objs, objs.len());
            let _ = std::slice::from_raw_parts_mut(mem, objs.len());
            return &*(objs as *const [u8]);
        }
    }

    pub fn register(&mut self, byte_str: &[u8]) -> &'static [u8] {
        let byte_str: &[u8] = self.arena.alloc_slice(byte_str);
        let byte_str: &'static [u8] = unsafe { &*(byte_str as *const [u8]) };
        return byte_str;
    }

    pub fn get_memo(&self, key: MemoKey) -> Option<&Object> {
        self.memos.get(&key)
    }
    pub fn set_memo(&mut self, key: MemoKey, result: Object) {
        self.memos.insert(key, result);
    }
    pub fn from_string(text: String) -> Self {
        let mut program = Program {
            arena: Default::default(),
            instructions: vec![],
            funcs: BTreeMap::new(),
            memos: HashMap::new(),
        };

        for line in text.split('\n') {
            if line.trim().is_empty() {
                continue;
            }
            let line_spl: Vec<&str> = line.split(' ').map(|x| x.trim()).collect();
            let op = line_spl[0];
            assert!(Operation::exists(op), "{} not a valid operation", op);

            if op == "func" {
                let mut saved_name = program.register_string(line_spl[1]);
                let arity = line_spl[2]
                    .parse::<usize>()
                    .expect("arity is not a number or something");
                saved_name.0 = ObjectKind::Func;
                let idx = program.instructions.len();
                program.funcs.insert(saved_name, (idx, arity));
            }

            let arg = line_spl[1..].join(" ");

            let saved_arg: Option<Object> = if !arg.is_empty() {
                if arg.chars().all(|c| c.is_digit(10) || c == '.') {
                    Some(program.register_as_a_number(arg.as_str()))
                } else if arg.starts_with('[') && arg.ends_with(']') {
                    todo!("pushing many literals at once");
                } else {
                    Some(program.register_string(arg.as_str()))
                }
            } else {
                None
            };

            if op == "call" {
                let mut key = saved_arg.unwrap();
                key.0 = ObjectKind::Func;

                let op_code = Operation::get_opcode(op);
                program
                    .instructions
                    .push(Operation::new(op_code, Some(key)))
            } else {
                let op_code = Operation::get_opcode(op);
                program
                    .instructions
                    .push(Operation::new(op_code, saved_arg));
            }
        }

        return program;
    }

    pub fn get_func(&self, lbl: &Object) -> Option<&(Index, Arity)> {
        self.funcs.get(lbl)
    }

    pub fn handle_bin_op(&mut self, kind: BinOpKind, lhs: Object, rhs: Object) -> Object {
        match (kind, lhs.0, rhs.0) {
            (BinOpKind::Add, ObjectKind::Integer, ObjectKind::Integer) => {
                self.register_integer(lhs.integer() + rhs.integer())
            }
            (BinOpKind::Add, ObjectKind::Float, ObjectKind::Float) => {
                self.register_float(lhs.float() + rhs.float())
            }
            (BinOpKind::Sub, ObjectKind::Integer, ObjectKind::Integer) => {
                self.register_integer(lhs.integer() - rhs.integer())
            }
            (BinOpKind::Sub, ObjectKind::Float, ObjectKind::Float) => {
                self.register_float(lhs.float() - rhs.float())
            }
            (BinOpKind::Mul, ObjectKind::Integer, ObjectKind::Integer) => {
                self.register_integer(lhs.integer() * rhs.integer())
            }
            (BinOpKind::Mul, ObjectKind::Float, ObjectKind::Float) => {
                self.register_float(lhs.float() * rhs.float())
            }
            (BinOpKind::Div, ObjectKind::Integer, ObjectKind::Integer) => {
                self.register_integer(lhs.integer() / rhs.integer())
            }
            (BinOpKind::Div, ObjectKind::Float, ObjectKind::Float) => {
                self.register_float(lhs.float() / rhs.float())
            }
            (BinOpKind::Eq, ObjectKind::Integer, ObjectKind::Integer) => {
                self.register_bool(lhs.integer() == rhs.integer())
            }
            (BinOpKind::Eq, ObjectKind::Float, ObjectKind::Float) => {
                self.register_bool(lhs.float() == rhs.float())
            }
            (BinOpKind::LessEq, ObjectKind::Integer, ObjectKind::Integer) => {
                self.register_bool(lhs.integer() <= rhs.integer())
            }
            (BinOpKind::LessEq, ObjectKind::Float, ObjectKind::Float) => {
                self.register_bool(lhs.float() <= rhs.float())
            }
            (BinOpKind::Lesser, ObjectKind::Integer, ObjectKind::Integer) => {
                self.register_bool(lhs.integer() < rhs.integer())
            }
            (BinOpKind::Lesser, ObjectKind::Float, ObjectKind::Float) => {
                self.register_bool(lhs.float() < rhs.float())
            }
            (BinOpKind::GreatEq, ObjectKind::Integer, ObjectKind::Integer) => {
                self.register_bool(lhs.integer() >= rhs.integer())
            }
            (BinOpKind::GreatEq, ObjectKind::Float, ObjectKind::Float) => {
                self.register_bool(lhs.float() >= rhs.float())
            }
            (BinOpKind::Greater, ObjectKind::Integer, ObjectKind::Integer) => {
                self.register_bool(lhs.integer() > rhs.integer())
            }
            (BinOpKind::Greater, ObjectKind::Float, ObjectKind::Float) => {
                self.register_bool(lhs.float() > rhs.float())
            }
            (BinOpKind::Mod, ObjectKind::Integer, ObjectKind::Integer) => {
                self.register_integer(lhs.integer() % rhs.integer())
            }
            (BinOpKind::Mod, ObjectKind::Float, ObjectKind::Float) => {
                self.register_float(lhs.float() % rhs.float())
            }
            _ => todo!("{:?}\n\tleft:{:?}\n\tright:{:?}", kind, lhs.0, rhs.0),
        }
    }
}
