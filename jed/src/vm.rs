use std::{collections::HashMap, fs::File, io};

use crate::{
    error::{ProgramError, ProgramErrorKind},
    frame::Frame,
    memory::{self, list::List, stack::Stack},
    modules::io::Io,
    object::Object,
    operation::{Operation, Response},
    program::Program,
    span::Span,
};

pub struct VM {
    pub program: Program,
    pub counter: usize,
    pub call_stack: Stack<Frame>,
    pub obj_stack: Stack<Object>,
    pub temp: Option<Object>,
    pub current_span: Span,
    pub debug: bool,
    pub locals: Vec<Object>,
    pub io: Io,
}

impl VM {
    pub fn new(program: Program, debug: bool) -> Self {
        let mut call_stack: Stack<Frame> = Stack::new();
        call_stack.push(Frame::new(0, program.instructions.len()));
        VM {
            call_stack,
            counter: 0,
            program,
            obj_stack: Stack::new(),
            temp: None,
            current_span: Span::empty(),
            debug,
            locals: vec![],
            io: Io::new(),
        }
    }

    pub fn create_object(&mut self, name: &'static [u8]) -> Response {
        if let Some(Operation::Object(_name, arity, block)) =
            self.program.constructors.get(name).cloned()
        {
            let obj = Object::new_data(Box::into_raw(Box::new(HashMap::new())));
            if self.obj_stack.len() >= arity {
                self.obj_stack.push(obj);
                self.run_block(&block.0, block.1);
                Response::Ok
            } else {
                Response::Error(ProgramErrorKind::StackError(arity))
            }
        } else {
            Response::Error(ProgramErrorKind::TodoError)
        }
    }

    pub fn call(&mut self, name: &'static [u8]) -> Response {
        match self.program.funcs.get(name).cloned() {
            Some(Operation::ExternFunc(_name, arity, func)) => {
                let args = {
                    match self.obj_stack.pop_n(arity) {
                        Ok(ts) => ts,
                        Err(_) => return Response::Error(ProgramErrorKind::StackError(arity)),
                    }
                };
                let res = func(args);
                match res {
                    Response::ExternReturn(maybe_obj) => {
                        if let Some(obj) = maybe_obj {
                            self.obj_stack.push(obj);
                        }
                        Response::Ok
                    }
                    _ => res,
                }
            }
            Some(Operation::Func(_name, arity, block)) => {
                // println!("{block}");
                match unsafe { self.obj_stack.last_n(arity) } {
                    Ok(ts) => ts,
                    Err(_) => return Response::Error(ProgramErrorKind::StackError(arity)),
                };
                let frame = Frame::new(self.locals.len(), self.counter);
                self.call_stack.push(frame);
                self.run_block(&block.0, block.1);
                let frame = self.call_stack.pop().unwrap();
                self.goto(frame.return_address);
                Response::Ok
            }
            _ => Response::Error(ProgramErrorKind::FunctionExists(name)),
        }
    }

    pub fn from_string(text: String, debug: bool) -> Self {
        let program = Program::from_string(text);
        Self::new(program, debug)
    }
    pub fn from_file(file: &mut File, debug: bool) -> io::Result<Self> {
        let program = Program::from_file(file)?;
        Ok(Self::new(program, debug))
    }

    // pub fn from_ops(ops: Vec<Operation>, debug: bool) -> Self {
    //     let program = Program::from_ops(ops);
    //     Self::new(program, debug)
    // }

    pub fn run(&mut self) {
        while let Some(op) = self.next() {
            // println!("{op}");
            if self.call_stack.len() > 100_000 {
                panic!("call stack overflow");
            }
            if self.obj_stack.len() > 1_000_000_000 {
                panic!("obj stack overflow");
            }

            let res = op.call(self);

            match res {
                Response::Exit(code) => self.exit(Some(code)),
                Response::Error(err) => {
                    println!("\nruntime failure:\n{}", self.error(err));
                    self.exit(Some(1));
                }
                Response::Ok => continue,
                Response::BlockReturn => unreachable!(),
                Response::ExternReturn(obj) => {
                    obj.map(|o| self.obj_stack.push(o));
                }
                Response::IterationDone => continue,
                Response::FunctionReturn(obj) => {
                    obj.map(|o| self.obj_stack.push(o));
                }
            }
            // if self.counter == self.program.instructions.len() {
            //     if !ran_main {
            //         let main = self.program.register_bytes(b"main");

            //         match self.program.funcs.get(main) {
            //             Some((idx, _)) => {
            //                 let _ = self.call(main);
            //                 ran_main = true;
            //             }
            //             None => return,
            //         }
            //     } else {
            //         return;
            //     }
            // }
        }
    }

