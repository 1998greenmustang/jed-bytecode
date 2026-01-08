mod arena;
// mod file;
mod binops;
mod builtin;
mod frame;
mod indexmap;
mod map;
mod mutable;
mod object;
mod operation;
mod program;
mod stack;
mod vm;

// use file::saveable::Saveable;
use vm::VM;

fn main() {
    let mut vm = VM::from_string(
        "
func fib 1
	store_name n
	
	push_name n
	push_lit 1
	bin_op <=
	return_if n
	
	push_name n
	push_lit 2
	bin_op -
	call fib

	push_name n
	push_lit 1
	bin_op -
	call fib
	
	bin_op +
done

func main 0
	push_lit 92
	call fib
	call_builtin println
exit
       "
        .to_owned(),
    );

    // match vm.program.save("./fib.jbc") {
    //     Ok(_) => println!("saed as a hell"),
    //     Err(_) => todo!(),
    // }

    // let program = program::Program::load("./fib.jbc");

    // match program {
    //     Ok(p) => {
    //         println!("loaded as a hell");
    //         let mut vm = VM::new(p);
    //         vm.run();
    //     }
    //     Err(_) => todo!(),
    // }

    // // let program = match unsafe { load_byc("./byc.jbc") } {
    // //     Ok(p) => p,
    // //     Err(_) => todo!(),
    // // };
    // println!("after");
    // // println!("{program:?}");
    // let mut vm = VM::new(program);
    println!("running as a hell");
    vm.run();
}
