use std::{cell::RefCell, convert::TryInto, fmt::Display, rc::Rc};

use jed_macros::{
    IndexFroms, IndexToSnakeCase, SnakeCaseDisplay, SnakeCaseExists, SnakeCaseToIndex,
};

use crate::{
    binops::BinOpKind,
    builtin::BuiltIn,
    error::{ProgramError, ProgramErrorKind},
    frame::{Frame, FrameKind},
    memory::list::List,
    modules,
    object::{MutableObject, Object, ObjectData, ObjectKind, RegObject},
    unops::UnOpKind,
    utils::{self, display_bytes, display_option_bytes, display_option_usize},
    vm::VM,
};

pub type Block = Rc<List<Operation>>;

#[repr(u8)]
#[derive(
    Clone, Debug, SnakeCaseDisplay, SnakeCaseToIndex, IndexToSnakeCase, SnakeCaseExists, IndexFroms,
)]
pub enum Operation {
    #[jed(type: Option<usize>, func: display_option_usize)]
    #[jed(type: &'static [u8], func: display_bytes)]
    #[jed(type: Option<&'static [u8]>, func: display_option_bytes)]
    BinOp(BinOpKind),
    UnaryOp(UnOpKind),
    Call(Option<&'static [u8]>, Option<&'static [u8]>),
    CallBuiltIn(BuiltIn),
    PushLit(&'static [u8]),
    PushManyLits(&'static [u8], Option<usize>),
    PushName(&'static [u8]),
    PushTemp,
    Pop,
    Dupe,
    Swap,
    ReturnIf(&'static [u8]),
    StoreConst(&'static [u8]),
    StoreName(&'static [u8]),
    StoreTemp,
    Func(&'static [u8], usize, Block),
    Exit,
    DoFor(Block),
    DoForIn(&'static [u8], Block),
    CreateList(Option<usize>),
    ListPush,
    ListGet(Option<usize>),
    ListSet(Option<usize>),
    ListAlloc(Option<usize>),
    ListFill(usize, &'static [u8]),
    PushRange,
    RangeLoop(Block),
    ReturnIfConst(&'static [u8]),
    GetPtr,
    ReadPtr,
    SetPtr,
    GetIter,
    IterNext,
    IterPrev,
    IterSkip,
    IterCurrent,
    Iterate(Block),
    DoIf(Block),
    Debug,
    Import(&'static [u8]),
    Empty,
}

pub enum Response {
    Ok,
    BlockReturn,
    Error(ProgramErrorKind),
    Exit(i32),
    IterationDone,
}

impl Operation {
    pub fn call(&self, vm: &mut VM) -> Response {
        match self {
            Operation::BinOp(bin_op_kind) => vm.handle_bin_op(*bin_op_kind),
            Operation::Call(maybe_first, maybe_second) => match (maybe_first, maybe_second) {
                (None, Some(func)) => vm.call(func),
                (Some(module), Some(func)) => match *module {
                    b"math" => {
                        use modules::math as lib;
                        todo!()
                    }
                    _ => todo!(),
                },
                (None, None) => {
                    let obj = match vm.obj_stack.pop() {
                        Ok(o) => o,
                        Err(e) => return Response::Error(e),
                    };
                    if let ObjectData::Func(name) = obj.data {
                        return vm.call(name);
                    } else {
                        return Response::Error(ProgramErrorKind::TypeError(
                            ObjectKind::Func,
                            obj.kind,
                        ));
                    }
                }
                (Some(_), None) => unreachable!(),
            },
            Operation::PushLit(literal) => match vm.parse_lit(literal) {
                Ok(obj) => {
                    vm.obj_stack.push(obj);
                    vm.store_const(literal, *obj);
                    Response::Ok
                }
                Err(e) => Response::Error(e),
            },
            Operation::PushName(name) => match vm.call_stack.last() {
                Ok(frame) => match frame.get_local(name) {
                    Some(v) => {
                        vm.obj_stack.push(v);
                        Response::Ok
                    }
                    None => Response::Error(ProgramErrorKind::VariableExists(name)),
                },
                Err(_) => return Response::Error(ProgramErrorKind::StackError(1)),
            },
            Operation::PushTemp => {
                if let Some(tmp) = vm.temp {
                    vm.obj_stack.push(tmp);
                    return Response::Ok;
                } else {
                    return Response::Error(ProgramErrorKind::TempPush);
                }
            }
            Operation::Pop => match { vm.obj_stack.pop() } {
                Ok(t) => Response::Ok,
                Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
            },
            Operation::ReturnIf(name) => {
                let b = {
                    match { vm.obj_stack.pop() } {
                        Ok(t) => t,
                        Err(_) => return Response::Error(ProgramErrorKind::StackError(1)),
                    }
                };
                assert_eq!(b.kind, ObjectKind::Bool, "Object is not a boolean");
                match b.data {
                    ObjectData::Bool(bol) => {
                        if bol {
                            let frame = {
                                match { vm.call_stack.last() } {
                                    Ok(t) => t,
                                    Err(_) => {
                                        return Response::Error(ProgramErrorKind::StackError(1));
                                    }
                                }
                            };
                            if let Some(obj) = frame.get_local(name) {
                                vm.obj_stack.push(obj);
                                vm.program.set_memo(frame.memo_key, *obj);
                                return Response::BlockReturn;
                            } else {
                                return Response::Error(ProgramErrorKind::VariableExists(name));
                            }
                        }
                        Response::Ok
                    }
                    _ => Response::Error(ProgramErrorKind::TypeError(ObjectKind::Bool, b.kind)),
                }
            }
            Operation::StoreConst(name) => match { vm.obj_stack.pop() } {
                Ok(t) => {
                    vm.store_const(*name, *t);
                    return Response::Ok;
                }
                Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
            },
            Operation::StoreName(name) => match vm.call_stack.last_mut() {
                Ok(frame) => {
                    match vm.obj_stack.pop() {
                        Ok(obj) => {
                            frame.add_local(*name, obj);
                            return Response::Ok;
                        }
                        Err(e) => return Response::Error(e),
                    };
                }
                Err(_) => unreachable!("There should never be an empty call stack"),
            },
            Operation::StoreTemp => match { vm.obj_stack.pop() } {
                Ok(obj) => {
                    vm.temp = Some(obj);
                    Response::Ok
                }
                Err(e) => Response::Error(e),
            },
            Operation::Func(_name, _arity, _block) => Response::Ok,

            Operation::Exit => Response::Exit(0),
            Operation::CallBuiltIn(built_in) => match { vm.obj_stack.pop() } {
                Ok(obj) => {
                    if let Some(val) = built_in.call(*obj) {
                        let val = vm.register_single(val);
                        vm.obj_stack.push(val);
                    };
                    Response::Ok
                }
                Err(e) => Response::Error(e),
            },
            Operation::DoFor(block) => match { vm.obj_stack.pop() } {
                Ok(obj) => {
                    if let ObjectData::Integer(times) = obj.data {
                        let pc = vm.counter.clone();
                        match vm.call_stack.last() {
                            Ok(last_frame) => {
                                let mut new_frame = Frame::new(pc, FrameKind::DoForLoop);
                                new_frame.copy_locals(last_frame);
                                vm.call_stack.push(new_frame.clone());
                                let block = Rc::clone(block);
                                for _ in 0..times {
                                    vm.run_block(&block);
                                }
                                let _ = vm.call_stack.pop();
                            }
                            Err(err) => unreachable!("There should never be an empty call stack"),
                        };
                    }
                    Response::Ok
                }
                Err(e) => Response::Error(e),
            },
            Operation::DoForIn(obj_name, block) => {
                let current_frame = {
                    match vm.call_stack.last() {
                        Ok(ts) => ts,
                        Err(_) => unreachable!("There should never be an empty call stack"),
                    }
                };
                let maybe_obj_ptr = current_frame.get_local(obj_name);
                let obj_ptr = maybe_obj_ptr
                    .unwrap_or_else(|| panic!("No such name '{}'", utils::display_bytes(obj_name)));
                let pc = vm.counter.clone();
                let mut new_frame = Frame::new(pc, FrameKind::DoForInLoop);
                new_frame.copy_locals(current_frame);
                vm.call_stack.push(new_frame);
                match obj_ptr.as_tuple() {
                    (ObjectKind::List, ObjectData::List(list)) => unsafe {
                        for _ in 0..(*list).borrow().len() {
                            vm.run_block(&Rc::clone(block));
                        }
                    },
                    (ObjectKind::Iterator, ObjectData::Iterator(list_ptr, _next)) => unsafe {
                        let list = *list_ptr;
                        if let ObjectData::List(list) = list {
                            for _ in 0..(*list).borrow().len() {
                                vm.counter = pc;
                                vm.run_block(&Rc::clone(block));
                            }
                        }
                    },

                    (kind, _data) => {
                        return Response::Error(ProgramErrorKind::TypeError(
                            ObjectKind::List,
                            kind,
                        ));
                    }
                }
                let _ = vm.call_stack.pop();
                return Response::Ok;
            }
            Operation::CreateList(maybe_num) => unsafe {
                // create an empty list
                let mut list = List::new();

                let num = match maybe_num {
                    Some(v) => *v,
                    None => vm.obj_stack.len(),
                };
                let pop_res = vm.obj_stack.pop_n(num);
                match pop_res {
                    Ok(objs) => objs.iter().map(|o| list.push(*o)).collect::<()>(),
                    Err(e) => return Response::Error(e),
                };
                let ptr = Box::new(Rc::new(RefCell::new(list)));
                let obj = Object {
                    kind: ObjectKind::List,
                    data: ObjectData::List(Box::into_raw(ptr)),
                };
                let obj: MutableObject = vm.register_single_mut(obj);
                vm.obj_stack.push(obj);

                Response::Ok
            },
            Operation::ListPush => unsafe {
                match vm.obj_stack.pop() {
                    Ok(new_item) => match { vm.obj_stack.pop_mut() } {
                        Ok(&mut &Object { ref kind, data }) => {
                            if let ObjectData::List(list) = data {
                                (*list).borrow_mut().push(&new_item);
                                return Response::Ok;
                            } else {
                                return Response::Error(ProgramErrorKind::TypeError(
                                    ObjectKind::List,
                                    *kind,
                                ));
                            }
                        }
                        Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
                    },
                    Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
                }
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
                                    return Response::Error(ProgramErrorKind::TypeError(
                                        ObjectKind::Integer,
                                        kind,
                                    ));
                                }
                            },

                            Err(e) => return Response::Error(e),
                        }
                    }
                };
                match { vm.obj_stack.pop() } {
                    Ok(list_obj) => match (list_obj.kind, list_obj.data) {
                        (ObjectKind::List, ObjectData::List(list)) => unsafe {
                            if let Some(obj) = (*list).borrow().get(idx) {
                                vm.obj_stack.push(obj);
                                Response::Ok
                            } else {
                                Response::Error(ProgramErrorKind::ListIndexError(
                                    idx,
                                    (*list).borrow().len(),
                                ))
                            }
                        },
                        (kind, _data) => {
                            Response::Error(ProgramErrorKind::TypeError(ObjectKind::List, kind))
                        }
                    },
                    Err(e) => Response::Error(e),
                }
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
                                    return Response::Error(ProgramErrorKind::TypeError(
                                        ObjectKind::Integer,
                                        kind,
                                    ));
                                }
                            },

                            Err(e) => return Response::Error(e),
                        }
                    }
                };
                match { vm.obj_stack.pop_n(2) } {
                    Ok(objects) => {
                        let list_obj = objects[1];
                        let obj = objects[0];

                        match (list_obj.kind, list_obj.data) {
                            (ObjectKind::List, ObjectData::List(list)) => {
                                if idx < (*list).borrow().len() {
                                    (*list).borrow_mut().insert(idx, &obj);
                                    Response::Ok
                                } else {
                                    Response::Error(ProgramErrorKind::ListIndexError(
                                        idx,
                                        (*list).borrow().len(),
                                    ))
                                }
                            }
                            (kind, _) => {
                                Response::Error(ProgramErrorKind::TypeError(ObjectKind::List, kind))
                            }
                        }
                    }
                    Err(e) => Response::Error(e),
                }
            },
            Operation::ListAlloc(maybe_num) => unsafe {
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
                                    return Response::Error(ProgramErrorKind::TypeError(
                                        ObjectKind::Integer,
                                        kind,
                                    ));
                                }
                            },

                            Err(e) => return Response::Error(e),
                        }
                    }
                };
                match { vm.obj_stack.pop_mut() } {
                    Ok(&mut &Object { ref kind, mut data }) => {
                        if let ObjectData::List(list) = data {
                            (*list).borrow_mut().alloc(num);
                            Response::Ok
                        } else {
                            Response::Error(ProgramErrorKind::TypeError(ObjectKind::List, *kind))
                        }
                    }
                    Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
                }
            },
            Operation::PushRange => {
                let (steps, end, start) = match unsafe { vm.obj_stack.pop_n(3) } {
                    Ok(xs) => (xs[2], xs[1], xs[0]),
                    Err(e) => return Response::Error(e),
                };
                match (start.data, end.data, steps.data) {
                    (ObjectData::Integer(s), ObjectData::Integer(n), ObjectData::Integer(ps)) => {
                        let p: usize = match ps.try_into() {
                            Ok(v) => v,
                            Err(_) => return Response::Error(ProgramErrorKind::IntegerToUnsigned),
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
                        return Response::Ok;
                    }
                    _ => todo!(),
                };
            }
            Operation::ReturnIfConst(name) => {
                let b = {
                    match { vm.obj_stack.pop() } {
                        Ok(t) => t,
                        Err(_) => return Response::Error(ProgramErrorKind::StackError(1)),
                    }
                };
                assert_eq!(b.kind, ObjectKind::Bool, "Object is not a boolean");
                match b.data {
                    ObjectData::Bool(bol) => {
                        if bol {
                            if let Some(obj) = vm.get_const(name) {
                                vm.obj_stack.push(obj);
                                return Response::BlockReturn;
                            } else {
                                return Response::Error(ProgramErrorKind::VariableExists(name));
                            }
                        }
                        Response::Ok
                    }
                    _ => Response::Error(ProgramErrorKind::TypeError(ObjectKind::Bool, b.kind)),
                }
            }
            Operation::GetPtr => match { vm.obj_stack.pop() } {
                Ok(obj) => {
                    let ptr_obj = {
                        Object {
                            kind: ObjectKind::Pointer,
                            data: ObjectData::Pointer(&mut &*obj as *mut &Object),
                        }
                    };
                    let ptr_obj: RegObject = vm.register_single(ptr_obj);
                    vm.obj_stack.push(ptr_obj);
                    Response::Ok
                }
                Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
            },
            Operation::ReadPtr => match { vm.obj_stack.pop() } {
                Ok(ptr_obj) => {
                    if let ObjectData::Pointer(real_ptr) = ptr_obj.data {
                        let val: RegObject = unsafe { real_ptr.read() };
                        vm.obj_stack.push(val);
                        Response::Ok
                    } else {
                        Response::Error(ProgramErrorKind::TypeError(
                            ObjectKind::Pointer,
                            ptr_obj.kind,
                        ))
                    }
                }
                Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
            },
            Operation::SetPtr => unsafe {
                match { vm.obj_stack.pop_n(2) } {
                    Ok(objects) => {
                        let obj = objects.get_unchecked(0);
                        let ptr_obj = objects.get_unchecked(1);

                        if let ObjectData::Pointer(real_ptr) = ptr_obj.data {
                            real_ptr.copy_from(obj, 1);
                            Response::Ok
                        } else {
                            Response::Error(ProgramErrorKind::TypeError(
                                ObjectKind::Pointer,
                                ptr_obj.kind,
                            ))
                        }
                    }
                    Err(_) => Response::Error(ProgramErrorKind::StackError(2)),
                }
            },
            Operation::GetIter => match { vm.obj_stack.pop() } {
                Ok(list_obj) => {
                    if let ObjectKind::List = list_obj.kind {
                        let initial_index = Box::new(0);
                        let iter_obj = Object {
                            kind: ObjectKind::Iterator,
                            data: ObjectData::Iterator(
                                &list_obj.data as *const ObjectData as *mut ObjectData,
                                Box::into_raw(initial_index),
                            ),
                        };
                        let iter_obj: RegObject = vm.register_single(iter_obj);
                        vm.obj_stack.push(iter_obj);
                        Response::Ok
                    } else {
                        Response::Error(ProgramErrorKind::TypeError(
                            ObjectKind::List,
                            list_obj.kind,
                        ))
                    }
                }
                Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
            },
            Operation::IterNext => unsafe {
                match { vm.obj_stack.pop_mut() } {
                    Ok(&mut &Object { kind, mut data }) => {
                        if let ObjectData::Iterator(list_ptr, ref mut next) = data {
                            let list = *list_ptr;
                            if let ObjectData::List(list) = list {
                                if **next < (*list).borrow().len() {
                                    vm.obj_stack.push((*list).borrow().get_unchecked(**next));
                                    **next += 1;
                                    Response::Ok
                                } else {
                                    Response::Error(ProgramErrorKind::IterNext(
                                        (*list).borrow().len(),
                                    ))
                                }
                            } else {
                                unreachable!();
                            }
                        } else {
                            Response::Error(ProgramErrorKind::TypeError(ObjectKind::Iterator, kind))
                        }
                    }
                    Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
                }
            },
            Operation::IterPrev => unsafe {
                match { vm.obj_stack.pop_mut() } {
                    Ok(&mut &Object { kind, mut data }) => {
                        if let ObjectData::Iterator(list_ptr, ref mut next) = data {
                            let list = *list_ptr;
                            if let ObjectData::List(list) = list {
                                let cur_val = **next;
                                if cur_val == 0 {
                                    Response::Error(ProgramErrorKind::IterPrevious)
                                } else {
                                    **next -= 1;
                                    vm.obj_stack.push((*list).borrow().get_unchecked(**next));
                                    Response::Ok
                                }
                            } else {
                                unreachable!();
                            }
                        } else {
                            return Response::Error(ProgramErrorKind::TypeError(
                                ObjectKind::Iterator,
                                kind,
                            ));
                        }
                    }
                    Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
                }
            },
            Operation::IterSkip => {
                // TODO
                Response::Ok
            }
            Operation::IterCurrent => unsafe {
                match { vm.obj_stack.pop_mut() } {
                    Ok(&mut Object { kind, data }) => {
                        if let ObjectData::Iterator(_list_ptr, next) = data {
                            let val = if **next != 0 {
                                (**next) - 1
                            } else {
                                return Response::Error(ProgramErrorKind::TodoError);
                            };
                            let obj = Object {
                                kind: ObjectKind::Integer,
                                data: ObjectData::Integer(val as isize),
                            };
                            let obj: RegObject = vm.register_single(obj);
                            vm.obj_stack.push(obj);
                            Response::Ok
                        } else {
                            Response::Error(ProgramErrorKind::TypeError(
                                ObjectKind::Iterator,
                                *kind,
                            ))
                        }
                    }
                    Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
                }
            },
            Operation::Iterate(block) => unsafe {
                match { vm.obj_stack.pop_mut() } {
                    Ok(&mut Object { kind, data }) => {
                        if let ObjectData::Iterator(list_ptr, next) = data {
                            let list = **list_ptr;
                            if let ObjectData::List(list) = list {
                                let len = (*list).borrow().len();
                                if len != 0 && **next < len {
                                    let last_frame = match vm.call_stack.last() {
                                        Ok(it) => it,
                                        Err(err) => return Response::Error(err),
                                    };
                                    let mut new_frame =
                                        Frame::new(vm.counter, FrameKind::IterateLoop);
                                    new_frame.copy_locals(last_frame);
                                    vm.call_stack.push(new_frame);

                                    for n in (**next)..len {
                                        **next = n + 1;
                                        vm.obj_stack.push((*list).borrow().get_unchecked(n));

                                        vm.run_block(&Rc::clone(block));
                                    }
                                    // let _ = vm.call_stack.pop();
                                }
                                return Response::IterationDone;
                            } else {
                                unreachable!()
                            }
                        } else {
                            Response::Error(ProgramErrorKind::TypeError(
                                ObjectKind::Iterator,
                                *kind,
                            ))
                        }
                    }
                    Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
                }
            },
            Operation::DoIf(block) => match { vm.obj_stack.pop() } {
                Ok(b) => {
                    if let ObjectData::Bool(bol) = b.data {
                        if bol {
                            let frame = {
                                match { vm.call_stack.last() } {
                                    Ok(t) => t,
                                    Err(_) => {
                                        unreachable!()
                                    }
                                }
                            };
                            let mut new_frame = Frame::new(vm.counter, FrameKind::DoIfBlock);
                            new_frame.copy_locals(frame);
                            vm.call_stack.push(new_frame);
                            vm.run_block(&Rc::clone(block));
                        }
                        return Response::Ok;
                    } else {
                        return Response::Error(ProgramErrorKind::TypeError(
                            ObjectKind::Bool,
                            b.kind,
                        ));
                    }
                }
                Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
            },
            Operation::Debug => {
                // let objs = match unsafe { vm.obj_stack.at_most_n(10) } {
                //     Ok(os) => Ok(os),
                //     Err(e) => vm.error(e),
                // }?;
                // let frames = match unsafe { vm.call_stack.at_most_n(10) } {
                //     Ok(fs) => Ok(fs),
                //     Err(e) => vm.error(e),
                // }?;
                // println!("object stack: {:?}", objs);
                // println!("call stack: {:?}", frames);
                Response::Ok
            }
            // Operation::Import(bytes) => {
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
            //     Ok(())
            // }
            Operation::Dupe => match vm.obj_stack.last() {
                Ok(o) => {
                    vm.obj_stack.push(o);
                    Response::Ok
                }
                Err(e) => Response::Error(e),
            },
            Operation::Swap => {
                if vm.obj_stack.len() > 2 {
                    vm.obj_stack.swap();
                    Response::Ok
                } else {
                    Response::Error(ProgramErrorKind::StackError(2))
                }
            }
            Operation::ListFill(_, items) => todo!(),
            Operation::RangeLoop(block) => {
                let (steps, end, start) = match { vm.obj_stack.pop_n(3) } {
                    Ok(xs) => (xs[2], xs[1], xs[0]),
                    Err(e) => return Response::Error(e),
                };
                match (start.data, end.data, steps.data) {
                    (ObjectData::Integer(s), ObjectData::Integer(n), ObjectData::Integer(ps)) => {
                        let p: usize = match ps.try_into() {
                            Ok(v) => v,
                            Err(_) => return Response::Error(ProgramErrorKind::IntegerToUnsigned),
                        };
                        let values = (s..n).step_by(p).map(|v| Object {
                            kind: ObjectKind::Integer,
                            data: ObjectData::Integer(v),
                        });
                        let last_frame = vm.call_stack.last().unwrap();
                        let mut new_frame = Frame::new(vm.counter, FrameKind::RangeLoop);
                        new_frame.copy_locals(last_frame);
                        vm.call_stack.push(new_frame);

                        for i in values {
                            let i = vm.register_single(i);
                            vm.obj_stack.push(i);
                            vm.run_block(&Rc::clone(block));
                        }
                        return Response::IterationDone;
                    }
                    _ => todo!(),
                }
            }
            Operation::PushManyLits(lit, maybe_n) => {
                let n = match maybe_n {
                    Some(n) => *n,
                    None => match vm.obj_stack.pop() {
                        Ok(o) => match o.data.try_into() {
                            Ok(n) => n,
                            Err(e) => return Response::Error(e),
                        },
                        Err(e) => return Response::Error(e),
                    },
                };
                let lit = vm.parse_lit(lit);
                match lit {
                    Ok(lit) => {
                        for _ in 0..n {
                            vm.obj_stack.push(&lit);
                        }
                        Response::Ok
                    }
                    Err(e) => Response::Error(e),
                }
            }
            _ => todo!("{}", self),
        }
    }
}
