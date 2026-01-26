# Jed Bytecode

This repo contains a stack-machine based bytecode interpreter for my self-study projects.
This will serve as the backend for many of my own languages.

## Building

`cargo build --release`

or use `cargo run`, I'm not your dad.

## Running

There are three commands:

 - Compile: convert string Jed Bytecode to Bytecode
 - Run: interpret either string Jed Bytecode or Bytecode
 - Validate: parse string Jed Bytecode or Bytecode


There are also options, but they do nothing:
 - --output, -o: path to the directory to save compiled files
 - --debug: print debug statements

To run examples with the built program:
```sh
  jed run ./examples/helloworld.jed
```
or with `cargo run`:
```sh
  cargo run run ./examples/helloworld.jed
```

### Hello World in Jed Bytecode
```text
func main 0
  push_lit "Hello world!"
  call_builtin println
done
```

## Future and Rational

This project is meant to serve as a simple RTS for various languages I decide to explore design-wise in the future.
I started this project when I was working on my firstever language 'jed' then once it could finally generate code, I didn't have a backend ready.
So, I made a very simple AST crawler runtime that was very annoying. So I wanted to create a bytecode interpreter that I don't have to worry about.

That's what this project is. This is not to replace any other backend, it's for my own exploration of what languages are.

## The Language

I am a Python developer by trade, so I _tried_ to take some inspiration from its bytecode. 
I don't think I did a very good job of actually doing that, **but** I enjoy the way it works.

### It's bad.

Objects get registered into static memory and never get removed. I...uh... added a deallocator arena but I never drop.

### Structure

The VM contains a map of constants, 1 register, an object stack, a call stack, and a memory arena.

The register is referred to as the `temp` storage.

The call stack gets new frames with each `call` and each loop in a `do_for` or `do_for_in`.


### Types

#### Literals

| Type in JBC | Type in Rust             |
| ---         | ---                      | 
| Integer     | `isize`                  |
| Float       | `(isize, usize)`         |
| String      | `&'static [u8]`          |
| Bool        | `bool`                   |
| Pointer     | `*mut &'static Object'`  |
| List        | `*const Object, usize`   |
| Iterator    | `(&'static Object, usize)`|
| Nil         | -                        |
 
There is 1 truly internal types: `Func`. `Func` is essentially used as more of a "label" to jumping between and forth.

### Operations

| Operation | Description | Argument | Stack Arguments |
| --- | --- | --- | --- |
| bin_op       | Apply a binary operation on the top 2    | BinOp | 2 same-typed operands |
| call         | Call a custom defined function           | Function name | Arguments (or not!) required of the function |
| call_builtin | Call a builtin function (ie println)     | Function name | Arguments (or not!) required of the function |
| push_lit     | Push a literal to the stack              | Literal | - |
| push_name    | Push a stored local to the stack         | Variable name | - |
| push_temp    | Push the `temp` storage to the stack     | - | - |
| pop          | Pop the top element off the stack        | - | - |
| return_if    | Return from function call if top element is true | Optional variable name | Bool |
| store_const  | Store a constant with a name             | Variable name | Object to store |
| store_name   | Store a variable with a name             | Variable name | Object to store |
| store_temp   | Store an object to `temp` storage        | - | Object to store |
| func         | Define a function                        | Function name, # of args | - |
| done         | Closing delimited for `do_for` and funcs | - | Return value (if wanted) |
| exit         | Stop and clear VM                        | - | - |
| do_for       | Loop for `x` times                       | - | Positive integer |
| do_for_in    | Loop for `len(LIST)` times               | Name of stored list | - |
| create_list  | Create a list of the top `x` elements    | Number of stack elements | The stack elements to use |
| list_push    | Push onto a list                         | - | Object, List, Index |
| list_get     | Get from a list                          | - | List, Index |
| list_set     | Set an index of a list                   | - | Object, List, Index |
| push_range   | Push int literals from `x` to `y` by `z` | - | Ints: Start, End, Steps |
| return_if_const | Return from function call with a const| Constant name | Bool |
| get_ptr      | Add a pointer to top object to stack     | | Any object |
| read_ptr     | Read the value from a pointer            | | Pointer |
| set_ptr      | Change the data at the pointer           | | Pointer, any object |
| get_iter     | Get iterable object from list on stack   | | List |
| iter_next    | Push current index and upcycle iterator  | | Iterator |
| iter_prev    | Push current index and downcycle iterator| | Iterator |
| iter_skip    | Skip n indeces of iterator               | | Iterator, Integer |
| iter_current | Push current index                       | | Iterator |
