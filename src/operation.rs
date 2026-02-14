use std::{convert::TryInto, fmt::Display};

use crate::{
    binops::BinOpKind,
    builtin::BuiltIn,
    error::{ProgramError, ProgramErrorKind},
    frame::{Frame, FrameKind},
    modules::{self, MODULES},
    object::{Object, ObjectData, ObjectKind},
    stack::Stack,
    utils::{self, bytes_to_string},
    vm::VM,
};

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Operation {
    BinOp(BinOpKind),
    Call(&'static [u8]),
    CallBuiltIn(BuiltIn),
    PushLit(&'static [u8]),
    PushName(&'static [u8]),
    PushTemp,
    Pop,
    ReturnIf(&'static [u8]),
    StoreConst(&'static [u8]),
    StoreName(&'static [u8]),
    StoreTemp,
    Func(&'static [u8], usize),
    Done,
    Exit,
    DoFor,
    DoForIn(&'static [u8]),
    CreateList(Option<usize>),
    ListPush,
    ListGet(Option<usize>),
    ListSet(Option<usize>),
    PushRange,
    ReturnIfConst(&'static [u8]),
    GetPtr,
    ReadPtr,
    SetPtr,
    GetIter,
    IterNext,
    IterPrev,
    IterSkip,
    IterCurrent,
    Iterate,
    DoIf,
    Debug,
    Import(&'static [u8]),
    Empty,
}

impl From<(u8, Option<usize>)> for Operation {
    fn from(value: (u8, Option<usize>)) -> Self {
        match value.0 {
            17 => Operation::CreateList(value.1),
            19 => Operation::ListGet(value.1),
            20 => Operation::ListSet(value.1),
            _ => panic!(),
        }
    }
}

impl From<(u8, &'static [u8])> for Operation {
    fn from(value: (u8, &'static [u8])) -> Self {
        match value.0 {
            2 => Operation::Call(value.1),
            4 => Operation::PushLit(value.1),
            5 => Operation::PushName(value.1),
            8 => Operation::ReturnIf(value.1),
            9 => Operation::StoreConst(value.1),
            10 => Operation::StoreName(value.1),
            16 => Operation::DoForIn(value.1),
            22 => Operation::ReturnIfConst(value.1),
            31 => Operation::ReturnIfConst(value.1),
            _ => panic!(),
        }
    }
}

impl From<u8> for Operation {
    fn from(value: u8) -> Self {
        match value {
            6 => Operation::PushTemp,
            7 => Operation::Pop,
            11 => Operation::StoreTemp,
            13 => Operation::Done,
            14 => Operation::Exit,
            15 => Operation::DoFor,
            18 => Operation::ListPush,
            21 => Operation::PushRange,
            23 => Operation::GetPtr,
            24 => Operation::ReadPtr,
            25 => Operation::SetPtr,
            26 => Operation::GetIter,
            27 => Operation::IterNext,
            28 => Operation::IterPrev,
            29 => Operation::IterSkip,
            30 => Operation::IterCurrent,
            33 => Operation::Debug,
            _ => panic!(),
        }
    }
}

impl From<Operation> for u8 {
    fn from(value: Operation) -> Self {
        match value {
            Operation::BinOp(_) => 1,
            Operation::Call(_) => 2,
            Operation::CallBuiltIn(_) => 3,
            Operation::PushLit(_) => 4,
            Operation::PushName(_) => 5,
            Operation::PushTemp => 6,
            Operation::Pop => 7,
            Operation::ReturnIf(_) => 8,
            Operation::StoreConst(_) => 9,
            Operation::StoreName(_) => 10,
            Operation::StoreTemp => 11,
            Operation::Func(_, _) => 12,
            Operation::Done => 13,
            Operation::Exit => 14,
            Operation::DoFor => 15,
            Operation::DoForIn(_) => 16,
            Operation::CreateList(_) => 17,
            Operation::ListPush => 18,
            Operation::ListGet(_) => 19,
            Operation::ListSet(_) => 20,
            Operation::PushRange => 21,
            Operation::ReturnIfConst(_) => 22,
            Operation::GetPtr => 23,
            Operation::ReadPtr => 24,
            Operation::SetPtr => 25,
            Operation::GetIter => 26,
            Operation::IterNext => 27,
            Operation::IterPrev => 28,
            Operation::IterSkip => 29,
            Operation::IterCurrent => 30,
            Operation::Iterate => 31,
            Operation::DoIf => 32,
            Operation::Debug => 33,
            Operation::Import(_) => 34,
            Operation::Empty => todo!(),
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::BinOp(bin_op_kind) => write!(f, "bin_op {}", bin_op_kind),
            Operation::Call(bytes) => write!(f, "call {}", bytes_to_string(bytes)),
            Operation::CallBuiltIn(built_in) => write!(f, "call_builtin {}", built_in),
            Operation::PushLit(bytes) => write!(f, "push_lit {}", bytes_to_string(bytes)),
            Operation::PushName(bytes) => write!(f, "push_name {}", bytes_to_string(bytes)),
            Operation::PushTemp => write!(f, "push_temp"),
            Operation::Pop => write!(f, "pop"),
            Operation::ReturnIf(bytes) => write!(f, "return_if {}", bytes_to_string(bytes)),
            Operation::StoreConst(bytes) => write!(f, "store_const {}", bytes_to_string(bytes)),
            Operation::StoreName(bytes) => write!(f, "store_name {}", bytes_to_string(bytes)),
            Operation::StoreTemp => write!(f, "store_temp"),
            Operation::Func(bytes, arity) => write!(f, "func {} {arity}", bytes_to_string(bytes)),
            Operation::Done => write!(f, "done"),
            Operation::Exit => write!(f, "exit"),
            Operation::DoFor => write!(f, "do_for"),
            Operation::DoForIn(bytes) => write!(f, "do_for_in {}", bytes_to_string(bytes)),
            Operation::CreateList(idx) => {
                write!(f, "create_list {}", utils::unwrap_as_string_or(*idx, ""))
            }
            Operation::ListPush => write!(f, "list_push"),
            Operation::ListGet(idx) => {
                write!(f, "list_get {}", utils::unwrap_as_string_or(*idx, ""))
            }
            Operation::ListSet(idx) => {
                write!(f, "list_set {}", utils::unwrap_as_string_or(*idx, ""))
            }
            Operation::PushRange => write!(f, "push_range"),
            Operation::ReturnIfConst(bytes) => {
                write!(f, "return_if_const {}", bytes_to_string(bytes))
            }
            Operation::GetPtr => write!(f, "get_ptr"),
            Operation::ReadPtr => write!(f, "read_ptr"),
            Operation::SetPtr => write!(f, "set_ptr"),
            Operation::GetIter => write!(f, "get_iter"),
            Operation::IterNext => write!(f, "iter_next"),
            Operation::IterPrev => write!(f, "iter_prev"),
            Operation::IterSkip => write!(f, "iter_skip"),
            Operation::IterCurrent => write!(f, "iter_current"),
            Operation::Iterate => write!(f, "iterate"),
            Operation::DoIf => write!(f, "do_if"),
            Operation::Debug => write!(f, "debug"),
            Operation::Import(bytes) => write!(f, "import {}", bytes_to_string(bytes)),
            Operation::Empty => write!(f, ""),
        }
    }
}

impl Operation {
    pub fn exists(s: &str) -> bool {
        match s {
            "bin_op" => true,
            "call" => true,
            "call_builtin" => true,
            "push_lit" => true,
            "push_name" => true,
            "push_temp" => true,
            "pop" => true,
            "return_if" => true,
            "store_const" => true,
            "store_name" => true,
            "store_temp" => true,
            "func" => true,
            "done" => true,
            "exit" => true,
            "do_for" => true,
            "do_for_in" => true,
            "create_list" => true,
            "list_push" => true,
            "list_get" => true,
            "list_set" => true,
            "push_range" => true,
            "return_if_const" => true,
            "get_ptr" => true,
            "read_ptr" => true,
            "set_ptr" => true,
            "get_iter" => true,
            "iter_next" => true,
            "iter_prev" => true,
            "iter_skip" => true,
            "iter_current" => true,
            "iterate" => true,
            "do_if" => true,
            "debug" => true,
            "import" => true,
            _ => false,
        }
    }

    pub fn get_opcode(op: &str) -> usize {
        match op {
            "bin_op" => 1,
            "call" => 2,
            "call_builtin" => 3,
            "push_lit" => 4,
            "push_name" => 5,
            "push_temp" => 6,
            "pop" => 7,
            "return_if" => 8,
            "store_const" => 9,
            "store_name" => 10,
            "store_temp" => 11,
            "func" => 12,
            "done" => 13,
            "exit" => 14,
            "do_for" => 15,
            "do_for_in" => 16,
            "create_list" => 17,
            "list_push" => 18,
            "list_get" => 19,
            "list_set" => 20,
            "push_range" => 21,
            "return_if_const" => 22,
            "get_ptr" => 23,
            "read_ptr" => 24,
            "set_ptr" => 25,
            "get_iter" => 26,
            "iter_next" => 27,
            "iter_prev" => 28,
            "iter_skip" => 29,
            "iter_current" => 30,
            "iterate" => 31,
            "do_if" => 32,
            "debug" => 33,
            "import" => 34,
            _ => 0,
        }
    }

    pub fn call(&self, vm: &mut VM) -> Result<(), ProgramError> {
        match self {
            Operation::BinOp(bin_op_kind) => vm.handle_bin_op(*bin_op_kind),
            Operation::Call(func) => {
                let (func_ptr, arity) = vm.unwrap_or_error(
                    vm.program.funcs.get(func).cloned(),
                    ProgramErrorKind::FunctionExists(func),
                )?;
                let args = {
                    match unsafe { vm.obj_stack.last_n(arity) } {
                        Ok(ts) => Ok(ts),
                        Err(_) => vm.error(ProgramErrorKind::StackError(arity)),
                    }
                }?;
                let args = if arity > 0 {
                    let deferenced: Vec<Object> = args.iter().map(|x| **x).collect();
                    vm.register_many(&deferenced)
                } else {
                    &[]
                };
                match vm.program.get_memo((func_ptr, args)) {
                    Some(value) => {
                        // println!("YES DUDE {:?}", args);
                        match unsafe { vm.obj_stack.pop_n(arity) } {
                            Ok(ts) => Ok(ts),
                            Err(_) => vm.error(ProgramErrorKind::StackError(arity)),
                        }?;
                        let value = vm.register_single(*value);
                        vm.obj_stack.push(value);
                        Ok(())
                    }
                    None => {
                        vm.call_stack.push(Frame::new(vm.counter, FrameKind::Call));
                        let current_frame = match vm.call_stack.last_mut() {
                            Ok(ts) => Ok(ts),
                            Err(e) => return vm.error(e),
                        }?;
                        let args = if args.len() > 0 {
                            vm.program.register_arguments(args)
                        } else {
                            args
                        };
                        current_frame.memo_key = (func_ptr, args);
                        vm.jump(&func);
                        Ok(())
                    }
                }
            }
            Operation::PushLit(literal) => {
                let get_const = vm.get_const(literal);
                if let Some(lit) = get_const {
                    vm.obj_stack.push(lit);
                } else {
                    let string = unsafe { String::from_utf8_unchecked(literal.to_vec()) };
                    let obj = {
                        if string.starts_with('[') && string.ends_with(']') {
                            // let literals = &string[1..string.len() - 1];
                            todo!("pushing many at a time")
                        } else if string.starts_with('"') && string.ends_with('"') {
                            let s = &string[1..string.len() - 1];
                            let sb = vm.program.register(s.to_owned());
                            vm.register_single(Object {
                                kind: ObjectKind::String,
                                data: ObjectData::String(sb),
                            })
                        } else if string == "true" {
                            vm.register_single(Object {
                                kind: ObjectKind::Bool,
                                data: ObjectData::Bool(true),
                            })
                        } else if string == "false" {
                            vm.register_single(Object {
                                kind: ObjectKind::Bool,
                                data: ObjectData::Bool(false),
                            })
                        } else if string == "Nil" {
                            vm.register_single(Object::nil())
                        } else if string.chars().all(|c| c.is_numeric()) {
                            let num: isize = match utils::string_to_t(string) {
                                Ok(v) => v,
                                Err(e) => return vm.error(e),
                            };
                            vm.register_single(Object {
                                kind: ObjectKind::Integer,
                                data: ObjectData::Integer(num),
                            })
                        } else if utils::string_is_float_like(string.clone()) {
                            let (wholestr, precstr) = string.split_at(string.find('.').unwrap());
                            let whole: i32 = match utils::string_to_t(wholestr.to_owned()) {
                                Ok(v) => v,
                                Err(e) => return vm.error(e),
                            };
                            let prec: u32 = match utils::string_to_t(precstr[1..].to_owned()) {
                                Ok(v) => v,
                                Err(e) => return vm.error(e),
                            };
                            vm.register_single(Object {
                                kind: ObjectKind::Float,
                                data: ObjectData::Float(whole, prec),
                            })
                        } else {
                            return vm.error(ProgramErrorKind::ParsingError(
                                utils::bytes_to_string(literal),
                            ));
                        }
                    };
                    vm.obj_stack.push(obj);
                    vm.store_const(literal, *obj);
                }
                Ok(())
            }
            Operation::PushName(name) => {
                // println!("{}", utils::bytes_to_string(name));
                let frame = {
                    match vm.call_stack.last() {
                        Ok(ts) => Ok(ts),
                        Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                    }
                }?;
                // println!("{:?}", frame.locals);
                let item = {
                    let option = frame.get_local(name);
                    let kind = ProgramErrorKind::VariableExists(name);
                    match utils::unwrap_or_error(option, kind) {
                        Ok(v) => Ok(v),
                        Err(e) => return vm.error(e),
                    }
                }?;
                vm.obj_stack.push(item);
                Ok(())
            }
            Operation::PushTemp => {
                let tmp = vm.unwrap_or_error(vm.temp, ProgramErrorKind::TempPush)?;
                vm.obj_stack.push(tmp);
                Ok(())
            }
            Operation::Pop => match {
                match { vm.obj_stack.pop() } {
                    Ok(t) => Ok(t),
                    Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                }
            } {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            Operation::ReturnIf(name) => {
                let b = {
                    match { vm.obj_stack.pop() } {
                        Ok(t) => Ok(t),
                        Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                    }
                }?;
                assert_eq!(b.kind, ObjectKind::Bool, "Object is not a boolean");
                if let ObjectData::Bool(bol) = b.data {
                    if bol {
                        let frame = {
                            match { vm.call_stack.pop() } {
                                Ok(t) => Ok(t),
                                Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                            }
                        }?;
                        vm.obj_stack.push(vm.unwrap_or_error(
                            frame.get_local(name),
                            ProgramErrorKind::VariableExists(name),
                        )?);
                        vm.counter = frame.return_address;
                        let return_value = *vm.obj_stack.last_option().unwrap_or(&&Object {
                            kind: ObjectKind::Nil,
                            data: ObjectData::Nil,
                        });
                        vm.program.set_memo(frame.memo_key, *return_value);
                    }
                }
                Ok(())
            }
            Operation::StoreConst(name) => {
                let obj = {
                    match { vm.obj_stack.pop() } {
                        Ok(t) => Ok(t),
                        Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                    }
                }?;
                vm.store_const(*name, *obj);
                Ok(())
            }
            Operation::StoreName(name) => {
                let frame = {
                    match vm.call_stack.last_mut() {
                        Ok(ts) => Ok(ts),
                        Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                    }
                }?;
                let obj = match vm.obj_stack.pop() {
                    Ok(t) => t,
                    Err(e) => return vm.error(e),
                };
                frame.add_local(*name, obj);
                Ok(())
            }
            Operation::StoreTemp => {
                let obj = {
                    match { vm.obj_stack.pop() } {
                        Ok(t) => Ok(t),
                        Err(e) => vm.error(e),
                    }
                }?;
                vm.temp = Some(obj);
                Ok(())
            }
            Operation::Func(_, _) => Ok(()),
            Operation::Done => {
                let frame = {
                    match { vm.call_stack.pop() } {
                        Ok(t) => Ok(t),
                        Err(e) => vm.error(e),
                    }
                }?;
                match frame.kind {
                    FrameKind::Call => {
                        let return_value = *vm.obj_stack.last_option().unwrap_or(&&Object {
                            kind: ObjectKind::Nil,
                            data: ObjectData::Nil,
                        });
                        vm.program.set_memo(frame.memo_key, *return_value);
                        vm.counter = frame.return_address;
                    }
                    FrameKind::IterateLoop | FrameKind::DoForLoop | FrameKind::DoForInLoop => {
                        vm.call_stack.push(frame);
                    }
                    FrameKind::DoIfBlock => {
                        return Ok(());
                    }
                    FrameKind::Main => vm.exit(Some(0)),
                }
                Ok(())
            }
            Operation::Exit => Ok(vm.exit(Some(0))),
            Operation::CallBuiltIn(built_in) => {
                let obj = {
                    match { vm.obj_stack.pop() } {
                        Ok(t) => Ok(t),
                        Err(e) => vm.error(e),
                    }
                }?;
                if let Some(val) = built_in.call(*obj) {
                    let val = vm.register_single(val);
                    vm.obj_stack.push(val);
                };
                Ok(())
            }
            Operation::DoFor => {
                let object = {
                    match { vm.obj_stack.pop() } {
                        Ok(t) => Ok(t),
                        Err(e) => vm.error(e),
                    }
                }?;
                if let ObjectData::Integer(times) = object.data {
                    let pc = vm.counter.clone();
                    let last_frame = match vm.call_stack.last() {
                        Ok(it) => it,
                        Err(err) => vm.error(err)?,
                    };
                    let mut new_frame = Frame::new(pc, FrameKind::DoForLoop);
                    new_frame.copy_locals(last_frame);
                    vm.call_stack.push(new_frame.clone());
                    for _ in 0..times {
                        vm.counter = pc;
                        vm.run_block(FrameKind::DoForLoop);
                    }
                    let _ = vm.call_stack.pop();
                    let done_address = vm.program.get_done(&(pc - 1));
                    match done_address {
                        Ok(addy) => vm.goto(*addy + 1),
                        Err(e) => vm.error(e)?,
                    }
                }
                Ok(())
            }
            Operation::DoForIn(obj_name) => {
                let current_frame = {
                    match vm.call_stack.last() {
                        Ok(ts) => Ok(ts),
                        Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                    }
                }?;
                let maybe_obj_ptr = current_frame.get_local(obj_name);
                let obj_ptr = maybe_obj_ptr.unwrap_or_else(|| {
                    panic!("No such name '{}'", utils::bytes_to_string(obj_name))
                });
                let pc = vm.counter.clone();
                let mut new_frame = Frame::new(pc, FrameKind::DoForInLoop);
                new_frame.copy_locals(current_frame);
                vm.call_stack.push(new_frame);
                match obj_ptr.as_tuple() {
                    (ObjectKind::List, ObjectData::List(_start, len)) => unsafe {
                        for _ in 0..*len {
                            vm.counter = pc;
                            vm.run_block(FrameKind::DoForInLoop);
                        }
                    },
                    (ObjectKind::Iterator, ObjectData::Iterator(list_ptr, _next)) => unsafe {
                        let list = *list_ptr;
                        if let ObjectData::List(_start, len) = list {
                            for _ in 0..*len {
                                vm.counter = pc;
                                vm.run_block(FrameKind::DoForInLoop);
                            }
                        }
                    },

                    (kind, _data) => {
                        return vm.error(ProgramErrorKind::TypeError(ObjectKind::List, kind))
                    }
                }
                let _ = vm.call_stack.pop();
                let done_address = vm.program.get_done(&(pc - 1));
                match done_address {
                    Ok(addy) => vm.goto(*addy + 1),
                    Err(e) => vm.error(e)?,
                }
                Ok(())
            }
            Operation::CreateList(maybe_num) => {
                let num = match maybe_num {
                    Some(v) => *v,
                    None => vm.obj_stack.len(),
                };
                let pop_res = unsafe { vm.obj_stack.pop_n(num) };
                let objects: Vec<Object> = match pop_res {
                    Ok(objs) => objs.iter().map(|o| **o).collect(),
                    Err(e) => return vm.error(e),
                };
                if objects.len() > 0 {
                    let objects: &'static [Object] = vm.register_many(objects.as_slice());
                    let len = Box::new(objects.len());
                    let obj_ptr = objects.as_ptr();
                    let obj_ptr: Box<usize> = Box::new(obj_ptr.addr());
                    let obj = Object {
                        kind: ObjectKind::List,
                        data: ObjectData::List(Box::into_raw(obj_ptr), Box::into_raw(len)),
                    };
                    let obj: &'static Object = vm.register_single(obj);

                    vm.obj_stack.push(obj);
                } else {
                    let len = Box::new(0);
                    let random_addr = Box::new(vm.memory.start().addr());
                    let obj = Object {
                        kind: ObjectKind::List,
                        data: ObjectData::List(Box::into_raw(random_addr), Box::into_raw(len)),
                    };
                    let obj: &'static Object = vm.register_single(obj);
                    vm.obj_stack.push(obj);
                }

                Ok(())
            }
            Operation::ListPush => unsafe {
                let new_item = match vm.obj_stack.pop().cloned() {
                    Ok(t) => Ok(t),
                    Err(e) => Err(ProgramError(e, vm.current_span.clone())),
                }?;
                match { vm.obj_stack.pop_mut() } {
                    Ok(mut t) => {
                        let Object { kind, mut data } = &mut t;
                        if let ObjectData::List(ref mut start, ref mut len) = data {
                            let starting_obj = **start as *const Object;

                            let obj: &'static Object = vm.register_single(new_item);
                            if start.add(**len) as *const Object == obj.as_ptr() {
                                **len += 1;
                            } else {
                                vm.drop(obj);
                                let mut objects: Vec<Object> = vec![];
                                for i in 0..**len {
                                    let obj = starting_obj.add(i);
                                    objects.push(*obj.clone());
                                    vm.drop(&*obj);
                                }
                                objects.push(new_item);
                                let objects: &'static [Object] = vm.register_many(&objects);
                                let new_start = start.with_addr(objects.as_ptr().addr());
                                **start = new_start as usize;
                                **len += 1;
                            }
                        } else {
                            return Err(ProgramError(
                                ProgramErrorKind::TypeError(ObjectKind::List, *kind),
                                vm.current_span.clone(),
                            ));
                        }
                    }
                    Err(_) => vm.error(ProgramErrorKind::StackError(1))?,
                }

                Ok(())
            },
            Operation::ListGet(maybe_idx) => {
                let idx = match maybe_idx {
                    Some(v) => *v,
                    None => {
                        let obj = vm.obj_stack.pop();
                        match obj {
                            Ok(v) => match v.as_tuple() {
                                (ObjectKind::Integer, ObjectData::Integer(n)) => {
                                    utils::isize_to_usize(n)
                                }
                                (kind, _data) => {
                                    return vm.error(ProgramErrorKind::TypeError(
                                        ObjectKind::Integer,
                                        kind,
                                    ))
                                }
                            },

                            Err(e) => return vm.error(e),
                        }
                    }
                };
                let list_obj = match { vm.obj_stack.pop() } {
                    Ok(t) => Ok(t),
                    Err(e) => vm.error(e),
                }?;
                match (list_obj.kind, list_obj.data) {
                    (ObjectKind::List, ObjectData::List(start, len)) => unsafe {
                        let start = *start as *const Object;
                        if idx < *len {
                            let obj_ptr = start.add(idx);
                            vm.obj_stack.push(&*obj_ptr);
                        } else {
                            return vm.error(ProgramErrorKind::ListIndexError(idx, *len));
                        }
                    },
                    (kind, _data) => {
                        return vm.error(ProgramErrorKind::TypeError(ObjectKind::List, kind))
                    }
                }
                Ok(())
            }
            Operation::ListSet(maybe_idx) => unsafe {
                let idx = match maybe_idx {
                    Some(v) => *v,
                    None => {
                        let obj = vm.obj_stack.pop();
                        match obj {
                            Ok(v) => match v.as_tuple() {
                                (ObjectKind::Integer, ObjectData::Integer(n)) => {
                                    utils::isize_to_usize(n)
                                }
                                (kind, _data) => {
                                    return vm.error(ProgramErrorKind::TypeError(
                                        ObjectKind::Integer,
                                        kind,
                                    ))
                                }
                            },

                            Err(e) => return vm.error(e),
                        }
                    }
                };
                let objects = {
                    match { vm.obj_stack.pop_n(2) } {
                        Ok(t) => Ok(t),
                        Err(e) => vm.error(e),
                    }
                }?;
                let list_obj = objects[1];
                let obj = objects[0];

                match (list_obj.kind, list_obj.data) {
                    (ObjectKind::List, ObjectData::List(start, len)) => {
                        let start = *start as *const Object;
                        if idx < *len {
                            let entry = start.add(idx) as *mut Object;
                            entry.copy_from(obj, 1);
                        } else {
                            return vm.error(ProgramErrorKind::ListIndexError(idx, *len));
                        }
                        Ok(())
                    }
                    (kind, _) => vm.error(ProgramErrorKind::TypeError(ObjectKind::List, kind)),
                }
            },
            Operation::PushRange => {
                let steps = vm.obj_stack.pop();
                let end = vm.obj_stack.pop();
                let start = vm.obj_stack.pop();
                match (start, end, steps) {
                    (Ok(strt), Ok(nd), Ok(stps)) => match (strt.data, nd.data, stps.data) {
                        (
                            ObjectData::Integer(s),
                            ObjectData::Integer(n),
                            ObjectData::Integer(ps),
                        ) => {
                            let p: usize = match ps.try_into() {
                                Ok(v) => v,
                                Err(_) => return vm.error(ProgramErrorKind::IntegerToUnsigned),
                            };
                            let values = (s..n)
                                .step_by(p)
                                .map(|v| Object {
                                    kind: ObjectKind::Integer,
                                    data: ObjectData::Integer(v),
                                })
                                .collect::<Vec<Object>>();
                            if values.len() > 0 {
                                let values: &'static [Object] = vm.register_many(&values);
                                values.iter().for_each(|v| vm.obj_stack.push(v));
                            }
                            Ok(())
                        }
                        _ => todo!(),
                    },
                    _ => todo!(),
                }
            }
            Operation::ReturnIfConst(name) => {
                let b = {
                    match { vm.obj_stack.pop() } {
                        Ok(t) => Ok(t),
                        Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                    }
                }?;
                assert_eq!(b.kind, ObjectKind::Bool, "Object is not a boolean");
                if let ObjectData::Bool(bol) = b.data {
                    if bol {
                        let frame = {
                            match { vm.call_stack.pop() } {
                                Ok(t) => Ok(t),
                                Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                            }
                        }?;
                        let obj = match vm.get_const(name) {
                            Some(v) => v,
                            None => return vm.error(ProgramErrorKind::ConstantExists(name)),
                        };
                        vm.obj_stack.push(obj);
                        vm.counter = frame.return_address;
                        let return_value = *vm.obj_stack.last_option().unwrap_or(&&Object {
                            kind: ObjectKind::Nil,
                            data: ObjectData::Nil,
                        });
                        vm.program.set_memo(frame.memo_key, *return_value);
                    }
                }
                Ok(())
            }
            Operation::GetPtr => {
                let obj: &'static Object = {
                    match { vm.obj_stack.pop() } {
                        Ok(t) => Ok(t),
                        Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                    }
                }?;

                let ptr_obj = {
                    Object {
                        kind: ObjectKind::Pointer,
                        data: ObjectData::Pointer(&mut &*obj as *mut &Object),
                    }
                };
                let ptr_obj: &'static Object = vm.register_single(ptr_obj);
                vm.obj_stack.push(ptr_obj);

                Ok(())
            }
            Operation::ReadPtr => {
                let ptr_obj: &'static Object = {
                    match { vm.obj_stack.pop() } {
                        Ok(t) => Ok(t),
                        Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                    }
                }?;

                if let ObjectData::Pointer(real_ptr) = ptr_obj.data {
                    let val: &'static Object = unsafe { real_ptr.read() };
                    vm.obj_stack.push(val);
                } else {
                    return vm.error(ProgramErrorKind::TypeError(
                        ObjectKind::Pointer,
                        ptr_obj.kind,
                    ));
                }

                Ok(())
            }
            Operation::SetPtr => unsafe {
                let objects = {
                    match { vm.obj_stack.pop_n(2) } {
                        Ok(t) => Ok(t),
                        Err(_) => vm.error(ProgramErrorKind::StackError(2)),
                    }
                }?;
                let obj = objects.get_unchecked(0);
                let ptr_obj = objects.get_unchecked(1);

                if let ObjectData::Pointer(real_ptr) = ptr_obj.data {
                    real_ptr.copy_from(obj, 1);
                } else {
                    return Err(ProgramError(
                        ProgramErrorKind::TypeError(ObjectKind::Pointer, ptr_obj.kind),
                        vm.current_span.clone(),
                    ));
                }

                Ok(())
            },
            Operation::GetIter => {
                let list_obj: &'static Object = {
                    match { vm.obj_stack.pop() } {
                        Ok(t) => Ok(t),
                        Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                    }
                }?;
                if let ObjectKind::List = list_obj.kind {
                    let initial_index = Box::new(0);
                    let iter_obj = Object {
                        kind: ObjectKind::Iterator,
                        data: ObjectData::Iterator(
                            &list_obj.data as *const ObjectData as *mut ObjectData,
                            Box::into_raw(initial_index),
                        ),
                    };
                    let iter_obj: &'static Object = vm.register_single(iter_obj);
                    vm.obj_stack.push(iter_obj);
                } else {
                    return vm.error(ProgramErrorKind::TypeError(ObjectKind::List, list_obj.kind));
                }
                Ok(())
            }
            Operation::IterNext => unsafe {
                let Object { kind, mut data } = {
                    match { vm.obj_stack.pop_mut() } {
                        Ok(&mut t) => Ok(t),
                        Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                    }
                }?;

                if let ObjectData::Iterator(list_ptr, ref mut next) = data {
                    let list = *list_ptr;
                    if let ObjectData::List(start, len) = list {
                        let start = *start as *const Object;
                        vm.obj_stack.push(&*start.add(**next));
                        if **next < *len {
                            **next += 1;
                        } else {
                            return vm.error(ProgramErrorKind::IterNext(*len));
                        }
                    }
                } else {
                    return vm.error(ProgramErrorKind::TypeError(ObjectKind::Iterator, *kind));
                }

                Ok(())
            },
            Operation::IterPrev => unsafe {
                let Object { kind, mut data } = {
                    match { vm.obj_stack.pop_mut() } {
                        Ok(&mut t) => Ok(t),
                        Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                    }
                }?;

                if let ObjectData::Iterator(list_ptr, ref mut next) = data {
                    let list = *list_ptr;
                    if let ObjectData::List(start, len) = list {
                        let start = *start as *const Object;
                        let cur_val = **next;
                        if cur_val == *len {
                            **next -= 1;
                            vm.obj_stack.push(&*start.add(**next));
                        } else if cur_val == 0 {
                            return vm.error(ProgramErrorKind::IterPrevious);
                        } else {
                            **next -= 1;
                            vm.obj_stack.push(&*start.add(**next));
                        }
                    }
                } else {
                    return vm.error(ProgramErrorKind::TypeError(ObjectKind::Iterator, *kind));
                }
                Ok(())
            },
            Operation::IterSkip => {
                // TODO
                Ok(())
            }
            Operation::IterCurrent => unsafe {
                let Object { kind, data } = {
                    match { vm.obj_stack.pop_mut() } {
                        Ok(&mut t) => Ok(t),
                        Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                    }
                }?;

                if let ObjectData::Iterator(_list_ptr, next) = data {
                    let val = if **next == 0 { 0 } else { (**next) - 1 };
                    let obj = Object {
                        kind: ObjectKind::Integer,
                        data: ObjectData::Integer(val as isize),
                    };
                    let obj: &'static Object = vm.register_single(obj);
                    vm.obj_stack.push(obj);
                } else {
                    return vm.error(ProgramErrorKind::TypeError(ObjectKind::Iterator, *kind));
                }
                Ok(())
            },
            Operation::Iterate => unsafe {
                let Object { kind, data } = {
                    match { vm.obj_stack.pop_mut() } {
                        Ok(&mut t) => Ok(t),
                        Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                    }
                }?;
                if let ObjectData::Iterator(list_ptr, next) = data {
                    let list = **list_ptr;
                    if let ObjectData::List(start, len) = list {
                        if *len != 0 && **next < *len {
                            let start = *start as *const Object;
                            let pc = vm.counter.clone();
                            let last_frame = match vm.call_stack.last() {
                                Ok(it) => it,
                                Err(err) => vm.error(err)?,
                            };
                            let mut new_frame = Frame::new(pc, FrameKind::IterateLoop);
                            new_frame.copy_locals(last_frame);
                            vm.call_stack.push(new_frame);

                            for n in (**next)..*len {
                                **next = n + 1;
                                vm.counter = pc;
                                // println!("iterate: {}, pc: {pc}", &*start.add(n));
                                vm.obj_stack.push(&*start.add(n));

                                vm.run_block(FrameKind::IterateLoop);
                            }
                            let _ = vm.call_stack.pop();
                            let done_address = vm.program.get_done(&(pc - 1));
                            match done_address {
                                Ok(addy) => vm.goto(*addy + 1),
                                Err(e) => vm.error(e)?,
                            }
                        } else {
                            let done_address = vm.program.get_done(&(vm.counter - 1));
                            match done_address {
                                Ok(addy) => vm.goto(*addy),
                                Err(e) => vm.error(e)?,
                            }
                        }
                    }
                } else {
                    vm.error(ProgramErrorKind::TypeError(ObjectKind::Iterator, *kind))?
                }
                Ok(())
            },
            Operation::DoIf => {
                let b = {
                    match { vm.obj_stack.pop() } {
                        Ok(t) => Ok(t),
                        Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                    }
                }?;
                assert_eq!(b.kind, ObjectKind::Bool, "Object is not a boolean");
                if let ObjectData::Bool(bol) = b.data {
                    if bol {
                        let frame = {
                            match { vm.call_stack.last() } {
                                Ok(t) => Ok(t),
                                Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                            }
                        }?;
                        let mut new_frame = Frame::new(vm.counter, FrameKind::DoIfBlock);
                        new_frame.copy_locals(frame);
                        vm.call_stack.push(new_frame);
                        vm.run_block(FrameKind::DoIfBlock);
                    } else {
                        let done_address = vm.program.get_done(&(vm.counter - 1));
                        match done_address {
                            Ok(addy) => vm.goto(*addy),
                            Err(e) => vm.error(e)?,
                        }
                    }
                }
                Ok(())
            }
            Operation::Debug => {
                let objs = match unsafe { vm.obj_stack.at_most_n(10) } {
                    Ok(os) => Ok(os),
                    Err(e) => vm.error(e),
                }?;
                let frames = match unsafe { vm.call_stack.at_most_n(10) } {
                    Ok(fs) => Ok(fs),
                    Err(e) => vm.error(e),
                }?;
                println!("object stack: {:?}", objs);
                // println!("call stack: {:?}", frames);
                Ok(())
            }
            Operation::Import(bytes) => {
                // if MODULES.contains(bytes) {
                //     if vm.debug {
                //         println!(
                //             "DEBUG: Using the internal module '{}'",
                //             bytes_to_string(bytes)
                //         );
                //         println!("DEBUG: If this isn't what you meant to do, rename your module");
                //     }
                //     // wait this isnt anything
                //     // ig if i do syscall things
                //     // but
                //     let module: &[Operation] = modules::get_module(bytes);
                //     vm.program.import_module(module);
                // } else {
                // }
                Ok(())
            }
            _ => todo!("{}", self),
        }
    }
}
