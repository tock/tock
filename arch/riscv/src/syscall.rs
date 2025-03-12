// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Kernel-userland system call interface for RISC-V architecture.

use core::fmt::{Display, Formatter, Write};
use core::mem::size_of;
use core::ops::Range;

use crate::csr::mcause;
use kernel;
use kernel::errorcode::ErrorCode;
use kernel::syscall::ContextSwitchReason;
use kernel::utilities::capability_ptr::CapabilityPtr;
use kernel::utilities::machine_register::MachineRegister;

/// This holds all of the state that the kernel must keep for the process when
/// the process is not executing.
#[derive(Default)]
#[repr(C)]
pub struct RiscvStoredState {
    /// Store all of the app registers.
    regs: [MachineRegister; 31],

    /// This holds the PC value of the app when the exception/syscall/interrupt
    /// occurred. We also use this to set the PC that the app should start
    /// executing at when it is resumed/started.
    pc: CapabilityPtr,

    /// This holds the default data capability. Switched with the kernel DDC if we trap from an app.
    #[cfg(target_feature = "xcheri")]
    ddc: CapabilityPtr,

    /// We need to store the mcause CSR between when the trap occurs and after
    /// we exit the trap handler and resume the context switching code.
    mcause: usize,

    /// We need to store the mtval CSR for the process in case the mcause
    /// indicates a fault. In that case, the mtval contains useful debugging
    /// information.
    mtval: usize,
}

pub struct DdcDisplay<'a> {
    _state: &'a RiscvStoredState,
}

impl Display for DdcDisplay<'_> {
    fn fmt(&self, _f: &mut Formatter<'_>) -> core::fmt::Result {
        #[cfg(target_feature = "xcheri")]
        {
            return _f.write_fmt(format_args!("DDC: {:#010X}", self._state.ddc));
        }
        #[cfg(not(target_feature = "xcheri"))]
        core::fmt::Result::Ok(())
    }
}

impl RiscvStoredState {
    pub fn get_ddc_display(&self) -> DdcDisplay {
        DdcDisplay { _state: self }
    }
}

// Because who would ever need offsetof?
#[cfg(target_feature = "xcheri")]
pub const CAUSE_OFFSET: usize = size_of::<MachineRegister>() * 33;
#[cfg(not(target_feature = "xcheri"))]
pub const CAUSE_OFFSET: usize = size_of::<MachineRegister>() * 32;

// Named offsets into the stored state registers.  These needs to be kept in
// sync with the register save logic in _start_trap() as well as the register
// restore logic in switch_to_process() below.
const R_RA: usize = 0;
const R_SP: usize = 1;
const R_A0: usize = 9;
const R_A1: usize = 10;
const R_A2: usize = 11;
const R_A3: usize = 12;
const R_A4: usize = 13;

/// Values for encoding the stored state buffer in a binary slice.
const VERSION: usize = 1;
const STORED_STATE_SIZE: usize = size_of::<RiscvStoredState>();
#[cfg(target_arch = "riscv32")]
const TAG: [u8; 4] = [b'r', b'v', b'5', b'i'];
#[cfg(any(target_arch = "riscv64", not(target_os = "none")))]
const TAG: [u8; 8] = [b'r', b'v', b'5', b'i', b'r', b'v', b'5', b'i'];
const METADATA_LEN: usize = 3;

// TODO: CHERI. This seems to be for swap or some such. Needs thinking about.

const VERSION_IDX: usize = 0;
const SIZE_IDX: usize = 1;
const TAG_IDX: usize = 2;
const PC_IDX: usize = 3;
const MCAUSE_IDX: usize = 4;
const MTVAL_IDX: usize = 5;
const REGS_IDX: usize = 6;
const REGS_RANGE: Range<usize> = REGS_IDX..REGS_IDX + 31;

const USIZE_SZ: usize = size_of::<usize>();
fn usize_byte_range(index: usize) -> Range<usize> {
    index * USIZE_SZ..(index + 1) * USIZE_SZ
}

