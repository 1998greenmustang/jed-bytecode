use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{self, BufReader, Read, Write},
    rc::Rc,
    str::CharIndices,
};

use jed_macros::match_ops;

use crate::{
    MAGIC_NUMBER,
    error::ProgramErrorKind,
    memory::{Dropless, list::List, peekableiterator::PeekableIterator},
    object::Object,
    operation::{Block, Operation},
    utils,
};

type Arity = usize;
type Index = usize;

pub type MemoKey = (&'static [u8], &'static [Object]);
type MemoTable = HashMap<MemoKey, Object>;

pub struct Program {
    pub string_arena: Dropless,
    pub saved_strings: BTreeMap<String, &'static [u8]>,
    pub instructions: Block,
    pub funcs: BTreeMap<&'static [u8], Operation>,
    pub memos: MemoTable,
    pub blocks: Vec<Block>,
}

impl Program {
    pub fn new() -> Program {
        let mut program = Program {
            string_arena: Default::default(),
            saved_strings: BTreeMap::new(),
            instructions: Default::default(),
            funcs: BTreeMap::new(),
            blocks: Vec::new(),
            memos: HashMap::new(),
        };
        // register keywords/stuff that not be added later
        // probably should be a macro but (:
        program.register("main".to_owned());

        return program;
    }

    pub fn get_op(&self, idx: usize) -> &Operation {
        match self.instructions.get(idx) {
            Some(op) => &op,
            None => &Operation::Exit,
        }
    }

