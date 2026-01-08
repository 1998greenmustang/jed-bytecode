# Jed Bytecode

This repo contains a stack-machine based bytecode interpreter for my self-study projects.
This will serve as the backend for many of my own languages.

## Running

Currently, this does NOT accept files as input. It produces a binary using whatever "byte"-code is in `src/main.rs`.

To run examples, copy & paste from the `examples` directory into the `VM::from_string()` call in `src/main.rs`.

Then use `cargo run` or `cargo build --release` for a more optimized version.

### Hello World in Jed Bytecode
```text
func main
  push_lit "Hello world!"
  call_builtin println
done
```

## Future and Rational

In the future, this will be able to accept files as input so other languages can compile into the bytecode served here.

This project is meant to serve as a simple RTS for various languages I decide to explore design-wise in the future.
I started this project when I was working on my firstever language 'jed' then once it could finally generate code, I didn't have a backend ready.
So, I made a very simple AST crawler runtime that was very annoying. So I wanted to create a bytecode interpreter that I don't have to worry about.

That's what this project is. This is not to replace any other thing like QBE or LLVM or even BEAM, it's for my own exploration of what languages are.

## The Language

I am a Python developer by trade, so I _tried_ to take some inspiration from its bytecode. 
I don't think I did a very good job of actually doing that, **but** I enjoy the way it works.

### It's bad.

A major issue of this is the memory usage. Every thing is a considered an "object" which is a tuple struct of `(ObjectKind, &'static [u8])`
When creating a new object, without exception, the VM will register the data into a _Dropless_ Arena.

This includes lists, which are defined as `&'static [Object]`. Each object in a list are a wrapped `MutablePtr`, which is a `usize`.
Any change to a list, will re-register the objects into the arena.

### Structure

The VM contains a map of constants, 1 register, an object stack, a call stack, and a "heap".

The register is referred to as the `temp` storage.

The call stack gets new frames with each `call` and each loop in a `do_for` or `do_for_in`.

The "heap" is just a `HashMap<usize, MutableObject>` to store dynamic data.

### Types

#### Literals

| Type in JBC | Type in Rust        |
| ---         | ---                 |
| Integer     | `i64`               |
| Float       | `f64`               |
| String      | `&[str]`            |
| Bool        | `"true" \| "false"` |

There are 2 truly internal types: `Func` and `MutablePtr`. `Func` is essentially used as more of a "label" to jumping between and forth.
`MutablePtr` is a `usize` which points to an object in `VM.heap`.

### Operations

| Operation | Description | Argument | Stack Arguments |
| --- | --- | --- | --- |
| bin_op       | Apply a binary operation on the top 2 | BinOp | 2 same-typed operands |
| call         | Call a custom defined function        | Function name | Arguments required of the function |
| call_builtin | Call a builtin function (ie println)  | Function name | Arguments required of the function |
| push_lit     | Push a literal to the stack           | Literal | - |
| push_name    | Push a stored local to the stack      | Variable name | - |
| push_temp    | Push the `temp` storage to the stack  | - | - |
| pop          | Pop the top element off the stack     | - | - |
| return_if    | Return from function call if top element is true | Optional variable name | Bool |
| store_const  | Store a constant with a name          | Variable name | Object to store |
| store_name   | Store a variable with a name          | Variable name | Object to store |
| store_temp   | Store an object to `temp` storage     | - | Object to store |
| func         | Define a function                     | Function name, # of args | - |
| done         | Closing delimited for `do_for` and funcs | - | Return value (if wanted) |
| exit         | Stop and clear VM                     | - | - |
| do_for       | Loop for `x` times                    | - | Positive integer |
| do_for_in    | Loop for `len(LIST)` times            | Name of stored list | - |
| create_list  | Create a list of the top `x` elements | Number of stack elements | The stack elements to use |
| list_push    | Push onto a list                      | - | Object, List, Index |
| list_get     | Get from a list                       | - | List, Index |
| list_set     | Set an index of a list                | - | Object, List, Index |
| push_range   | Push int literals from `x` to `y` by `z` | - | Ints: Start, End, Steps |
| return_if_const | Return from function call with a const | Constant name | Bool |
  