fn usize_from_u8_slice(slice: &[u8], index: usize) -> Result<usize, ErrorCode> {
    let range = usize_byte_range(index);
    Ok(usize::from_le_bytes(
        slice
            .get(range)
            .ok_or(ErrorCode::SIZE)?
            .try_into()
            .or(Err(ErrorCode::FAIL))?,
    ))
}

fn write_usize_to_u8_slice(val: usize, slice: &mut [u8], index: usize) {
    let range = usize_byte_range(index);
    slice[range].copy_from_slice(&val.to_le_bytes());
}

impl core::convert::TryFrom<&[u8]> for RiscvStoredState {
    type Error = ErrorCode;
    fn try_from(ss: &[u8]) -> Result<RiscvStoredState, Self::Error> {
        if ss.len() == size_of::<RiscvStoredState>() + METADATA_LEN * USIZE_SZ
            && usize_from_u8_slice(ss, VERSION_IDX)? == VERSION
            && usize_from_u8_slice(ss, SIZE_IDX)? == STORED_STATE_SIZE
            && usize_from_u8_slice(ss, TAG_IDX)? == usize::from_le_bytes(TAG)
        {
            let mut res = RiscvStoredState {
                regs: [0usize.into(); 31],
                pc: (usize_from_u8_slice(ss, PC_IDX)? as usize).into(),
                #[cfg(target_feature = "xcheri")]
                ddc: 0usize.into(),
                mcause: usize_from_u8_slice(ss, MCAUSE_IDX)?,
                mtval: usize_from_u8_slice(ss, MTVAL_IDX)?,
            };
            for (i, v) in (REGS_RANGE).enumerate() {
                res.regs[i] = (usize_from_u8_slice(ss, v)? as usize).into();
            }
            Ok(res)
        } else {
            Err(ErrorCode::FAIL)
        }
    }
}

/// Implementation of the `UserspaceKernelBoundary` for the RISC-V architecture.
pub struct SysCall(());

impl SysCall {
    pub const unsafe fn new() -> SysCall {
        SysCall(())
    }
}

impl kernel::syscall::UserspaceKernelBoundary for SysCall {
    type StoredState = RiscvStoredState;

    fn initial_process_app_brk_size(&self) -> usize {
        // The RV32I UKB implementation does not use process memory for any
        // context switch state. Therefore, we do not need any process-accessible
        // memory to start with to successfully context switch to the process the
        // first time.
        0
    }

    unsafe fn initialize_process(
        &self,
        accessible_memory_start: *const u8,
        _app_brk: *const u8,
        state: &mut Self::StoredState,
    ) -> Result<(), ()> {
        // Need to clear the stored state when initializing.
        state.regs.iter_mut().for_each(|x| *x = usize::into(0));
        // CHERI note: this PC cannot be executed. It will always be replaced with an initial fn.
        state.pc = usize::into(0);

        #[cfg(target_feature = "xcheri")]
        {
            use kernel::utilities::capability_ptr::CapabilityPtrPermissions::ReadWrite;
            let start = accessible_memory_start as usize;

            state.ddc = CapabilityPtr::new_with_authority(
                start as *const (),
                start,
                (_app_brk as usize) - start,
                ReadWrite,
            );
        }

        state.mcause = 0;

        // The first time the process runs we need to set the initial stack
        // pointer in the sp register.
        //
        // We do not pre-allocate any stack for RV32I processes.
        state.regs[R_SP] = usize::into(accessible_memory_start as usize);

        // We do not use memory for UKB, so just return ok.
        Ok(())
    }

