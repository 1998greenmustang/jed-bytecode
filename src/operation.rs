use std::{convert::TryInto, fmt::Display};

use crate::{
    binops::BinOpKind,
    builtin::BuiltIn,
    error::{ProgramError, ProgramErrorKind},
    frame::{Frame, FrameKind},
    object::{Object, ObjectData, ObjectKind},
    utils::{self, bytes_to_string},
    vm::VM,
};

#[derive(Copy, Clone, Debug)]
#[repr(usize)]
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
    ListPush(&'static [u8]),
    ListGet(Option<usize>),
    ListSet(Option<usize>),
    PushRange,
    ReturnIfConst(&'static [u8]),
    Empty,
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::BinOp(bin_op_kind) => write!(f, "bin_op {}", bin_op_kind),
            Operation::Call(items) => write!(f, "call {}", bytes_to_string(items)),
            Operation::CallBuiltIn(built_in) => write!(f, "call_builtin {}", built_in),
            Operation::PushLit(items) => write!(f, "push_lit {}", bytes_to_string(items)),
            Operation::PushName(items) => write!(f, "push_name {}", bytes_to_string(items)),
            Operation::PushTemp => write!(f, "push_temp"),
            Operation::Pop => write!(f, "pop"),
            Operation::ReturnIf(items) => write!(f, "return_if {}", bytes_to_string(items)),
            Operation::StoreConst(items) => write!(f, "store_const {}", bytes_to_string(items)),
            Operation::StoreName(items) => write!(f, "store_name {}", bytes_to_string(items)),
            Operation::StoreTemp => write!(f, "store_temp"),
            Operation::Func(items, arity) => write!(f, "func {} {arity}", bytes_to_string(items)),
            Operation::Done => write!(f, "done"),
            Operation::Exit => write!(f, "exit"),
            Operation::DoFor => write!(f, "do_for"),
            Operation::DoForIn(items) => write!(f, "do_for_in {}", bytes_to_string(items)),
            Operation::CreateList(idx) => {
                write!(f, "create_list {}", utils::unwrap_as_string_or(*idx, ""))
            }
            Operation::ListPush(items) => write!(f, "list_push {}", bytes_to_string(items)),
            Operation::ListGet(idx) => {
                write!(f, "list_get {}", utils::unwrap_as_string_or(*idx, ""))
            }
            Operation::ListSet(idx) => {
                write!(f, "list_set {}", utils::unwrap_as_string_or(*idx, ""))
            }
            Operation::PushRange => write!(f, "push_range"),
            Operation::ReturnIfConst(items) => {
                write!(f, "return_if_const {}", bytes_to_string(items))
            }
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
                            let whole: isize = match utils::string_to_t(wholestr.to_owned()) {
                                Ok(v) => v,
                                Err(e) => return vm.error(e),
                            };
                            let prec: usize = match utils::string_to_t(precstr[1..].to_owned()) {
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
                let frame = {
                    match vm.call_stack.last() {
                        Ok(ts) => Ok(ts),
                        Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                    }
                }?;
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
                frame.add_local(*name, {
                    match { vm.obj_stack.pop() } {
                        Ok(t) => Ok(t),
                        Err(_) => Err(ProgramError(
                            ProgramErrorKind::StackError(1),
                            vm.current_span.clone(),
                        )),
                    }
                }?);
                Ok(())
            }
            Operation::StoreTemp => {
                let obj = {
                    match { vm.obj_stack.pop() } {
                        Ok(t) => Ok(t),
                        Err(_) => Err(ProgramError(
                            ProgramErrorKind::StackError(1),
                            vm.current_span.clone(),
                        )),
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
                        Err(_) => Err(ProgramError(
                            ProgramErrorKind::StackError(1),
                            vm.current_span.clone(),
                        )),
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
                    FrameKind::Loop => {
                        let next_frame = vm.call_stack.last_mut_option();
                        if let Some(nxt) = next_frame {
                            if nxt.return_address == frame.return_address {
                                nxt.copy_locals(&frame);
                                vm.counter = frame.return_address;
                            } else {
                                vm.counter = vm.counter + 1;
                                return Ok(());
                            }
                        }
                    }
                    FrameKind::Main => todo!(),
                }
                Ok(())
            }
            Operation::Exit => {
                // println!("i have been called");
                std::process::exit(0);
            }
            Operation::CallBuiltIn(built_in) => {
                let obj = {
                    match { vm.obj_stack.pop() } {
                        Ok(t) => Ok(t),
                        Err(_) => Err(ProgramError(
                            ProgramErrorKind::StackError(1),
                            vm.current_span.clone(),
                        )),
                    }
                }?;
                built_in.call(*obj);
                Ok(())
            }
            Operation::DoFor => {
                // TODO let done work with this
                let object = {
                    match { vm.obj_stack.pop() } {
                        Ok(t) => Ok(t),
                        Err(_) => Err(ProgramError(
                            ProgramErrorKind::StackError(1),
                            vm.current_span.clone(),
                        )),
                    }
                }?;
                if let ObjectData::Integer(times) = object.data {
                    let pc = vm.counter.clone();
                    for _ in 0..times {
                        let mut frame = Frame::new(pc, FrameKind::Loop);
                        frame.copy_locals({
                            match vm.call_stack.last() {
                                Ok(ts) => Ok(ts),
                                Err(_) => Err(ProgramError(
                                    ProgramErrorKind::StackError(1),
                                    vm.current_span.clone(),
                                )),
                            }
                        }?);
                        vm.call_stack.push(frame);
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
                match obj_ptr.as_tuple() {
                    (ObjectKind::List, ObjectData::List(start, len)) => {
                        let pc = vm.counter.clone();
                        for _ in 0..len {
                            let mut frame = Frame::new(pc, FrameKind::Loop);
                            frame.copy_locals({
                                match vm.call_stack.last() {
                                    Ok(ts) => Ok(ts),
                                    Err(_) => vm.error(ProgramErrorKind::StackError(1)),
                                }
                            }?);
                            vm.call_stack.push(frame);
                        }
                    }
                    (kind, _data) => {
                        return vm.error(ProgramErrorKind::TypeError(ObjectKind::List, kind))
                    }
                }
                Ok(())
            }
            Operation::CreateList(maybe_num) => {
                let num = match maybe_num {
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
                let pop_res = unsafe { vm.obj_stack.pop_n(num) };
                let objects: Vec<Object> = match pop_res {
                    Ok(objs) => objs.iter().map(|o| **o).collect(),
                    Err(e) => return Err(ProgramError(e, vm.current_span.clone())),
                };
                let objects: &'static [Object] = vm.register_many(objects.as_slice());
                let obj = Object {
                    kind: ObjectKind::List,
                    data: ObjectData::List(objects.as_ptr(), objects.len()),
                };
                let obj: &'static Object = vm.register_single(obj);
                vm.obj_stack.push(obj);

                Ok(())
            }
            // Operation::ListPush(bytes) => {
            // TODO
            // }
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
                    Err(e) => Err(ProgramError(e, vm.current_span.clone())),
                }?;
                match (list_obj.kind, list_obj.data) {
                    (ObjectKind::List, ObjectData::List(start, len)) => unsafe {
                        if idx < len {
                            let obj_ptr = start.add(idx);
                            vm.obj_stack.push(&*obj_ptr);
                        } else {
                            return vm.error(ProgramErrorKind::ListIndexError(idx, len));
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
                        Err(e) => Err(ProgramError(e, vm.current_span.clone())),
                    }
                }?;
                let list_obj = objects[1];
                let obj = objects[0];

                match (list_obj.kind, list_obj.data) {
                    (ObjectKind::List, ObjectData::List(start, len)) => {
                        if idx < len {
                            let entry = start.add(idx) as *mut Object;
                            entry.copy_from(obj, 1);
                        } else {
                            return vm.error(ProgramErrorKind::ListIndexError(idx, len));
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
                            let values: &'static [Object] = vm.register_many(
                                (s..n)
                                    .step_by(p)
                                    .map(|v| Object {
                                        kind: ObjectKind::Integer,
                                        data: ObjectData::Integer(v),
                                    })
                                    .collect::<Vec<Object>>()
                                    .as_slice(),
                            );
                            values.iter().for_each(|v| vm.obj_stack.push(v));
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
            _ => todo!("{}", self),
        }
    }
}
