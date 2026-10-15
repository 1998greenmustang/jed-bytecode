use std::{convert::TryInto, fmt::Display, mem::ManuallyDrop};

use jed_macros::{
    IndexFroms, IndexToSnakeCase, SnakeCaseDisplay, SnakeCaseExists, SnakeCaseToIndex,
};

use crate::{
    error::ProgramErrorKind::{self, TodoError, TypeError},
    memory::list::List,
    modules::{
        MODULES,
        jed::{self, BinOpKind},
    },
    object::{Object, ObjectKind},
    utils::{display_bytes, display_option_bytes, display_option_usize},
    vm::VM,
};

// Operations, nlocals
pub type Block = (ManuallyDrop<List<Operation>>, usize);

// add a label and jump
// what if theres a conditional jump, but it uses temp :thinking:
#[repr(u8)]
#[derive(
    Clone, Debug, SnakeCaseDisplay, SnakeCaseToIndex, IndexToSnakeCase, SnakeCaseExists, IndexFroms,
)]
pub enum Operation {
    #[jed(type: Option<usize>, func: display_option_usize)]
    #[jed(type: &'static [u8], func: display_bytes)]
    #[jed(type: Option<&'static [u8]>, func: display_option_bytes)]
    #[jed(type: fn(&[Object]) -> Response, func: (|_| "external function"))]
    #[jed(type: Block, func: (|b: &Block| format!("{}", *b.0)))]
    Call(Option<&'static [u8]>),
    PushConst(usize),
    PushManyConst(usize, Option<usize>),
    Push(usize),
    PushTemp,
    Pop,
    Dupe,
    Swap,
    ReturnIf(usize),
    Store(usize),
    StoreTemp,
    Func(&'static [u8], usize, Block),
    Exit,
    DoFor(Block),
    DoForIn(usize, Block),
    CreateList(Option<usize>),
    ListPush,
    ListGet(Option<usize>),
    ListSet(Option<usize>),
    ListAlloc(Option<usize>),
    ListFill(usize, usize),
    PushRange,
    RangeLoop(Block),
    GetPtr,
    ReadPtr,
    SetPtr,
    CreateIter,
    IterNext,
    IterPrev,
    IterSkip,
    IterCurrent,
    Iterate(Block),
    DoIf(Block),
    Debug,
    Import(&'static [u8]),
    Empty,
    ExternFunc(&'static [u8], usize, fn(&[Object]) -> Response),
    Object(&'static [u8], usize, Block),
    CreateObject(&'static [u8]),
    GetAttribute(&'static [u8]),
    SetAttribute(&'static [u8]),
    Consts,
    BinOp(BinOpKind),
    Name,
    Return, // essentially just a label, just here for macros
}

pub enum Response {
    Ok,
    BlockReturn,
    Error(ProgramErrorKind),
    Exit(i32),
    IterationDone,
    ExternReturn(Option<Object>),
    FunctionReturn(Option<Object>),
}

impl Operation {
    pub fn call(&self, vm: &mut VM) -> Response {
        // println!("{self}");
        match self {
            Operation::Call(_) => self.op_call(vm),
            Operation::PushConst(_) => self.op_push_lit(vm),
            Operation::Push(_) => self.op_push(vm),
            Operation::PushTemp => self.op_push_temp(vm),
            Operation::Pop => self.op_pop(vm),
            Operation::Store(_) => self.op_store(vm),
            Operation::StoreTemp => self.op_store_temp(vm),
            Operation::Func(_, _, _) => self.op_func(vm),
            Operation::Exit => self.op_exit(vm),
            Operation::DoFor(_) => self.op_do_for(vm),
            Operation::DoForIn(_, _) => self.op_do_for_in(vm),
            Operation::CreateList(_) => self.op_create_list(vm),
            Operation::ListPush => self.op_list_push(vm),
            Operation::ListGet(_) => self.op_list_get(vm),
            Operation::ListSet(_) => self.op_list_set(vm),
            Operation::ListAlloc(_) => self.op_list_alloc(vm),
            Operation::PushRange => self.op_push_range(vm),
            Operation::GetPtr => self.op_get_ptr(vm),
            Operation::ReadPtr => self.op_read_ptr(vm),
            Operation::SetPtr => self.op_set_ptr(vm),
            Operation::CreateIter => self.op_create_iter(vm),
            Operation::IterCurrent => self.op_iter_current(vm),
            Operation::Iterate(_) => self.op_iterate(vm),
            Operation::DoIf(_) => self.op_do_if(vm),
            Operation::Debug => self.op_debug(vm),
            Operation::Import(_) => self.op_import(vm),
            Operation::Dupe => self.op_dupe(vm),
            Operation::Swap => self.op_swap(vm),
            Operation::ReturnIf(_) => self.op_return_if(vm),
            Operation::Return => self.op_return(vm),
            Operation::ListFill(_, _) => self.op_list_fill(vm),
            Operation::RangeLoop(_) => self.op_range_loop(vm),
            Operation::PushManyConst(_, _) => self.op_push_many_const(vm),
            Operation::Object(_, _, _) => self.op_object(vm),
            Operation::CreateObject(_) => self.op_create_object(vm),
            Operation::SetAttribute(_) => self.op_set_attribute(vm),
            Operation::GetAttribute(_) => self.op_get_attribute(vm),
            Operation::Consts => Response::Ok,
            Operation::Name => Response::Ok,
            Operation::BinOp(_) => self.op_bin_op(vm),
            _ => todo!("{}", self),
        }
    }

    #[inline(never)]
    fn op_call(&self, vm: &mut VM) -> Response {
        match self {
            Operation::Call(maybe) => match maybe {
                Some(func) => vm.call(func),
                None => {
                    let obj = match vm.obj_stack.pop() {
                        Ok(o) => o,
                        Err(e) => return Response::Error(e),
                    };
                    if obj.is(ObjectKind::Func) {
                        return vm.call(obj.func());
                    } else {
                        return Response::Error(ProgramErrorKind::TypeError(
                            ObjectKind::Func,
                            obj.kind,
                        ));
                    }
                }
            },
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_push_lit(&self, vm: &mut VM) -> Response {
        match self {
            Operation::PushConst(idx) => match vm.program.get_const(*idx) {
                Ok(obj) => {
                    vm.obj_stack.push(obj);
                    Response::Ok
                }
                Err(e) => Response::Error(e),
            },
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_push(&self, vm: &mut VM) -> Response {
        match self {
            Operation::Push(name) => {
                let obj = vm.get_local(*name);
                vm.obj_stack.push(obj);
                Response::Ok
            }
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_push_temp(&self, vm: &mut VM) -> Response {
        match self {
            Operation::PushTemp => {
                if let Some(tmp) = vm.temp {
                    vm.obj_stack.push(tmp);
                    return Response::Ok;
                } else {
                    return Response::Error(ProgramErrorKind::TempPush);
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_pop(&self, vm: &mut VM) -> Response {
        match self {
            Operation::Pop => match { vm.obj_stack.pop() } {
                Ok(_) => Response::Ok,
                Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
            },
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_store(&self, vm: &mut VM) -> Response {
        match self {
            Operation::Store(name) => match vm.obj_stack.pop() {
                Ok(obj) => {
                    vm.set_local(*name, obj);
                    return Response::Ok;
                }
                Err(e) => return Response::Error(e),
            },
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_store_temp(&self, vm: &mut VM) -> Response {
        match self {
            Operation::StoreTemp => match { vm.obj_stack.pop() } {
                Ok(obj) => {
                    vm.temp = Some(obj);
                    Response::Ok
                }
                Err(e) => Response::Error(e),
            },
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_func(&self, _vm: &mut VM) -> Response {
        match self {
            Operation::Func(_name, _arity, _block) => Response::Ok,
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_exit(&self, _vm: &mut VM) -> Response {
        match self {
            Operation::Exit => Response::Exit(0),
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_do_for(&self, vm: &mut VM) -> Response {
        match self {
            Operation::DoFor(block) => match { vm.obj_stack.pop() } {
                Ok(obj) => {
                    if obj.is(ObjectKind::Integer) {
                        let times = obj.integer();
                        for _ in 0..times {
                            vm.run_block(&block.0, block.1);
                        }
                    }
                    Response::Ok
                }
                Err(e) => Response::Error(e),
            },
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_do_for_in(&self, vm: &mut VM) -> Response {
        match self {
            Operation::DoForIn(obj_name, block) => {
                let obj = vm.get_local(*obj_name).clone();
                if obj.is(ObjectKind::List) {
                    unsafe {
                        for _ in 0..(*obj.list()).len() {
                            vm.run_block(&block.0, block.1);
                        }
                    }
                } else if obj.is(ObjectKind::Iterator) {
                    while let Some(_obj) = (unsafe { *obj.iterator() }).next() {
                        // vm.counter = pc;
                        vm.run_block(&block.0, block.1);
                    }
                } else {
                    return Response::Error(ProgramErrorKind::TypeError(
                        ObjectKind::List,
                        obj.kind,
                    ));
                }
                let _ = vm.call_stack.pop();
                return Response::Ok;
            }
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_create_list(&self, vm: &mut VM) -> Response {
        match self {
            Operation::CreateList(maybe_num) => {
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
                let obj = Object::new_list(Box::into_raw(Box::new(ManuallyDrop::new(list))));
                vm.obj_stack.push(obj);

                Response::Ok
            }
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_list_push(&self, vm: &mut VM) -> Response {
        match self {
            Operation::ListPush => unsafe {
                match vm.obj_stack.pop() {
                    Ok(new_item) => match { vm.obj_stack.pop_mut() } {
                        Ok(obj) => {
                            if obj.is(ObjectKind::List) {
                                (*obj.list()).push(new_item);
                                return Response::Ok;
                            } else {
                                return Response::Error(ProgramErrorKind::TypeError(
                                    ObjectKind::List,
                                    obj.kind,
                                ));
                            }
                        }
                        Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
                    },
                    Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
                }
            },
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_list_get(&self, vm: &mut VM) -> Response {
        match self {
            Operation::ListGet(maybe_idx) => {
                let idx = match maybe_idx {
                    Some(v) => *v,
                    None => {
                        let obj = vm.obj_stack.pop();
                        match obj {
                            Ok(v) if v.is_usizeable() => v.usize(),
                            Ok(v) => {
                                return Response::Error(ProgramErrorKind::TypeError(
                                    ObjectKind::Integer,
                                    v.kind,
                                ));
                            }
                            Err(e) => return Response::Error(e),
                        }
                    }
                };
                match { vm.obj_stack.pop() } {
                    Ok(object) if object.is(ObjectKind::List) => unsafe {
                        if let Some(obj) = (*object.list()).get(idx) {
                            vm.obj_stack.push(*obj);
                            Response::Ok
                        } else {
                            Response::Error(ProgramErrorKind::ListIndexError(
                                idx,
                                (*object.data.list).len(),
                            ))
                        }
                    },
                    Ok(object) => {
                        Response::Error(ProgramErrorKind::TypeError(ObjectKind::List, object.kind))
                    }
                    Err(e) => Response::Error(e),
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_list_set(&self, vm: &mut VM) -> Response {
        match self {
            Operation::ListSet(maybe_idx) => unsafe {
                let idx = match maybe_idx {
                    Some(v) => *v,
                    None => {
                        let obj = vm.obj_stack.pop();
                        match obj {
                            Ok(obj) if obj.is_usizeable() => obj.usize(),
                            Ok(obj) => {
                                return Response::Error(ProgramErrorKind::TypeError(
                                    ObjectKind::Integer,
                                    obj.kind,
                                ));
                            }

                            Err(e) => return Response::Error(e),
                        }
                    }
                };
                match { vm.obj_stack.pop_n(2) } {
                    Ok(objects) => {
                        let list = objects[1];
                        let obj = objects[0];

                        if list.is(ObjectKind::List) {
                            if idx < (*list.list()).len() {
                                (*list.list()).insert(idx, obj);
                                Response::Ok
                            } else {
                                Response::Error(ProgramErrorKind::ListIndexError(
                                    idx,
                                    (*list.list()).len(),
                                ))
                            }
                        } else {
                            Response::Error(ProgramErrorKind::TypeError(
                                ObjectKind::List,
                                list.kind,
                            ))
                        }
                    }
                    Err(e) => Response::Error(e),
                }
            },
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_list_alloc(&self, vm: &mut VM) -> Response {
        match self {
            Operation::ListAlloc(maybe_num) => unsafe {
                let num = match maybe_num {
                    Some(v) => *v,
                    None => {
                        let obj = vm.obj_stack.pop();
                        match obj {
                            Ok(v) if v.is_usizeable() => v.usize(),
                            Ok(v) => {
                                return Response::Error(ProgramErrorKind::TypeError(
                                    ObjectKind::Integer,
                                    v.kind,
                                ));
                            }
                            Err(e) => return Response::Error(e),
                        }
                    }
                };
                match { vm.obj_stack.pop_mut() } {
                    Ok(&mut Object { kind, data }) => {
                        if kind == ObjectKind::List
                            && let list = data.list
                        {
                            (*list).alloc(num);
                            Response::Ok
                        } else {
                            Response::Error(ProgramErrorKind::TypeError(ObjectKind::List, kind))
                        }
                    }
                    Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
                }
            },
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_push_range(&self, vm: &mut VM) -> Response {
        match self {
            Operation::PushRange => {
                let (steps, end, start) = match { vm.obj_stack.pop_n(3) } {
                    Ok(xs) => (xs[2], xs[1], xs[0]),
                    Err(e) => return Response::Error(e),
                };
                let steps = if steps.is_usizeable() {
                    steps.usize()
                } else {
                    return Response::Error(TodoError);
                };
                match (start.kind, end.kind) {
                    (ObjectKind::Integer, ObjectKind::Integer) => {
                        for v in (start.integer()..end.integer()).step_by(steps) {
                            vm.obj_stack.push(v.into());
                        }

                        return Response::Ok;
                    }
                    (ObjectKind::UnsignedInt, ObjectKind::UnsignedInt) => {
                        for v in (start.unsigned()..end.unsigned()).step_by(steps) {
                            vm.obj_stack.push(v.into());
                        }

                        return Response::Ok;
                    }
                    _ => todo!(),
                };
            }
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_get_ptr(&self, _vm: &mut VM) -> Response {
        match self {
            //     Operation::GetPtr => match { vm.obj_stack.pop() } {
            //         Ok(obj) => {
            //             let ptr_obj = Object::new(ObjectData::Pointer(&mut &*obj as *mut &Object));
            //             let ptr_obj: RegObject = vm.register_single(ptr_obj);
            //             vm.obj_stack.push(ptr_obj);
            //             Response::Ok
            //         }
            //         Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
            //     },
            _ => todo!("pointer ops"),
        }
    }

    #[inline(never)]
    fn op_read_ptr(&self, _vm: &mut VM) -> Response {
        match self {
            // Operation::ReadPtr => match { vm.obj_stack.pop() } {
            //     Ok(ptr_obj) => {
            //         if let ObjectData::Pointer(real_ptr) = ptr_obj.data {
            //             let val: RegObject = unsafe { real_ptr.read() };
            //             vm.obj_stack.push(val);
            //             Response::Ok
            //         } else {
            //             Response::Error(ProgramErrorKind::TypeError(
            //                 ObjectKind::Pointer,
            //                 ptr_obj.kind,
            //             ))
            //         }
            //     }
            //     Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
            // },
            _ => todo!("pointer ops"),
        }
    }

    #[inline(never)]
    fn op_set_ptr(&self, _vm: &mut VM) -> Response {
        match self {
            // Operation::SetPtr => unsafe {
            // match { vm.obj_stack.pop_n(2) } {
            //     Ok(objects) => {
            //         let obj = objects.get_unchecked(0);
            //         let ptr_obj = objects.get_unchecked(1);

            //         if let ObjectData::Pointer(real_ptr) = ptr_obj.data {
            //             real_ptr.copy_from(obj, 1);
            //             Response::Ok
            //         } else {
            //             Response::Error(ProgramErrorKind::TypeError(
            //                 ObjectKind::Pointer,
            //                 ptr_obj.kind,
            //             ))
            //         }
            //     }
            //     Err(_) => Response::Error(ProgramErrorKind::StackError(2)),
            // }
            // },
            _ => todo!("pointer ops"),
        }
    }

    #[inline(never)]
    fn op_create_iter(&self, vm: &mut VM) -> Response {
        match self {
            Operation::CreateIter => match { vm.obj_stack.pop() } {
                Ok(list_obj) if list_obj.is(ObjectKind::List) => {
                    let ls = list_obj.list();
                    let iter = Box::new(unsafe { (*ls).clone() }.iter());
                    let iter_obj = Object::new_iterator(Box::into_raw(iter));
                    vm.obj_stack.push(iter_obj);
                    Response::Ok
                }
                Ok(obj) => Response::Error(ProgramErrorKind::TypeError(ObjectKind::List, obj.kind)),
                Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
            },
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_iter_current(&self, vm: &mut VM) -> Response {
        match self {
            Operation::IterCurrent => {
                let data = vm.call_stack.last().unwrap().internal;
                // let locals = vm.call_stack.last().unwrap().locals.borrow().iter();
                match data {
                    Some(data) if data.is(ObjectKind::Iterator) => {
                        let obj = Object::new_unsigned(
                            (unsafe { (*data.iterator()).clone() }.current - 1)
                                .try_into()
                                .unwrap(),
                        );
                        vm.obj_stack.push(obj);

                        Response::Ok
                    }
                    Some(data) => Response::Error(TypeError(ObjectKind::Iterator, data.kind)),
                    None => todo!(),
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_iterate(&self, vm: &mut VM) -> Response {
        match self {
            Operation::Iterate(block) => match { vm.obj_stack.pop_mut() } {
                Ok(&mut object) if object.is(ObjectKind::Iterator) => {
                    let last_frame = vm.call_stack.last_mut_unchecked();
                    last_frame.set_internal(object);
                    while let Some(obj) = (unsafe { object.iterator().as_mut_unchecked() }).next() {
                        vm.obj_stack.push(*obj);
                        vm.run_block(&block.0, block.1);
                    }
                    let _ = vm.call_stack.pop();
                    Response::IterationDone
                }
                Ok(object) => Response::Error(ProgramErrorKind::TypeError(
                    ObjectKind::Iterator,
                    object.kind,
                )),
                Err(_) => Response::Error(ProgramErrorKind::StackError(1)),
            },
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_do_if(&self, vm: &mut VM) -> Response {
        match self {
            Operation::DoIf(block) => {
                let b = match vm.obj_stack.pop() {
                    Ok(v) => v,
                    Err(e) => return Response::Error(e),
                };

                if b.is(ObjectKind::Bool) {
                    if b.bool() {
                        vm.run_block(&block.0, block.1);
                    }
                    Response::Ok
                } else {
                    Response::Error(ProgramErrorKind::TypeError(ObjectKind::Bool, b.kind))
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_debug(&self, _vm: &mut VM) -> Response {
        match self {
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
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_import(&self, vm: &mut VM) -> Response {
        match self {
            Operation::Import(module) => {
                if let Some(m) = MODULES.iter().find(|m| m.1 == *module) {
                    // TODO save imported modules?
                    for func in m.2 {
                        let Operation::ExternFunc(name, _, _) = func else {
                            unreachable!()
                        };
                        let name = &[module, b":" as &[u8], name].concat();
                        let name: &'static [u8] = vm.program.register_bytes(name);
                        vm.program.funcs.insert(name, func.clone());
                    }
                    Response::Ok
                } else {
                    Response::Error(ProgramErrorKind::TodoError)
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_dupe(&self, vm: &mut VM) -> Response {
        match self {
            Operation::Dupe => match vm.obj_stack.last() {
                Ok(o) => {
                    vm.obj_stack.push(*o);
                    Response::Ok
                }
                Err(e) => Response::Error(e),
            },
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_swap(&self, vm: &mut VM) -> Response {
        match self {
            Operation::Swap => {
                if vm.obj_stack.len() > 2 {
                    vm.obj_stack.swap();
                    Response::Ok
                } else {
                    Response::Error(ProgramErrorKind::StackError(2))
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_return_if(&self, vm: &mut VM) -> Response {
        match self {
            Operation::ReturnIf(idx) => match vm.obj_stack.pop() {
                Ok(b) if b.is(ObjectKind::Bool) => {
                    if b.bool() {
                        Response::FunctionReturn(Some(vm.get_local(*idx)))
                    } else {
                        Response::Ok
                    }
                }
                Ok(o) => Response::Error(TypeError(ObjectKind::Bool, o.kind)),
                Err(e) => Response::Error(e),
            },

            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_return(&self, vm: &mut VM) -> Response {
        match self {
            Operation::Return => Response::FunctionReturn(None),
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_list_fill(&self, _vm: &mut VM) -> Response {
        match self {
            Operation::ListFill(_, _) => todo!(),
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_range_loop(&self, vm: &mut VM) -> Response {
        match self {
            Operation::RangeLoop(block) => {
                let (steps, end, start) = match { vm.obj_stack.pop_n(3) } {
                    Ok(xs) => (xs[2], xs[1], xs[0]),
                    Err(e) => return Response::Error(e),
                };
                let steps = if steps.is_usizeable() {
                    steps.usize()
                } else {
                    todo!()
                };
                match (start.kind, end.kind) {
                    (ObjectKind::Integer, ObjectKind::Integer) => {
                        for i in (start.integer()..end.integer()).step_by(steps) {
                            vm.obj_stack.push(i.into());
                            vm.run_block(&block.0, block.1);
                        }
                        return Response::IterationDone;
                    }
                    (ObjectKind::UnsignedInt, ObjectKind::UnsignedInt) => {
                        for i in (start.unsigned()..end.unsigned()).step_by(steps) {
                            vm.obj_stack.push(i.into());
                            vm.run_block(&block.0, block.1);
                        }
                        return Response::IterationDone;
                    }
                    _ => todo!(),
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_push_many_const(&self, vm: &mut VM) -> Response {
        match self {
            Operation::PushManyConst(idx, maybe_n) => {
                let n = match maybe_n {
                    Some(n) => *n,
                    None => match vm.obj_stack.pop() {
                        Ok(o) if o.is_usizeable() => o.usize(),
                        Ok(o) => {
                            return Response::Error(TypeError(ObjectKind::UnsignedInt, o.kind));
                        }
                        Err(e) => return Response::Error(e),
                    },
                };
                let lit = vm.program.get_const(*idx);
                match lit {
                    Ok(lit) => {
                        for _ in 0..n {
                            vm.obj_stack.push(lit);
                        }
                        Response::Ok
                    }
                    Err(e) => Response::Error(e),
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_object(&self, _vm: &mut VM) -> Response {
        match self {
            Operation::Object(_name, _arity, _block) => Response::Ok,
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_create_object(&self, vm: &mut VM) -> Response {
        match self {
            Operation::CreateObject(name) => vm.create_object(name),
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_set_attribute(&self, vm: &mut VM) -> Response {
        match self {
            Operation::SetAttribute(name) => {
                let object = vm.obj_stack.pop();
                match object {
                    Ok(obj) if obj.is(ObjectKind::Data) => {
                        let ptr = obj.data();
                        if let Ok(attr) = vm.obj_stack.pop() {
                            unsafe { ptr.as_mut_unchecked() }.insert(name, attr);
                            vm.obj_stack.push(object.unwrap());
                            Response::Ok
                        } else {
                            Response::Error(ProgramErrorKind::StackError(1))
                        }
                    }
                    Ok(_) => todo!(),
                    Err(_e) => todo!(),
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_get_attribute(&self, vm: &mut VM) -> Response {
        match self {
            Operation::GetAttribute(name) => {
                let object = vm.obj_stack.pop();
                match object {
                    Ok(obj) if obj.is(ObjectKind::Data) => {
                        let ptr = obj.data();
                        if let Some(attr) = unsafe { ptr.as_ref_unchecked() }.get(name) {
                            vm.obj_stack.push(*attr);
                            Response::Ok
                        } else {
                            Response::Error(ProgramErrorKind::TodoError)
                        }
                    }
                    Ok(_) => todo!(),
                    Err(_e) => todo!(),
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline(never)]
    fn op_bin_op(&self, vm: &mut VM) -> Response {
        match self {
            Operation::BinOp(kind) => {
                let res = match vm.obj_stack.pop_n(2) {
                    Ok(objs) => {
                        match kind {
                            BinOpKind::Add => jed::add(objs),
                            BinOpKind::Sub => jed::sub(objs),
                            BinOpKind::Mul => jed::mul(objs),
                            BinOpKind::Div => jed::div(objs),
                            BinOpKind::Mod => jed::modulus(objs),
                            BinOpKind::Power => jed::pow(objs),
                            BinOpKind::Root => jed::root(objs),
                            BinOpKind::Eq => jed::eq(objs),
                            BinOpKind::LessEq => jed::lesseq(objs),
                            BinOpKind::GreatEq => jed::greateq(objs),
                            BinOpKind::Lesser => jed::lesser(objs),
                            BinOpKind::Greater => jed::greater(objs),
                            BinOpKind::And => jed::and(objs),
                            BinOpKind::Or => jed::or(objs),
                            // BinOpKind::BitAnd => jed::bitand(objs),
                            // BinOpKind::BitOr => jed::bitor(objs),
                            // BinOpKind::Xor => jed::xor(objs),
                            // BinOpKind::BitShLeft => jed::bitshleft(objs),
                            // BinOpKind::BitShRight => jed::bitshright(objs)
                            _ => todo!("{kind}"),
                        }
                    }
                    Err(e) => return Response::Error(e),
                };
                match res {
                    Response::FunctionReturn(object) => {
                        object.map(|o| vm.obj_stack.push(o));
                        Response::Ok
                    }
                    _ => res,
                }
            }
            _ => unreachable!(),
        }
    }
}