    unsafe fn set_syscall_return_value(
        &self,
        _accessible_memory_start: *const u8,
        _app_brk: *const u8,
        state: &mut Self::StoredState,
        return_value: kernel::syscall::SyscallReturn,
    ) -> Result<(), ()> {
        // Encode the system call return value into registers,
        // available for when the process resumes

        // We need to use a bunch of split_at_mut's to have multiple
        // mutable borrows into the same slice at the same time.
        //
        // Since the compiler knows the size of this slice, and these
        // calls will be optimized out, we use one to get to the first
        // register (A0)
        let (_, r) = state.regs.split_at_mut(R_A0);

        // This comes with the assumption that the respective
        // registers are stored at monotonically increasing indices
        // in the register slice
        let (a0slice, r) = r.split_at_mut(R_A1 - R_A0);
        let (a1slice, r) = r.split_at_mut(R_A2 - R_A1);
        let (a2slice, a3slice) = r.split_at_mut(R_A3 - R_A2);

        if (core::mem::size_of::<usize>() == core::mem::size_of::<u32>())
            && !kernel::config::CONFIG.is_cheri
        {
            // On non-CHERI, 32-bit platforms we use trd104 for backwards compatability
            kernel::utilities::arch_helpers::encode_syscall_return_with_variant::<
                kernel::utilities::arch_helpers::TRD104SyscallReturnVariant,
            >(
                &return_value,
                &mut a0slice[0],
                &mut a1slice[0],
                &mut a2slice[0],
                &mut a3slice[0],
            );
        } else {
            // Other platforms can use TRD105
            kernel::utilities::arch_helpers::encode_syscall_return_with_variant::<
                kernel::utilities::arch_helpers::TRD105SyscallReturnVariant,
            >(
                &return_value,
                &mut a0slice[0],
                &mut a1slice[0],
                &mut a2slice[0],
                &mut a3slice[0],
            );
        }

        // We do not use process memory, so this cannot fail.
        Ok(())
    }

    unsafe fn set_process_function(
        &self,
        _accessible_memory_start: *const u8,
        _app_brk: *const u8,
        state: &mut RiscvStoredState,
        callback: kernel::process::FunctionCall,
    ) -> Result<(), ()> {
        // Set the register state for the application when it starts
        // executing. These are the argument registers.
        state.regs[R_A0] = callback.argument0.into();
        state.regs[R_A1] = callback.argument1.into();
        state.regs[R_A2] = callback.argument2.into();
        state.regs[R_A3] = callback.argument3;

        // We also need to set the return address (ra) register so that the new
        // function that the process is running returns to the correct location.
        // Note, however, that if this function happens to be the first time the
        // process is executing then `state.pc` is invalid/useless, but the
        // application must ignore it anyway since there is nothing logically
        // for it to return to. So this doesn't hurt anything.
        state.regs[R_RA] = MachineRegister::from(state.pc);

        // Save the PC we expect to execute.
        // On CHERI we are basically forcing a jump, so caller better have the correct bounds.
        state.pc = callback.pc;

        Ok(())
    }

    // Mock implementation for tests on Travis-CI.
    #[cfg(not(any(doc, target_os = "none")))]
    unsafe fn switch_to_process(
        &self,
        _accessible_memory_start: *const u8,
        _app_brk: *const u8,
        _state: &mut RiscvStoredState,
    ) -> (ContextSwitchReason, Option<*const u8>) {
        // Convince lint that 'mcause' and 'R_A4' are used during test build
        let _cause = mcause::Trap::from(_state.mcause);
        let _arg4 = _state.regs[R_A4];
        unimplemented!()
    }

