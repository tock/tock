// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! System call interface for userspace processes implemented by capsules.
//!
//! Drivers implement these interfaces to expose operations to processes.

use crate::errorcode::ErrorCode;
use crate::process::ProcessId;
use crate::processbuffer::UserspaceReadableProcessBuffer;
use crate::syscall::SyscallReturn;
use crate::{debug, process};

/// Possible return values of a `command` driver method, as specified in TRD104.
///
/// This is just a wrapper around [`SyscallReturn`] since a `command` driver
/// method may only return primitive integer types as payload.
///
/// It is important for this wrapper to only be constructable over variants of
/// [`SyscallReturn`] that are deemed safe for a capsule to construct and return
/// to an application (e.g. not
/// [`SubscribeSuccess`](crate::syscall::SyscallReturn::SubscribeSuccess)). This
/// means that the inner value **must** remain private.
pub struct CommandReturn(SyscallReturn);

impl CommandReturn {
    pub(crate) fn into_inner(self) -> SyscallReturn {
        self.0
    }

    /// Command error
    pub fn failure(rc: ErrorCode) -> Self {
        CommandReturn(SyscallReturn::Failure(rc))
    }

    /// Command error with an additional 32-bit data field
    pub fn failure_u32(rc: ErrorCode, data0: u32) -> Self {
        CommandReturn(SyscallReturn::FailureU32(rc, data0))
    }

    /// Command error with two additional 32-bit data fields
    pub fn failure_u32_u32(rc: ErrorCode, data0: u32, data1: u32) -> Self {
        CommandReturn(SyscallReturn::FailureU32U32(rc, data0, data1))
    }

    /// Command error with an additional 64-bit data field
    pub fn failure_u64(rc: ErrorCode, data0: u64) -> Self {
        CommandReturn(SyscallReturn::FailureU64(rc, data0))
    }

    /// Successful command
    pub fn success() -> Self {
        CommandReturn(SyscallReturn::Success)
    }

    /// Successful command with an additional 32-bit data field
    pub fn success_u32(data0: u32) -> Self {
        CommandReturn(SyscallReturn::SuccessU32(data0))
    }

    /// Successful command with two additional 32-bit data fields
    pub fn success_u32_u32(data0: u32, data1: u32) -> Self {
        debug!("receive data0:{}, data1:{}", data0, data1);
        CommandReturn(SyscallReturn::SuccessU32U32(data0, data1))
    }

    /// Successful command with three additional 32-bit data fields
    pub fn success_u32_u32_u32(data0: u32, data1: u32, data2: u32) -> Self {
        CommandReturn(SyscallReturn::SuccessU32U32U32(data0, data1, data2))
    }

    /// Successful command with an additional 64-bit data field
    pub fn success_u64(data0: u64) -> Self {
        CommandReturn(SyscallReturn::SuccessU64(data0))
    }

    /// Successful command with an additional 64-bit and 32-bit data field
    pub fn success_u32_u64(data0: u32, data1: u64) -> Self {
        CommandReturn(SyscallReturn::SuccessU32U64(data0, data1))
    }
}

impl From<Result<(), ErrorCode>> for CommandReturn {
    fn from(rc: Result<(), ErrorCode>) -> Self {
        match rc {
            Ok(()) => CommandReturn::success(),
            Err(e) => CommandReturn::failure(e),
        }
    }
}

impl From<process::Error> for CommandReturn {
    fn from(perr: process::Error) -> Self {
        CommandReturn::failure(perr.into())
    }
}

/// Trait for capsules implementing peripheral driver system calls specified in
/// TRD104.
///
/// The kernel translates the values passed from userspace into Rust
/// types and includes which process is making the call. All of these
/// system calls perform very little synchronous work; long running
/// computations or I/O should be split-phase, with an upcall
/// indicating their completion.
///
/// The exact instances of each of these methods (which identifiers are valid
/// and what they represents) are specific to the peripheral system call driver.
///
/// Note about `subscribe`, `read-only allow`, and `read-write allow` syscalls:
/// those are handled entirely by the core kernel, and there is no corresponding
/// function for capsules to implement.
#[allow(unused_variables)]
pub trait SyscallDriver {
    /// System call for a process to perform a short synchronous operation or
    /// start a long-running split-phase operation (whose completion is signaled
    /// with an upcall). Command 0 is a reserved command to detect if a
    /// peripheral system call driver is installed and must always return a
    /// [`CommandReturn::success`].
    fn command(
        &self,
        command_num: usize,
        r2: usize,
        r3: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        debug!("--------Syscall invoked with command_num: {}", command_num);
        CommandReturn::failure(ErrorCode::NOSUPPORT)
    }

