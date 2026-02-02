use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{self, BufReader, Read, Write},
};

use crate::{arena::Dropless, object::Object, operation::Operation, utils, MAGIC_NUMBER};

type Arity = usize;
type Index = usize;

pub type MemoKey = (Index, &'static [Object]);
type MemoTable = HashMap<MemoKey, Object>;

pub struct Program {
    pub string_arena: Dropless,
    pub saved_strings: BTreeMap<String, &'static [u8]>,
    pub instructions: Vec<Operation>,
    pub funcs: BTreeMap<&'static [u8], (Index, Arity)>,
    pub memos: MemoTable,
}

impl Program {
    pub fn new() -> Program {
        let mut program = Program {
            string_arena: Default::default(),
            saved_strings: BTreeMap::new(),
            instructions: vec![],
            funcs: BTreeMap::new(),
            memos: HashMap::new(),
        };
        // register keywords/stuff that not be added later
        // probably should be a macro but (:
        program.register("main".to_owned());

        return program;
    }

    pub fn get_main(&self) -> Index {
        let main = self.saved_strings.get("main").unwrap();
        let (idx, _) = self
            .funcs
            .get(main)
            .unwrap_or_else(|| panic!("No main func!"));
        return *idx;
    }
    pub fn get_op(&self, idx: usize) -> &Operation {
        match self.instructions.get(idx) {
            Some(op) => op,
            None => &Operation::Exit,
        }
    }

    pub fn register_bytes(&mut self, byte_str: &[u8]) -> &'static [u8] {
        let string = utils::bytes_to_string(byte_str);
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
        file.write(MAGIC_NUMBER)?;
        for op in self.instructions.clone() {
            match op {
                Operation::BinOp(bin_op_kind) => {
                    let _ = file.write(&[op.into(), bin_op_kind as u8])?;
                }
                Operation::CallBuiltIn(built_in) => {
                    let _ = file.write(&[op.into(), built_in as u8])?;
                }
                Operation::Call(items)
                | Operation::PushLit(items)
                | Operation::PushName(items)
                | Operation::ReturnIf(items)
                | Operation::ReturnIfConst(items)
                | Operation::StoreConst(items)
                | Operation::StoreName(items)
                | Operation::DoForIn(items) => {
                    // op, usize (len), slice
                    let mut data = Vec::<u8>::from(&[op.into()]);
                    data.extend_from_slice(items.len().to_be_bytes().as_slice());
                    data.extend_from_slice(items);
                    let _ = file.write(&data)?;
                }
                Operation::Func(items, arity) => {
                    // op, usize (len), slice, usize (arity)
                    let mut data = Vec::<u8>::from(&[op.into()]);
                    data.extend_from_slice(items.len().to_be_bytes().as_slice());
                    data.extend_from_slice(items);
                    data.extend_from_slice(arity.to_be_bytes().as_slice());
                    let _ = file.write(&data)?;
                }
                Operation::CreateList(option)
                | Operation::ListGet(option)
                | Operation::ListSet(option) => {
                    // op, ok || none, maybe usize
                    let mut data = Vec::<u8>::from(&[op.into(), u8::from(option.is_some())]);
                    match option {
                        Some(num) => data.extend_from_slice(&num.to_be_bytes()),
                        None => {}
                    }
                    let _ = file.write(&data)?;
                }
                _ => {
                    // everything else doesnt have args
                    let _ = file.write(&[op.into()])?;
                }
            }
        }
        Ok(())
    }
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
        let mut reader = BufReader::new(file);
        let mut program = Self::new();
        let mut magic_number: [u8; 3] = [0; 3];
        reader.read(&mut magic_number[..])?;
        assert_eq!(magic_number, MAGIC_NUMBER, "not a jed file");
        let mut op_buffer: [u8; 1] = [0; 1];
        loop {
            let n = reader.read(&mut op_buffer[..])?;
            // println!("buf: {:?} ({})", op_buffer, n);
            if n != 1 {
                break;
            }
            match op_buffer[0] {
                // "bin_op"
                // BinOpKind
                // in file: u8
                1 => {
                    let mut binopbuffer: [u8; 1] = [0; 1];
                    reader.read(&mut binopbuffer[..])?;
                    program
                        .instructions
                        .push(Operation::BinOp(binopbuffer[0].into()))
                }
                // "call_builtin"
                // BuiltIn
                // in file: u8
                3 => {
                    let mut builtinbuffer: [u8; 1] = [0; 1];
                    reader.read(&mut builtinbuffer[..])?;
                    program
                        .instructions
                        .push(Operation::CallBuiltIn(builtinbuffer[0].into()))
                }
                // "call", "push_lit", "push_name", "return_if", "store_const",
                // "store_name", "do_for_in", "return_if_const"
                // &'static [u8]
                // in file: usize (length), [u8; length]
                2 | 4 | 5 | 8 | 9 | 10 | 16 | 22 => {
                    let mut slice_length: [u8; size_of::<usize>()] = [0; size_of::<usize>()];
                    let n = reader.read(&mut slice_length[..])?;
                    assert_eq!(n, size_of::<usize>(), "did not receive enough data");
                    let slice_length: usize = usize::from_be_bytes(slice_length);
                    println!("slice_length {}", slice_length);

                    let mut args: Vec<u8> = vec![0; slice_length];
                    let n = reader.read(&mut args)?;
                    assert_eq!(n, slice_length, "did not receive enough data");
                    let args = program.register_bytes(&args);

                    program.instructions.push((op_buffer[0], args).into());
                }

                // create_list, list_get, list_set
                // Option<usize>
                // in file: Bool, usize
                17 | 19 | 20 => {
                    let mut boolean: [u8; 1] = [0; 1];
                    let n = reader.read(&mut boolean[..])?;
                    assert_eq!(n, 1, "did not receive enough data");
                    let boolean: bool = unsafe { std::mem::transmute(boolean) };
                    if boolean {
                        let mut number: [u8; size_of::<usize>()] = [0; size_of::<usize>()];
                        let n = reader.read(&mut number[..])?;
                        assert_eq!(n, size_of::<usize>(), "did not receive enough data");
                        let number: usize = usize::from_be_bytes(number);
                        program
                            .instructions
                            .push((op_buffer[0], Some(number)).into())
                    } else {
                        program.instructions.push((op_buffer[0], None).into())
                    }
                }
                // func
                // &'static [u8], usize
                // in file: usize (length), [u8; length]
                12 => {
                    let mut slice_length: [u8; size_of::<usize>()] = [0; size_of::<usize>()];
                    let n = reader.read(&mut slice_length[..])?;
                    assert_eq!(n, size_of::<usize>(), "did not receive enough data");
                    let slice_length: usize = usize::from_be_bytes(slice_length);

                    let mut name: Vec<u8> = vec![0; slice_length];
                    let n = reader.read(&mut name)?;
                    assert_eq!(n, slice_length, "did not receive enough data");
                    let name = program.register_bytes(&name);

                    let mut arity: [u8; size_of::<usize>()] = [0; size_of::<usize>()];
                    let n = reader.read(&mut arity[..])?;
                    assert_eq!(n, size_of::<usize>(), "did not receive enough data");
                    let arity: usize = usize::from_be_bytes(arity);

                    program.instructions.push(Operation::Func(name, arity));

                    // register the function
                    program
                        .funcs
                        .insert(name, (program.instructions.len(), arity));
                }

                // push_temp, pop, store_temp, done, exit, do_for, list_push, push_range,
                // get_ptr, read_ptr, set_ptr, get_iter, iter_next, iter_prev, iter_skip,
                // iter_current
                6 | 7 | 11 | 13 | 14 | 15 | 18 | 21 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 30 => {
                    program.instructions.push(op_buffer[0].into())
                }
                _ => break,
            }
        }
        return Ok(program);
    }
    pub fn from_string(text: String) -> Self {
        let mut program = Self::new();
        for line in text.split('\n') {
            if line.trim().is_empty() {
                continue;
            }
            let line_spl: Vec<&str> = line.split(' ').map(|x| x.trim()).collect();
            let op = line_spl[0];
            assert!(Operation::exists(op), "'{}' not a valid operation", op);

            let arg = line_spl[1..].join(" ");

            let op_code = Operation::get_opcode(op);
            let operation = match op_code {
                1 => Operation::BinOp(arg.as_str().into()),
                2 => {
                    let name = program.register(arg);
                    match program.funcs.get(name) {
                        Some(_) => Operation::Call(name),
                        None => panic!(
                            "Call to nonexistent function '{}'",
                            utils::bytes_to_string(name)
                        ),
                    }
                }
                3 => Operation::CallBuiltIn(arg.as_str().into()),
                4 => Operation::PushLit(program.register(arg)),
                5 => Operation::PushName(program.register(arg)),
                6 => Operation::PushTemp,
                7 => Operation::Pop,
                8 => Operation::ReturnIf(program.register(arg)),
                9 => Operation::StoreConst(program.register(arg)),
                10 => Operation::StoreName(program.register(arg)),
                11 => Operation::StoreTemp,
                12 => {
                    let saved_name = program.register(line_spl[1].to_owned());
                    let arity = line_spl[2]
                        .parse::<usize>()
                        .expect("arity is not a number or something");
                    let idx = program.instructions.len();
                    program.funcs.insert(saved_name, (idx, arity));
                    Operation::Func(saved_name, arity)
                }
                13 => Operation::Done,
                14 => Operation::Exit,
                15 => Operation::DoFor,
                16 => Operation::DoForIn(program.register(arg)),
                17 => Operation::CreateList(utils::string_to_t(arg).ok()),
                18 => Operation::ListPush,
                19 => Operation::ListGet(utils::string_to_t(arg).ok()),
                20 => Operation::ListSet(utils::string_to_t(arg).ok()),
                21 => Operation::PushRange,
                22 => Operation::ReturnIfConst(program.register(arg)),
                23 => Operation::GetPtr,
                24 => Operation::ReadPtr,
                25 => Operation::SetPtr,
                26 => Operation::GetIter,
                27 => Operation::IterNext,
                28 => Operation::IterPrev,
                29 => Operation::IterSkip,
                30 => Operation::IterCurrent,
                31 => Operation::Iterate,

                0 | _ => panic!("No such operation '{}'", op),
            };
            program.instructions.push(operation);
        }
        return program;
    }

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
