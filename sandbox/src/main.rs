extern crate rand;

mod state_machine {
    use rand::prelude::*;

    struct Data {
        pub n: u32,
    }

    trait State {
        fn update(&mut self, data: &mut Data) -> Option<Box<dyn State>>;
    }

    struct S1 {}

    impl State for S1 {
        fn update(&mut self, data: &mut Data) -> Option<Box<dyn State>> {
            data.n += 1;
            if random() {
                None
            } else {
                Some(Box::new(S2 {}))
            }
        }
    }

    struct S2 {}

    impl State for S2 {
        fn update(&mut self, data: &mut Data) -> Option<Box<dyn State>> {
            data.n += 10;
            None
        }
    }

    pub fn run() {
        let mut data = Data { n: 0 };
        let mut s: Box<dyn State> = Box::new(S1 {});
        for _i in 0..10 {
            if let Some(next) = s.update(&mut data) {
                s = next;
            }
        }
        println!("{}", data.n);
    }
}

mod associated_trait {
    trait Foo {
        type Value;
        fn value(&self) -> Self::Value;
    }
    struct Bar<Value> {
        value: Value,
    }
    impl<Value: Copy> Foo for Bar<Value> {
        type Value = Value;
        fn value(&self) -> Self::Value {
            self.value
        }
    }
    pub fn run() {
        let x = Bar::<u32> { value: 100 };
        println!("{}", x.value());
    }
}

mod trait_method_return_self {
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
}

mod impl_trait {
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
}

fn main() {
    state_machine::run();
    associated_trait::run();
    trait_method_return_self::run();
    impl_trait::run();
}
