use std::{cell::RefCell, collections::HashMap, fs::File, io, rc::Rc, time::Instant};

use crate::{
    error::{ProgramError, ProgramErrorKind},
    frame::{Frame, FrameKind},
    memory::{self, list::List, stack::Stack},
    object::{MutableObject, Object, ObjectData, ObjectKind, RegObject},
    operation::{Operation, Response},
    program::Program,
    span::Span,
    utils,
};

pub struct VM {
    pub program: Program,
    pub consts: Rc<RefCell<List<(&'static [u8], RegObject)>>>,
    pub counter: usize,
    pub call_stack: Stack<Frame>,
    pub obj_stack: Stack<RegObject>,
    pub temp: Option<RegObject>,
    pub memory: memory::Manual<Object>,
    pub current_span: Span,
    pub debug: bool,
    start: Instant,
}

impl VM {
    pub fn new(program: Program, debug: bool) -> Self {
        let mut call_stack: Stack<Frame> = Stack::new();
        call_stack.push(Frame::new(program.instructions.len(), FrameKind::Initial));
        VM {
            call_stack,
            counter: 0,
            program,
            consts: Rc::new(RefCell::new(List::new())),
            obj_stack: Stack::new(),
            temp: None,
            memory: Default::default(),
            current_span: Span::empty(),
            debug,
            start: Instant::now(),
        }
    }

