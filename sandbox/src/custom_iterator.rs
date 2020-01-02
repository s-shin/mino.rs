struct Foo<T> {
    vs: Vec<T>,
}

impl<'a, T> Foo<T> {
    fn iter(&'a self) -> FooIterator<'a, T> {
        FooIterator { i: 0, foo: self }
    }
}

struct FooIterator<'a, T> {
    i: usize,
    foo: &'a Foo<T>,
}

impl<'a, T> Iterator for FooIterator<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.i;
        if i < self.foo.vs.len() {
            self.i += 1;
            Some((i, &self.foo.vs[i]))
        } else {
            None
        }
    }
}

// IntoIterator probably cannot be implemented by following reason.
// https://github.com/rust-lang/rfcs/blob/master/text/1598-generic_associated_types.md

pub fn run() {
    let foo = Foo { vs: vec![1, 2, 3] };
    for (i, v) in foo.iter() {
        println!("{} {}", i, v);
    }
}
