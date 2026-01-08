use std::collections::HashMap;

use crate::{
    binops::BinOpKind,
    frame::{Frame, FrameKind},
    indexmap::{IndexMap, IndexSet},
    mutable::MutableObject,
    object::{Object, ObjectKind},
    operation::Operation,
    program::Program,
    stack::Stack,
};

pub struct VM {
    pub program: Program,
    pub consts: HashMap<Object, Object>,
    pub counter: usize,
    pub call_stack: Stack<Frame>,
    pub heap: IndexMap<MutableObject>,
    pub obj_stack: Stack<Object>,
    pub temp: Option<Object>,
}

impl VM {
    // TODO add "consts" so we don't to CONSTANTLY fresh new objects for the same literal
    pub fn new(program: Program) -> Self {
        let mut call_stack = Stack::new();
        call_stack.push(Frame::new(program.instructions.len(), FrameKind::Main));
        VM {
            call_stack,
            counter: program.get_main(),
            program,
            consts: HashMap::new(),
            heap: IndexMap::new(),
            obj_stack: Stack::new(),
            temp: None,
        }
    }

    pub fn store_const(&mut self, name: Object, obj: Object) {
        self.consts.insert(name, obj);
    }

    pub fn get_const(&self, name: &Object) -> &Object {
        self.consts.get(name).unwrap()
    }

    pub fn create_list(&mut self, n: usize) -> usize {
        let objects = unsafe { self.obj_stack.pop_n(n) };
        let items: Vec<MutableObject> = objects.iter().map(|o| (*o).into()).collect();
        let idxs: Vec<usize> = items.iter().map(|i| self.heap.push(*i)).collect();
        let objs: Vec<Object> = idxs
            .iter()
            .map(|idx| {
                Object(
                    ObjectKind::MutablePtr,
                    self.program.register(&idx.to_be_bytes()),
                )
            })
            .collect();
        let registered_objs = self.program.register_objects(objs.as_slice());
        self.heap.push(MutableObject::List(registered_objs))
    }

    pub fn from_string(text: String) -> Self {
        let program = Program::from_string(text);
        Self::new(program)
    }

    pub fn run(&mut self) {
        self.counter = self.program.get_main();
        loop {
            if self.counter == self.program.instructions.len() - 1 {
                return;
            }
            if self.call_stack.len() > 100_000 {
                panic!("call stack overflow");
            }
            if self.obj_stack.len() > 1_000_000 {
                unsafe {
                    panic!(
                        "object stack overflow, {:?}",
                        self.obj_stack
                            .last_n(10)
                            .iter()
                            .map(|x| format!("{}", x))
                            .collect::<Vec<String>>()
                    )
                }
            }
            let op = self.next();
            // println!(
            // "op: {:?}, pc: {}, tmp: {:?}, objstack: {:?}",
            // op.unwrap(),
            // self.counter,
            // self.temp,
            // self.obj_stack
            // );
            op.unwrap().call(self);
            if let Operation::Exit = self.program.get_op(self.counter) {
                self.counter = self.program.get_main();
                self.obj_stack = Stack::new();
                self.call_stack = Stack::new();
                self.program.memos.clear();
                return;
            }
        }
    }

    #[inline]
    pub fn next(&mut self) -> Option<Operation> {
        let op = self.program.get_op(self.counter);
        self.counter += 1;
        return Some(*op);
    }

    pub fn jump(&mut self, func: &Object) {
        let (idx, _arity) = *self
            .program
            .get_func(func)
            .unwrap_or_else(|| panic!("No such function: {:?}", func));
        self.counter = idx;
    }

    pub fn goto(&mut self, counter: usize) {
        self.counter = counter
    }

    pub fn handle_bin_op(&mut self, kind: &BinOpKind) {
        let pair = unsafe { self.obj_stack.pop_n(2) };
        let lhs = pair[0];
        let rhs = pair[1];

        self.obj_stack
            .push(self.program.handle_bin_op(*kind, lhs, rhs))
    }
}
