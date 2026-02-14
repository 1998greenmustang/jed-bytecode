use std::collections::BTreeMap;

use crate::{object::Object, program::MemoKey};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum FrameKind {
    DoForLoop,
    IterateLoop,
    DoForInLoop,
    DoIfBlock,
    Call,
    Main,
}

#[derive(Debug, Clone)]
pub struct Frame {
    // String -> Literal
    pub locals: BTreeMap<&'static [u8], &'static Object>,
    pub return_address: usize,
    pub memo_key: MemoKey,
    pub kind: FrameKind,
}

impl Frame {
    pub fn new(return_address: usize, kind: FrameKind) -> Self {
        Frame {
            memo_key: (0, &[]),
            locals: BTreeMap::new(),
            return_address,
            kind,
        }
    }

    pub fn add_local(&mut self, name: &'static [u8], obj: &'static Object) {
        // if self.locals.contains_key(name) {
        //     panic!("{} has already been declared");
        // }
        self.locals.insert(name, obj);
    }

    pub fn get_local(&self, name: &'static [u8]) -> Option<&'static Object> {
        self.locals.get(name).cloned()
    }

    pub fn copy_locals(&mut self, other: &Self) {
        self.locals = other.locals.clone()
    }
}