    #[cfg(any(doc, target_os = "none"))]
    unsafe fn switch_to_process(
        &self,
        _accessible_memory_start: *const u8,
        _app_brk: *const u8,
        state: &mut RiscvStoredState,
    ) -> (ContextSwitchReason, Option<*const u8>) {
        use kernel::{csr_op, csr_ptr, is_cheri, ldptr, ptrreg, ptrregn, stptr, stx};

        // We need to ensure that the compiler does not reorder
        // kernel memory writes to after the userspace context switch
        // to ensure we provide a consistent memory view of
        // application-accessible buffers.
        //
        // The compiler will not be able to reorder memory accesses
        // beyond this point, as the "nomem" option on the asm!-block
        // is not set, hence the compiler has to assume the assembly
        // will issue arbitrary memory accesses (acting as a compiler
        // fence).
        core::arch::asm!(crate::easm!("
          // Before switching to the app we need to save some kernel registers
          // to the kernel stack, specifically ones which we can't mark as
          // clobbered in the asm!() block. We then save the stack pointer in
          // the mscratch CSR so we can retrieve it after returning to the
          // kernel from the app.
          //
          // A few values get saved to the kernel stack, including an app
          // register temporarily after entering the trap handler. Here is a
          // memory map to make it easier to keep track:
          //
          // ```
          //  8*CLEN(sp):          <- original stack pointer
          //  7*CLEN(sp):
          //  6*CLEN(sp): x9  / s1
          //  5*CLEN(sp): x8  / s0 / fp
          //  4*CLEN(sp): x4  / tp
          //  3*CLEN(sp): x3  / gp
          //  2*CLEN(sp): x10 / a0 (*state, Per-process StoredState struct)
          //  1*CLEN(sp): custom trap handler address
          //  0*CLEN(sp): scratch space, having s1 written to by the trap handler
          //                    <- new stack pointer
          // ```

          addi sp, sp, -8*{CLEN_BYTES}  // Move the stack pointer down to make room.

          // Load the address of `_start_app_trap` into `1*CLEN(sp)`. We swap our
          // stack pointer into the mscratch CSR and the trap handler will load
          // and jump to the address at this offset.
          la    t0, 100f      // t0 = _start_app_trap

          // Save all registers on the kernel stack which cannot be clobbered
          // by an asm!() block. These are mostly registers which have a
          // designated purpose (e.g. stack pointer) or are used internally
          // by LLVM. These are:
          // s1 (used internally by LLVM)
          // fp (can't be clobbered / used as an operand)
          // tp (can't be clobbered / used as an operand)
          // gp (can't be clobbered / used as an operand)
          // and a0/to which is the process state pointer/label to jump to
          " FOR_EACH("Reg" in ["s1","s0","tp","gp","a0","t0"] :
            stptr!() ptrreg!() "\\()\\Reg, (6-FOR_N)*{CLEN_BYTES}(sp)"
          )"

          // sw x0, 0*4(sp)   // Reserved as scratch space for the trap handler

          // -----> All required registers saved to the stack.
          //        sp holds the updated stack pointer, a0 the per-process state

          // From here on we can't allow the CPU to take interrupts anymore, as
          // we re-route traps to `_start_app_trap` below (by writing our stack
          // pointer into the mscratch CSR), and we rely on certain CSRs to not
          // be modified or used in their intermediate states (e.g., mepc).
          //
          // We atomically switch to user-mode and re-enable interrupts using
          // the `mret` instruction below.
          //
          // If interrupts are disabled _after_ setting mscratch, this result in
          // the race condition of [PR 2308](https://github.com/tock/tock/pull/2308)

          // Therefore, clear the following bits in mstatus first:
          //   0x00000008 -> bit 3 -> MIE (disabling interrupts here)
          // + 0x00001800 -> bits 11,12 -> MPP (switch to usermode on mret)
          li    t0, 0x00001808
          csrc  mstatus, t0         // clear bits in mstatus

          // Afterwards, set the following bits in mstatus:
          //   0x00000080 -> bit 7 -> MPIE (enable interrupts on mret)
          li    t0, 0x00000080
          csrs  mstatus, t0         // set bits in mstatus
          " csr_op!("mscratch" <- "sp")" // Store `sp` in mscratch CSR. Discard the
                                       // prior value, must have been set to zero.

          // We have to set the mepc CSR with the PC we want the app to start
          // executing at. This has been saved in Riscv32iStoredState for us
          // (either when the app returned back to the kernel or in the
          // `set_process_function()` function).
          // Retrieve the PC from Riscv32iStoredState
          " ldptr!() ptrreg!("t0") ", 31*{CLEN_BYTES}(a0)
          " csr_op!("mepc" <- "t0") "// Set mepc CSR to the app's PC.

          // Restore all of the app registers from what we saved. If this is the
          // first time running the app then most of these values are
          // irrelevant, However we do need to set the four arguments to the
          // `_start_ function in the app. If the app has been executing then
          // this allows the app to correctly resume.

          // We do a little switcheroo here, and place the per-process stored
          // state pointer into the `sp` register instead of `a0`. Doing so
          // allows us to use compressed instructions for all of these loads:
          mv    sp,  a0             // sp <- a0 (per-process stored state)
          " FOR_RANGE("regn" in 1 .. 32 :
                ".if \\regn != 2 // do last since we overwrite our pointer
                    " ldptr!() ptrregn!() "\\()\\regn, (\\regn-1)*{CLEN_BYTES}(sp)
                 .endif"
          )

          ".if " is_cheri!() "
              // Load processes DDC. We cannot restore it before the last load has happened.
              // We can use mtdc as a scratch register (have it hold ct1), so ct1 can hold ddc.
              // DDC should currently hold the kernel DDC, which should eventually go in mtdc
              cspecialw   mtdc, ct1
              " ldptr!() ptrreg!("t1") ", 32*{CLEN_BYTES}(sp)
          .endif

          " ldptr!() ptrregn!(2) ", 1*{CLEN_BYTES}(sp) // sp. Do last since we overwrite our pointer.

          .if " is_cheri!() "
              // Currently:
              //    mtdc holds ct1
              //    ct1 holds ddc
              //    ddc holds mdtc
              cspecialrw ct1, ddc, ct1
              cspecialrw ct1, mtdc, ct1
          .endif

          // Call mret to jump to where mepc points, switch to user mode, and
          // start running the app.
          mret

          // The global trap handler will jump to this address when catching a
          // trap while the app is executing (address loaded into the mscratch
          // CSR).
          //
          // This custom trap handler is responsible for saving application
          // state, clearing the custom trap handler (mscratch = 0), and
          // restoring the kernel context.
        100: // _start_app_trap

          // At this point all we know is that we entered the trap handler from
          // an app. We don't know _why_ we got a trap, it could be from an
          // interrupt, syscall, or fault (or maybe something else). Therefore
          // we have to be very careful not to overwrite any registers before we
          // have saved them.
          //
          // The global trap handler has swapped the app's `s0` into the
          // mscratch CSR, which now contains the address of our stack pointer.
          // The global trap handler further clobbered `s1`, which now contains
          // the address of `_start_app_trap`. The app's `s1` is saved at
          // `0*CLEN(s0)`.
          //
          // Thus we can clobber `s1` and load the address of the per-process
          // stored state:
          //
          " ldptr!() ptrreg!("s1") ", 2*{CLEN_BYTES}(s0)
          // With the per-process stored state address in `s1`, save all
          // non-clobbered registers. Save the `sp` first, then do the same
          // switcheroo as above, moving the per-process stored state pointer
          // into `sp`. This allows us to use compressed instructions for all
          // these stores:
          " stptr!() ptrregn!(2) ", 1*{CLEN_BYTES}(s1) // Save app's sp

          mv    sp,  s1             // sp <- s1 (per-process stored state)

          // Now, store relative to `sp` (per-process stored state) with
          // compressed instructions:

          // ------------------------> sp, saved above
          // ------------------------> s0, in mscratch right now
          // ------------------------> s1, stored at 0*4(s0) right now

          "  FOR_RANGE("regn" in 1 .. 32 :
                ".if \\regn != 2 && \\regn != 8 && \\regn != 9
                " stptr!() ptrregn!() "\\()\\regn, (\\regn-1)*{CLEN_BYTES}(sp)
                .endif"
            )
          "

