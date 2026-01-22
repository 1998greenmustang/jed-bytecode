use std::collections::{BTreeMap, HashMap};

use crate::{arena::Dropless, object::Object, operation::Operation, utils};

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
        self.instructions.get(idx).unwrap()
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
                18 => Operation::ListPush(program.register(arg)),
                19 => Operation::ListGet(utils::string_to_t(arg).ok()),
                20 => Operation::ListSet(utils::string_to_t(arg).ok()),
                21 => Operation::PushRange,
                22 => Operation::ReturnIfConst(program.register(arg)),
                0 | _ => panic!("No such operation '{}'", op),
            };
            program.instructions.push(operation);
        }
        return program;
    }
}
