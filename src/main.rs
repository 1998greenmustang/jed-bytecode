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

use vm::VM;

fn main() {
    let mut vm = VM::from_string(
        "
func fizzbuzz 1
	store_name n

	push_name n
	push_lit 3
	bin_op %
	push_lit 0
	bin_op ==
	store_name div_by_three

	push_name n
	push_lit 5
	bin_op %
	push_lit 0
	bin_op ==
	store_name div_by_five

	push_name div_by_three
	push_name div_by_five
	bin_op &&
	return_if_const fizzbuzz

	push_name div_by_three
	return_if_const fizz
	
	push_name div_by_five
	return_if_const buzz
	
	push_name n
done

func main 0
	push_lit 1
	push_lit 10001
	push_lit 1
	push_range
	create_list 10000
	store_name nums
	
	push_lit \"FizzBuzz\"
	store_const fizzbuzz
	push_lit \"Fizz\"
	store_const fizz
	push_lit \"Buzz\"
	store_const buzz

	push_lit 0
	store_name i
	do_for_in nums
		push_name nums
		push_name i
		list_get
		call fizzbuzz

		push_name nums
		push_name i
		list_set 

		push_name i
		push_lit 1
		bin_op +
		store_name i
	done

	push_name nums
	push_name nums
	call_builtin println
exit
       "
        .to_owned(),
    );

    vm.run();
}
