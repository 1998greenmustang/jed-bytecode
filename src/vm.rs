use std::{collections::HashMap, fs::File, io};

use crate::{
    arena,
    binops::{self, BinOpKind},
    error::{ProgramError, ProgramErrorKind},
    frame::{Frame, FrameKind},
    object::Object,
    operation::Operation,
    program::Program,
    span::Span,
    stack::Stack,
    utils,
};

pub struct VM {
    pub program: Program,
    pub consts: HashMap<&'static [u8], &'static Object>,
    pub counter: usize,
    pub call_stack: Stack<Frame>,
    pub obj_stack: Stack<&'static Object>,
    pub temp: Option<&'static Object>,
    pub memory: arena::Manual<Object>,
    pub current_span: Span,
    pub debug: bool,
}

impl VM {
    pub fn new(program: Program, debug: bool) -> Self {
        let mut call_stack = Stack::new();
        call_stack.push(Frame::new(program.instructions.len(), FrameKind::Main));
        let main = program.get_main();
        VM {
            call_stack,
            counter: main,
            program,
            consts: HashMap::new(),
            obj_stack: Stack::new(),
            temp: None,
            memory: Default::default(),
            current_span: Span::empty(),
            debug,
        }
    }

    pub fn register_single(&mut self, obj: Object) -> &'static Object {
        unsafe { self.register_many([obj].as_slice()).get_unchecked(0) }
    }

    pub fn register_many(&mut self, objs: &[Object]) -> &'static [Object] {
        let saved_bytes = self.memory.alloc_slice(objs);
        let saved_bytes: &'static [Object] = unsafe { &*(saved_bytes as *const [Object]) };
        saved_bytes
    }

    pub fn drop(&mut self, obj: &'static Object) {
        self.memory.deallocate(
            obj as *const Object as *mut Object,
            std::alloc::Layout::for_value(obj),
        );
    }
    pub fn drop_list(&mut self, start: *const Object, len: usize) -> &[Object] {
        let size = size_of::<Object>() * len;
        self.memory.deallocate(
            start as *mut Object,
            std::alloc::Layout::from_size_align(size, align_of::<Object>()).expect("a"),
        );
        return &[];
    }

    pub fn store_const(&mut self, name: &'static [u8], obj: Object) {
        let obj: &'static Object = self.register_single(obj);
        self.consts.insert(name, obj);
    }

    pub fn get_const(&self, name: &'static [u8]) -> Option<&'static Object> {
        self.consts.get(name).map(|v| &**v)
    }

    pub fn from_string(text: String, debug: bool) -> Self {
        let program = Program::from_string(text);
        Self::new(program, debug)
    }
    pub fn from_file(file: &mut File, debug: bool) -> io::Result<Self> {
        let program = Program::from_file(file)?;
        Ok(Self::new(program, debug))
    }

    pub fn run(&mut self) {
        self.counter = self.program.get_main();
        loop {
            self.update_span();
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
                            .map(|x| format!("{:?}", x))
                            .collect::<Vec<String>>()
                    )
                }
            }
            let op = self.next();
            let res = op.unwrap().call(self);

            match res {
                Ok(_) => {}
                Err(e) => {
                    println!("\nruntime failure:\n{}", e);
                    std::process::exit(1);
                }
            }

            if let Operation::Exit = self.program.get_op(self.counter) {
                return;
            }
        }
    }

    pub fn run_block(&mut self, frame_type: FrameKind) {
        loop {
            self.update_span();
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
                            .map(|x| format!("{:?}", x))
                            .collect::<Vec<String>>()
                    )
                }
            }
            let op = self.next();
            let res = op.unwrap().call(self);

            match res {
                Ok(_) => {}
                Err(e) => {
                    println!("\nruntime failure:\n{}", e);
                    std::process::exit(1);
                }
            }

            if let Ok(frame) = self.call_stack.last() {
                let op = self.program.get_op(self.counter);
                if frame.kind == frame_type {
                    match op {
                        Operation::Done | Operation::Exit => return,
                        _ => (),
                    }
                }
            }
        }
    }

    pub fn exit(&mut self, code: Option<i32>) {
        self.counter = self.program.get_main();
        self.obj_stack = Stack::new();
        self.call_stack = Stack::new();
        self.program.memos.clear();
        std::process::exit(code.unwrap_or_default());
    }

    fn update_span(&mut self) {
        let prev_op_pc = self.counter.checked_sub(1);
        let prev_op = match prev_op_pc {
            Some(s) => self
                .program
                .instructions
                .get(s)
                .unwrap_or(&Operation::Empty),
            None => &Operation::Empty,
        };
        self.current_span = Span {
            current_op: *self
                .program
                .instructions
                .get(self.counter)
                .unwrap_or(&Operation::Empty),
            program_count: self.counter,
            prev_op: *prev_op,
            next_op: *self
                .program
                .instructions
                .get(self.counter + 1)
                .unwrap_or(&Operation::Empty),
        };
    }

    #[inline]
    pub fn next(&mut self) -> Option<Operation> {
        let op = self.program.get_op(self.counter);
        self.counter += 1;
        return Some(*op);
    }

    pub fn jump(&mut self, func: &'static [u8]) {
        let (idx, _arity) = *self
            .program
            .funcs
            .get(func)
            .unwrap_or_else(|| panic!("No such function: {:?}", func));
        self.counter = idx;
    }

    pub fn goto(&mut self, counter: usize) {
        self.counter = counter
    }

    pub fn handle_bin_op(&mut self, kind: BinOpKind) -> Result<(), ProgramError> {
        let pair = {
            match unsafe { self.obj_stack.pop_n(2) } {
                Ok(ts) => Ok(ts),
                Err(_) => Err(ProgramError(
                    ProgramErrorKind::StackError(2),
                    self.current_span.clone(),
                )),
            }
        }?;
        let lhs = pair[0].data;
        let rhs = pair[1].data;

        let result = match kind {
            BinOpKind::Add => binops::add(lhs, rhs),
            BinOpKind::Sub => binops::sub(lhs, rhs),
            BinOpKind::Mul => binops::mul(lhs, rhs),
            BinOpKind::Div => binops::div(lhs, rhs),
            BinOpKind::Mod => binops::modulus(lhs, rhs),
            BinOpKind::Eq => binops::eq(lhs, rhs),
            BinOpKind::LessEq => binops::lesseq(lhs, rhs),
            BinOpKind::GreatEq => binops::greateq(lhs, rhs),
            BinOpKind::Lesser => binops::lesser(lhs, rhs),
            BinOpKind::Greater => binops::greater(lhs, rhs),
            BinOpKind::And => binops::and(lhs, rhs),
            BinOpKind::Or => binops::or(lhs, rhs),
            BinOpKind::Power => binops::pow(lhs, rhs),
            BinOpKind::Root => binops::root(lhs, rhs),
        };

        match result {
            Ok(value) => {
                let value = self.register_single(value);
                Ok(self.obj_stack.push(value))
            }
            Err(e) => Err(ProgramError(e, self.current_span.clone())),
        }
    }

    pub fn unwrap_or_error<T>(
        &self,
        option: Option<T>,
        kind: ProgramErrorKind,
    ) -> Result<T, ProgramError> {
        match utils::unwrap_or_error(option, kind) {
            Ok(v) => Ok(v),
            Err(e) => return Err(ProgramError(e, self.current_span.clone())),
        }
    }

    pub fn error<T>(&self, e: ProgramErrorKind) -> Result<T, ProgramError> {
        match e {
            ProgramErrorKind::VariableExists(_) => {
                match self.call_stack.last().cloned() {
                    Ok(frame) => println!(
                        "current variables: {:?}",
                        frame
                            .locals
                            .into_keys()
                            .map(|x| utils::bytes_to_string(x))
                            .collect::<Vec<String>>()
                    ),
                    Err(_) => todo!(),
                };
            }
            ProgramErrorKind::StackError(_) => unsafe {
                println!(
                    "call stack: {:?}",
                    self.call_stack
                        .last_n(10)
                        .iter()
                        .map(|x| format!("{:?}", x))
                        .collect::<Vec<String>>()
                );
                println!(
                    "object stack: {:?}",
                    self.obj_stack
                        .last_n(10)
                        .iter()
                        .map(|x| format!("{:?}", x))
                        .collect::<Vec<String>>()
                );
            },
            _ => {}
        };

        Err(ProgramError(e, self.current_span.clone()))
    }
}
