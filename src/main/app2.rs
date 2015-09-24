use core::raw::Slice;

use self::syscalls::wait;

#[derive(Clone,Copy)]
struct Chunk {
    inuse: bool,
    slice: Slice<u8>
}

struct App {
    mem: Slice<u8>,
    offset: usize,
    chunks: [Option<Chunk>; 100]
}

static mut app : *mut App = 0 as *mut App;

pub fn _start(mem_start: *mut u8, mem_size: usize) {
    let myapp = unsafe {
        app = mem_start as *mut App;
        &mut *app
    };
    myapp.mem = Slice { data: mem_start, len: mem_size };
    myapp.offset = 0;
    myapp.chunks = [None; 100];

    init();

    loop {
        wait();
    }
}

fn init(){
}

struct Box<T: ?Sized>{ pointer: *mut T }

impl<T> Box<T> {

    fn new(x: T) -> Box<T> {
        use core::mem;
        let myapp = unsafe { &mut *app };
        let size = mem::size_of::<T>();

        // First, see if there is an available chunk of the right size
        for chunk in myapp.chunks.iter_mut() {
            match *chunk {
                Some(mut chunk) => {
                    if !chunk.inuse && chunk.slice.len >= size {
                        chunk.inuse = true;
                        return Box { pointer: chunk.slice.data as *mut T };
                    }
                },
                None => { }
            }
        }

        // No existing chunks match, so allocate a new one
        match myapp.chunks.iter_mut().filter(|c| c.is_none()).next() {
            Some(slot) => {
                let chunk = Chunk {
                    slice: Slice {
                        data: unsafe {
                            myapp.mem.data.offset(myapp.offset as isize)
                        },
                        len: size
                    },
                    inuse: true
                };
                myapp.offset += size;
                *slot = Some(chunk);
                Box{ pointer: chunk.slice.data as *mut T }
            },
            None => {
                panic!("OOM")
            }
        }
    }
}

mod syscalls {

    #[allow(improper_ctypes)]
    extern {
        fn __allow(driver_num: usize, allownum: usize, ptr: *mut (), len: usize) -> isize;
        fn __subscribe(driver_num: usize, subnum: usize, cb: usize) -> isize;
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

    pub fn subscribe(driver_num: usize, cmdnum: usize, callback: usize) -> isize {
        unsafe {
            __subscribe(driver_num, cmdnum, callback)
        }
    }

    pub fn wait() -> isize {
        unsafe {
            __wait()
        }
    }
}

mod tmp006 {
    use super::syscalls::{command, subscribe};

    pub fn enable_tmp006() {
        command(2, 0, 0);
    }

    pub fn subscribe_temperature(f: fn(i16)) {
        subscribe(2, 0, f as usize);
    }
}

mod console {
    use super::syscalls::{allow, command, subscribe};

    pub fn putc(c: char) {
        command(0, 0, c as usize);
    }

    pub fn subscribe_read_line(buf: *mut u8, len: usize,
                               f: fn(usize, *mut u8)) -> isize {
        let res =  allow(0, 0, buf as *mut (), len);
        if res < 0 {
            res
        } else {
            subscribe(0, 0, f as usize)
        }
    }

    pub fn subscribe_write_done(f: fn()) -> isize {
        subscribe(0, 1, f as usize)
    }
}

mod gpio {
    use super::syscalls::command;

    pub fn enable_pin(pin: usize) {
        command(1, 0, pin);
    }

    pub fn set_pin(pin: usize) {
        command(1, 2, pin);
    }

    pub fn clear_pin(pin: usize) {
        command(1, 3, pin);
    }

    pub fn toggle_pin(pin: usize) {
        command(1, 4, pin);
    }
}

