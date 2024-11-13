// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Helper functions and types shared between multiple `arch` crates.
//!
//! This function contains functions and types that do not have to be in the
//! core kernel and are architecture-specific, but are shared by two or more
//! `arch` crates. While these could also live in a dedicated crate, we use the
//! `kernel` crate as all `arch` crates already depend on it.

use crate::syscall::SyscallReturn;
use crate::utilities::capability_ptr::{CapabilityPtr, CapabilityPtrPermissions};
use crate::ErrorCode;

/// Helper function to split a [`u64`] into a higher and lower [`u32`].
///
/// Used in encoding 64-bit wide system call return values on 32-bit
/// platforms.
#[inline]
fn u64_to_be_u32s(src: u64) -> (u32, u32) {
    let src_bytes = src.to_be_bytes();
    let src_msb = u32::from_be_bytes([src_bytes[0], src_bytes[1], src_bytes[2], src_bytes[3]]);
    let src_lsb = u32::from_be_bytes([src_bytes[4], src_bytes[5], src_bytes[6], src_bytes[7]]);

    (src_msb, src_lsb)
}

/// Enumeration of the system call return type variant identifiers described in
/// TRD104.
///
/// Each variant is associated with the respective variant identifier that would
/// be passed along with the return value to userspace.
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum TRD104SyscallReturnVariant {
    Failure = 0,
    FailureU32 = 1,
    FailureU32U32 = 2,
    FailureU64 = 3,
    Success = 128,
    SuccessU32 = 129,
    SuccessU32U32 = 130,
    SuccessU64 = 131,
    SuccessU32U32U32 = 132,
    SuccessU32U64 = 133,
}

/// System call return variants defined as defined in TRD104. These are a strict
/// subset of the variants defined in the core kernel's [`SyscallReturn`]
/// enum. For documentation on the individual variants, refer to this type
/// instead.
#[derive(Copy, Clone, Debug)]
pub enum TRD104SyscallReturn {
    Failure(ErrorCode),
    FailureU32(ErrorCode, u32),
    FailureU32U32(ErrorCode, u32, u32),
    FailureU64(ErrorCode, u64),
    Success,
    SuccessU32(u32),
    SuccessU32U32(u32, u32),
    SuccessU32U32U32(u32, u32, u32),
    SuccessU64(u64),
    SuccessU32U64(u32, u64),
    AllowReadWriteSuccess(*mut u8, usize),
    AllowReadWriteFailure(ErrorCode, *mut u8, usize),
    UserspaceReadableAllowSuccess(*mut u8, usize),
    UserspaceReadableAllowFailure(ErrorCode, *mut u8, usize),
    AllowReadOnlySuccess(*const u8, usize),
    AllowReadOnlyFailure(ErrorCode, *const u8, usize),
    SubscribeSuccess(*const (), usize),
    SubscribeFailure(ErrorCode, *const (), usize),
    YieldWaitFor(usize, usize, usize),
}

