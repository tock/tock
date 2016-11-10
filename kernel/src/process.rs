use callback::AppId;
use common::{RingBuffer, Queue, VolatileCell};

use container;
use core::{mem, ptr, slice};
use core::cell::Cell;
use core::intrinsics::breakpoint;
use core::ptr::{read_volatile, write_volatile};

#[no_mangle]
pub static mut SYSCALL_FIRED: usize = 0;

#[allow(improper_ctypes)]
extern "C" {
    pub fn switch_to_user(user_stack: *const u8, mem_base: *const u8) -> *mut u8;
}

pub static mut PROCS: &'static mut [Option<Process<'static>>] = &mut [];

pub fn schedule(callback: Callback, appid: AppId) -> bool {
    let procs = unsafe { &mut PROCS };
    let idx = appid.idx();
    if idx >= procs.len() {
        return false;
    }

    match procs[idx] {
        None => false,
        Some(ref mut p) => {
            // TODO(alevy): validate appid liveness
            unsafe {
                HAVE_WORK.set(HAVE_WORK.get() + 1);
            }

            p.callbacks.enqueue(GCallback::Callback(callback))
        }
    }
}

#[derive(Copy,Clone,PartialEq,Eq)]
pub enum Error {
    NoSuchApp,
    OutOfMemory,
    AddressOutOfBounds,
}

#[derive(Copy,Clone,PartialEq,Eq)]
pub enum State {
    Running,
    Yielded,
}

#[derive(Copy, Clone)]
pub enum IPCType {
    Service,
    Client,
}

#[derive(Copy, Clone)]
pub enum GCallback {
    Callback(Callback),
    IPCCallback((AppId, IPCType)),
}

#[derive(Copy, Clone)]
pub struct Callback {
    pub r0: usize,
    pub r1: usize,
    pub r2: usize,
    pub r3: usize,
    pub pc: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct LoadInfo {
    total_size: u32,
    entry_offset: u32,
    rel_data_offset: u32,
    rel_data_size: u32,
    text_offset: u32,
    text_size: u32,
    got_offset: u32,
    got_size: u32,
    data_offset: u32,
    data_size: u32,
    bss_start_offset: u32,
    bss_size: u32,
    pkg_name_offset: u32,
    pkg_name_size: u32,
}

pub struct Process<'a> {
    /// The process's memory.
    memory: &'static mut [u8],

    app_memory_break: *const u8,
    kernel_memory_break: *const u8,

    /// Process text segment
    text: &'static [u8],

    /// The offset in `memory` to use for the process stack.
    cur_stack: *const u8,

    yield_pc: usize,
    psr: usize,

    state: State,

    /// MPU regions are saved as a pointer-size pair.
    ///
    /// size is encoded as X where
    /// SIZE = 2^(X + 1) and X >= 4.
    ///
    /// A null pointer represents an empty region.
    ///
    /// # Invariants
    ///
    /// The pointer must be aligned to the size. E.g. if the size is 32 bytes, the pointer must be
    /// 32-byte aligned.
    ///
    mpu_regions: [Cell<(*const u8, usize)>; 5],

    callbacks: RingBuffer<'a, GCallback>,

    pub pkg_name: &'static [u8],
}

fn closest_power_of_two(mut num: u32) -> u32 {
    num -= 1;
    num |= num >> 1;
    num |= num >> 2;
    num |= num >> 4;
    num |= num >> 8;
    num |= num >> 16;
    num += 1;
    num
}

// Stores the current number of callbacks enqueued + processes in Running state
static mut HAVE_WORK: VolatileCell<usize> = VolatileCell::new(0);

pub fn processes_blocked() -> bool {
    unsafe { HAVE_WORK.get() == 0 }
}

impl<'a> Process<'a> {
    pub fn schedule_ipc(&mut self, from: AppId, cb_type: IPCType) {
        unsafe {
            HAVE_WORK.set(HAVE_WORK.get() + 1);
        }
        self.callbacks.enqueue(GCallback::IPCCallback((from, cb_type)));
    }

    pub fn current_state(&self) -> State {
        self.state
    }

    pub fn yield_state(&mut self) {
        if self.state == State::Running {
            self.state = State::Yielded;
            unsafe {
                HAVE_WORK.set(HAVE_WORK.get() - 1);
            }
        }
    }

