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
    print!("You have {} days left...\r\n", 1234);
    print!("Welcome to Tock\r\n");
    let stats = (unsafe { &*app }).memory.stats();
    print!("Memory Stats:\r\n");
    print!("\tNum Allocated: {}\r\n", stats.num_allocated);
    print!("\tAllocated Bytes: {}\r\n", stats.allocated_bytes);
    print!("\tActive: {}\r\n", stats.active);
    print!("\r\n");
}

