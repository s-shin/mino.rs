trait Foo {
    fn foo(&self) -> Self;
}

struct MyFoo {
    i: i32,
}

impl Foo for MyFoo {
    fn foo(&self) -> MyFoo {
        MyFoo { i: self.i + 1 }
    }
}

pub fn run() {
    println!("{}", MyFoo { i: 0 }.foo().foo().foo().i);
}
