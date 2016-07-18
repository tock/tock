use process::{AppId, Callback};
use hil::Driver;

pub struct Noop {
    count: isize,
}

impl Noop {
    pub fn new() -> Noop {
        Noop {
            count: 0
        }
    }
}
impl Driver for Noop {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        -1
    }

    fn command(&self, command_num: usize, data: usize, _: AppId) -> isize {
        self.count
    }
}