impl TRD104SyscallReturn {
    /// Map from the kernel's [`SyscallReturn`] enum to the subset of return
    /// values specified in TRD104. This ensures backwards compatibility with
    /// architectures implementing the ABI as specified in TRD104.
    pub fn from_syscall_return(syscall_return: SyscallReturn) -> Self {
        match syscall_return {
            // Identical variants:
            SyscallReturn::Failure(a) => TRD104SyscallReturn::Failure(a),
            SyscallReturn::FailureU32(a, b) => TRD104SyscallReturn::FailureU32(a, b),
            SyscallReturn::FailureU32U32(a, b, c) => TRD104SyscallReturn::FailureU32U32(a, b, c),
            SyscallReturn::FailureU64(a, b) => TRD104SyscallReturn::FailureU64(a, b),
            SyscallReturn::Success => TRD104SyscallReturn::Success,
            SyscallReturn::SuccessU32(a) => TRD104SyscallReturn::SuccessU32(a),
            SyscallReturn::SuccessU32U32(a, b) => TRD104SyscallReturn::SuccessU32U32(a, b),
            SyscallReturn::SuccessU32U32U32(a, b, c) => {
                TRD104SyscallReturn::SuccessU32U32U32(a, b, c)
            }
            SyscallReturn::SuccessU64(a) => TRD104SyscallReturn::SuccessU64(a),
            SyscallReturn::SuccessU32U64(a, b) => TRD104SyscallReturn::SuccessU32U64(a, b),
            SyscallReturn::AllowReadWriteSuccess(a, b) => {
                TRD104SyscallReturn::AllowReadWriteSuccess(a, b)
            }
            SyscallReturn::AllowReadWriteFailure(a, b, c) => {
                TRD104SyscallReturn::AllowReadWriteFailure(a, b, c)
            }
            SyscallReturn::UserspaceReadableAllowSuccess(a, b) => {
                TRD104SyscallReturn::UserspaceReadableAllowSuccess(a, b)
            }
            SyscallReturn::UserspaceReadableAllowFailure(a, b, c) => {
                TRD104SyscallReturn::UserspaceReadableAllowFailure(a, b, c)
            }
            SyscallReturn::AllowReadOnlySuccess(a, b) => {
                TRD104SyscallReturn::AllowReadOnlySuccess(a, b)
            }
            SyscallReturn::AllowReadOnlyFailure(a, b, c) => {
                TRD104SyscallReturn::AllowReadOnlyFailure(a, b, c)
            }
            SyscallReturn::SubscribeSuccess(a, b) => TRD104SyscallReturn::SubscribeSuccess(a, b),
            SyscallReturn::SubscribeFailure(a, b, c) => {
                TRD104SyscallReturn::SubscribeFailure(a, b, c)
            }
            SyscallReturn::YieldWaitFor(a, b, c) => TRD104SyscallReturn::YieldWaitFor(a, b, c),

            // Compatibility mapping:
            SyscallReturn::SuccessUsize(a) => TRD104SyscallReturn::SuccessU32(a as u32),
            SyscallReturn::SuccessPtr(a) => {
                TRD104SyscallReturn::SuccessU32(a.as_ptr::<()>() as u32)
            }
        }
    }
}

