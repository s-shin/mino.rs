trait Foo {
    fn foo(&self) -> i32;
}

struct MyFoo {
    i: i32,
}

impl Foo for MyFoo {
    fn foo(&self) -> i32 {
        self.i
    }
}

fn call_foo(foo: impl Foo) -> i32 {
    foo.foo()
}

fn call_foo_ref(foo: &impl Foo) -> i32 {
    foo.foo()
}

pub fn run() {
    println!(
        "{} {}",
        call_foo(MyFoo { i: 0 }),
        call_foo_ref(&MyFoo { i: 1 })
    );
}