    pub fn run_block(&mut self, block: &List<Operation>, nlocals: usize) {
        // check if i need to add more variable space
        if self.locals.len() == self.call_stack.last_unchecked().base {
            self.locals.resize(
                nlocals + self.call_stack.last_unchecked().base,
                Object::new_nil(),
            );
        }
        let mut iter = block.iter();
        while let Some(op) = iter.next() {
            // println!("{}", op);
            if self.call_stack.len() > 100_000 {
                panic!("call stack overflow");
            }
            if self.obj_stack.len() > 10_000_000 {
                panic!("object stack overflow");
            }
            let res = op.call(self);

            match res {
                Response::Exit(code) => std::process::exit(code),
                Response::BlockReturn => break,
                Response::Error(err) => {
                    println!("\nruntime failure:\n{}", self.error(err));
                    std::process::exit(1);
                }
                Response::Ok => continue,
                Response::IterationDone => {
                    // break;
                }
                Response::ExternReturn(_) => unreachable!(),
                Response::FunctionReturn(Some(obj)) => {
                    self.obj_stack.push(obj);
                    break;
                }
                Response::FunctionReturn(None) => break,
            }
        }
    }

    pub fn exit(&mut self, code: Option<i32>) {
        self.counter = 0;
        self.obj_stack = Stack::new();
        self.call_stack = Stack::new();
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
            current_op: self
                .program
                .instructions
                .get(self.counter)
                .unwrap_or(&Operation::Empty)
                .clone(),
            program_count: self.counter,
            prev_op: prev_op.clone(),
            next_op: self
                .program
                .instructions
                .get(self.counter + 1)
                .unwrap_or(&Operation::Empty)
                .clone(),
        };
    }

    #[inline]
    pub fn next(&mut self) -> Option<Operation> {
        let op = self.program.get_op(self.counter);
        self.counter += 1;
        return Some(op.clone());
    }

    #[inline]
    pub fn goto(&mut self, counter: usize) {
        self.counter = counter
    }

    pub fn error(&mut self, e: ProgramErrorKind) -> ProgramError {
        self.update_span();
        match e {
            ProgramErrorKind::VariableExists(_) => {
                match self.call_stack.last().cloned() {
                    Ok(_frame) => todo!(),
                    // println!(
                    //     "frame type: {:?}, current variables: {:?}",
                    //     (frame.kind),
                    //     (*frame.locals)
                    //         .borrow()
                    //         .iter()
                    //         .collect::<Vec<&'static &Object>>()
                    // ),
                    Err(_) => todo!(),
                };
            }
            ProgramErrorKind::StackError(_) => {
                // println!(
                //     "call stack: {:?}",
                //     self.call_stack
                //         .last_n(10)
                //         .iter()
                //         .map(|x| format!("{:?}", x))
                //         .collect::<Vec<String>>()
                // );
                // println!(
                //     "object stack: {:?}",
                //     self.obj_stack
                //         .last_n(10)
                //         .iter()
                //         .map(|x| format!("{:?}", x))
                //         .collect::<Vec<String>>()
                // );
                todo!()
            }
            _ => {}
        };

        ProgramError(e, self.current_span.clone())
    }

    #[inline]
    pub fn get_local(&self, idx: usize) -> Object {
        unsafe {
            // println!("{:?}", self.locals);
            // println!("{idx}, {}", self.call_stack.last_unchecked().base);
            *self
                .locals
                .get_unchecked(self.call_stack.last_unchecked().base + idx)
        }
    }

    #[inline]
    pub fn set_local(&mut self, idx: usize, obj: Object) {
        self.locals
            .insert(self.call_stack.last_unchecked().base + idx, obj);
    }
}