/// Encode the system call return value into 4 registers, following the encoding
/// specified in TRD104. Architectures which do not follow TRD104 are free to
/// define their own encoding.
pub fn encode_syscall_return_32bit_trd104(
    syscall_return: &TRD104SyscallReturn,
    a0: &mut u32,
    a1: &mut u32,
    a2: &mut u32,
    a3: &mut u32,
) {
    match *syscall_return {
        TRD104SyscallReturn::Failure(e) => {
            *a0 = TRD104SyscallReturnVariant::Failure as u32;
            *a1 = usize::from(e) as u32;
        }
        TRD104SyscallReturn::FailureU32(e, data0) => {
            *a0 = TRD104SyscallReturnVariant::FailureU32 as u32;
            *a1 = usize::from(e) as u32;
            *a2 = data0;
        }
        TRD104SyscallReturn::FailureU32U32(e, data0, data1) => {
            *a0 = TRD104SyscallReturnVariant::FailureU32U32 as u32;
            *a1 = usize::from(e) as u32;
            *a2 = data0;
            *a3 = data1;
        }
        TRD104SyscallReturn::FailureU64(e, data0) => {
            let (data0_msb, data0_lsb) = u64_to_be_u32s(data0);
            *a0 = TRD104SyscallReturnVariant::FailureU64 as u32;
            *a1 = usize::from(e) as u32;
            *a2 = data0_lsb;
            *a3 = data0_msb;
        }
        TRD104SyscallReturn::Success => {
            *a0 = TRD104SyscallReturnVariant::Success as u32;
        }
        TRD104SyscallReturn::SuccessU32(data0) => {
            *a0 = TRD104SyscallReturnVariant::SuccessU32 as u32;
            *a1 = data0;
        }
        TRD104SyscallReturn::SuccessU32U32(data0, data1) => {
            *a0 = TRD104SyscallReturnVariant::SuccessU32U32 as u32;
            *a1 = data0;
            *a2 = data1;
        }
        TRD104SyscallReturn::SuccessU32U32U32(data0, data1, data2) => {
            *a0 = TRD104SyscallReturnVariant::SuccessU32U32U32 as u32;
            *a1 = data0;
            *a2 = data1;
            *a3 = data2;
        }
        TRD104SyscallReturn::SuccessU64(data0) => {
            let (data0_msb, data0_lsb) = u64_to_be_u32s(data0);

            *a0 = TRD104SyscallReturnVariant::SuccessU64 as u32;
            *a1 = data0_lsb;
            *a2 = data0_msb;
        }
        TRD104SyscallReturn::SuccessU32U64(data0, data1) => {
            let (data1_msb, data1_lsb) = u64_to_be_u32s(data1);

            *a0 = TRD104SyscallReturnVariant::SuccessU32U64 as u32;
            *a1 = data0;
            *a2 = data1_lsb;
            *a3 = data1_msb;
        }
        TRD104SyscallReturn::AllowReadWriteSuccess(ptr, len) => {
            *a0 = TRD104SyscallReturnVariant::SuccessU32U32 as u32;
            *a1 = ptr as u32;
            *a2 = len as u32;
        }
        TRD104SyscallReturn::UserspaceReadableAllowSuccess(ptr, len) => {
            *a0 = TRD104SyscallReturnVariant::SuccessU32U32 as u32;
            *a1 = ptr as u32;
            *a2 = len as u32;
        }
        TRD104SyscallReturn::AllowReadWriteFailure(err, ptr, len) => {
            *a0 = TRD104SyscallReturnVariant::FailureU32U32 as u32;
            *a1 = usize::from(err) as u32;
            *a2 = ptr as u32;
            *a3 = len as u32;
        }
        TRD104SyscallReturn::UserspaceReadableAllowFailure(err, ptr, len) => {
            *a0 = TRD104SyscallReturnVariant::FailureU32U32 as u32;
            *a1 = usize::from(err) as u32;
            *a2 = ptr as u32;
            *a3 = len as u32;
        }
        TRD104SyscallReturn::AllowReadOnlySuccess(ptr, len) => {
            *a0 = TRD104SyscallReturnVariant::SuccessU32U32 as u32;
            *a1 = ptr as u32;
            *a2 = len as u32;
        }
        TRD104SyscallReturn::AllowReadOnlyFailure(err, ptr, len) => {
            *a0 = TRD104SyscallReturnVariant::FailureU32U32 as u32;
            *a1 = usize::from(err) as u32;
            *a2 = ptr as u32;
            *a3 = len as u32;
        }
        TRD104SyscallReturn::SubscribeSuccess(ptr, data) => {
            *a0 = TRD104SyscallReturnVariant::SuccessU32U32 as u32;
            *a1 = ptr as u32;
            *a2 = data as u32;
        }
        TRD104SyscallReturn::SubscribeFailure(err, ptr, data) => {
            *a0 = TRD104SyscallReturnVariant::FailureU32U32 as u32;
            *a1 = usize::from(err) as u32;
            *a2 = ptr as u32;
            *a3 = data as u32;
        }
        TRD104SyscallReturn::YieldWaitFor(data0, data1, data2) => {
            *a0 = data0 as u32;
            *a1 = data1 as u32;
            *a2 = data2 as u32;
        }
    }
}

