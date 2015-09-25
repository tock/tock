use core::fmt::{self, Write};
use core::raw::Slice;
use core::ops::{Deref,DerefMut};

use self::syscalls::wait;

#[derive(Clone,Copy)]
struct Chunk {
    inuse: bool,
    slice: Slice<u8>
}

struct App {
    mem: Slice<u8>,
    offset: usize,
    chunks: [*mut Chunk; 100]
}

static mut app : *mut App = 0 as *mut App;

pub fn _start(mem_start: *mut u8, mem_size: usize) {
    let myapp = unsafe {
        app = mem_start as *mut App;
        &mut *app
    };
    myapp.mem = Slice { data: mem_start, len: mem_size };
    myapp.offset = 0;
    myapp.chunks = [0 as *mut Chunk; 100];

    init();

    loop {
        wait();
    }
}

fn dummy() {}

fn init() {
    use self::console::{puts, subscribe_write_done};
    //print(format_args!("Welcome to Tock\r\n"));
    puts("Welcome to Tock\r\n");
}

struct Box<T: ?Sized>{ pointer: *mut T }

impl<T> Box<T> {

    pub unsafe fn uninitialized(size: usize) -> Box<T> {
        use core::mem;
        unsafe {
            let myapp = &mut *app;

            // First, see if there is an available chunk of the right size
            for chunk in myapp.chunks.iter_mut().filter(|c| !c.is_null()) {
                let c = &mut **chunk;
                if !c.inuse && c.slice.len >= size {
                    c.inuse = true;
                    return Box { pointer: c.slice.data as *mut T };
                }
            }

            // No existing chunks match, so allocate a new one
            match myapp.chunks.iter_mut().filter(|c| c.is_null()).next() {
                Some(slot) => {
                    let freemem = myapp.mem.data.offset(myapp.offset as isize);
                    let chunk = &mut *(freemem as *mut Chunk);
                    myapp.offset += mem::size_of::<Chunk>();

                    chunk.slice = Slice {
                        data: unsafe {
                            myapp.mem.data.offset(myapp.offset as isize)
                        },
                        len: size
                    };
                    myapp.offset += size;

                    chunk.inuse = true;
                    *slot = chunk;
                    let data = chunk.slice.data as *mut T;
                    Box{ pointer: data }
                },
                None => {
                    panic!("OOM")
                }
            }
        }
    }

    pub fn new(x: T) -> Box<T> {
        use core::mem;
        let size = mem::size_of::<T>();
        let mut d = unsafe { Self::uninitialized(size) };
        *d = x;
        d
    }
}

impl<T> Deref for Box<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            &*self.pointer
        }
    }
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.pointer
        }
    }
}

impl<T: ?Sized> Drop for Box<T> {
    fn drop(&mut self) {
        unsafe {
            use core::mem;
            let chunk_size = mem::size_of::<Chunk>() as isize;
            let chunk = (self.pointer as *mut T as *mut u8)
                            .offset(0 - chunk_size) as *mut Chunk;
            (&mut *chunk).inuse = false;
        }
    }
}

unsafe fn uninitialized_box_slice<T>(size: usize) -> Box<&'static mut [T]> {
    use core::mem;
    use core::slice;
    let slice_size = mem::size_of::<Slice<u8>>();
    let mut bx : Box<Slice<u8>> =
        Box::uninitialized(slice_size + size * mem::size_of::<T>());
    bx.len = size;
    bx.data = (bx.pointer as *const u8).offset(slice_size as isize);
    mem::transmute(bx)
}

pub struct String { bx: Option<Box<&'static mut [u8]>> }

impl String {
    pub fn new(x: &str) -> String {
        if x.len() == 0 {
            return String { bx: None };
        }
        unsafe {
            let mut slice = uninitialized_box_slice(x.len());
            for (i,c) in x.bytes().enumerate() {
                slice[i] = c;
            }
            String { bx: Some(slice) }
        }
    }

    pub fn len(&self) -> usize {
        self.bx.as_ref().map(|b| b.len()).unwrap_or(0)
    }

    pub fn as_str(&self) -> &str {
        use core::mem;
        use core::raw::Repr;

        match self.bx.as_ref() {
            Some(bx) => unsafe {
                mem::transmute((*bx.pointer).repr())
            },
            None => ""
        }
    }
}

impl Write for String {
    fn write_char(&mut self, c: char) -> fmt::Result {
        let charlen = c.len_utf8();
        let oldlen = self.len();
        let mut newbox = unsafe { uninitialized_box_slice(oldlen + charlen) };
        
        self.bx.as_ref().map(|bx| {
            for (i, c) in bx.iter().enumerate() {
                newbox[i] = *c;
            }
        });

        match charlen {
            1 => newbox[oldlen] = c as u8,
            len => {
                c.encode_utf8(&mut newbox[oldlen..]);
            }
        }

        self.bx = Some(newbox);
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        let oldlen = self.len();
        let mut newbox = unsafe { uninitialized_box_slice(oldlen + s.len()) };

        self.bx.as_ref().map(|bx| {
            for (i, c) in bx.iter().enumerate() {
                newbox[i] = *c;
            }
        });

        for (i,c) in s.bytes().enumerate() {
            newbox[i + oldlen] = c;
        }

        self.bx = Some(newbox);

        Ok(())
    }
}

pub trait ToString {
    fn to_string(&self) -> String;
}

impl<T: fmt::Display + ?Sized> ToString for T {
    fn to_string(&self) -> String {
        use core::fmt::Write;
        let mut buf = String::new("");
        let _ = buf.write_fmt(format_args!("{}", self));
        buf
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
    use core::mem;
    use super::syscalls::*;
    use super::*;

    pub fn putc(c: char) {
        command(0, 0, c as usize);
    }

    pub fn print(args: ::core::fmt::Arguments) {
        use core::fmt::Write;
        let mut buf = String::new("");
        buf.write_fmt(args);
        puts("hello");
    }

    pub fn puts(string: &str) {
        unsafe {
            let bstr = String::new(string);
            allow(0, 1, bstr.as_str() as *const str as *mut (), string.len());
            mem::forget(bstr);
            command(0, 1, 0);
        }
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

