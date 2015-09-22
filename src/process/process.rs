use core::intrinsics::{atomic_xadd, atomic_xsub, breakpoint, volatile_load, volatile_store};
use core::mem;
use core::raw;

use common::{RingBuffer, Queue};

#[allow(improper_ctypes)]
extern {
    pub fn switch_to_user(user_stack: *mut u8) -> *mut u8;
}

/// Size of each processes's memory region in bytes
pub const PROC_MEMORY_SIZE : usize = 2048;
pub const NUM_PROCS : usize = 1;

static mut MEMORIES: [[u8; PROC_MEMORY_SIZE]; NUM_PROCS] = [[0; PROC_MEMORY_SIZE]; NUM_PROCS];
static mut FREE_MEMORY_IDX: usize = 0;

pub static mut PROCS : [Option<Process<'static>>; NUM_PROCS] = [None; NUM_PROCS];

pub fn schedule(callback: Callback, appid: ::AppId) -> bool {
    let procs = unsafe { &mut PROCS };
    let idx = appid.idx();
    if idx >= procs.len() {
        return false
    }

    match procs[idx] {
        None => false,
        Some(ref mut p) => {
            // TODO(alevy): validate appid liveness
            p.callbacks.enqueue(callback)
        }
    }
}

#[derive(Copy,Clone,PartialEq,Eq)]
pub enum State {
    Running,
    Waiting
}


#[derive(Copy,Clone)]
pub struct Callback {
    pub r0: usize,
    pub r1: usize,
    pub r2: usize,
    pub pc: usize
}

pub struct Process<'a> {
    /// The process's memory.
    memory: &'a mut [u8],

    exposed_memory_start: *mut u8,

    /// The offset in `memory` to use for the process stack.
    cur_stack: *mut u8,

    wait_pc: usize,

    pub state: State,

    pub callbacks: RingBuffer<'a, Callback>
}

impl<'a> Process<'a> {
    pub unsafe fn create(init_fn: fn(*mut u8, usize)) -> Option<Process<'a>> {
        let cur_idx = atomic_xadd(&mut FREE_MEMORY_IDX, 1);
        if cur_idx > MEMORIES.len() {
            atomic_xsub(&mut FREE_MEMORY_IDX, 1);
            None
        } else {
            let memory = &mut MEMORIES[cur_idx];

            let stack_bottom = &mut memory[PROC_MEMORY_SIZE - 4] as *mut u8;

            // Take callback buffer from bottom of process memory
            let callback_len = 10;
            let callback_buf = mem::transmute(raw::Slice {
                data: &mut memory[0] as *mut u8 as *mut Option<Callback>,
                len: callback_len
            });
            let callback_size = mem::size_of::<Option<Callback>>();

            let mut callbacks = RingBuffer::new(callback_buf);

            let exposed_memory_start =
                    &mut memory[callback_len * callback_size] as *mut u8;

            let exposed_mem_len = memory.len() - callback_len * callback_size;
            callbacks.enqueue(Callback {
                pc: init_fn as usize,
                r0: exposed_memory_start as usize,
                r1: exposed_mem_len,
                r2: 0
            });

            Some(Process {
                memory: memory,
                exposed_memory_start: exposed_memory_start,
                cur_stack: stack_bottom as *mut u8,
                wait_pc: 0,
                state: State::Waiting,
                callbacks: callbacks
            })
        }
    }

    pub fn in_exposed_bounds(&self, buf_start_addr: *const u8, size: usize)
            -> bool {
        use core::raw::Repr;

        let buf_end_addr = ((buf_start_addr as usize) + size) as *const u8;

        let mem = self.memory.repr();
        let mem_end = ((mem.data as usize) + mem.len) as *const u8;

        buf_start_addr >= self.exposed_memory_start && buf_end_addr <= mem_end
    }

    pub unsafe fn alloc(&mut self, size: usize) -> Option<&mut [u8]> {
        use core::raw::Slice;

        let mem_len = self.memory.len();
        let end_mem = &mut self.memory[mem_len - 1] as *mut u8;
        let new_start = self.exposed_memory_start.offset(size as isize);
        if new_start >= end_mem {
            None
        } else {
            let buf = Slice {
                data: self.exposed_memory_start,
                len: size
            };
            self.exposed_memory_start = new_start;
            Some(mem::transmute(buf))
        }
    }

    pub unsafe fn free<T>(&mut self, _: *mut T) {}

    pub fn pop_syscall_stack(&mut self) {
        let pspr = self.cur_stack as *const usize;
        unsafe {
            self.wait_pc = volatile_load(pspr.offset(6));
            self.cur_stack =
                (self.cur_stack as *mut usize).offset(8) as *mut u8;
        }
    }

    /// Context switch to the process.
    pub unsafe fn switch_to_callback(&mut self, callback: Callback) {
        // Fill in initial stack expected by SVC handler
        // Top minus 8 u32s for r0-r3, r12, lr, pc and xPSR
        let stack_bottom = (self.cur_stack as *mut usize).offset(-8);
        volatile_store(stack_bottom.offset(7), 0x01000000);
        volatile_store(stack_bottom.offset(6), callback.pc);
        // Set the LR register to the saved PC so the callback returns to
        // wherever wait was called. Set lowest bit to one because of THUMB
        // instruction requirements.
        volatile_store(stack_bottom.offset(5), self.wait_pc | 0x1);
        volatile_store(stack_bottom, callback.r0);
        volatile_store(stack_bottom.offset(1), callback.r1);
        volatile_store(stack_bottom.offset(2), callback.r2);

        self.cur_stack = stack_bottom as *mut u8;
        self.switch_to();
    }

    /// Context switch to the process.
    pub unsafe fn switch_to(&mut self) {
        if self.cur_stack < self.exposed_memory_start {
            breakpoint();
        }
        let psp = switch_to_user(self.cur_stack);
        self.cur_stack = psp;
    }

    pub fn svc_number(&self) -> Option<u8> {
        let psp = self.cur_stack as *const *const u16;
        unsafe {
            let pcptr = volatile_load((psp as *const *const u16).offset(6));
            let svc_instr = volatile_load(pcptr.offset(-1));
            Some((svc_instr & 0xff) as u8)
        }
    }

    pub fn lr(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { volatile_load(pspr.offset(5)) }
    }


    pub fn r0(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { volatile_load(pspr) }
    }

    pub fn set_r0(&mut self, val: isize) {
        let pspr = self.cur_stack as *mut isize;
        unsafe { volatile_store(pspr, val) }
    }

    pub fn r1(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { volatile_load(pspr.offset(1)) }
    }

    pub fn r2(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { volatile_load(pspr.offset(2)) }
    }

    pub fn r3(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { volatile_load(pspr.offset(3)) }
    }

}

