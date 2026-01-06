#![allow(dead_code)]
pub mod saveable;

use crate::object::{Object, ObjectKind};
use crate::operation::*;
use crate::program::Program;
use std::convert::TryInto;
use std::fs::File;
use std::io::{self, Read, Write};

#[derive(Clone)]
pub struct ObjectSaveable(Vec<u8>);

impl From<Object> for ObjectSaveable {
    fn from(value: Object) -> Self {
        let mut vec = value.1.to_vec();
        vec.insert(0, value.0 as u8);
        Self(vec)
    }
}

#[derive(Clone)]
enum OperationSaveable {
    BinOp(BinOpKind),
    Call(ObjectSaveable),
    CallBuiltIn(BuiltIn),
    PushLit(ObjectSaveable),
    PushName(ObjectSaveable),
    PushTemp,
    Pop,
    ReturnIf(ObjectSaveable),
    StoreConst(ObjectSaveable),
    StoreName(ObjectSaveable),
    StoreTemp,
    Func(ObjectSaveable),
    Done,
}

impl OperationSaveable {
    fn to_operation(self, object: Option<Object>) -> Operation {
        match self {
            OperationSaveable::CallBuiltIn(built_in) => Operation::CallBuiltIn(built_in),
            OperationSaveable::BinOp(bin_op_kind) => Operation::BinOp(bin_op_kind),
            OperationSaveable::Call(_) => Operation::Call(object.unwrap()),
            OperationSaveable::Func(_) => Operation::Func(object.unwrap()),
            OperationSaveable::PushLit(_) => Operation::PushLit(object.unwrap()),
            OperationSaveable::PushName(_) => Operation::PushName(object.unwrap()),
            OperationSaveable::ReturnIf(_) => Operation::ReturnIf(object.unwrap()),
            OperationSaveable::StoreConst(_) => Operation::StoreConst(object.unwrap()),
            OperationSaveable::StoreName(_) => Operation::StoreName(object.unwrap()),
            OperationSaveable::StoreTemp => Operation::StoreTemp,
            OperationSaveable::PushTemp => Operation::PushTemp,
            OperationSaveable::Pop => Operation::Pop,
            OperationSaveable::Done => Operation::Done,
        }
    }
}

impl From<super::operation::Operation> for OperationSaveable {
    fn from(value: super::operation::Operation) -> Self {
        match value {
            Operation::BinOp(bin_op_kind) => Self::BinOp(bin_op_kind),
            Operation::Call(object) => Self::Call(object.into()),
            Operation::CallBuiltIn(built_in) => Self::CallBuiltIn(built_in),
            Operation::PushLit(object) => Self::PushLit(object.into()),
            Operation::PushName(object) => Self::PushName(object.into()),
            Operation::PushTemp => Self::PushTemp,
            Operation::Pop => Self::Pop,
            Operation::ReturnIf(object) => Self::ReturnIf(object.into()),
            Operation::StoreConst(object) => Self::StoreConst(object.into()),
            Operation::StoreName(object) => Self::StoreName(object.into()),
            Operation::StoreTemp => Self::StoreTemp,
            Operation::Func(object) => Self::Func(object.into()),
            Operation::Done => Self::Done,
        }
    }
}

pub struct FileOptions {}

struct ProgramFile(Vec<OperationSaveable>);

impl From<&super::program::Program> for ProgramFile {
    fn from(value: &super::program::Program) -> Self {
        let vec = value
            .instructions
            .iter()
            .map(|x| Into::<OperationSaveable>::into(*x))
            .collect();
        ProgramFile(vec)
    }
}

impl From<ProgramFile> for Program {
    fn from(value: ProgramFile) -> Self {
        let mut this = Program {
            arena: Default::default(),
            instructions: Default::default(),
            funcs: Default::default(),
        };
        for (idx, op_saveable) in value.0.iter().enumerate() {
            let maybe_obj = match op_saveable {
                OperationSaveable::Call(object_saveable)
                | OperationSaveable::PushLit(object_saveable)
                | OperationSaveable::PushName(object_saveable)
                | OperationSaveable::ReturnIf(object_saveable)
                | OperationSaveable::StoreConst(object_saveable)
                | OperationSaveable::StoreName(object_saveable)
                | OperationSaveable::Func(object_saveable) => {
                    let read_kind = object_saveable.0.first().unwrap().try_into();
                    match read_kind {
                        Ok(kind) => {
                            println!("{kind:?}");
                            let bytes = this.register(object_saveable.0.as_slice());
                            Some(Object(kind, bytes))
                        }
                        Err(msg) => panic!("{}", msg),
                    }
                }
                _ => None,
            };
            let op = op_saveable.clone().to_operation(maybe_obj);
            println!("op: {:?}", op);
            if let Operation::Func(obj) = op {
                println!("{idx}");
                this.funcs.insert(obj, idx);
            }
            this.instructions.push(op);
        }
        println!("funcs {:?}", this.funcs);
        return this;
    }
}