          // At this point, we can restore s0 into our stack pointer:
          mv   sp, s0

          // Now retrieve the original value of s1 and save that as well. We
          // must not clobber s1, our per-process stored state pointer.
          " ldptr!() ptrreg!("s0") ", 0*{CLEN_BYTES}(sp)  // s0 = app s1 (from trap handler scratch space)
          " stptr!() ptrreg!("s0") ", 8*{CLEN_BYTES}(s1) // Save app s1 to per-process state

          // Retrieve the original value of s0 from the mscratch CSR, save it.
          //
          // This will also restore the kernel trap handler by writing zero to
          // the CSR. `csrrw` allows us to read and write the CSR in a single
          // instruction:
          li s0, 0
          " csr_op!("s0" <- "mscratch" <- "s0") " // s0 <- mscratch[app s0] <- zero
          " stptr!() ptrreg!("s0") ", 7*{CLEN_BYTES}(s1) // Save app s0 to per-process state

          .if " is_cheri!() "
              // We now need to save the trapped DDC (which is in mtdc)
              // and restore mtdc it to what it was for the next trap
              cspecialr ct0, ddc
              cspecialrw ct0, mtdc, ct0
              sc        ct0, 32*{CLEN_BYTES}(s1)
          .endif

          // -------------------------------------------------------------------
          // At this point, the entire app register file is saved. We also
          // restored the kernel trap handler. We have restored the following
          // kernel registers:
          //
          // - sp: kernel stack pointer
          // - s1: per-process stored state pointer
          //
          // We avoid clobbering those registers from this point onward.
          // -------------------------------------------------------------------

