use std::{convert::TryInto, fmt::Display};

use jed_macros::{
    IndexFroms, IndexToSnakeCase, SnakeCaseDisplay, SnakeCaseExists, SnakeCaseToIndex,
};

use crate::{
    binops::BinOpKind,
    builtin::BuiltIn,
    error::{ProgramError, ProgramErrorKind},
    frame::{Frame, FrameKind},
    object::{Object, ObjectData, ObjectKind},
    utils::{self, bytes_to_string, display_option_usize},
    vm::VM,
};

#[repr(u8)]
#[derive(
    Copy,
    Clone,
    Debug,
    SnakeCaseDisplay,
    SnakeCaseToIndex,
    IndexToSnakeCase,
    SnakeCaseExists,
    IndexFroms,
)]
pub enum Operation {
    #[jed(type: Option<usize>, func: display_option_usize)]
    #[jed(type: &'static [u8], func: bytes_to_string)]
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
    ListAlloc(Option<usize>),
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

impl Operation {
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
            Operation::CreateList(maybe_num) => unsafe {
                // create an empty list
                let len = Box::new(0);
                let random_addr = Box::new(vm.memory.start().addr());
                let obj = Object {
                    kind: ObjectKind::List,
                    data: ObjectData::List(Box::into_raw(random_addr), Box::into_raw(len)),
                };
                let obj: &'static mut Object = vm.register_single_mut(obj);

                let num = match maybe_num {
                    Some(v) => *v,
                    None => vm.obj_stack.len(),
                };
                let pop_res = vm.obj_stack.pop_n(num);
                let objects: Vec<Object> = match pop_res {
                    Ok(objs) => objs.iter().map(|o| **o).collect(),
                    Err(e) => return vm.error(e),
                };
                if objects.len() > 0 {
                    let objects: &'static [Object] = vm.register_many(objects.as_slice());
                    let obj_ptr = objects.as_ptr();
                    if let ObjectData::List(ref mut start, ref mut len) = obj.data {
                        **len = objects.len();
                        **start = obj_ptr.addr();
                    }
                }
                vm.obj_stack.push(obj);

                Ok(())
            },
            Operation::ListPush => unsafe {
                let new_item = match vm.obj_stack.pop().cloned() {
                    Ok(t) => Ok(t),
                    Err(e) => Err(ProgramError(e, vm.current_span.clone())),
                }?;
                match { vm.obj_stack.pop_mut() } {
                    Ok(mut t) => {
                        let Object { kind, mut data } = &mut t;
                        if let ObjectData::List(ref mut start, ref mut len) = data {
                            let starting_obj = dbg!(**start as *const Object);

                            let pretend_ptr = (**start as *const Object).add(**len).addr();
                            if dbg!(pretend_ptr as *const Object)
                                == dbg!(vm.memory.start().addr() as *const Object)
                            {
                                **len += 1;
                                vm.register_single(new_item);
                            } else {
                                vm.exit(Some(1));
                                println!("new list");
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
