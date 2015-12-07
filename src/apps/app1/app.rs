use super::boxed::BoxMgr;

pub struct App {
    pub memory: BoxMgr
}

pub fn init() {
    print!("Welcome to Tock!\r\n");

    let stats = (unsafe { &*super::app }).memory.stats();
    print!("Memory Stats:{}\r\n", "");
    print!("\tNum Allocated: {}\r\n", stats.num_allocated);
    print!("\tNum Allocs: {}\r\n", stats.allocs);
    print!("\tDrops: {}\r\n", stats.drops);
    print!("\tAllocated Bytes: {}\r\n", stats.allocated_bytes);
    print!("\tFree Bytes: {}\r\n", stats.free);
    print!("\tActive: {}\r\n", stats.active);
}

