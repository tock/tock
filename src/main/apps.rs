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

pub fn app1_init() {
    command(1, 0, 'c' as usize);
    command(1, 0, 'm' as usize);
    loop {
        command(1, 0, 'd' as usize);
        //wait();
    }
}