    /// System call for a process to pass a buffer (a
    /// [`UserspaceReadableProcessBuffer`]) to the kernel that the kernel can
    /// either read or write. The kernel calls this method only after it checks
    /// that the entire buffer is within memory the process can both read and
    /// write.
    ///
    /// This is different to `allow_readwrite` in that the app is allowed to
    /// read the buffer once it has been passed to the kernel. For more details
    /// on how this can be done safely see the userspace readable allow syscalls
    /// TRDXXX.
    fn allow_userspace_readable(
        &self,
        app: ProcessId,
        which: usize,
        slice: UserspaceReadableProcessBuffer,
    ) -> Result<UserspaceReadableProcessBuffer, (UserspaceReadableProcessBuffer, ErrorCode)> {
        Err((slice, ErrorCode::NOSUPPORT))
    }

    /// Request to allocate a capsule's grant for a specific process.
    ///
    /// The core kernel uses this function to instruct a capsule to ensure its
    /// grant (if it has one) is allocated for a specific process. The core
    /// kernel needs the capsule to initiate the allocation because only the
    /// capsule knows the type `T` (and therefore the size of `T`) that will be
    /// stored in the grant.
    ///
    /// The typical implementation will look like:
    /// ```rust, ignore
    /// fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
    ///    self.apps.enter(processid, |_, _| {})
    /// }
    /// ```
    ///
    /// No default implementation is provided to help prevent accidentally
    /// forgetting to implement this function.
    ///
    /// If a capsule fails to successfully implement this function, subscribe
    /// calls from userspace for the [`SyscallDriver`] may fail.
    //
    // The inclusion of this function originates from the method for ensuring
    // correct upcall swapping semantics in the kernel starting with Tock 2.0.
    // To ensure upcalls are always swapped correctly all upcall handling is
    // done in the core kernel. Capsules only have access to a handle which
    // permits them to schedule upcalls, but capsules no longer manage upcalls.
    //
    // The core kernel stores upcalls in the process's grant region along with
    // the capsule's grant object. A simultaneous Tock 2.0 change requires that
    // capsules wishing to use upcalls must also use grants. Storing upcalls in
    // the grant requires that the grant be allocated for that capsule in that
    // process. This presents a challenge as grants are dynamically allocated
    // only when actually used by a process. If a subscribe syscall happens
    // first, before the capsule has allocated the grant, the kernel has no way
    // to store the upcall. The kernel cannot allocate the grant itself because
    // it does not know the type T the capsule will use for the grant (or more
    // specifically the kernel does not know the size of T to use for the memory
    // allocation).
    //
    // There are a few ideas on how to handle this case where the kernel must
    // store an upcall before the capsule has allocated the grant.
    //
    // 1. The kernel could allocate space for the grant type T, but not actually
    //    initialize it, based only on the size of T. However, this would
    //    require the kernel to keep track of the size of T for each grant, and
    //    there is no convenient place to store that information.
    //
    // 2. The kernel could store upcalls and grant types separately in the grant
    //    region.
    //
    //    a. One approach is to store upcalls completely dynamically. That is,
    //       whenever a new subscribe_num is used for a particular driver the
    //       core kernel allocates new memory from the grant region to store it.
    //       This would work, but would have high memory and runtime overhead to
    //       manage all of the dynamic upcall stores.
    //    b. To reduce the tracking overhead, all upcalls for a particular
    //       driver could be stored together as one allocation. This would only
    //       cost one additional pointer per grant to point to the upcall array.
    //       However, the kernel does not know how many upcalls a particular
    //       driver needs, and there is no convenient place for it to store that
    //       information.
    //
    // 3. The kernel could allocate a fixed region for all upcalls across all
    //    drivers in the grant region. When each grant is created it could tell
    //    the kernel how many upcalls it will use and the kernel could easily
    //    keep track of the total. Then, when a process's memory is allocated
    //    the kernel would reserve room for that many upcalls. There are two
    //    issues, however. The kernel would not know how many upcalls each
    //    driver individually requires, so it would not be able to index into
    //    this array properly to store each upcall. Second, the upcall array
    //    memory would be statically allocated, and would be wasted for drivers
    //    the process never uses.
    //
    //    A version of this approach would assume a maximum limit of a certain
    //    number of upcalls per driver. This would address the indexing
    //    challenge, but would still have the memory overhead problem. It would
    //    also limit capsule flexibility by capping the number of upcalls any
    //    capsule could ever use.
    //
    // 4. The kernel could have some mechanism to ask a capsule to allocate its
    //    grant, and since the capsule knows the size of T and the number of
    //    upcalls it uses the grant type and upcall storage could be allocated
    //    together.
    //
    // Based on the available options, the Tock developers decided go with
    // option 4 and add the `allocate_grant` method to the `SyscallDriver`
    // trait. This mechanism may find more uses in the future if the kernel
    // needs to store additional state on a per-driver basis and therefore needs
    // a mechanism to force a grant allocation.
    //
    // This same mechanism was later extended to handle allow calls as well.
    // Capsules that do not need upcalls but do use process buffers must also
    // implement this function.
    fn allocate_grant(&self, process_id: ProcessId) -> Result<(), crate::process::Error>;
}
