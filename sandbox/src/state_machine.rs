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