/// Enumeration of all system call return type variant identifiers, as a
/// superset of those defined in TRD 104. This is not yet standardized.
///
/// Each variant is associated with the respective variant identifier that would
/// be passed along with the return value to userspace.
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum SyscallReturnVariant {
    Failure = 0,
    FailureU32 = 1,
    FailureU32U32 = 2,
    FailureU64 = 3,
    FailurePtrUsize = 4,
    FailurePtrPtr = 5,
    Success = 128,
    SuccessU32 = 129,
    SuccessU32U32 = 130,
    SuccessU64 = 131,
    SuccessU32U32U32 = 132,
    SuccessU32U64 = 133,
    SuccessUsize = 134,
    SuccessPtr = 135,
    SuccessPtrUsize = 136,
    SuccessPtrPtr = 137,
}

/// Encode the system call return value into 4 registers, with an encoding that
/// is a superset of the return variants specified in TRD104. This extension
/// behaves identical to TRD104 for the [`SyscallReturn`] variants that are
/// defined in TRD104. This does not implement any standardized encoding.
///
/// On 64-bit platforms, both 64-bit and usize values are passed as a single register,
/// Does not handle usize other than 4 and 8 bytes.
/// Pointers from allow'd buffers have permissions and length reattached matching
/// those that were checked at the syscall boundary.
pub fn encode_syscall_return_usize(
    syscall_return: &SyscallReturn,
    a0: &mut CapabilityPtr,
    a1: &mut CapabilityPtr,
    a2: &mut CapabilityPtr,
    a3: &mut CapabilityPtr,
) {
    // On 32-bit CHERI, given that capabilities cannot be used as 64-bit integers, 64-bit
    // integers will still be returned as two 32-bit values in different registers.
    fn write_64(a: &mut CapabilityPtr, b: &mut CapabilityPtr, val: u64) {
        let is_64_bit = core::mem::size_of::<usize>() == 8;
        if !is_64_bit {
            let (msb, lsb) = u64_to_be_u32s(val);
            *a = (lsb as usize).into();
            *b = (msb as usize).into();
        } else {
            *a = (val as usize).into();
        }
    }

    match *syscall_return {
        SyscallReturn::Failure(e) => {
            *a0 = (SyscallReturnVariant::Failure as usize).into();
            *a1 = (usize::from(e)).into();
        }
        SyscallReturn::FailureU32(e, data0) => {
            *a0 = (SyscallReturnVariant::FailureU32 as usize).into();
            *a1 = usize::from(e).into();
            *a2 = (data0 as usize).into();
        }
        SyscallReturn::FailureU32U32(e, data0, data1) => {
            *a0 = (SyscallReturnVariant::FailureU32U32 as usize).into();
            *a1 = (usize::from(e)).into();
            *a2 = (data0 as usize).into();
            *a3 = (data1 as usize).into();
        }
        SyscallReturn::FailureU64(e, data0) => {
            *a0 = (SyscallReturnVariant::FailureU64 as usize).into();
            *a1 = (usize::from(e)).into();
            write_64(a2, a3, data0)
        }
        SyscallReturn::Success => {
            *a0 = (SyscallReturnVariant::Success as usize).into();
        }
        SyscallReturn::SuccessU32(data0) => {
            *a0 = (SyscallReturnVariant::SuccessU32 as usize).into();
            *a1 = (data0 as usize).into();
        }
        SyscallReturn::SuccessU32U32(data0, data1) => {
            *a0 = (SyscallReturnVariant::SuccessU32U32 as usize).into();
            *a1 = (data0 as usize).into();
            *a2 = (data1 as usize).into();
        }
        SyscallReturn::SuccessU32U32U32(data0, data1, data2) => {
            *a0 = (SyscallReturnVariant::SuccessU32U32U32 as usize).into();
            *a1 = (data0 as usize).into();
            *a2 = (data1 as usize).into();
            *a3 = (data2 as usize).into();
        }
        SyscallReturn::SuccessU64(data0) => {
            *a0 = (SyscallReturnVariant::SuccessU64 as usize).into();
            write_64(a1, a2, data0);
        }
        SyscallReturn::SuccessU32U64(data0, data1) => {
            *a0 = (SyscallReturnVariant::SuccessU32U64 as usize).into();
            *a1 = (data0 as usize).into();
            write_64(a2, a3, data1);
        }
        SyscallReturn::AllowReadWriteSuccess(ptr, len) => {
            *a0 = (SyscallReturnVariant::SuccessPtrUsize as usize).into();
            *a1 = unsafe {
                CapabilityPtr::new_with_metadata(
                    ptr as *const (),
                    ptr as usize,
                    len,
                    CapabilityPtrPermissions::ReadWrite,
                )
            };
            *a2 = len.into();
        }
        SyscallReturn::UserspaceReadableAllowSuccess(ptr, len) => {
            *a0 = (SyscallReturnVariant::SuccessPtrUsize as usize).into();
            *a1 = unsafe {
                CapabilityPtr::new_with_metadata(
                    ptr as *const (),
                    ptr as usize,
                    len,
                    CapabilityPtrPermissions::Read,
                )
            };
            *a2 = len.into();
        }
        SyscallReturn::AllowReadWriteFailure(err, ptr, len) => {
            *a0 = (SyscallReturnVariant::FailurePtrUsize as usize).into();
            *a1 = (usize::from(err)).into();
            *a2 = unsafe {
                CapabilityPtr::new_with_metadata(
                    ptr as *const (),
                    ptr as usize,
                    len,
                    CapabilityPtrPermissions::ReadWrite,
                )
            };
            *a3 = len.into();
        }
        SyscallReturn::UserspaceReadableAllowFailure(err, ptr, len) => {
            *a0 = (SyscallReturnVariant::FailurePtrUsize as usize).into();
            *a1 = (usize::from(err)).into();
            *a2 = unsafe {
                CapabilityPtr::new_with_metadata(
                    ptr as *const (),
                    ptr as usize,
                    len,
                    CapabilityPtrPermissions::Read,
                )
            };
            *a3 = len.into();
        }
        SyscallReturn::AllowReadOnlySuccess(ptr, len) => {
            *a0 = (SyscallReturnVariant::SuccessPtrUsize as usize).into();
            *a1 = unsafe {
                CapabilityPtr::new_with_metadata(
                    ptr as *const (),
                    ptr as usize,
                    len,
                    CapabilityPtrPermissions::Read,
                )
            };
            *a2 = len.into();
        }
        SyscallReturn::AllowReadOnlyFailure(err, ptr, len) => {
            *a0 = (SyscallReturnVariant::FailurePtrUsize as usize).into();
            *a1 = (usize::from(err)).into();
            *a2 = unsafe {
                CapabilityPtr::new_with_metadata(
                    ptr as *const (),
                    ptr as usize,
                    len,
                    CapabilityPtrPermissions::Read,
                )
            };
            *a3 = len.into();
        }
        SyscallReturn::SubscribeSuccess(ptr, data) => {
            *a0 = (SyscallReturnVariant::SuccessPtrPtr as usize).into();
            *a1 = (ptr as usize).into();
            *a2 = data.into();
        }
        SyscallReturn::SubscribeFailure(err, ptr, data) => {
            *a0 = (SyscallReturnVariant::FailurePtrPtr as usize).into();
            *a1 = (usize::from(err)).into();
            *a2 = (ptr as usize).into();
            *a3 = data.into();
        }
        SyscallReturn::SuccessPtr(ptr) => {
            *a0 = (SyscallReturnVariant::SuccessPtr as usize).into();
            *a1 = ptr;
        }
        SyscallReturn::YieldWaitFor(data0, data1, data2) => {
            *a0 = data0.into();
            *a1 = data1.into();
            *a2 = data2.into();
        }
        SyscallReturn::SuccessUsize(data) => {
            *a0 = (SyscallReturnVariant::SuccessUsize as usize).into();
            *a1 = data.into();
        }
    }
}
