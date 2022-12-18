// taken from https://github.com/rust-lang/miri/blob/master/tests/pass/calls.rs

extern crate intrinsics;
use intrinsics::*;

fn call() -> i32 {
    fn increment(x: i32) -> i32 {
        x + 1
    }
    increment(1)
}

fn call_generic() -> (i16, bool) {
    fn id<T>(t: T) -> T {
        t
    }
    (id(42), id(true))
}


const fn foo(i: i64) -> i64 {
    *&i + 1
}

fn const_fn_call() -> i64 {
    let x = 5 + foo(5);
    x
}

fn main() {
    print(call());
    print(call_generic().0);
    print(call_generic().1);
    print(const_fn_call());
}