          // We also need to store some other information about the trap reason,
          // present in CSRs:
          //
          // - the app's PC (mepc),
          // - the trap reason (mcause),
          // - the trap 'value' (mtval, e.g., faulting address).
          //
          // We need to store mcause because we use that to determine why the
          // app stopped executing and returned to the kernel. We store mepc
          // because it is where we need to return to in the app at some
          // point. We need to store mtval in case the app faulted and we need
          // mtval to help with debugging.
          //
          // We use `s0` as a scratch register, as it fits into the 3-bit
          // register argument of RISC-V compressed loads / stores:

          // Save the PC to the stored state struct. We also load the address
          // of _return_to_kernel into it, as this will be where we jump on
          // the mret instruction, which leaves the trap handler.
          la    s0, 300f            // Load _return_to_kernel into t0.

          .if " is_cheri!() "
              // On CHERI, we must add some metadata to s0 as we will use it to take traps
              cspecialr ct1, pcc
              csetaddr  cs0, ct1, s0
          .endif

          " csr_op!("s0" <- "mepc" <- "s0") " // s0 <- mepc[app pc] <- _return_to_kernel
          " stptr!() ptrreg!("s0") ", 31*{CLEN_BYTES}(s1) // Store app's pc in stored state struct.

          // Save mtval to the stored state struct
          csrr  s0, mtval
          " stx!() " s0, ({CAUSE_OFFSET} + {XLEN_BYTES})(s1)

          // Save mcause and leave it loaded into a0, as we call a function
          // with it below:
          csrr  a0, mcause
          " stx!() " a0, {CAUSE_OFFSET}(s1)

          // Depending on the value of a0, we might be calling into a function
          // while still in the trap handler. The callee may rely on the `gp`,
          // `tp`, and `fp` (s0) registers to be set correctly. Thus we restore
          // them here, as we need to do anyways. They are saved registers,
          // and so we avoid clobbering them beyond this point.
          //
          // We do not restore `s1`, as we need to move it back into `a0`
          // _after_ potentially invoking the _disable_interrupt_... function.
          // LLVM relies on it to not be clobbered internally, but it is not
          // part of the RISC-V C ABI, which we need to follow here.
          //
          " ldptr!() ptrreg!("s0") ", 5*{CLEN_BYTES}(sp)  // fp/s0: Restore the frame pointer
          " ldptr!() ptrreg!("tp") ", 4*{CLEN_BYTES}(sp)  // fp/s0: Restore the frame pointer
          " ldptr!() ptrreg!("gp") ", 3*{CLEN_BYTES}(sp)  // fp/s0: Restore the frame pointer

          // --------------------------------------------------------------------
          // From this point onward, avoid clobbering the following registers:
          //
          // - x2 / sp: kernel stack pointer
          // - x3 / gp: kernel global pointer
          // - x4 / tp: kernel thread pointer
          // - x8 / s0 / fp: kernel frame pointer
          // - x9 / s1: per-process stored state pointer
          //
          // --------------------------------------------------------------------

          // Now we need to check if this was an interrupt, and if it was,
          // then we need to disable the interrupt before returning from this
          // trap handler so that it does not fire again.
          //
          // If mcause is greater than or equal to zero this was not an
          // interrupt (i.e. the most significant bit is not 1). In this case,
          // jump to _start_app_trap_continue.
          bge   a0, zero, 200f

