#[allow(improper_ctypes)]
extern {
    fn __subscribe(driver_num: usize, arg1: usize, arg2: fn());
    fn __command(driver_num: usize, subnum: usize, arg1: usize);
    fn __wait(a: usize, b: usize, c: usize);
}

fn command(driver_num: usize, subnum: usize, arg1: usize) {
    unsafe {
        __command(driver_num, subnum, arg1);
    }
}

fn wait() {
    unsafe {
        __wait(0, 0, 0);
    }
}

pub mod app1 {
    use super::{wait,command};

    pub fn _start() {
        init();
        loop {
            wait();
        }
    }

    fn init() {
        command(0, 0, '>' as usize);
        command(0, 0, ' ' as usize);
        command(1, 0, 0);
        command(1, 2, 0);
    }
}

