use super::syscalls::wait;
use super::boxed::BoxMgr;

pub struct App {
    pub memory: BoxMgr
}

pub static mut app : *mut App = 0 as *mut App;

pub fn _start(mem_start: *mut u8, mem_size: usize) {
    let myapp = unsafe {
        app = mem_start as *mut App;
        &mut *app
    };
    let appsize = ::core::mem::size_of::<App>();
    myapp.memory = BoxMgr::new(mem_start, mem_size, appsize);

    init();

    loop {
        wait();
    }
}

fn init() {
    use super::console::{print, puts};
    
    puts("Hello\r\n");
    print(format_args!("Welcome to Tock\r\n"));
}

