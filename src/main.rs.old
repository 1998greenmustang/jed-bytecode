mod arena;
mod frame;
mod literal;
mod map;
mod object;
mod operation;
mod program;
mod stack;
mod vm;
use std::{
    fs::File,
    io::{self, Read, Write},
    mem,
};

use program::Program;
use vm::VM;
unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    core::slice::from_raw_parts((p as *const T) as *const u8, core::mem::size_of::<T>())
}

fn save_byc(filepath: &str, data: &[u8]) -> io::Result<()> {
    let mut file = File::create(filepath)?;
    file.write_all(data)?;
    Ok(())
}

unsafe fn load_byc(filepath: &str) -> io::Result<Program> {
    let mut file = File::open(filepath)?;

    let bytes: &mut [u8; mem::size_of::<Program>()] = &mut [0 as u8; mem::size_of::<Program>()];
    File::read(&mut file, bytes)?;
    // let data: *const [u8; mem::size_of::<Program>()] =
    //     bytes as *const [u8; mem::size_of::<Program>()];

    let p: *const [u8; std::mem::size_of::<Program>()] =
        bytes as *const [u8; std::mem::size_of::<Program>()];
    let s: Program = unsafe { std::mem::transmute(*p) };
    Ok(s)
}

fn main() {
    let mut vm = VM::from_string(
        "func fib\n\t\
             store_name n\n\t\

             push_name n\n\t\
             push_lit 1\n\t\
          	bin_op <=\n\t\
          	return_if n\n\n\t\

          	push_name n\n\t\
          	push_lit 2\n\t\
          	bin_op -\n\t\
          	call fib\n\t\

          	push_name n\n\t\
          	push_lit 1\n\t\
          	bin_op -\n\t\
          	call fib\n\n\t\

          	bin_op +\n\
          done\n\

          func main\n\t\
              push_lit 25\n\t\
              call fib\n\t\
              call_builtin println\n\
          done"
            .to_owned(),
    );

    let bytes: &[u8] = unsafe { any_as_u8_slice(&vm.program) };
    let res = save_byc("./byc.jbc", bytes);
    match res {
        Ok(_) => {}
        Err(_) => todo!(),
    }

    // let program = match unsafe { load_byc("./byc.jbc") } {
    //     Ok(p) => p,
    //     Err(_) => todo!(),
    // };
    // println!("after");
    // // println!("{program:?}");
    // let mut vm = VM::new(program);
    vm.run();
}