    pub fn dequeue_callback(&mut self) -> Option<GCallback> {
        self.callbacks.dequeue().map(|cb| {
            unsafe {
                HAVE_WORK.set(HAVE_WORK.get() - 1);
            }
            cb
        })
    }

    pub fn mem_start(&self) -> *const u8 {
        self.memory.as_ptr()
    }

    pub fn mem_end(&self) -> *const u8 {
        unsafe { self.memory.as_ptr().offset(self.memory.len() as isize) }
    }

    pub fn setup_mpu(&self, mpu: &::platform::MPU) {
        let data_start = self.memory.as_ptr() as usize;
        let data_len = 12;

        let text_start = self.text.as_ptr() as usize;
        let text_len = ((32 - self.text.len().leading_zeros()) - 2) as u32;

        let mut grant_size = unsafe {
            self.memory.as_ptr().offset(self.memory.len() as isize) as u32 -
            (self.kernel_memory_break as u32)
        };
        grant_size = closest_power_of_two(grant_size);
        let grant_base = unsafe {
            self.memory
                .as_ptr()
                .offset(self.memory.len() as isize)
                .offset(-(grant_size as isize))
        };
        let mgrant_size = grant_size.trailing_zeros() - 1;

        // Data segment read/write/execute
        mpu.set_mpu(0, data_start as u32, data_len, true, 0b011);
        // Text segment read/execute (no write)
        mpu.set_mpu(1, text_start as u32, text_len, true, 0b111);

        // Disallow access to grant region
        mpu.set_mpu(2, grant_base as u32, mgrant_size, false, 0b001);

        for (i, region) in self.mpu_regions.iter().enumerate() {
            mpu.set_mpu((i + 3) as u32,
                        region.get().0 as u32,
                        region.get().1 as u32,
                        true,
                        0b011);
        }
    }


    pub fn add_mpu_region(&self, base: *const u8, size: usize) -> bool {
        if size >= 16 && size.count_ones() == 1 && (base as usize) % size == 0 {
            let mpu_size = (size.trailing_zeros() - 1) as usize;
            for region in self.mpu_regions.iter() {
                if region.get().0 == ptr::null() {
                    region.set((base, mpu_size));
                    return true;
                } else if region.get().0 == base {
                    if region.get().1 < mpu_size {
                        region.set((base, mpu_size));
                    }
                    return true;
                }
            }
        }
        return false;
    }

    pub unsafe fn create(start_addr: *const u8,
                         length: usize,
                         memory: &'static mut [u8])
                         -> Process<'a> {
        let mut kernel_memory_break = {
            // make room for container pointers
            let psz = mem::size_of::<*const usize>();
            let num_ctrs = read_volatile(&container::CONTAINER_COUNTER);
            let container_ptrs_size = num_ctrs * psz;
            let res = memory.as_mut_ptr().offset((memory.len() - container_ptrs_size) as isize);
            // set all ptrs to null
            let opts = slice::from_raw_parts_mut(res as *mut *const usize, num_ctrs);
            for opt in opts.iter_mut() {
                *opt = ptr::null()
            }
            res
        };

        // Take callback buffer from of memory
        let callback_size = mem::size_of::<GCallback>();
        let callback_len = 10;
        let callback_offset = callback_len * callback_size;
        // Set kernel break to beginning of callback buffer
        kernel_memory_break = kernel_memory_break.offset(-(callback_offset as isize));
        let callback_buf = slice::from_raw_parts_mut(kernel_memory_break as *mut GCallback,
                                                     callback_len);

        let callbacks = RingBuffer::new(callback_buf);

        let load_result = load(start_addr, memory.as_mut_ptr());

        let stack_bottom = load_result.app_mem_start.offset(512);

        let mut process = Process {
            memory: memory,
            app_memory_break: stack_bottom,
            kernel_memory_break: kernel_memory_break,
            text: slice::from_raw_parts(start_addr, length),
            cur_stack: stack_bottom,
            yield_pc: 0,
            psr: 0x01000000,
            mpu_regions: [Cell::new((ptr::null(), 0)),
                          Cell::new((ptr::null(), 0)),
                          Cell::new((ptr::null(), 0)),
                          Cell::new((ptr::null(), 0)),
                          Cell::new((ptr::null(), 0))],
            pkg_name: load_result.pkg_name,
            state: State::Yielded,
            callbacks: callbacks,
        };

        if (load_result.init_fn - 1) % 8 != 0 {
            panic!("{}", (load_result.init_fn - 1) % 8);
        }

        process.callbacks.enqueue(GCallback::Callback(Callback {
            pc: load_result.init_fn,
            r0: load_result.app_mem_start as usize,
            r1: process.app_memory_break as usize,
            r2: process.kernel_memory_break as usize,
            r3: 0,
        }));

        HAVE_WORK.set(HAVE_WORK.get() + 1);

        process
    }

