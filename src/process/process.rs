use core::intrinsics::{atomic_xadd, atomic_xsub, breakpoint, volatile_load, volatile_store};
use core::mem;
use core::raw;

use common::ring_buffer::RingBuffer;

#[allow(improper_ctypes)]
extern {
    pub fn switch_to_user(user_stack: *mut u8) -> *mut u8;
}

/// Size of each processes's memory region in bytes
pub const PROC_MEMORY_SIZE : usize = 2048;

static mut MEMORIES: [[u8; PROC_MEMORY_SIZE]; 8] = [[0; PROC_MEMORY_SIZE]; 8];
static mut FREE_MEMORY_IDX: usize = 0;

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
    pub memory: &'static mut [u8],

    /// The process's memory exposed to the process (the rest is reserved for the
    /// kernel, drivers, etc).
    pub exposed_memory: &'a mut [u8],

    /// The offset in `memory` to use for the process stack.
    pub cur_stack: *mut u8,

    pub wait_pc: usize,

    pub state: State,

    pub callbacks: RingBuffer<'a, Callback>
}

impl<'a> Process<'a> {
    pub unsafe fn create(init_fn: fn()) -> Option<Process<'a>> {
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
            callbacks.enqueue(Callback {
                pc: init_fn as usize, r0: 0, r1: 0, r2:0
            });

            Some(Process {
                memory: memory,
                exposed_memory: &mut memory[callback_len * callback_size..],
                cur_stack: stack_bottom as *mut u8,
                wait_pc: 0,
                state: State::Waiting,
                callbacks: callbacks
            })
        }
    }

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
        if self.cur_stack < (&mut self.exposed_memory[0] as *mut u8) {
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

}

