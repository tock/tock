//! A temporary support module to ease the tock 2.0 migration
//!
//! This module contains structs and traits that have the same interface as tock 2.0 versions, yet
//! these structs and traits adapt their implementations to the Tock 1.x APIs under the hood.

use crate::AppSlice;
use crate::Callback;
use crate::ErrorCode;
use crate::ReturnCode;
use crate::Shared;
pub type ProcessId = crate::AppId;

/// Possible return values of a `command` driver method, as specified
/// in TRD104. This has been artificially limited to only the subset of
/// CommandReturn types Tock 1.0 supports. Once we switch to Tock OS 2.0, more
/// options will be available
pub struct CommandReturn(ReturnCode);
impl CommandReturn {
    /// Command error
    pub fn failure(rc: ErrorCode) -> Self {
        CommandReturn(rc.into())
    }

    /// Successful command
    pub fn success() -> Self {
        CommandReturn(ReturnCode::SUCCESS)
    }

    /// Successful command with an additional 32-bit data field
    pub fn success_u32(data0: u32) -> Self {
        CommandReturn(ReturnCode::SuccessWithValue {
            value: data0 as usize,
        })
    }
}

/// Trait for capsules implementing peripheral driver system calls
/// specified in TRD104. The kernel translates the values passed from
/// userspace into Rust types and includes which process is making the
/// call. All of these system calls perform very little synchronous work;
/// long running computations or I/O should be split-phase, with an upcall
/// indicating their completion.
///
/// The exact instances of each of these methods (which identifiers are valid
/// and what they represents) are specific to the peripheral system call
/// driver.
#[allow(unused_variables)]
pub trait Driver {
    /// System call for a process to provide an upcall function pointer to
    /// the kernel. Peripheral system call driver capsules invoke
    /// upcalls to indicate events have occurred. These events are typically triggered
    /// in response to `command` calls. For example, a command that sets a timer to
    /// fire in the future will cause an upcall to invoke after the command returns, when
    /// the timer expires, while a command to sample a sensor will cause an upcall to
    /// invoke when the sensor value is ready.
    fn subscribe(
        &self,
        subscribe_num: usize,
        upcall: Upcall,
        process_id: ProcessId,
    ) -> Result<Upcall, (Upcall, ErrorCode)> {
        Err((upcall, ErrorCode::NOSUPPORT))
    }

    /// System call for a process to perform a short synchronous operation
    /// or start a long-running split-phase operation (whose completion
    /// is signaled with an upcall). Command 0 is a reserved command to
    /// detect if a peripheral system call driver is installed and must
    /// always return a CommandReturn::Success.
    fn command(
        &self,
        cmd_num: usize,
        r2: usize,
        r3: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        CommandReturn::failure(ErrorCode::NOSUPPORT)
    }

    /// System call for a process to pass a buffer (a ReadWriteAppSlice) to
    /// the kernel that the kernel can either read or write. The kernel calls
    /// this method only after it checks that the entire buffer is
    /// within memory the process can both read and write.
    fn allow_readwrite(
        &self,
        allow_num: usize,
        slice: ReadWriteAppSlice,
        process_id: ProcessId,
    ) -> Result<ReadWriteAppSlice, (ReadWriteAppSlice, ErrorCode)> {
        Err((slice, ErrorCode::NOSUPPORT))
    }
}

// Auto implementing the Tock OS 1.x driver from the 2.0 driver implementation
impl<T: Driver> crate::Driver for T {
    fn subscribe(
        &self,
        minor_num: usize,
        callback: Option<Callback>,
        app_id: crate::AppId,
    ) -> ReturnCode {
        let result = self.subscribe(minor_num, Upcall(callback), app_id);
        result.map(|_| ()).map_err(|e| e.1).into()
    }

    fn command(
        &self,
        minor_num: usize,
        r2: usize,
        r3: usize,
        caller_id: crate::AppId,
    ) -> ReturnCode {
        self.command(minor_num, r2, r3, caller_id).0
    }

    fn allow(
        &self,
        app: crate::AppId,
        minor_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        let result = self.allow_readwrite(minor_num, ReadWriteAppSlice(slice), app);
        result.map(|_| ()).map_err(|e| e.1).into()
    }
}

#[derive(Clone, Copy, Default)]
pub struct Upcall(Option<Callback>);

impl Upcall {
    /// Tell the scheduler to run this upcall for the process.
    ///
    /// The three arguments are passed to the upcall in userspace.
    pub fn schedule(&mut self, r0: usize, r1: usize, r2: usize) -> bool {
        self.0.as_mut().map_or(false, |cb| cb.schedule(r0, r1, r2))
    }
}

/// Read-writable memory region of a process, shared with the kernel. This interface will change
/// before Tock 2.0 is released: https://gist.github.com/aec17cf82fc4ce05c9cf32edda4ce088
#[derive(Default)]
pub struct ReadWriteAppSlice(Option<AppSlice<Shared, u8>>);

impl ReadWriteAppSlice {
    /// Length of the memory region.
    ///
    /// If the process is no longer alive and the memory has been
    /// reclaimed, this method must return 0.
    ///
    /// # Default AppSlice
    ///
    /// A default instance of an AppSlice must return 0.
    pub fn len(&self) -> usize {
        self.0.as_ref().map_or(0, |b| b.len())
    }

    /// Pointer to the first byte of the userspace memory region.
    ///
    /// If the length of the initially shared memory region
    /// (irrespective of the return value of [`len`](Read::len)) is 0,
    /// this function returns a pointer to address `0x0`. This is
    /// because processes may allow buffers with length 0 to share no
    /// no memory with the kernel. Because these buffers have zero
    /// length, they may have any pointer value. However, these
    /// _dummy addresses_ should not be leaked, so this method returns
    /// 0 for zero-length slices.
    ///
    /// # Default AppSlice
    ///
    /// A default instance of an AppSlice must return a pointer to
    /// address `0x0`.
    pub fn ptr(&self) -> *const u8 {
        self.0.as_ref().map_or(0 as *const u8, |b| b.ptr())
    }

    /// Applies a function to the (read only) slice reference pointed
    /// to by the AppSlice.
    ///
    /// If the process is no longer alive and the memory has been
    /// reclaimed, this method must return the default value.
    ///
    /// # Default AppSlice
    ///
    /// A default instance of an AppSlice must return the passed
    /// default value without executing the closure.
    pub fn map_or<F, R>(&self, default: R, fun: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        self.0.as_ref().map_or(default, |b| fun(b.as_ref()))
    }

    /// Applies a function to the mutable slice reference pointed to
    /// by the AppSlice.
    ///
    /// If the process is no longer alive and the memory has been
    /// reclaimed, this method must return the default value.
    ///
    /// # Default AppSlice
    ///
    /// A default instance of an AppSlice must return the passed
    /// default value without executing the closure.
    pub fn mut_map_or<F, R>(&mut self, default: R, fun: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        self.0.as_mut().map_or(default, |b| fun(b.as_mut()))
    }
}
