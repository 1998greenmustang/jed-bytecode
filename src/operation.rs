use std::convert::TryInto;

use crate::{
    binops::BinOpKind,
    builtin::BuiltIn,
    frame::{Frame, FrameKind},
    mutable::MutableObject,
    object::{Object, ObjectKind},
    vm::VM,
};

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    BinOp(BinOpKind),
    Call(Object),
    CallBuiltIn(BuiltIn),
    PushLit(Object),
    PushName(Object),
    PushTemp,
    Pop,
    ReturnIf(Object),
    StoreConst(Object),
    StoreName(Object),
    StoreTemp,
    Func(Object, u8),
    Done,
    Exit,
    DoFor,
    DoForIn(Object),
    CreateList(Object),
    ListPush(Object),
    ListGet,
    ListSet,
    PushRange,
    ReturnIfConst(Object),
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

    pub fn arg_error() -> ! {
        panic!("no arg alert!")
    }

    pub fn new(op: usize, arg: Option<Object>) -> Self {
        match op {
            1 => Operation::BinOp(BinOpKind::from_object(
                arg.unwrap_or_else(|| Self::arg_error()),
            )),
            2 => Operation::Call(arg.unwrap_or_else(|| Self::arg_error())),
            3 => Operation::CallBuiltIn(BuiltIn::PrintLn), // TODO
            4 => Operation::PushLit(arg.unwrap_or_else(|| Self::arg_error())),
            5 => Operation::PushName(arg.unwrap_or_else(|| Self::arg_error())),
            6 => Operation::PushTemp,
            7 => Operation::Pop,
            8 => Operation::ReturnIf(arg.unwrap_or_else(|| Self::arg_error())),
            9 => Operation::StoreConst(arg.unwrap_or_else(|| Self::arg_error())),
            10 => Operation::StoreName(arg.unwrap_or_else(|| Self::arg_error())),
            11 => Operation::StoreTemp,
            12 => {
                let argument = arg.unwrap_or_else(|| Self::arg_error());
                let (arity, func) = argument.1.split_last().unwrap_or_else(|| Self::arg_error());
                Operation::Func(Object(argument.0, func), *arity)
            }
            13 => Operation::Done,
            14 => Operation::Exit,
            15 => Operation::DoFor,
            16 => Operation::DoForIn(arg.unwrap_or_else(|| Self::arg_error())),
            17 => Operation::CreateList(arg.unwrap_or_else(|| Self::arg_error())),
            18 => Operation::ListPush(arg.unwrap_or_else(|| Self::arg_error())),
            19 => Operation::ListGet,
            20 => Operation::ListSet,
            21 => Operation::PushRange,
            22 => Operation::ReturnIfConst(arg.unwrap_or_else(|| Self::arg_error())),
            _ => panic!("Invalid operation: '{:?}' '{:?}'", op, arg),
        }
    }

    pub fn call(&self, vm: &mut VM) {
        match self {
            Operation::BinOp(bin_op_kind) => vm.handle_bin_op(bin_op_kind),
            Operation::Call(func) => {
                let (func_ptr, arity) = vm
                    .program
                    .get_func(func)
                    .cloned()
                    .unwrap_or_else(|| panic!("invalid function"));
                if arity > 0 {
                    let args = unsafe {
                        vm.obj_stack
                            .last_n((arity).try_into().expect("i dont know the error message"))
                    };
                    let args = vm.program.register_objects(args);
                    match vm.program.get_memo((func_ptr, args)) {
                        Some(value) => {
                            // println!("YES DUDE {:?}", args);
                            unsafe { vm.obj_stack.pop_n(arity) };
                            vm.obj_stack.push(*value);
                        }
                        None => {
                            vm.call_stack.push(Frame::new(vm.counter, FrameKind::Call));
                            let current_frame = vm.call_stack.last_mut();
                            let args = vm.program.register_objects(args);
                            current_frame.memo_key = (func_ptr, args);
                            vm.jump(&func);
                        }
                    }
                }
            }
            Operation::PushLit(literal) => vm.obj_stack.push(*literal),
            Operation::PushName(name) => {
                let frame = vm.call_stack.last();
                let item = frame
                    .get_local(name)
                    .unwrap_or_else(|| panic!("No such variable '{:?}'", name));
                vm.obj_stack.push(
                    frame
                        .get_local(name)
                        .unwrap_or_else(|| panic!("No such variable '{:?}'", name)),
                );

                // println!("pushing {name}: {item}");
            }
            Operation::PushTemp => {
                let tmp = vm
                    .temp
                    .unwrap_or_else(|| panic!("No object stored in the temp!"));
                vm.obj_stack.push(tmp);
            }
            Operation::Pop => {
                vm.obj_stack.pop();
            }
            Operation::ReturnIf(name) => {
                let b = vm.obj_stack.pop();
                assert_eq!(b.0, ObjectKind::Bool, "Object is not a boolean");
                let bol: &str = unsafe { std::mem::transmute(b.1) };
                match bol {
                    "true" => {
                        let frame = vm.call_stack.pop();
                        vm.obj_stack.push(
                            frame
                                .get_local(name)
                                .unwrap_or_else(|| panic!("No such object '{}'!", name)),
                        );
                        vm.counter = frame.return_address;
                        let return_value = *vm.obj_stack.last_option().unwrap_or(&Object::dummy());
                        vm.program.set_memo(frame.memo_key, return_value);
                    }
                    "false" => return,
                    _ => unreachable!(),
                }
            }
            Operation::StoreConst(name) => {
                let obj = vm.obj_stack.pop();
                vm.store_const(*name, obj);
            }
            Operation::StoreName(name) => {
                let frame = vm.call_stack.last_mut();
                frame.add_local(*name, vm.obj_stack.pop());
            }
            Operation::StoreTemp => {
                let obj = vm.obj_stack.pop();
                vm.temp = Some(obj);
            }
            Operation::Func(_, _) => {}
            Operation::Done => {
                let frame = vm.call_stack.pop();
                match frame.kind {
                    FrameKind::Call => {
                        let return_value = *vm.obj_stack.last_option().unwrap_or(&Object::dummy());
                        vm.program.set_memo(frame.memo_key, return_value);
                        vm.counter = frame.return_address;
                    }
                    FrameKind::Loop => {
                        let next_frame = vm.call_stack.last_mut_option();
                        if let Some(nxt) = next_frame {
                            // println!("{nxt:?}\n{frame:?}");
                            if nxt.return_address == frame.return_address {
                                // println!("YES DUDE !!! {}", vm.counter);
                                nxt.copy_locals(&frame);
                                vm.counter = frame.return_address;
                            } else {
                                // println!("YES DUDE !!! {}", vm.counter);
                                vm.counter = vm.counter + 1;
                                return;
                            }
                        }
                    }
                    FrameKind::Main => todo!(),
                }
            }
            Operation::Exit => {
                // println!("i have been called");
                std::process::exit(0);
            }
            Operation::CallBuiltIn(built_in) => {
                let obj = vm.obj_stack.pop();
                if ObjectKind::MutablePtr == obj.0 {
                    let pointer = obj.pointer();
                    let mobj = vm.heap.get(&pointer).unwrap();
                    if let MutableObject::List(l) = mobj {
                        let objs: Vec<Object> = l
                            .iter()
                            .map(|o| vm.heap.get(&o.pointer()).unwrap().into())
                            .collect();
                        built_in.call_with_slice(&objs);
                        return;
                    }
                }
                built_in.call(obj);
            }
            Operation::DoFor => {
                // TODO let done work with this
                let times = vm.obj_stack.pop().integer();
                let pc = vm.counter.clone();
                for _ in 0..times {
                    let mut frame = Frame::new(pc, FrameKind::Loop);
                    frame.copy_locals(vm.call_stack.last());
                    vm.call_stack.push(frame);
                }
            }
            Operation::DoForIn(obj_name) => {
                let current_frame = vm.call_stack.last();
                let maybe_obj_ptr = current_frame.get_local(obj_name);
                let pointer = match maybe_obj_ptr {
                    Some(pointer) => pointer,
                    None => panic!("No such name '{}'", obj_name),
                };
                let obj = vm.heap.get(&pointer.pointer());
                let times = match obj {
                    Some(MutableObject::List(l)) => l.len(),
                    _ => panic!("Not a list"),
                };
                let pc = vm.counter.clone();
                for _ in 0..times {
                    let mut frame = Frame::new(pc, FrameKind::Loop);
                    frame.copy_locals(vm.call_stack.last());
                    vm.call_stack.push(frame);
                }
            }
            Operation::CreateList(num) => {
                // create a mutableobject in the heap
                // push a mutableptr to the stack
                let idx = vm.create_list(num.pointer());
                vm.obj_stack.push(Object(
                    ObjectKind::MutablePtr,
                    vm.program.register(&idx.to_be_bytes()),
                ))
            }
            Operation::ListPush(num) => {
                let number_of_items = num.integer();
                let list_ptr = vm.obj_stack.pop().pointer();
                let items = unsafe { vm.obj_stack.pop_n(number_of_items.try_into().unwrap()) };
                let mut ptrs: Vec<Object> = vec![];
                for i in items {
                    let index = vm.heap.push((*i).into());
                    ptrs.push(Object(
                        ObjectKind::MutablePtr,
                        vm.program.register(&index.to_be_bytes()),
                    ));
                }
                let list = vm
                    .heap
                    .get_mut(&list_ptr)
                    .unwrap_or_else(|| panic!("no object there"));
                if let MutableObject::List(l) = list {
                    let combined = [l, ptrs.as_slice()].concat();
                    let new_list = vm.program.register_objects(combined.as_slice());
                    vm.heap.insert(list_ptr, MutableObject::List(new_list));
                }
            }
            Operation::ListGet => {
                let index = vm.obj_stack.pop().integer();
                let list_ptr = vm.obj_stack.pop().pointer();

                let list = vm
                    .heap
                    .get(&list_ptr)
                    .unwrap_or_else(|| panic!("no object there"));
                // println!("{list_ptr} {index}");
                if let MutableObject::List(l) = list {
                    let item_ptr: Object = *l
                        .get(index as usize)
                        .unwrap_or_else(|| panic!("nothing there broseph"));
                    let item = vm
                        .heap
                        .get(&item_ptr.pointer())
                        .unwrap_or_else(|| panic!("nothing there bro"));
                    vm.obj_stack.push(item.into());
                    // println!("{item:?}")
                }
            }
            Operation::ListSet => {
                let index = vm.obj_stack.pop().pointer();
                let list_ptr = vm.obj_stack.pop().pointer();
                let obj = vm.obj_stack.pop();

                let obj_ptr = vm.heap.push(obj.into());

                // println!("set: {index} {list_ptr} {obj} @ {obj_ptr}");
                let list = vm
                    .heap
                    .get_mut(&list_ptr)
                    .unwrap_or_else(|| panic!("no object there"));
                if let MutableObject::List(l) = list {
                    let mut l_vec = l.to_vec();
                    l_vec.remove(index);
                    let _ = l_vec.insert(
                        index,
                        Object(
                            ObjectKind::MutablePtr,
                            vm.program.register(&obj_ptr.to_be_bytes()),
                        ),
                    );

                    let registered = vm.program.register_objects(&l_vec);
                    vm.heap.insert(list_ptr, MutableObject::List(registered));
                }
            }
            Operation::PushRange => {
                let steps = vm.obj_stack.pop();
                let end = vm.obj_stack.pop();
                let start = vm.obj_stack.pop();
                println!("range: {start}..{end} by {steps}");
                for i in (start.integer()..end.integer()).step_by(steps.pointer()) {
                    vm.obj_stack.push(vm.program.register_integer(i));
                }
            }
            Operation::ReturnIfConst(name) => {
                let b = vm.obj_stack.pop();
                assert_eq!(b.0, ObjectKind::Bool, "Object is not a boolean");
                let bol: &str = unsafe { std::mem::transmute(b.1) };
                match bol {
                    "true" => {
                        let frame = vm.call_stack.pop();
                        vm.obj_stack.push(*vm.get_const(name));
                        vm.counter = frame.return_address;
                        let return_value = *vm.obj_stack.last_option().unwrap_or(&Object::dummy());
                        vm.program.set_memo(frame.memo_key, return_value);
                    }
                    "false" => return,
                    _ => unreachable!(),
                }
            }
        }
    }
}
