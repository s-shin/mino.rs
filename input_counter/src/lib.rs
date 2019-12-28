extern crate num_traits;

use num_traits::NumAssign;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Copy, Clone)]
pub enum InputState {
    Inactive,
    Delay,
    Repeat,
    End,
}

#[derive(Debug, Copy, Clone)]
pub struct InputCounter<Num = u8> {
    opt_repeat: Num,
    opt_first_delay: Num,
    state: InputState,
    can_handle: bool,
    is_handled: bool,
    n: Num,
}

impl<Num: NumAssign + Copy> InputCounter<Num> {
    pub fn new(repeat: Num, first_delay: Num) -> Self {
        Self {
            opt_repeat: repeat,
            opt_first_delay: if first_delay == Num::zero() {
                repeat
            } else {
                first_delay
            },
            state: InputState::Inactive,
            can_handle: false,
            is_handled: false,
            n: Num::zero(),
        }
    }
    pub fn update(&mut self, active: bool) {
        if !active {
            self.state = InputState::Inactive;
            self.can_handle = false;
            self.is_handled = false;
            self.n = Num::zero();
            return;
        }
        if self.can_handle && !self.is_handled {
            return;
        }
        self.is_handled = false;
        match self.state {
            InputState::Inactive => {
                self.can_handle = true;
                self.state = if self.opt_repeat.is_zero() {
                    InputState::End
                } else {
                    InputState::Delay
                };
            }
            InputState::Delay => {
                self.n += Num::one();
                self.can_handle = self.n == self.opt_first_delay;
                if self.can_handle {
                    self.n = Num::zero();
                    self.state = InputState::Repeat;
                }
            }
            InputState::Repeat => {
                self.n = (self.n + Num::one()) % self.opt_repeat;
                self.can_handle = self.n.is_zero();
            }
            InputState::End => {
                // do nothing
            }
        }
    }
    pub fn can_handle(&self) -> bool {
        self.can_handle
    }
    pub fn handle(&mut self) -> bool {
        if self.can_handle {
            self.can_handle = false;
            self.is_handled = true;
            return true;
        }
        false
    }
}

pub trait Contains<T> {
    fn contains(&self, v: T) -> bool;
}

#[derive(Debug, Clone, Default)]
pub struct InputManager<Input: Eq + Hash, Num> {
    inputs: HashMap<Input, InputCounter<Num>>,
}

impl<Input: Eq + Hash + Clone, Num: NumAssign + Copy> InputManager<Input, Num> {
    pub fn register(
        &mut self,
        input: Input,
        counter: InputCounter<Num>,
    ) -> Option<InputCounter<Num>> {
        self.inputs.insert(input, counter)
    }
    pub fn update(&mut self, inputs: impl Contains<Input>) {
        for (i, c) in &mut self.inputs {
            c.update(inputs.contains(i.clone()));
        }
    }
    pub fn can_handle(&self, input: Input) -> bool {
        if let Some(c) = self.inputs.get(&input) {
            c.can_handle()
        } else {
            false
        }
    }
    pub fn handle(&mut self, input: Input) -> bool {
        if let Some(c) = self.inputs.get_mut(&input) {
            c.handle()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_shot() {
        let mut c = InputCounter::new(0, 0);
        assert!(!c.can_handle);
        c.update(true);
        assert!(c.can_handle);
        c.update(true); // ignored
        assert!(c.can_handle);
        assert!(c.handle());
        assert!(!c.can_handle);
        c.update(true);
        assert!(!c.can_handle);
        c.update(false);
        assert!(!c.can_handle);
        c.update(true);
        assert!(c.can_handle);
    }
    #[test]
    fn repeatable() {
        let mut c = InputCounter::new(1, 0);
        assert!(!c.can_handle);
        c.update(true);
        assert!(c.can_handle);
        c.update(true);
        assert!(c.can_handle);
        assert!(c.handle());
        assert!(!c.can_handle);
        c.update(true);
        assert!(c.can_handle);
        c.update(true); // ignored
        assert!(c.can_handle);
        assert!(c.handle());
        assert!(!c.can_handle);
    }
    #[test]
    fn repeatable2() {
        let mut c = InputCounter::new(2, 3);
        assert!(!c.handle());
        c.update(true);
        assert!(c.handle());
        c.update(true);
        assert!(!c.handle());
        c.update(true);
        assert!(!c.handle());
        c.update(true);
        assert!(c.handle());
        c.update(true);
        assert!(!c.handle());
        c.update(true);
        assert!(c.handle());
    }
}
