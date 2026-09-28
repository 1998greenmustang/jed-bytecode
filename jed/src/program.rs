use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{self},
    mem::ManuallyDrop,
    ptr::slice_from_raw_parts_mut,
    rc::Rc,
};

use jed_macros::match_ops;

use crate::{
    error::{
        self,
        ProgramErrorKind::{ParsingError, TodoError},
        Result,
    },
    memory::{Dropless, list::List, peekableiterator::PeekableIterator},
    modules,
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
    pub constructors: BTreeMap<&'static [u8], Operation>,
    pub consts: ManuallyDrop<List<Object>>,
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
            constructors: BTreeMap::new(),
            blocks: Vec::new(),
            memos: HashMap::new(),
            consts: ManuallyDrop::new(List::new()),
        };
        // register keywords/stuff that not be added later
        // probably should be a macro but (:
        program.register("main".to_owned());
        for func in modules::jed::FUNCTIONS {
            let Operation::ExternFunc(name, _, _) = func else {
                unreachable!()
            };
            let name: &'static [u8] = program.register_bytes(name);
            program.funcs.insert(name, func.clone());
        }
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

    pub fn to_file(&self, _file: &mut File) -> io::Result<()> {
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
    pub fn from_file(_file: &mut File) -> io::Result<Self> {
        todo!()
    }

    fn parse_token(text: &mut PeekableIterator<char>) -> Option<String> {
        let _ = text.until(|c| !&[' ', '\t', '\n', ','].contains(c));
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
            Some(c) if ['{', '}', ','].contains(&c) => {
                let c = text.next()?;
                return Some(c.to_string());
            }
            Some(_) => {
                if let Some(u) = text.until_any(&[' ', '\t', '{', '}', '\n', ',']) {
                    return Some(u.iter().collect());
                } else {
                    return None;
                }
            }
            None => return None,
        }
    }

    fn parse_consts(&mut self, text: &mut PeekableIterator<char>) -> Operation {
        // im in the block here
        while let Some(literal) = Self::parse_token(text) {
            if literal == "," {
                continue;
            }
            if literal == "}" {
                break;
            }
            let literal = self.register(literal);
            let _ = self.parse_lit(literal); // TODO handle error
        }
        return Operation::Consts;
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
                    CreateIter,
                    IterNext,
                    IterPrev,
                    IterSkip,
                    IterCurrent,
                    Debug
                ]},
                // bytes
                {[PushName, ReturnIf, StoreConst, StoreName, Import, SetAttribute, GetAttribute, CreateObject],
                    self.register(Self::parse_token(text).unwrap().into())},
                {[PushLit],
                    utils::string_to_t(match Self::parse_token(text) {
                        Some(v) if v.chars().all(|c| c.is_numeric()) => v,
                        _ => {
                            text.undo();
                            "".to_string()
                        }
                    }).expect("") // TODO handle parser error :D
                    },
                // option<usize>
                {[CreateList, ListAlloc, ListSet, ListGet],
                    utils::string_to_t(match Self::parse_token(text) {
                        Some(v) if v.chars().all(|c| c.is_numeric()) => v,
                        _ => {
                            text.undo();
                            "".to_string()
                        }
                    }).ok()},
                // usize, option<usize>
                {PushManyLits, {
                    let lit: usize = utils::string_to_t(match Self::parse_token(text) {
                        Some(v) if v.chars().all(|c| c.is_numeric()) => v,
                        _ => {
                            text.undo();
                            "".to_string()
                        }
                    }).expect(""); // TODO handle parser error :D
                    let us: Option<usize> = utils::string_to_t(match Self::parse_token(text) {
                        Some(v) if v.chars().all(|c| c.is_numeric()) => v,
                        _ => {
                            text.undo();
                            "".to_string()
                        }
                    }).ok();
                    Operation::PushManyLits(lit, us)
                }},
                {Call, {
                    match Self::parse_token(text) {
                        Some(arg) => Operation::Call(Some(self.register(arg.to_string()))),
                        _ => Operation::Call(None)
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
                {Object, {
                    let saved_name = self.register(Self::parse_token(text).unwrap());
                    let arity = Self::parse_token(text).unwrap()
                        .parse::<usize>()
                        .expect("arity is not a number or something");
                    let idx = self.instructions.len();
                    match Self::parse_token(text) {
                        Some(bracket) if bracket == "{" => {
                            let b = self.parse_block(text);
                            let op = Operation::Object(saved_name, arity, b);
                            self.constructors.insert(saved_name, op.clone());
                            op
                        }
                        _ => panic!("start blocks with {{ plz")
                    }
                }}
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
                },
                {Consts, {
                    match Self::parse_token(text) {
                        Some(bracket) if bracket == "{" => {
                            self.parse_consts(text)
                        }
                        _ => panic!("start blocks with {{ plz")
                    }
                }}
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

    pub fn parse_lit(&mut self, bytes: &'static [u8]) -> Result<()> {
        let string = unsafe { String::from_utf8_unchecked(bytes.to_vec()) };
        if string.starts_with('[') && string.ends_with(']') {
            // let bytess = &string[1..string.len() - 1];
            todo!("pushing many at a time")
        } else if string.starts_with('"') && string.ends_with('"') {
            let s = &string[1..string.len() - 1];
            let sb = self.register(s.to_owned());
            self.consts.push(sb.into());
            Ok(())
        } else if string == "true" {
            self.consts.push(true.into());
            Ok(())
        } else if string == "false" {
            self.consts.push(false.into());
            Ok(())
        } else if string == "Nil" {
            self.consts.push(Object::nil());
            Ok(())
        } else if string.chars().all(|c| c.is_numeric()) {
            let num: isize = match utils::string_to_t(string) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };
            self.consts.push(num.into());
            Ok(())
        } else if utils::string_is_float_like(string.clone()) {
            let num: f64 = str::parse(&string).unwrap();
            self.consts.push(num.into());
            Ok(())
        } else {
            return Err(ParsingError(utils::display_bytes(bytes)));
        }
    }

    pub fn get_const(&self, idx: usize) -> Result<Object> {
        self.consts.get(idx).ok_or(TodoError).copied()
    }
    // pub fn get_done(&self, pc: &usize) -> Result<&usize, ProgramErrorKind> {
    //     match self.block_returns.get(pc) {
    //         Some(address) => Ok(address),
    //         None => Err(ProgramErrorKind::DoneAddress),
    //     }
    // }
}
