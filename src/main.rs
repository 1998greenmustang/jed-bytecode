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

	push_lit 0
	store_name a
	
	push_lit 1
	store_name b

	push_name n
	do_for
		push_name b
		store_name a

		push_name a
		push_name b
		bin_op +
		store_name b
	done

	push_name a
done
	
		
done

func main 0
	push_lit 0
	push_lit 1
	push_lit 2
	push_lit 3
	push_lit 4
	create_list 5
	store_name nums

	push_lit 0
	store_name i
	push_lit 5
	do_for
		push_name nums
		push_name i
		list_get
		call fib

		push_name nums
		push_name i
		list_set
			
		push_name nums
		push_name i
		push_lit 1
		bin_op +
		store_name i
	done

	push_name nums
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
