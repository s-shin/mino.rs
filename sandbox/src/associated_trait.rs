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