    pub fn sbrk(&mut self, increment: isize) -> Result<*const u8, Error> {
        let new_break = unsafe { self.app_memory_break.offset(increment) };
        self.brk(new_break)
    }

    pub fn brk(&mut self, new_break: *const u8) -> Result<*const u8, Error> {
        if new_break < self.mem_start() || new_break >= self.mem_end() {
            Err(Error::AddressOutOfBounds)
        } else if new_break > self.kernel_memory_break {
            Err(Error::OutOfMemory)
        } else {
            let old_break = self.app_memory_break;
            self.app_memory_break = new_break;
            Ok(old_break)
        }
    }

    pub fn in_exposed_bounds(&self, buf_start_addr: *const u8, size: usize) -> bool {

        let buf_end_addr = unsafe { buf_start_addr.offset(size as isize) };

        buf_start_addr >= self.mem_start() && buf_end_addr <= self.mem_end()
    }

    pub unsafe fn alloc(&mut self, size: usize) -> Option<&mut [u8]> {
        let new_break = self.kernel_memory_break.offset(-(size as isize));
        if new_break < self.app_memory_break {
            None
        } else {
            self.kernel_memory_break = new_break;
            Some(slice::from_raw_parts_mut(new_break as *mut u8, size))
        }
    }

    pub unsafe fn free<T>(&mut self, _: *mut T) {}

    pub unsafe fn container_for<T>(&mut self, container_num: usize) -> *mut *mut T {
        let container_num = container_num as isize;
        let ptr = (self.mem_end() as *mut usize).offset(-(container_num + 1));
        ptr as *mut *mut T
    }

    pub unsafe fn container_for_or_alloc<T: Default>(&mut self,
                                                     container_num: usize)
                                                     -> Option<*mut T> {
        let ctr_ptr = self.container_for::<T>(container_num);
        if (*ctr_ptr).is_null() {
            self.alloc(mem::size_of::<T>()).map(|root_arr| {
                let root_ptr = root_arr.as_mut_ptr() as *mut T;
                *root_ptr = Default::default();
                write_volatile(ctr_ptr, root_ptr);
                root_ptr
            })
        } else {
            Some(*ctr_ptr)
        }
    }


    pub fn pop_syscall_stack(&mut self) {
        let pspr = self.cur_stack as *const usize;
        unsafe {
            self.yield_pc = read_volatile(pspr.offset(6));
            self.psr = read_volatile(pspr.offset(7));
            self.cur_stack = (self.cur_stack as *mut usize).offset(8) as *mut u8;
        }
    }

    /// Context switch to the process.
    pub unsafe fn push_callback(&mut self, callback: Callback) {
        HAVE_WORK.set(HAVE_WORK.get() + 1);

        self.state = State::Running;
        // Fill in initial stack expected by SVC handler
        // Top minus 8 u32s for r0-r3, r12, lr, pc and xPSR
        let stack_bottom = (self.cur_stack as *mut usize).offset(-8);
        write_volatile(stack_bottom.offset(7), self.psr);
        write_volatile(stack_bottom.offset(6), callback.pc | 1);
        // Set the LR register to the saved PC so the callback returns to
        // wherever wait was called. Set lowest bit to one because of THUMB
        // instruction requirements.
        write_volatile(stack_bottom.offset(5), self.yield_pc | 0x1);
        write_volatile(stack_bottom, callback.r0);
        write_volatile(stack_bottom.offset(1), callback.r1);
        write_volatile(stack_bottom.offset(2), callback.r2);
        write_volatile(stack_bottom.offset(3), callback.r3);

        self.cur_stack = stack_bottom as *mut u8;
    }

