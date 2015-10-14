use super::syscalls::wait;
use super::boxed::BoxMgr;

pub struct App {
    pub memory: BoxMgr
}

pub static mut app : *mut App = 0 as *mut App;

pub fn _start(mem_start: *mut u8, mem_size: usize) {
    use core::mem::size_of;

    let myapp = unsafe {
        app = mem_start as *mut App;
        &mut *app
    };
    let appsize = size_of::<App>();
    myapp.memory = BoxMgr::new(mem_start, mem_size, appsize);

    init();

    loop {
        wait();
    }
}

fn init() {
    print!("Welcome to Tock!\r\n");

    let stats = (unsafe { &*app }).memory.stats();
    print!("Memory Stats:{}\r\n", "");
    print!("\tNum Allocated: {}\r\n", stats.num_allocated);
    print!("\tNum Allocs: {}\r\n", stats.allocs);
    print!("\tDrops: {}\r\n", stats.drops);
    print!("\tAllocated Bytes: {}\r\n", stats.allocated_bytes);
    print!("\tFree Bytes: {}\r\n", stats.free);
    print!("\tActive: {}\r\n", stats.active);
}