    pub fn create_object(&mut self, name: &'static [u8]) -> Response {
        if let Some(Operation::Object(_name, arity, block)) =
            self.program.constructors.get(name).cloned()
        {
            let obj = self.register_single(Object::new(ObjectData::Data(Box::into_raw(Box::new(
                HashMap::new(),
            )))));
            if self.obj_stack.len() >= arity {
                self.obj_stack.push(obj);
                self.run_block(&block);
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
                            let obj = self.register_single(obj);
                            self.obj_stack.push(obj);
                        }
                        Response::Ok
                    }
                    _ => res,
                }
            }
            Some(Operation::Func(name, arity, block)) => {
                // println!("{block}");
                let args = {
                    match unsafe { self.obj_stack.last_n(arity) } {
                        Ok(ts) => ts,
                        Err(_) => return Response::Error(ProgramErrorKind::StackError(arity)),
                    }
                };
                let args = if arity > 0 {
                    let deferenced: Vec<Object> = args.iter().map(|x| **x).collect();
                    self.register_many(&deferenced)
                } else {
                    &[]
                };
                // match self.program.get_memo((name, args)).cloned() {
                //     Some(value) => {
                //         // println!("YES DUDE {:?}", args);
                //         match self.obj_stack.pop_n(arity) {
                //             Ok(ts) => ts,
                //             Err(_) => return Response::Error(ProgramErrorKind::StackError(arity)),
                //         };
                //         let value = self.register_single(value);
                //         self.obj_stack.push(value);
                //         Response::Ok
                //     }
                //     None => {
                self.call_stack
                    .push(Frame::new(self.counter, FrameKind::Call));
                let current_frame = match self.call_stack.last_mut() {
                    Ok(ts) => ts,
                    Err(e) => return Response::Error(e),
                };
                let args = if args.len() > 0 {
                    self.program.register_arguments(args)
                } else {
                    args
                };
                // current_frame.memo_key = (name, args);
                self.run_block(&block.clone());
                let frame = self.call_stack.pop().unwrap();
                self.goto(frame.return_address);
                Response::Ok
                //     }
                // }
                // Response::Ok
            }
            _ => Response::Error(ProgramErrorKind::FunctionExists(name)),
        }
    }

    pub fn parse_lit(&mut self, bytes: &'static [u8]) -> Result<RegObject, ProgramErrorKind> {
        let get_const = self.get_const(bytes);
        if let Some(lit) = get_const {
            Ok(lit)
        } else {
            let string = unsafe { String::from_utf8_unchecked(bytes.to_vec()) };
            if string.starts_with('[') && string.ends_with(']') {
                // let bytess = &string[1..string.len() - 1];
                todo!("pushing many at a time")
            } else if string.starts_with('"') && string.ends_with('"') {
                let s = &string[1..string.len() - 1];
                let sb = self.program.register(s.to_owned());
                Ok(self.register_single(Object::new(ObjectData::String(sb))))
            } else if string == "true" {
                Ok(self.register_single(Object::new(ObjectData::Bool(true))))
            } else if string == "false" {
                Ok(self.register_single(Object::new(ObjectData::Bool(false))))
            } else if string == "Nil" {
                Ok(self.register_single(Object::nil()))
            } else if string.chars().all(|c| c.is_numeric()) {
                let num: isize = match utils::string_to_t(string) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };
                Ok(self.register_single(num.into()))
            } else if utils::string_is_float_like(string.clone()) {
                let num: f64 = str::parse(&string).unwrap();
                Ok(self.register_single(num.into()))
            } else {
                return Err(ProgramErrorKind::ParsingError(utils::display_bytes(bytes)));
            }
        }
    }

    pub fn register_single(&mut self, obj: Object) -> RegObject {
        let saved_bytes = self.memory.alloc(&obj);
        let saved_bytes: &'static mut Object = saved_bytes;
        saved_bytes
    }
    pub fn register_single_mut(&mut self, obj: Object) -> MutableObject {
        let saved_bytes = self.memory.alloc_slice(&[obj]);
        let saved_bytes: &'static mut [Object] = unsafe { &mut *(saved_bytes as *mut [Object]) };
        unsafe { saved_bytes.get_unchecked_mut(0) }
    }

    pub fn register_many(&mut self, objs: &[Object]) -> &'static [Object] {
        let saved_bytes = self.memory.alloc_slice(objs);
        let saved_bytes: &'static [Object] = unsafe { &*(saved_bytes as *const [Object]) };
        saved_bytes
    }

    pub fn resize_list(
        &mut self,
        starting_ptr: *mut Object,
        len: usize,
        alloc: usize,
        n: usize,
    ) -> Result<*const Object, ProgramErrorKind> {
        if n <= alloc {
            // shrink it baby
            // i dont actually want to do
            // because i do not care
            return Ok(starting_ptr);
        } else if n > alloc {
            // grow it baby
            let amt_to_alloc = n - alloc;
            let pretend_ptr = unsafe { starting_ptr.add(alloc).addr() as *const Object };
            if pretend_ptr == self.memory.start().addr() as *const Object {
                self.memory.extend_from(starting_ptr, amt_to_alloc);
                return Ok(starting_ptr);
            } else {
                unsafe {
                    // create new list
                    let mut objects: Vec<Object> = vec![];
                    for i in 0..len {
                        let obj = starting_ptr.add(i);
                        objects.push(*obj.clone());
                        self.drop(&*obj);
                    }
                    while objects.len() != n {
                        objects.push(Object::nil());
                    }
                    let objects: &'static [Object] = self.register_many(&objects);
                    return Ok(objects.as_ptr());
                }
            }
        }
        Err(ProgramErrorKind::TodoError)
    }

    pub fn drop(&mut self, obj: RegObject) {
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
        let obj: RegObject = self.register_single(obj);
        (*self.consts).borrow_mut().push((name, obj));
    }

    pub fn get_const(&self, name: &'static [u8]) -> Option<RegObject> {
        (*self.consts)
            .borrow()
            .iter()
            .find(|tpl| tpl.0 == name)
            .map(|tpl| tpl.1)
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
        self.start = Instant::now();
        while let Some(op) = self.next() {
            // println!(
            //     "{}/{} {:?}",
            //     self.counter,
            //     self.program.instructions.len() - 1,
            //     self.program.instructions.get(self.counter)
            // );
            // self.update_span();
            if self.call_stack.len() > 100_000 {
                panic!("call stack overflow");
            }
            if self.obj_stack.len() > 1_000_000_000 {
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

            let res = op.call(self);

            match res {
                Response::Exit(code) => self.exit(Some(code)),
                Response::Error(err) => {
                    println!("\nruntime failure:\n{}", self.error(err));
                    self.exit(Some(1));
                }
                Response::Ok => continue,
                Response::BlockReturn => unreachable!(),
                Response::ExternReturn(_) => unreachable!(),
                Response::IterationDone => break,
                Response::FunctionReturn(_) => todo!(),
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

    pub fn run_block(&mut self, block: &List<Operation>) {
        let mut iter = block.iter();
        while let Some(op) = iter.next() {
            // println!("{}", op);
            if self.call_stack.len() > 100_000 {
                panic!("call stack overflow");
            }
            if self.obj_stack.len() > 10_000_000 {
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
                    let _ = self.call_stack.pop();
                    // break;
                }
                Response::ExternReturn(_) => unreachable!(),
                Response::FunctionReturn(Some(obj)) => {
                    let obj = self.register_single(obj);
                    self.obj_stack.push(obj)
                }
                Response::FunctionReturn(None) => continue,
            }
        }
    }

    pub fn exit(&mut self, code: Option<i32>) {
        self.counter = 0;
        self.obj_stack = Stack::new();
        self.call_stack = Stack::new();
        self.program.memos.clear();
        println!("{:?}", Instant::now() - self.start);
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
                    Ok(frame) => println!(
                        "current variables: {:?}",
                        (*frame.locals)
                            .borrow()
                            .iter()
                            .map(|tpl| utils::display_bytes(tpl.0))
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

        ProgramError(e, self.current_span.clone())
    }
}
