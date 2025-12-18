use std::collections::HashMap;

use crate::{
    arena::Dropless,
    frame::Frame,
    object::{Object, ObjectKind},
    operation::{BinOpKind, Operation},
    program::Program,
    stack::Stack,
};

pub struct VM {
    pub program: Program,
    pub counter: usize,
    pub call_stack: Stack<Frame>,
    // pub heap: HashMap<String, Object>,
    pub obj_stack: Stack<Object>,
    pub temp: Option<Object>,
}

impl VM {
    pub fn new(program: Program) -> Self {
        let mut call_stack = Stack::new();
        call_stack.push(Frame::new(program.instructions.len()));
        VM {
            call_stack,
            counter: *program
                .get_func(&Object(ObjectKind::Func, "main".as_bytes()))
                .unwrap_or_else(|| panic!("No main func!")),
            program,
            // heap: HashMap::<String, Object>::new(),
            obj_stack: Stack::new(),
            temp: None,
        }
    }

    pub fn from_string(text: String) -> Self {
        let program = Program::from_string(text);
        Self::new(program)
    }

    pub fn run(&mut self) {
        loop {
            if self.counter == self.program.instructions.len() - 1 {
                return;
            }
            if self.call_stack.len() > 100_000 || self.obj_stack.len() > 1_000_000 {
                panic!("stack overflow");
            }
            // println!(
            //     "pc: {}, tmp: {:?}, objstack: {:?}",
            //     self.counter, self.temp, self.obj_stack
            // );
            let op = self.next();
            op.unwrap().call(self);
        }
    }

    #[inline]
    pub fn next(&mut self) -> Option<Operation> {
        let op = self.program.instructions[self.counter];
        self.counter += 1;
        return Some(op);
    }

    pub fn jump(&mut self, func: &Object) {
        self.counter = *self
            .program
            .get_func(func)
            .unwrap_or_else(|| panic!("No such function: {:?}", func));
    }

    pub fn handle_bin_op(&mut self, kind: &BinOpKind) {
        let rhs = self
            .obj_stack
            .pop()
            .unwrap_or_else(|| panic!("Not enough on the stack"));
        let lhs = self
            .obj_stack
            .pop()
            .unwrap_or_else(|| panic!("Not enough on the stack"));

        self.obj_stack
            .push(self.program.handle_bin_op(*kind, lhs, rhs))
    }
}