    pub fn register_bytes(&mut self, byte_str: &[u8]) -> &'static [u8] {
        let string = utils::display_bytes(byte_str);
        if let Some(saved) = self.saved_strings.get(&string) {
            return saved;
        } else {
            let byte_str: &[u8] = self.string_arena.alloc_slice(byte_str);
            let byte_str: &'static [u8] = unsafe { &*(byte_str as *const [u8]) };
            self.saved_strings.insert(string, byte_str);
            return byte_str;
        }
    }

    pub fn register(&mut self, string: String) -> &'static [u8] {
        if let Some(saved) = self.saved_strings.get(&string) {
            return saved;
        } else {
            let byte_str = string.as_bytes();
            let byte_str: &[u8] = self.string_arena.alloc_slice(byte_str);
            let byte_str: &'static [u8] = unsafe { &*(byte_str as *const [u8]) };
            self.saved_strings.insert(string, byte_str);
            return byte_str;
        }
    }

    pub fn register_arguments(&mut self, objects: &[Object]) -> &'static [Object] {
        let saved_bytes = self.string_arena.alloc_slice(objects);
        let saved_bytes: &'static [Object] = unsafe { &*(saved_bytes as *const [Object]) };
        saved_bytes
    }

    pub fn get_memo(&self, key: MemoKey) -> Option<&Object> {
        self.memos.get(&key)
    }
    pub fn set_memo(&mut self, key: MemoKey, result: Object) {
        self.memos.insert(key, result);
    }

    pub fn to_file(&self, file: &mut File) -> io::Result<()> {
        todo!()
    }

    /// TODO
    /// Bytecode files look like
    /// [
    ///  jed_magicnumber ("jed"),
    ///  Operation as u8,
    ///    if operation has `&'static [u8]` as arg: `usize` then n amount of bytes
    ///    if operation has `Option<usize>`: `true` | `false` then `usize`
    ///    if operation has `BinOpKind`: BinOpKind-able `u8`
    ///    if operation has `BuiltIn`: BuiltIn-able `u8`
    ///    else: nothing,
    ///  ...
    /// ]
    /// Spans will be added later for error reporting
    pub fn from_file(file: &mut File) -> io::Result<Self> {
        todo!()
    }

    fn parse_token(text: &mut PeekableIterator<char>) -> Option<String> {
        let _ = text.until(|c| !&[' ', '\t', '\n'].contains(c));
        let c = text.peek();
        match c {
            Some(c) if c == &'"' => {
                text.next();
                if let Some(u) = text.until_any_inclusive(&['"']) {
                    let mut u: Vec<char> = u.iter().map(|e| *e).collect();
                    u.insert(0, '"');
                    return Some(u.iter().collect());
                } else {
                    // TODO parsing error
                    panic!()
                }
            }
            Some(c) if ['{', '}'].contains(&c) => {
                let c = text.next()?;
                return Some(c.to_string());
            }
            Some(_) => {
                if let Some(u) = text.until_any(&[' ', '\t', '{', '}', '\n']) {
                    return Some(u.iter().collect());
                } else {
                    return None;
                }
            }
            None => return None,
        }
    }

    fn parse_block(&mut self, text: &mut PeekableIterator<char>) -> Block {
        let mut block = List::new();

        while let Some(token) = Self::parse_token(text) {
            if token == "}" {
                break;
            }
            let op = token.as_str();
            block.push(match_ops!(
                // no argument
                {[
                    Empty,
                    Pop,
                    Dupe,
                    Swap,
                    Exit,
                    PushTemp,
                    StoreTemp,
                    ListPush,
                    PushRange,
                    GetPtr,
                    ReadPtr,
                    SetPtr,
                    GetIter,
                    IterNext,
                    IterPrev,
                    IterSkip,
                    IterCurrent,
                    Debug
                ]},
                // single custom type arg
                {[BinOp, CallBuiltIn, UnaryOp], Self::parse_token(text).unwrap().into()},
                // bytes
                {[PushLit, PushName, ReturnIf, StoreConst, StoreName, ReturnIfConst, Import],
                    self.register(Self::parse_token(text).unwrap().into())},
                // option<usize>
                {[CreateList, ListAlloc, ListSet, ListGet],
                    utils::string_to_t(match Self::parse_token(text) {
                        Some(v) if v.chars().all(|c| c.is_numeric()) => v,
                        _ => {
                            text.undo();
                            "".to_string()
                        }
                    }).ok()},
                {Call, {
                    let arg = Self::parse_token(text).unwrap();
                    let split: Vec<&str> = arg.split(' ').filter(|x| x != &"").collect();
                    match split.len() {
                        2 => unsafe {
                            let modname = self.register(split.get_unchecked(0).to_string());
                            let funcname = self.register(split.get_unchecked(1).to_string());
                            Operation::Call(Some(modname), Some(funcname))
                        }
                        1 => unsafe {
                            let funcname = self.register(split.get_unchecked(0).to_string());
                            Operation::Call(None, Some(funcname))
                        }
                        0 => Operation::Call(None, None),
                        _ => panic!()
                    }
                }}
                {Func, {
                    let saved_name = self.register(Self::parse_token(text).unwrap());
                    let arity = Self::parse_token(text).unwrap()
                        .parse::<usize>()
                        .expect("arity is not a number or something");
                    let idx = self.instructions.len();
                    match Self::parse_token(text) {
                        Some(bracket) if bracket == "{" => {
                            let b = self.parse_block(text);
                            let op = Operation::Func(saved_name, arity, b);
                            self.funcs.insert(saved_name, op.clone());
                            op
                        }
                        _ => panic!("start blocks with {{ plz")
                    }
                }},
                {DoForIn, {
                    let arg = self.register(Self::parse_token(text).unwrap());
                    match Self::parse_token(text) {
                        Some(bracket) if bracket == "{" => {
                            let b = self.parse_block(text);
                            Operation::DoForIn(arg, b)
                        }
                        _ => panic!("start blocks with {{ plz")
                    }
                }},
                {[RangeLoop, Iterate, DoIf, DoFor,]
                    match Self::parse_token(text) {
                        Some(bracket) if bracket == "{" => {
                            self.parse_block(text)
                        }
                        _ => panic!("start blocks with {{ plz")
                    }
                }
            ));
            // buffer.push(c);
        }
        // println!("{}", block);
        let b = Rc::new(block);
        self.blocks.push(Rc::clone(&b));
        return Rc::clone(&b);
    }

    fn remove_comments(text: String) -> String {
        let mut new_text = text.clone();
        while let Some(o) = new_text.find('#') {
            new_text.replace_range(o..o + new_text[o..].find('\n').unwrap(), "");
        }
        return new_text;
    }

    pub fn from_string(text: String) -> Self {
        let text = Self::remove_comments(text);
        let iter = text.chars().into_iter();
        let mut program = Self::new();
        program.instructions = program.parse_block(&mut iter.collect());
        program
    }

    // pub fn get_done(&self, pc: &usize) -> Result<&usize, ProgramErrorKind> {
    //     match self.block_returns.get(pc) {
    //         Some(address) => Ok(address),
    //         None => Err(ProgramErrorKind::DoneAddress),
    //     }
    // }

    // pub fn import_module(&mut self, other: &mut Program) {
    //     // update: vm.program.instructions, vm.program.funcs, vm.program.string_arena, vm.program.saved_strings
    //     let length_of_other = other.instructions.len();
    //     self.instructions.append(&mut other.instructions);

    //     for (_name, tpl) in self.funcs.iter_mut() {
    //         tpl.0 += length_of_other;
    //     }
    //     self.funcs.append(&mut other.funcs);

    //     for (string, _) in other.saved_strings.clone() {
    //         other.register(string);
    //     }
    //     drop(other);
    // }
}
