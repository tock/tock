use std::ffi::CString;
use std::slice;

pub static mut SHARED_BUFFER: SharedBuffer = SharedBuffer::new();

#[allow(dead_code)]
pub struct SharedBuffer {
    buf_start: *mut u8,
    buf_head: *mut u8,
    capacity: usize,
    size: usize,
}

#[allow(dead_code)]
impl SharedBuffer {
    const fn new() -> Self {
        SharedBuffer {
            buf_start: std::ptr::null_mut(),
            buf_head: std::ptr::null_mut(),
            capacity: 0,
            size: 0,
        }
    }

    unsafe fn handle_libc_error(error: bool, msg: &str) {
        if error {
            panic!("{}: errno {}", msg, *libc::__errno_location());
        }
    }

    pub unsafe fn initialize(&mut self, size: usize) {
        let name = CString::new("he_sharedbuf").unwrap();
        let fd = libc::shm_open(name.as_ptr(), libc::O_CREAT | libc::O_RDWR, libc::S_IRWXU);
        Self::handle_libc_error(fd < 0, "Could not create sharedmem buffer");

        let result = libc::ftruncate(fd, size as i64);
        Self::handle_libc_error(result < 0, "Could not truncate sharedmem buffer");

        let addr = libc::mmap(
            0 as *mut libc::c_void,
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            0,
        );
        Self::handle_libc_error((addr as isize) < 0, "Could not mmap sharedmem");

        self.buf_start = addr as *mut u8;
        self.buf_head = addr as *mut u8;
        self.capacity = size;
        self.size = size;
    }

    pub fn alloc_buf(&mut self, size: usize) -> &'static mut [u8] {
        assert!(self.buf_start != std::ptr::null_mut());

        if size > self.capacity {
            panic!("Not enough capacity");
        }

        let slice = unsafe { slice::from_raw_parts_mut(self.buf_head, size) };
        self.buf_head = unsafe { self.buf_head.add(size) };
        self.capacity -= size;
        slice
    }

    pub fn get_start_addr(&self) -> *const u8 {
        self.buf_start
    }

    pub fn get_size(&self) -> usize {
        self.capacity
    }
}
