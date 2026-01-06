extern crate criterion;
extern crate jar;
use criterion::{criterion_group, criterion_main, Criterion};
use jar::vm::{self, VM};
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("vm - mystack", |b| {
        b.iter(|| {
            black_box(VM::from_string(
                "func fib 1\n\t\
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

          func main 0\n\t\
              push_lit 35\n\t\
              call fib\n\t\
          done"
                    .to_owned(),
            ))
            .run();
        })
    });
}
criterion_group!(benches, criterion_benchmark,);
criterion_main!(benches);