          // This was an interrupt. Call the interrupt disable function, with
          // mcause already loaded in a0.
          //
          // This may clobber all caller-saved registers. However, at this
          // stage, we only restored `sp`, `s1`, and the registers above, all of
          // which are saved. Thus we don't have to worry about the function
          // call clobbering these registers.
          //
          jal  ra, _disable_interrupt_trap_rust_from_app

        200: // _start_app_trap_continue

          // Need to set mstatus.MPP to 0b11 so that we stay in machine mode.
          //
          // We use `a0` as a scratch register, as we are allowed to clobber it
          // here, and it fits into a compressed load instruction. We must avoid
          // using restored saved registers like `s0`, etc.
          //
          li    a0, 0x1800          // Load 0b11 to the MPP bits location in a0
          csrs  mstatus, a0         // mstatus |= a0

          // Use mret to exit the trap handler and return to the context
          // switching code. We loaded the address of _return_to_kernel
          // into mepc above.
          mret

          // This is where the trap handler jumps back to after the app stops
          // executing.
        300: // _return_to_kernel

          // We have already stored the app registers in the trap handler. We
          // have further restored `gp`, `tp`, `fp`/`s0` and the stack pointer.
          //
          // The only other non-clobbered registers are `s1` and `a0`, where
          // `a0` needs to hold the per-process state pointer currently stored
          // in `s1`, and the original value of `s1` is saved on the stack.
          // Restore them:
          //
          mv    a0, s1              // a0 = per-process stored state
          " ldptr!() ptrreg!("s1") ", 6*{CLEN_BYTES}(sp)  // restore s1 (used by LLVM internally)

          // We need thus need to mark all registers as clobbered, except:
          //
          // - x2  (sp)
          // - x3  (gp)
          // - x4  (tp)
          // - x8  (fp)
          // - x9  (s1)
          // - x10 (a0)

