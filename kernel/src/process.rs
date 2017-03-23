use callback::AppId;
use common::{RingBuffer, Queue, VolatileCell};

use container;
use core::{mem, ptr, slice, str};
use core::cell::Cell;
use core::fmt::Write;
use core::intrinsics;
use core::ptr::{read_volatile, write_volatile};

use platform::mpu;
use returncode::ReturnCode;
use syscall::Syscall;
use common::math;

/// Takes a value and rounds it up to be aligned % 8
macro_rules! align8 {
    ( $e:expr ) => ( ($e) + ((8 - (($e) % 8)) % 8 ) );
}

#[no_mangle]
pub static mut SYSCALL_FIRED: usize = 0;

#[no_mangle]
pub static mut APP_FAULT: usize = 0;

#[no_mangle]
pub static mut SCB_REGISTERS: [u32; 5] = [0; 5];

#[allow(improper_ctypes)]
extern "C" {
    pub fn switch_to_user(user_stack: *const u8,
                          got_base: *const u8,
                          process_regs: &mut [usize; 8])
                          -> *mut u8;
}

pub static mut PROCS: &'static mut [Option<Process<'static>>] = &mut [];

pub fn schedule(callback: FunctionCall, appid: AppId) -> bool {
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

            p.tasks.enqueue(Task::FunctionCall(callback))
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    NoSuchApp,
    OutOfMemory,
    AddressOutOfBounds,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum State {
    Running,
    Yielded,
    Fault,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FaultResponse {
    Panic,
    Restart,
}

#[derive(Copy, Clone, Debug)]
pub enum IPCType {
    Service,
    Client,
}

#[derive(Copy, Clone, Debug)]
pub enum Task {
    FunctionCall(FunctionCall),
    IPC((AppId, IPCType)),
}

#[derive(Copy, Clone, Debug)]
pub struct FunctionCall {
    pub r0: usize,
    pub r1: usize,
    pub r2: usize,
    pub r3: usize,
    pub pc: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct LoadInfo {
    version: u32,
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
    bss_mem_offset: u32,
    bss_size: u32,
    min_stack_len: u32,
    min_app_heap_len: u32,
    min_kernel_heap_len: u32,
    pkg_name_offset: u32,
    pkg_name_size: u32,
    checksum: u32,
}

/// Converts a pointer to memory to a LoadInfo struct
///
/// This function takes a pointer to arbitrary memory and Optionally returns a
/// LoadInfo struct. This function will validate the header checksum, but does
/// not perform sanity or security checking on the structure
unsafe fn parse_and_validate_load_info(address: *const u8) -> Option<&'static LoadInfo> {
    let load_info = &*(address as *const LoadInfo);

    if load_info.version != 1 {
        return None;
    }

    let checksum =
        load_info.version ^ load_info.total_size ^ load_info.entry_offset ^
        load_info.rel_data_offset ^ load_info.rel_data_size ^ load_info.text_offset ^
        load_info.text_size ^ load_info.got_offset ^
        load_info.got_size ^
        load_info.data_offset ^ load_info.data_size ^ load_info.bss_mem_offset ^
        load_info.bss_size ^
        load_info.min_stack_len ^ load_info.min_app_heap_len ^
        load_info.min_kernel_heap_len ^ load_info.pkg_name_offset ^ load_info.pkg_name_size;

    if checksum != load_info.checksum {
        return None;
    }

    Some(load_info)
}

#[derive(Default)]
struct StoredRegs {
    r4: usize,
    r5: usize,
    r6: usize,
    r7: usize,
    r8: usize,
    r9: usize,
    r10: usize,
    r11: usize,
}

pub struct Process<'a> {
    /// Application memory layout:
    ///
    ///     |======== <- memory[memory.len()]
    ///  ╔═ | Grant
    ///     |   ↓
    ///  D  |  ----   <- kernel_memory_break
    ///  Y  |
    ///  N  |  ----   <- app_heap_break
    ///  A  |
    ///  M  |   ↑
    ///     |  Heap
    ///  ╠═ |  ----   <- app_heap_start
    ///     |  Data
    ///  F  |  ----   <- stack_data_boundary
    ///  I  | Stack
    ///  X  |   ↓
    ///  E  |
    ///  D  |  ----   <- cur_stack
    ///     |
    ///  ╚═ |======== <- memory[0]

    /// The process's memory.
    memory: &'static mut [u8],

    kernel_memory_break: *const u8,
    app_heap_break: *const u8,
    app_heap_start: *const u8,
    stack_data_boundary: *const u8,
    cur_stack: *const u8,

    /// How low have we ever seen the stack pointer
    min_stack_pointer: *const u8,

    /// How many syscalls have occurred since the process started
    syscall_count: Cell<usize>,

    /// What was the most recent syscall
    last_syscall: Cell<Option<Syscall>>,

    /// Process text segment
    text: &'static [u8],

    stored_regs: StoredRegs,

    yield_pc: usize,
    psr: usize,

    state: State,

    /// How to deal with Faults occuring in the process
    fault_response: FaultResponse,

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

    tasks: RingBuffer<'a, Task>,

    pub package_name: &'static str,
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
        self.tasks.enqueue(Task::IPC((from, cb_type)));
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

    pub unsafe fn fault_state(&mut self) {
        write_volatile(&mut APP_FAULT, 0);
        self.state = State::Fault;

        match self.fault_response {
            FaultResponse::Panic => {
                // process faulted. Panic and print status
                panic!("Process {} had a fault", self.package_name);
            }
            FaultResponse::Restart => {
                //XXX: unimplemented
                panic!("Process {} had a fault and could not be restarted",
                       self.package_name);
                /*
                // HAVE_WORK is really screwed up in this case
                // the tasks ring buffer needs to be cleared
                // need to re-load() the app
                 */
            }
        }
    }

    pub fn dequeue_task(&mut self) -> Option<Task> {
        self.tasks.dequeue().map(|cb| {
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

    pub fn setup_mpu<MPU: mpu::MPU>(&self, mpu: &MPU) {
        let data_start = self.memory.as_ptr() as usize;
        let data_len = self.memory.len();
        if data_len.count_ones() != 1 {
            panic!("Tock MPU does not currently handle complex region sizes");
        }
        let data_region_len = math::log_base_two(data_len as u32);

        let text_start = self.text.as_ptr() as usize;
        let text_len = self.text.len();
        if text_len.count_ones() != 1 {
            panic!("Tock MPU does not currently handle complex region sizes");
        }
        let text_region_len = math::log_base_two(text_len as u32);

        let mut grant_size = unsafe {
            self.memory.as_ptr().offset(self.memory.len() as isize) as u32 -
            (self.kernel_memory_break as u32)
        };
        grant_size = math::closest_power_of_two(grant_size);
        let grant_base = unsafe {
            self.memory
                .as_ptr()
                .offset(self.memory.len() as isize)
                .offset(-(grant_size as isize))
        };
        let mgrant_size = grant_size.trailing_zeros() - 1;

        // Data segment read/write/execute
        mpu.set_mpu(0,
                    data_start as u32,
                    data_region_len,
                    mpu::ExecutePermission::ExecutionPermitted,
                    mpu::AccessPermission::ReadWrite);
        // Text segment read/execute (no write)
        mpu.set_mpu(1,
                    text_start as u32,
                    text_region_len,
                    mpu::ExecutePermission::ExecutionPermitted,
                    mpu::AccessPermission::ReadOnly);

        // Disallow access to grant region
        mpu.set_mpu(2,
                    grant_base as u32,
                    mgrant_size,
                    mpu::ExecutePermission::ExecutionNotPermitted,
                    mpu::AccessPermission::PrivilegedOnly);

        for (i, region) in self.mpu_regions.iter().enumerate() {
            mpu.set_mpu((i + 3) as u32,
                        region.get().0 as u32,
                        region.get().1 as u32,
                        mpu::ExecutePermission::ExecutionPermitted,
                        mpu::AccessPermission::ReadWrite);
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

    pub unsafe fn create(app_flash_address: *const u8,
                         remaining_app_memory: *mut u8,
                         remaining_app_memory_size: usize,
                         fault_response: FaultResponse)
                         -> (Option<Process<'a>>, usize, usize) {
        if let Some(load_info) = parse_and_validate_load_info(app_flash_address) {
            let app_flash_size = load_info.total_size as usize;

            // Load the process into memory
            if let Some(load_result) =
                load(load_info,
                     app_flash_address,
                     remaining_app_memory,
                     remaining_app_memory_size) {
                let app_heap_len = align8!(load_info.min_app_heap_len);
                let kernel_heap_len = align8!(load_info.min_kernel_heap_len);

                let app_slice_size_unaligned = load_result.fixed_len + app_heap_len +
                                               kernel_heap_len;
                let app_slice_size = math::closest_power_of_two(app_slice_size_unaligned) as usize;
                // TODO round app_slice_size up to a closer MPU unit.
                // This is a very conservative approach that rounds up to power of
                // two. We should be able to make this closer to what we actually need.

                if app_slice_size > remaining_app_memory_size {
                    panic!("{:?} failed to load. Insufficient memory. Requested {} have {}",
                           load_result.package_name,
                           app_slice_size,
                           remaining_app_memory_size);
                }

                let app_memory = slice::from_raw_parts_mut(remaining_app_memory, app_slice_size);

                // Set up initial grant region
                let mut kernel_memory_break = app_memory.as_mut_ptr()
                    .offset(app_memory.len() as isize);

                // make room for container pointers
                let pointer_size = mem::size_of::<*const usize>();
                let num_ctrs = read_volatile(&container::CONTAINER_COUNTER);
                let container_ptrs_size = num_ctrs * pointer_size;
                kernel_memory_break = kernel_memory_break.offset(-(container_ptrs_size as isize));

                // set all pointers to null
                let opts = slice::from_raw_parts_mut(kernel_memory_break as *mut *const usize,
                                                     num_ctrs);
                for opt in opts.iter_mut() {
                    *opt = ptr::null()
                }

                // Allocate memory for callback ring buffer
                let callback_size = mem::size_of::<Task>();
                let callback_len = 10;
                let callback_offset = callback_len * callback_size;
                kernel_memory_break = kernel_memory_break.offset(-(callback_offset as isize));

                // Set up ring buffer
                let callback_buf = slice::from_raw_parts_mut(kernel_memory_break as *mut Task,
                                                             callback_len);
                let tasks = RingBuffer::new(callback_buf);

                let mut process = Process {
                    memory: app_memory,

                    kernel_memory_break: kernel_memory_break,
                    app_heap_break: load_result.app_heap_start,
                    app_heap_start: load_result.app_heap_start,
                    stack_data_boundary: load_result.stack_data_boundary,
                    cur_stack: load_result.stack_data_boundary,

                    min_stack_pointer: load_result.stack_data_boundary,

                    syscall_count: Cell::new(0),
                    last_syscall: Cell::new(None),

                    text: slice::from_raw_parts(app_flash_address, app_flash_size),

                    stored_regs: Default::default(),
                    yield_pc: load_result.init_fn,
                    // Set the Thumb bit and clear everything else
                    psr: 0x01000000,

                    state: State::Yielded,
                    fault_response: fault_response,

                    mpu_regions: [Cell::new((ptr::null(), 0)),
                                  Cell::new((ptr::null(), 0)),
                                  Cell::new((ptr::null(), 0)),
                                  Cell::new((ptr::null(), 0)),
                                  Cell::new((ptr::null(), 0))],
                    tasks: tasks,
                    package_name: load_result.package_name,
                };

                if (load_result.init_fn & 0x1) != 1 {
                    panic!("{:?} process image invalid. \
                           init_fn address must end in 1 to be Thumb, got {:#X}",
                           load_result.package_name,
                           load_result.init_fn);
                }

                process.tasks.enqueue(Task::FunctionCall(FunctionCall {
                    pc: load_result.init_fn,
                    r0: process.memory.as_ptr() as usize,
                    r1: process.app_heap_break as usize,
                    r2: process.kernel_memory_break as usize,
                    r3: 0,
                }));

                HAVE_WORK.set(HAVE_WORK.get() + 1);

                return (Some(process), app_flash_size, app_slice_size);
            }
        }
        (None, 0, 0)
    }

    pub fn sbrk(&mut self, increment: isize) -> Result<*const u8, Error> {
        let new_break = unsafe { self.app_heap_break.offset(increment) };
        self.brk(new_break)
    }

    pub fn brk(&mut self, new_break: *const u8) -> Result<*const u8, Error> {
        if new_break < self.mem_start() || new_break >= self.mem_end() {
            Err(Error::AddressOutOfBounds)
        } else if new_break > self.kernel_memory_break {
            Err(Error::OutOfMemory)
        } else {
            let old_break = self.app_heap_break;
            self.app_heap_break = new_break;
            Ok(old_break)
        }
    }

    pub fn in_exposed_bounds(&self, buf_start_addr: *const u8, size: usize) -> bool {

        let buf_end_addr = unsafe { buf_start_addr.offset(size as isize) };

        buf_start_addr >= self.mem_start() && buf_end_addr <= self.mem_end()
    }

    pub unsafe fn alloc(&mut self, size: usize) -> Option<&mut [u8]> {
        let new_break = self.kernel_memory_break.offset(-(size as isize));
        if new_break < self.app_heap_break {
            None
        } else {
            self.kernel_memory_break = new_break;
            Some(slice::from_raw_parts_mut(new_break as *mut u8, size))
        }
    }

    pub unsafe fn free<T>(&mut self, _: *mut T) {}

    unsafe fn container_ptr<T>(&self, container_num: usize) -> *mut *mut T {
        let container_num = container_num as isize;
        (self.mem_end() as *mut *mut T).offset(-(container_num + 1))
    }

    pub unsafe fn container_for<T>(&mut self, container_num: usize) -> *mut T {
        *self.container_ptr(container_num)
    }

    pub unsafe fn container_for_or_alloc<T: Default>(&mut self,
                                                     container_num: usize)
                                                     -> Option<*mut T> {
        let ctr_ptr = self.container_ptr::<T>(container_num);
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
            if self.cur_stack < self.min_stack_pointer {
                self.min_stack_pointer = self.cur_stack;
            }
        }
    }

    /// Context switch to the process.
    pub unsafe fn push_function_call(&mut self, callback: FunctionCall) {
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
        if self.cur_stack < self.min_stack_pointer {
            self.min_stack_pointer = self.cur_stack;
        }
    }

    pub unsafe fn app_fault(&self) -> bool {
        read_volatile(&APP_FAULT) != 0
    }

    pub unsafe fn syscall_fired(&self) -> bool {
        read_volatile(&SYSCALL_FIRED) != 0
    }

    /// Context switch to the process.
    pub unsafe fn switch_to(&mut self) {
        write_volatile(&mut SYSCALL_FIRED, 0);
        let psp = switch_to_user(self.cur_stack,
                                 self.stack_data_boundary,
                                 mem::transmute(&mut self.stored_regs));
        self.cur_stack = psp;
        if self.cur_stack < self.min_stack_pointer {
            self.min_stack_pointer = self.cur_stack;
        }
    }

    pub fn svc_number(&self) -> Option<Syscall> {
        let psp = self.cur_stack as *const *const u16;
        unsafe {
            let pcptr = read_volatile((psp as *const *const u16).offset(6));
            let svc_instr = read_volatile(pcptr.offset(-1));
            let svc_num = (svc_instr & 0xff) as u8;
            match svc_num {
                0 => Some(Syscall::YIELD),
                1 => Some(Syscall::SUBSCRIBE),
                2 => Some(Syscall::COMMAND),
                3 => Some(Syscall::ALLOW),
                4 => Some(Syscall::MEMOP),
                _ => None,
            }
        }
    }

    pub fn incr_syscall_count(&self) {
        self.syscall_count.set(self.syscall_count.get() + 1);
        self.last_syscall.set(self.svc_number());
    }

    pub fn sp(&self) -> usize {
        self.cur_stack as usize
    }

    pub fn lr(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { read_volatile(pspr.offset(5)) }
    }

    pub fn pc(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { read_volatile(pspr.offset(6)) }
    }

    pub fn r0(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { read_volatile(pspr) }
    }

    pub fn set_return_code(&mut self, return_code: ReturnCode) {
        let r: isize = return_code.into();
        self.set_r0(r);
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

    pub fn r12(&self) -> usize {
        let pspr = self.cur_stack as *const usize;
        unsafe { read_volatile(pspr.offset(3)) }
    }

    pub unsafe fn fault_str<W: Write>(&mut self, writer: &mut W) {
        let _ccr = SCB_REGISTERS[0];
        let cfsr = SCB_REGISTERS[1];
        let hfsr = SCB_REGISTERS[2];
        let mmfar = SCB_REGISTERS[3];
        let bfar = SCB_REGISTERS[4];

        let iaccviol = (cfsr & 0x01) == 0x01;
        let daccviol = (cfsr & 0x02) == 0x02;
        let munstkerr = (cfsr & 0x08) == 0x08;
        let mstkerr = (cfsr & 0x10) == 0x10;
        let mlsperr = (cfsr & 0x20) == 0x20;
        let mmfarvalid = (cfsr & 0x80) == 0x80;

        let ibuserr = ((cfsr >> 8) & 0x01) == 0x01;
        let preciserr = ((cfsr >> 8) & 0x02) == 0x02;
        let impreciserr = ((cfsr >> 8) & 0x04) == 0x04;
        let unstkerr = ((cfsr >> 8) & 0x08) == 0x08;
        let stkerr = ((cfsr >> 8) & 0x10) == 0x10;
        let lsperr = ((cfsr >> 8) & 0x20) == 0x20;
        let bfarvalid = ((cfsr >> 8) & 0x80) == 0x80;

        let undefinstr = ((cfsr >> 16) & 0x01) == 0x01;
        let invstate = ((cfsr >> 16) & 0x02) == 0x02;
        let invpc = ((cfsr >> 16) & 0x04) == 0x04;
        let nocp = ((cfsr >> 16) & 0x08) == 0x08;
        let unaligned = ((cfsr >> 16) & 0x100) == 0x100;
        let divbysero = ((cfsr >> 16) & 0x200) == 0x200;

        let vecttbl = (hfsr & 0x02) == 0x02;
        let forced = (hfsr & 0x40000000) == 0x40000000;


        let _ = writer.write_fmt(format_args!("\r\n---| Fault Status |---\r\n"));

        if iaccviol {
            let _ =
                writer.write_fmt(format_args!("Instruction Access Violation:       {}\r\n",
                                              iaccviol));
        }
        if daccviol {
            let _ =
                writer.write_fmt(format_args!("Data Access Violation:              {}\r\n",
                                              daccviol));
        }
        if munstkerr {
            let _ =
                writer.write_fmt(format_args!("Memory Management Unstacking Fault: {}\r\n",
                                              munstkerr));
        }
        if mstkerr {
            let _ = writer.write_fmt(format_args!("Memory Management Stacking Fault:   {}\r\n",
                                                  mstkerr));
        }
        if mlsperr {
            let _ = writer.write_fmt(format_args!("Memory Management Lazy FP Fault:    {}\r\n",
                                                  mlsperr));
        }

        if ibuserr {
            let _ = writer.write_fmt(format_args!("Instruction Bus Error:              {}\r\n",
                                                  ibuserr));
        }
        if preciserr {
            let _ =
                writer.write_fmt(format_args!("Precise Data Bus Error:             {}\r\n",
                                              preciserr));
        }
        if impreciserr {
            let _ =
                writer.write_fmt(format_args!("Imprecise Data Bus Error:           {}\r\n",
                                              impreciserr));
        }
        if unstkerr {
            let _ =
                writer.write_fmt(format_args!("Bus Unstacking Fault:               {}\r\n",
                                              unstkerr));
        }
        if stkerr {
            let _ = writer.write_fmt(format_args!("Bus Stacking Fault:                 {}\r\n",
                                                  stkerr));
        }
        if lsperr {
            let _ = writer.write_fmt(format_args!("Bus Lazy FP Fault:                  {}\r\n",
                                                  lsperr));
        }

        if undefinstr {
            let _ =
                writer.write_fmt(format_args!("Undefined Instruction Usage Fault:  {}\r\n",
                                              undefinstr));
        }
        if invstate {
            let _ =
                writer.write_fmt(format_args!("Invalid State Usage Fault:          {}\r\n",
                                              invstate));
        }
        if invpc {
            let _ =
                writer.write_fmt(format_args!("Invalid PC Load Usage Fault:        {}\r\n", invpc));
        }
        if nocp {
            let _ =
                writer.write_fmt(format_args!("No Coprocessor Usage Fault:         {}\r\n", nocp));
        }
        if unaligned {
            let _ =
                writer.write_fmt(format_args!("Unaligned Access Usage Fault:       {}\r\n",
                                              unaligned));
        }
        if divbysero {
            let _ =
                writer.write_fmt(format_args!("Divide By Zero:                     {}\r\n",
                                              divbysero));
        }

        if vecttbl {
            let _ = writer.write_fmt(format_args!("Bus Fault on Vector Table Read:     {}\r\n",
                                                  vecttbl));
        }
        if forced {
            let _ = writer.write_fmt(format_args!("Forced Hard Fault:                  {}\r\n",
                                                  forced));
        }

        if mmfarvalid {
            let _ =
                writer.write_fmt(format_args!("Faulting Memory Address:            {:#010X}\r\n",
                                              mmfar));
        }
        if bfarvalid {
            let _ =
                writer.write_fmt(format_args!("Bus Fault Address:                  {:#010X}\r\n",
                                              bfar));
        }

        if cfsr == 0 && hfsr == 0 {
            let _ = writer.write_fmt(format_args!("No faults detected.\r\n"));
        } else {
            let _ =
                writer.write_fmt(format_args!("Fault Status Register (CFSR):       {:#010X}\r\n",
                                              cfsr));
            let _ =
                writer.write_fmt(format_args!("Hard Fault Status Register (HFSR):  {:#010X}\r\n",
                                              hfsr));
        }
    }

    pub unsafe fn statistics_str<W: Write>(&mut self, writer: &mut W) {

        if let Some(load_info) = parse_and_validate_load_info(self.text.as_ptr()) {
            // Flash addresses
            let flash_end = self.text.as_ptr().offset(self.text.len() as isize) as usize;
            let flash_data_end = self.text
                .as_ptr()
                .offset(load_info.pkg_name_offset as isize + load_info.pkg_name_size as isize) as
                                 usize;
            let flash_data_start = self.text.as_ptr().offset(load_info.got_offset as isize) as
                                   usize;
            let flash_text_start = self.text.as_ptr().offset(load_info.text_offset as isize) as
                                   usize;
            let flash_start = self.text.as_ptr() as usize;

            // Flash sizes
            let flash_data_size = load_info.got_size + load_info.data_size +
                                  load_info.pkg_name_size;
            let flash_text_size = load_info.text_size;
            let flash_header_size = mem::size_of::<LoadInfo>() + load_info.rel_data_size as usize;

            // SRAM addresses
            let sram_end = self.memory.as_ptr().offset(self.memory.len() as isize) as usize;
            let sram_grant_start = self.kernel_memory_break as usize;
            let sram_heap_end = self.app_heap_break as usize;
            let sram_heap_start = self.app_heap_start as usize;
            let sram_stack_data_boundary = self.stack_data_boundary as usize;
            let sram_stack_bottom = self.min_stack_pointer as usize;
            let sram_start = self.memory.as_ptr() as usize;

            // SRAM sizes
            let sram_grant_size = sram_end - sram_grant_start;
            let sram_heap_size = sram_heap_end - sram_heap_start;
            let sram_data_size = sram_heap_start - sram_stack_data_boundary;
            let sram_stack_size = sram_stack_data_boundary - sram_stack_bottom;
            let sram_grant_allocated = load_info.min_kernel_heap_len as usize;
            let sram_heap_allocated = load_info.min_app_heap_len as usize;
            let sram_stack_allocated = load_info.min_stack_len as usize;
            let sram_data_allocated = sram_data_size as usize;

            // checking on sram
            let mut sram_grant_error_str = "          ";
            if sram_grant_size > sram_grant_allocated {
                sram_grant_error_str = " EXCEEDED!"
            }
            let mut sram_heap_error_str = "          ";
            if sram_heap_size > sram_heap_allocated {
                sram_heap_error_str = " EXCEEDED!"
            }
            let mut sram_stack_error_str = "          ";
            if sram_stack_size > sram_stack_allocated {
                sram_stack_error_str = " EXCEEDED!"
            }

            // application statistics
            let events_queued = self.tasks.len();
            let syscall_count = self.syscall_count.get();
            let last_syscall = self.last_syscall.get();

            // register values
            let (r0, r1, r2, r3, r12, sp, lr, pc) = (self.r0(),
                                                     self.r1(),
                                                     self.r2(),
                                                     self.r3(),
                                                     self.r12(),
                                                     self.sp(),
                                                     self.lr(),
                                                     self.pc());

            // lst-file relative LR and PC
            let lr_lst_relative = 0x80000000 | (0xFFFFFFFE & (lr - flash_text_start as usize));
            let pc_lst_relative = 0x80000000 | (0xFFFFFFFE & (pc - flash_text_start as usize));

            let ypc_lst_relative = 0x80000000 |
                                   (0xFFFFFFFE & (self.yield_pc - flash_text_size as usize));


            let _ = writer.write_fmt(format_args!("\
            App: {}   -   [{:?}]\
            \r\n Events Queued: {}   Syscall Count: {}   ",
                                                  self.package_name,
                                                  self.state,
                                                  events_queued,
                                                  syscall_count,
                                                  ));

            let _ = match last_syscall {
                Some(syscall) => writer.write_fmt(format_args!("Last Syscall: {:?}", syscall)),
                None => writer.write_fmt(format_args!("Last Syscall: None")),
            };

            let _ = writer.write_fmt(format_args!("\
\r\n\
\r\n ╔═══════════╤══════════════════════════════════════════╗\
\r\n ║  Address  │ Region Name    Used | Allocated (bytes)  ║\
\r\n ╚{:#010X}═╪══════════════════════════════════════════╝\
\r\n             │ ▼ Grant      {:6} | {:6}{}\
  \r\n  {:#010X} ┼┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈\
\r\n             │ Unused\
  \r\n  {:#010X} ┼┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈\
\r\n             │ ▲ Heap       {:6} | {:6}{}    S\
  \r\n  {:#010X} ┼┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈ R\
\r\n             │ Data         {:6} | {:6}              A\
  \r\n  {:#010X} ┼┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈ M\
\r\n             │ ▼ Stack      {:6} | {:6}{}\
  \r\n  {:#010X} ┼┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈\
\r\n             │ Unused\
  \r\n  {:#010X} ┴───────────────────────────────────────────\
\r\n             .....\
  \r\n  {:#010X} ┬───────────────────────────────────────────\
\r\n             │ Unused\
  \r\n  {:#010X} ┼┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈ F\
\r\n             │ Data         {:6}                       L\
  \r\n  {:#010X} ┼┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈ A\
\r\n             │ Text         {:6}                       S\
  \r\n  {:#010X} ┼┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈ H\
\r\n             │ Header       {:6}\
  \r\n  {:#010X} ┴───────────────────────────────────────────\
\r\n\
  \r\n  R0 : {:#010X}    R6 : {:#010X}\
  \r\n  R1 : {:#010X}    R7 : {:#010X}\
  \r\n  R2 : {:#010X}    R8 : {:#010X}\
  \r\n  R3 : {:#010X}    R10: {:#010X}\
  \r\n  R4 : {:#010X}    R11: {:#010X}\
  \r\n  R5 : {:#010X}    R12: {:#010X}\
  \r\n  R9 : {:#010X} (Static Base Register)\
  \r\n  SP : {:#010X} (Process Stack Pointer)\
  \r\n  LR : {:#010X} [{:#010X} in lst file]\
  \r\n  PC : {:#010X} [{:#010X} in lst file]\
  \r\n YPC : {:#010X} [{:#010X} in lst file]\
\r\n\r\n",
  sram_end,
  sram_grant_size, sram_grant_allocated, sram_grant_error_str,
  sram_grant_start,
  sram_heap_end,
  sram_heap_size, sram_heap_allocated, sram_heap_error_str,
  sram_heap_start,
  sram_data_size, sram_data_allocated,
  sram_stack_data_boundary,
  sram_stack_size, sram_stack_allocated, sram_stack_error_str,
  sram_stack_bottom,
  sram_start,
  flash_end,
  flash_data_end,
  flash_data_size,
  flash_data_start,
  flash_text_size,
  flash_text_start,
  flash_header_size,
  flash_start,
  r0, self.stored_regs.r6,
  r1, self.stored_regs.r7,
  r2, self.stored_regs.r8,
  r3, self.stored_regs.r10,
  self.stored_regs.r4, self.stored_regs.r11,
  self.stored_regs.r5, r12,
  self.stored_regs.r9,
  sp,
  lr, lr_lst_relative,
  pc, pc_lst_relative,
  self.yield_pc, ypc_lst_relative,
  ));
        } else {
            let _ = writer.write_fmt(format_args!("Unknown Load Info\r\n"));
        }
    }
}

#[derive(Debug)]
struct LoadResult {
    /// The absolute address of the process entry point (i.e. `_start`).
    init_fn: usize,

    /// The lowest free address in process memory after allocating space for
    /// the stack, loading the GOT, data and BSS
    app_heap_start: *const u8,

    /// The initial stack pointer
    stack_data_boundary: *const u8,

    /// The length of the fixed segment, including the stack, GOT, .data, /
    /// BSS, and any necessary alignment
    fixed_len: u32,

    /// The process's package name (used for IPC)
    package_name: &'static str,
}

/// Loads the process into memory
///
/// Loads the process whos binary starts at `flash_start_addr` into the memory
/// region beginning at `mem_base`. The process binary must fit within
/// `mem_size` bytes.
///
/// This function will copy the GOT and data segment into memory as well as
/// zero out the BSS section. It performs relocation on the GOT and on
/// variables named in the relocation section of the binary.
///
/// Note: We place the stack at the bottom of the memory space so that a stack
/// overflow will trigger an MPU violation rather than overwriting GOT/BSS/.data
/// sections. The stack is not included in the flash data, however, which means
/// that the offset values for everything above the stack in the elf header need
/// to have the stack offset added.
///
/// The function returns a `LoadResult` containing metadata about the loaded
/// process or None if loading failed.
unsafe fn load(load_info: &'static LoadInfo,
               flash_start_addr: *const u8,
               mem_base: *mut u8,
               mem_size: usize)
               -> Option<LoadResult> {
    let package_name_byte_array =
        slice::from_raw_parts(flash_start_addr.offset(load_info.pkg_name_offset as isize),
                              load_info.pkg_name_size as usize);
    let mut app_name_str = "";
    let _ = str::from_utf8(package_name_byte_array).map(|name_str| { app_name_str = name_str; });

    let mut load_result = LoadResult {
        init_fn: 0,
        app_heap_start: ptr::null(),
        stack_data_boundary: ptr::null(),
        fixed_len: 0,
        package_name: app_name_str,
    };

    let text_start = flash_start_addr.offset(load_info.text_offset as isize);

    let rel_data: &[u32] =
        slice::from_raw_parts(flash_start_addr.offset(load_info.rel_data_offset as isize) as
                              *const u32,
                              (load_info.rel_data_size as usize) / mem::size_of::<u32>());

    let aligned_stack_len = align8!(load_info.min_stack_len);

    let got: &[u8] =
        slice::from_raw_parts(flash_start_addr.offset(load_info.got_offset as isize),
                              load_info.got_size as usize) as &[u8];

    let data: &[u8] =
        slice::from_raw_parts(flash_start_addr.offset(load_info.data_offset as isize),
                              load_info.data_size as usize);

    let got_base = mem_base.offset(aligned_stack_len as isize);
    let got_andthen_data: &mut [u8] =
        slice::from_raw_parts_mut(got_base,
                                  (load_info.got_size + load_info.data_size) as usize);

    let bss = mem_base.offset(aligned_stack_len as isize + load_info.bss_mem_offset as isize);

    // Total size of fixed segment
    let aligned_fixed_len = align8!(aligned_stack_len + load_info.data_size + load_info.got_size +
                                    load_info.bss_size);

    // Verify target data fits in memory before writing anything
    if (aligned_fixed_len) > mem_size as u32 {
        // When a kernel warning mechanism exists, this panic should be
        // replaced with that, but for now it seems more useful to bail out to
        // alert developers of why the app failed to load
        panic!("{:?} failed to load. Stack + Data + GOT + BSS ({}) > available memory ({})",
               load_result.package_name,
               aligned_fixed_len,
               mem_size);
    }

    // Copy the GOT and data into base memory
    for (orig, dest) in got.iter().chain(data.iter()).zip(got_andthen_data.iter_mut()) {
        *dest = *orig
    }

    // Zero out BSS
    intrinsics::write_bytes(bss, 0, load_info.bss_size as usize);


    // Helper function that fixes up GOT entries
    let fixup = |addr: &mut u32| {
        let entry = *addr;
        if (entry & 0x80000000) == 0 {
            // Regular data (memory relative)
            *addr = entry + (got_base as u32);
        } else {
            // rodata or function pointer (code relative)
            *addr = (entry ^ 0x80000000) + (text_start as u32);
        }
    };

    // Fixup Global Offset Table
    let mem_got: &mut [u32] = slice::from_raw_parts_mut(got_base as *mut u32,
                                                        (load_info.got_size as usize) /
                                                        mem::size_of::<u32>());

    for got_cur in mem_got {
        fixup(got_cur);
    }

    // Fixup relocation data
    for (i, addr) in rel_data.iter().enumerate() {
        if i % 2 == 0 {
            // Only the first of every 2 entries is an address
            fixup(&mut *(got_base.offset(*addr as isize) as *mut u32));
        }
    }

    // Entry point is offset from app code
    load_result.init_fn = flash_start_addr.offset(load_info.entry_offset as isize) as usize;

    load_result.app_heap_start = mem_base.offset(aligned_fixed_len as isize);
    load_result.stack_data_boundary = mem_base.offset(aligned_stack_len as isize);
    load_result.fixed_len = aligned_fixed_len;

    Some(load_result)
}
