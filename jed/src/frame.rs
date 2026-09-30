use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::{memory::list::List, object::Object, program::MemoKey};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum FrameKind {
    DoForLoop,
    IterateLoop,
    DoForInLoop,
    DoIfBlock,
    Call,
    Main,
    Initial,
    RangeLoop,
}

#[derive(Debug, Clone)]
pub struct Frame {
    // String -> Literal
    pub locals: Rc<RefCell<List<&'static Object>>>,
    pub return_address: usize,
    pub memo_key: MemoKey,
    pub kind: FrameKind,
    pub internal: Option<&'static Object>,
}

impl Frame {
    pub fn new(return_address: usize, kind: FrameKind) -> Self {
        Frame {
            memo_key: (&[], &[]),
            locals: Rc::new(RefCell::new(List::new())),
            return_address,
            kind,
            internal: None,
        }
    }

    pub fn set_internal(&mut self, obj: &'static Object) {
        self.internal = Some(obj)
    }

    pub fn add_local(&mut self, name: usize, obj: &'static Object) {
        // if self.locals.contains_key(name) {
        //     panic!("{} has already been declared");
        // }
        (*self.locals).borrow_mut().insert(name, &obj);
    }

    pub fn get_local(&self, name: usize) -> Option<&'static Object> {
        (*self.locals).borrow().get(name).map(|v| *v)
    }

    pub fn copy_locals(&mut self, other: &Self) {
        self.locals = other.locals.clone()
    }
}