          addi sp, sp, 8*{CLEN_BYTES}   // Reset kernel stack pointer"),
            // The register to put the state struct pointer in is not
            // particularly relevant, however we must avoid using t0
            // as that is overwritten prior to being accessed
            // (although stored and later restored) in the assembly
            CLEN_BYTES = const size_of::<MachineRegister>(),
            XLEN_BYTES = const size_of::<usize>(),
            CAUSE_OFFSET = const crate::syscall::CAUSE_OFFSET,
            in("a0") kernel::polyfill::core::ptr::from_mut(state),
            // Clobber all registers which can be marked as clobbered, except
            // for `a0` / `x10`. By making it retain the value of `&mut state`,
            // which we need to stack manually anyway, we can avoid Rust/LLVM
            // stacking it redundantly for us.
            out("x1") _, out("x5") _, out("x6") _, out("x7") _, out("x11") _,
            out("x12") _, out("x13") _, out("x14") _, out("x15") _, out("x16") _,
            out("x17") _, out("x18") _, out("x19") _, out("x20") _, out("x21") _,
            out("x22") _, out("x23") _, out("x24") _, out("x25") _, out("x26") _,
            out("x27") _, out("x28") _, out("x29") _, out("x30") _, out("x31") _,
        );

        let ret = match mcause::Trap::from(state.mcause) {
            mcause::Trap::Interrupt(_intr) => {
                // An interrupt occurred while the app was running.
                ContextSwitchReason::Interrupted
            }
            mcause::Trap::Exception(excp) => {
                match excp {
                    // The SiFive HiFive1 board allegedly does not support
                    // u-mode, so the m-mode ecall is handled here too.
                    mcause::Exception::UserEnvCall | mcause::Exception::MachineEnvCall => {
                        // Need to increment the PC so when we return we start at the correct
                        // instruction. The hardware does not do this for us.
                        state.pc += 4;

                        let syscall = kernel::syscall::Syscall::from_register_arguments(
                            state.regs[R_A4].as_usize() as u8,
                            state.regs[R_A0].as_usize(),
                            state.regs[R_A1],
                            state.regs[R_A2],
                            state.regs[R_A3],
                        );

                        match syscall {
                            Some(s) => ContextSwitchReason::SyscallFired { syscall: s },
                            None => ContextSwitchReason::Fault,
                        }
                    }
                    _ => {
                        // All other exceptions result in faulted state
                        ContextSwitchReason::Fault
                    }
                }
            }
        };
        let new_stack_pointer = state.regs[R_SP];
        (ret, Some(new_stack_pointer.as_capability_ptr().as_ptr()))
    }

    unsafe fn print_context(
        &self,
        _accessible_memory_start: *const u8,
        _app_brk: *const u8,
        state: &RiscvStoredState,
        writer: &mut dyn Write,
    ) {
        let _ = writer.write_fmt(format_args!(
            "\
             \r\n R0 : {:#010X}    R16: {:#010X}\
             \r\n R1 : {:#010X}    R17: {:#010X}\
             \r\n R2 : {:#010X}    R18: {:#010X}\
             \r\n R3 : {:#010X}    R19: {:#010X}\
             \r\n R4 : {:#010X}    R20: {:#010X}\
             \r\n R5 : {:#010X}    R21: {:#010X}\
             \r\n R6 : {:#010X}    R22: {:#010X}\
             \r\n R7 : {:#010X}    R23: {:#010X}\
             \r\n R8 : {:#010X}    R24: {:#010X}\
             \r\n R9 : {:#010X}    R25: {:#010X}\
             \r\n R10: {:#010X}    R26: {:#010X}\
             \r\n R11: {:#010X}    R27: {:#010X}\
             \r\n R12: {:#010X}    R28: {:#010X}\
             \r\n R13: {:#010X}    R29: {:#010X}\
             \r\n R14: {:#010X}    R30: {:#010X}\
             \r\n R15: {:#010X}    R31: {:#010X}\
             \r\n PC : {:#010X}    {}           \
             \r\n\
             \r\n mcause: {:#010X} (",
            <MachineRegister as From<usize>>::from(0usize),
            state.regs[15],
            state.regs[0],
            state.regs[16],
            state.regs[1],
            state.regs[17],
            state.regs[2],
            state.regs[18],
            state.regs[3],
            state.regs[19],
            state.regs[4],
            state.regs[20],
            state.regs[5],
            state.regs[21],
            state.regs[6],
            state.regs[22],
            state.regs[7],
            state.regs[23],
            state.regs[8],
            state.regs[24],
            state.regs[9],
            state.regs[25],
            state.regs[10],
            state.regs[26],
            state.regs[11],
            state.regs[27],
            state.regs[12],
            state.regs[28],
            state.regs[13],
            state.regs[29],
            state.regs[14],
            state.regs[30],
            state.pc,
            state.get_ddc_display(),
            state.mcause,
        ));
        let cause = mcause::Trap::from(state.mcause);
        crate::print_mcause(cause, writer);
        let _ = writer.write_fmt(format_args!(
            ")\
             \r\n mtval:  {:#010X} (",
            state.mtval,
        ));
        crate::print_mtval(cause, state.mtval, writer);
        let _ = writer.write_fmt(format_args!(")\r\n\r\n",));
    }

    fn store_context(&self, state: &RiscvStoredState, out: &mut [u8]) -> Result<usize, ErrorCode> {
        const USIZE_SZ: usize = size_of::<usize>();
        if out.len() >= size_of::<RiscvStoredState>() + METADATA_LEN * USIZE_SZ {
            write_usize_to_u8_slice(VERSION, out, VERSION_IDX);
            write_usize_to_u8_slice(STORED_STATE_SIZE, out, SIZE_IDX);
            write_usize_to_u8_slice(usize::from_le_bytes(TAG), out, TAG_IDX);
            write_usize_to_u8_slice(state.pc.addr(), out, PC_IDX);
            write_usize_to_u8_slice(state.mcause, out, MCAUSE_IDX);
            write_usize_to_u8_slice(state.mtval, out, MTVAL_IDX);
            for (i, v) in state.regs.iter().enumerate() {
                write_usize_to_u8_slice(v.as_usize(), out, REGS_IDX + i);
            }
            // +3 for pc, mcause, mtval
            Ok((state.regs.len() + 3 + METADATA_LEN) * USIZE_SZ)
        } else {
            Err(ErrorCode::SIZE)
        }
    }
}