    pub unsafe fn syscall_fired(&self) -> bool {
        read_volatile(&SYSCALL_FIRED) != 0
    }

    /// Context switch to the process.
    pub unsafe fn switch_to(&mut self) {
        if self.cur_stack < self.memory.as_ptr() {
            breakpoint();
        }
        write_volatile(&mut SYSCALL_FIRED, 0);
        let psp = switch_to_user(self.cur_stack, self.memory.as_ptr());
        self.cur_stack = psp;
    }

    pub fn svc_number(&self) -> Option<u8> {
        let psp = self.cur_stack as *const *const u16;
        unsafe {
            let pcptr = read_volatile((psp as *const *const u16).offset(6));
            let svc_instr = read_volatile(pcptr.offset(-1));
            Some((svc_instr & 0xff) as u8)
        }
    }

    pub fn lr(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { read_volatile(pspr.offset(5)) }
    }


    pub fn r0(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { read_volatile(pspr) }
    }

    pub fn set_r0(&mut self, val: isize) {
        let pspr = self.cur_stack as *mut isize;
        unsafe { write_volatile(pspr, val) }
    }

    pub fn r1(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { read_volatile(pspr.offset(1)) }
    }

    pub fn r2(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { read_volatile(pspr.offset(2)) }
    }

    pub fn r3(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { read_volatile(pspr.offset(3)) }
    }
}

#[derive(Debug)]
struct LoadResult {
    init_fn: usize,
    app_mem_start: *const u8,
    pkg_name: &'static [u8],
}

unsafe fn load(start_addr: *const u8, mem_base: *mut u8) -> LoadResult {

    let load_info = &*(start_addr as *const LoadInfo);

    let mut result = LoadResult {
        pkg_name: slice::from_raw_parts(start_addr.offset(load_info.pkg_name_offset as isize),
                                        load_info.pkg_name_size as usize),
        init_fn: 0,
        app_mem_start: ptr::null(),
    };

    let text_start = start_addr.offset(load_info.text_offset as isize);

    let rel_data: &[u32] =
        slice::from_raw_parts(start_addr.offset(load_info.rel_data_offset as isize) as *const u32,
                              (load_info.rel_data_size as usize) / mem::size_of::<u32>());

    let got: &[u8] =
        slice::from_raw_parts(start_addr.offset(load_info.got_offset as isize),
                              load_info.got_size as usize) as &[u8];

    let data: &[u8] = slice::from_raw_parts(start_addr.offset(load_info.data_offset as isize),
                                            load_info.data_size as usize);

    let target_data: &mut [u8] =
        slice::from_raw_parts_mut(mem_base,
                                  (load_info.data_size + load_info.got_size) as usize);

    for (orig, dest) in got.iter().chain(data.iter()).zip(target_data.iter_mut()) {
        *dest = *orig
    }

    // Zero out BSS
    ::core::intrinsics::write_bytes(mem_base.offset(load_info.bss_start_offset as isize),
                                    0,
                                    load_info.bss_size as usize);


    let fixup = |addr: &mut u32| {
        let entry = *addr;
        if (entry & 0x80000000) == 0 {
            // Regular data (memory relative)
            *addr = entry + (mem_base as u32);
        } else {
            // rodata or function pointer (code relative)
            *addr = (entry ^ 0x80000000) + (text_start as u32);
        }
    };

    // Fixup Global Offset Table
    let mem_got: &mut [u32] = slice::from_raw_parts_mut(mem_base as *mut u32,
                                                        (load_info.got_size as usize) /
                                                        mem::size_of::<u32>());

    for got_cur in mem_got {
        fixup(got_cur);
    }

    // Fixup relocation data
    for (i, addr) in rel_data.iter().enumerate() {
        if i % 2 == 0 {
            // Only the first of every 2 entries is an address
            fixup(&mut *(mem_base.offset(*addr as isize) as *mut u32));
        }
    }

    // Entry point is offset from app code
    result.init_fn = start_addr.offset(load_info.entry_offset as isize) as usize;

    let mut aligned_mem_start = load_info.bss_start_offset + load_info.bss_size;
    aligned_mem_start += (8 - (aligned_mem_start % 8)) % 8;
    result.app_mem_start = mem_base.offset(aligned_mem_start as isize);

    result
}
