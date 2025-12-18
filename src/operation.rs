use crate::{
    frame::Frame,
    object::{Object, ObjectKind},
    vm::VM,
};

#[derive(Debug, Clone, Copy)]
pub enum BuiltIn {
    PrintLn,
}

impl BuiltIn {
    pub fn call(&self, arg: Object) {
        match self {
            BuiltIn::PrintLn => println!("{arg}"),
        }
    }
}

impl From<&String> for BuiltIn {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "println" => BuiltIn::PrintLn,
            _ => panic!("No such builtin '{}'", value),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,

    Eq,
    LessEq,
    GreatEq,
    Lesser,
    Greater,
}

impl BinOpKind {
    pub fn from_string(s: &String) -> Self {
        match s.as_str() {
            "+" => BinOpKind::Add,
            "-" => BinOpKind::Sub,
            "*" => BinOpKind::Mul,
            "/" => BinOpKind::Div,
            "==" => BinOpKind::Eq,
            "<=" => BinOpKind::LessEq,
            ">=" => BinOpKind::GreatEq,
            "<" => BinOpKind::Lesser,
            ">" => BinOpKind::Greater,
            _ => panic!("Not implemented: {}", s),
        }
    }

    fn from_object(arg: Object) -> BinOpKind {
        match arg {
            val if val.1 == b"+" => BinOpKind::Add,
            val if val.1 == b"-" => BinOpKind::Sub,
            val if val.1 == b"*" => BinOpKind::Mul,
            val if val.1 == b"/" => BinOpKind::Div,
            val if val.1 == b"==" => BinOpKind::Eq,
            val if val.1 == b"<=" => BinOpKind::LessEq,
            val if val.1 == b">=" => BinOpKind::GreatEq,
            val if val.1 == b"<" => BinOpKind::Lesser,
            val if val.1 == b">" => BinOpKind::Greater,
            _ => panic!("Not implemented: {:?}", arg),
        }
    }
}

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
    Func(Object),
    Done,
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
            12 => Operation::Func(arg.unwrap_or_else(|| Self::arg_error())),
            13 => Operation::Done,
            _ => panic!("Invalid operation: '{}' '{:?}'", op, arg),
        }
    }

    pub fn call(&self, vm: &mut VM) {
        match self {
            Operation::BinOp(bin_op_kind) => vm.handle_bin_op(bin_op_kind),
            Operation::Call(func) => {
                // "func" is actually a string Object
                vm.call_stack.push(Frame::new(vm.counter));
                let func: Object = Object(ObjectKind::Func, func.1);
                vm.jump(&func);
            }
            Operation::PushLit(literal) => vm.obj_stack.push(*literal),
            Operation::PushName(name) => {
                let frame = vm
                    .call_stack
                    .last()
                    .unwrap_or_else(|| panic!("No frame on the call stack!"));
                vm.obj_stack.push(
                    frame
                        .get_local(name)
                        .unwrap_or_else(|| panic!("No such variable '{:?}'", name)),
                );
            }
            Operation::PushTemp => {
                let tmp = vm
                    .temp
                    .unwrap_or_else(|| panic!("No object stored in the temp!"));
                vm.obj_stack.push(tmp);
            }
            Operation::Pop => todo!(),
            Operation::ReturnIf(name) => {
                let b = vm
                    .obj_stack
                    .pop()
                    .unwrap_or_else(|| panic!("No object on the object stack"));
                assert_eq!(b.0, ObjectKind::Bool, "Object is not a boolean");
                let bol: &str = unsafe { std::mem::transmute(b.1) };
                match bol {
                    "true" => {
                        let frame = vm
                            .call_stack
                            .pop()
                            .unwrap_or_else(|| panic!("No frame on the call stack!"));
                        vm.obj_stack.push(
                            frame
                                .get_local(name)
                                .unwrap_or_else(|| panic!("No object on the object stack!")),
                        );
                        vm.counter = frame.return_address;
                    }
                    "false" => return,
                    _ => unreachable!(),
                }
            }
            Operation::StoreConst(_) => todo!(),
            Operation::StoreName(name) => {
                let frame = vm
                    .call_stack
                    .last_mut()
                    .unwrap_or_else(|| panic!("No frame on the call stack!"));
                frame.add_local(
                    *name,
                    vm.obj_stack
                        .pop()
                        .unwrap_or_else(|| panic!("No object on the obj stack!")),
                );
            }
            Operation::StoreTemp => {
                let obj = vm
                    .obj_stack
                    .pop()
                    .unwrap_or_else(|| panic!("No object on the obj stack!"));
                vm.temp = Some(obj);
            }
            Operation::Func(_) => {}
            Operation::Done => {
                let frame = vm
                    .call_stack
                    .pop()
                    .unwrap_or_else(|| panic!("No frame on the call stack!"));
                vm.counter = frame.return_address;
            }
            Operation::CallBuiltIn(built_in) => {
                let obj = vm
                    .obj_stack
                    .pop()
                    .unwrap_or_else(|| panic!("No object on the obj stack!"));
                built_in.call(obj);
            }
        }
    }
}
