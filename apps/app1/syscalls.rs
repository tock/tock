#[allow(improper_ctypes)]
extern {
    fn __allow(driver_num: usize, allownum: usize, ptr: *mut (), len: usize) -> isize;
    fn __subscribe(driver_num: usize, subnum: usize, cb: usize, appdata: usize) -> isize;
    fn __command(driver_num: usize, cmdnum: usize, arg1: usize) -> isize;
    fn __wait() -> isize;
}


pub fn allow(driver_num: usize, allownum: usize, ptr: *mut (), len: usize) -> isize {
    unsafe {
        __allow(driver_num, allownum, ptr, len)
    }
}

pub fn command(driver_num: usize, cmdnum: usize, arg1: usize) -> isize {
    unsafe {
        __command(driver_num, cmdnum, arg1)
    }
}

pub fn subscribe(driver_num: usize, cmdnum: usize, callback: usize, appdata: usize) -> isize {
    unsafe {
        __subscribe(driver_num, cmdnum, callback, appdata)
    }
}

pub fn wait() -> isize {
    unsafe {
        __wait()
    }
}
