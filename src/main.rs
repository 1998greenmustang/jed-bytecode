extern crate lexopt;
mod arena;
mod binops;
mod builtin;
mod error;
mod frame;
mod indexmap;
mod map;
mod object;
mod operation;
mod program;
mod span;
mod stack;
mod utils;
mod vm;
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::Path,
};

use program::Program;
use vm::VM;

const HELP: &str = "jed [COMMAND] [OPTIONS]";
const MAGIC_NUMBER: &[u8] = "JED".as_bytes();

// jed commands:
//  - compile (string -> bytecode)
//  - run (string | bytecode)
//  - validate (string | bytecode)
//
// flags:
//  - --output/-o (path to cache dir)
//  - --debug

struct Args {
    command: Command,
    file: String,
    output: String,
    debug: bool,
}

enum Command {
    Compile,
    Run,
    Validate,
}

fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut command: Option<Command> = None;
    let mut file: Option<String> = None;
    let mut output: Option<String> = None;
    let mut debug = false;

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Value(cmd) if command.is_none() => {
                let cmd = cmd.string()?;
                command = match cmd.as_str() {
                    "compile" => Some(Command::Compile),
                    "run" => Some(Command::Run),
                    "validate" => Some(Command::Validate),
                    _ => Err(format!("unknown command '{}'", cmd))?,
                }
            }
            Value(fl) if file.is_none() => file = Some(fl.string()?),
            Short('o') | Long("output") => output = Some(parser.value()?.parse()?),
            Long("debug") => debug = true,
            Short('h') | Long("help") => {
                println!("{}", HELP);
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }
    Ok(Args {
        command: command.unwrap_or_else(|| panic!("missing command")),
        file: file.unwrap_or_else(|| panic!()),
        output: output.unwrap_or(".jedcache/".to_owned()),
        debug,
    })
}

fn compile(text: String, output: &Path) -> io::Result<()> {
    let program = Program::from_string(text);

    let file = File::create(output);
    match file {
        Ok(mut f) => program.to_file(&mut f),
        Err(_) => todo!(),
    }
}

fn main() -> io::Result<()> {
    let opts = match parse_args() {
        Ok(opts) => opts,
        Err(e) => panic!("{}", e),
    };
    let filepath = Path::new(&opts.file);
    let output = Path::new(&opts.output);
    // output doesnt work
    // but yk

    match opts.command {
        Command::Compile => {
            let mut file = File::open(filepath)?;
            let mut string = String::new();
            file.read_to_string(&mut string)?;
            let program = Program::from_string(string);

            let mut output_filepath = output.to_path_buf();
            let file_name = filepath.file_stem().unwrap();
            output_filepath.set_file_name(file_name);
            output_filepath.set_extension("jbc");

            let mut output_file = File::create(&output_filepath)?;
            program.to_file(&mut output_file)?;
            println!("wrote to {}", output_filepath.to_str().unwrap());
        }
        Command::Run => {
            let mut file = File::open(filepath)?;
            let mut magic_number_buffer = [0 as u8; 3];
            let n = file.read(&mut magic_number_buffer)?;
            if n == 3 && magic_number_buffer == MAGIC_NUMBER {
                file.seek(SeekFrom::Start(0))?;
                let mut vm = VM::from_file(&mut file)?;
                vm.run();
            } else {
                file.seek(SeekFrom::Start(0))?;
                let mut string = String::new();
                file.read_to_string(&mut string)?;
                let mut vm = VM::from_string(string);
                vm.run();
            }
        }
        Command::Validate => {
            let mut file = File::open(filepath)?;
            let mut magic_number_buffer = [0 as u8; 3];
            let n = file.read(&mut magic_number_buffer)?;
            if n == 3 && magic_number_buffer == MAGIC_NUMBER {
                file.seek(SeekFrom::Start(0))?;
                let _program = Program::from_file(&mut file)?;
            } else {
                file.seek(SeekFrom::Start(0))?;
                let mut string = String::new();
                file.read_to_string(&mut string)?;
                let _program = Program::from_string(string);
            }
        }
    }
    Ok(())
}
