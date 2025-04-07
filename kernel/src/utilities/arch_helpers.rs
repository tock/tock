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
use crate::utilities::machine_register::MachineRegister;

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

/// Enumeration of the system call return type variant identifiers described in
/// the (not yet existant) TRD105.
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum TRD105SyscallReturnVariant {
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
    SuccessAddr = 134,
    SucessPtr = 135,
    SucessPtrUsize = 136,
    SucessPtrPtr = 137,
}

/// A (kernel private) set of variants that matches SyscallReturn.
/// These must be mapped into ABI variants.
pub enum SyscallReturnVariant {
    Failure,
    FailureU32,
    FailureU32U32,
    FailureU64,
    FailurePtrUsize,
    FailurePtrPtr,
    Success,
    SuccessU32,
    SuccessU32U32,
    SuccessU64,
    SuccessU32U32U32,
    SuccessU32U64,
    SuccessAddr,
    SucessPtr,
    SucessPtrUsize,
    SucessPtrPtr,
}

impl From<TRD104SyscallReturnVariant> for usize {
    fn from(value: TRD104SyscallReturnVariant) -> Self {
        value as usize
    }
}

impl From<TRD105SyscallReturnVariant> for usize {
    fn from(value: TRD105SyscallReturnVariant) -> Self {
        value as usize
    }
}

impl From<SyscallReturnVariant> for TRD104SyscallReturnVariant {
    /// Map from the kernel's [`SyscallReturn`] enum to the subset of return
    /// values specified in TRD104. This ensures backwards compatibility with
    /// architectures implementing the ABI as specified in TRD104.
    fn from(value: SyscallReturnVariant) -> Self {
        match value {
            // Identical variants:
            SyscallReturnVariant::Failure => TRD104SyscallReturnVariant::Failure,
            SyscallReturnVariant::FailureU32 => TRD104SyscallReturnVariant::FailureU32,
            SyscallReturnVariant::FailureU32U32 => TRD104SyscallReturnVariant::FailureU32U32,
            SyscallReturnVariant::FailureU64 => TRD104SyscallReturnVariant::FailureU64,
            SyscallReturnVariant::Success => TRD104SyscallReturnVariant::Success,
            SyscallReturnVariant::SuccessU32 => TRD104SyscallReturnVariant::SuccessU32,
            SyscallReturnVariant::SuccessU32U32 => TRD104SyscallReturnVariant::SuccessU32U32,
            SyscallReturnVariant::SuccessU64 => TRD104SyscallReturnVariant::SuccessU64,
            SyscallReturnVariant::SuccessU32U32U32 => TRD104SyscallReturnVariant::SuccessU32U32U32,
            SyscallReturnVariant::SuccessU32U64 => TRD104SyscallReturnVariant::SuccessU32U64,
            // Compatibility mapping:
            SyscallReturnVariant::FailurePtrUsize => TRD104SyscallReturnVariant::FailureU32U32,
            SyscallReturnVariant::FailurePtrPtr => TRD104SyscallReturnVariant::FailureU32U32,
            SyscallReturnVariant::SuccessAddr => TRD104SyscallReturnVariant::SuccessU32,
            SyscallReturnVariant::SucessPtr => TRD104SyscallReturnVariant::SuccessU32,
            SyscallReturnVariant::SucessPtrUsize => TRD104SyscallReturnVariant::SuccessU32U32,
            SyscallReturnVariant::SucessPtrPtr => TRD104SyscallReturnVariant::SuccessU32U32,
        }
    }
}

impl From<SyscallReturnVariant> for TRD105SyscallReturnVariant {
    fn from(value: SyscallReturnVariant) -> Self {
        match value {
            // Same as TRD104
            SyscallReturnVariant::Failure => TRD105SyscallReturnVariant::Failure,
            SyscallReturnVariant::FailureU32 => TRD105SyscallReturnVariant::FailureU32,
            SyscallReturnVariant::FailureU32U32 => TRD105SyscallReturnVariant::FailureU32U32,
            SyscallReturnVariant::FailureU64 => TRD105SyscallReturnVariant::FailureU64,
            SyscallReturnVariant::Success => TRD105SyscallReturnVariant::Success,
            SyscallReturnVariant::SuccessU32 => TRD105SyscallReturnVariant::SuccessU32,
            SyscallReturnVariant::SuccessU32U32 => TRD105SyscallReturnVariant::SuccessU32U32,
            SyscallReturnVariant::SuccessU64 => TRD105SyscallReturnVariant::SuccessU64,
            SyscallReturnVariant::SuccessU32U32U32 => TRD105SyscallReturnVariant::SuccessU32U32U32,
            SyscallReturnVariant::SuccessU32U64 => TRD105SyscallReturnVariant::SuccessU32U64,
            // TRD105 only mappings
            SyscallReturnVariant::FailurePtrUsize => TRD105SyscallReturnVariant::FailurePtrUsize,
            SyscallReturnVariant::FailurePtrPtr => TRD105SyscallReturnVariant::FailurePtrPtr,
            SyscallReturnVariant::SuccessAddr => TRD105SyscallReturnVariant::SuccessAddr,
            SyscallReturnVariant::SucessPtr => TRD105SyscallReturnVariant::SucessPtr,
            SyscallReturnVariant::SucessPtrUsize => TRD105SyscallReturnVariant::SucessPtrUsize,
            SyscallReturnVariant::SucessPtrPtr => TRD105SyscallReturnVariant::SucessPtrPtr,
        }
    }
}

/// Encode the system call return value into 4 registers, following the encoding
/// specified in TRD104. Architectures which do not follow TRD104 are free to
/// define their own encoding.
pub fn encode_syscall_return_trd104(
    syscall_return: &SyscallReturn,
    a0: &mut u32,
    a1: &mut u32,
    a2: &mut u32,
    a3: &mut u32,
) {
    if core::mem::size_of::<MachineRegister>() == core::mem::size_of::<u32>() {
        // SAFETY: if the two unsized integers are the same size references to them
        // can be safely transmuted. The size checks that there is no extra metadata.
        // NOTE: This could be made safe via an extra copy to the stack, but this would be an
        // extra copy and would have subtly different semantics of replacing unused registers
        // with a default.
        unsafe {
            let a0 = &mut *(crate::polyfill::core::ptr::from_mut(a0) as *mut MachineRegister);
            let a1 = &mut *(crate::polyfill::core::ptr::from_mut(a1) as *mut MachineRegister);
            let a2 = &mut *(crate::polyfill::core::ptr::from_mut(a2) as *mut MachineRegister);
            let a3 = &mut *(crate::polyfill::core::ptr::from_mut(a3) as *mut MachineRegister);
            encode_syscall_return_with_variant::<TRD104SyscallReturnVariant>(
                syscall_return,
                a0,
                a1,
                a2,
                a3,
            );
        }
    } else {
        panic!("encode_syscall_return_trd104 used on a 64-bit platform or CHERI platform")
    }
}

/// Trait alias for syscall variants
pub trait Variant: From<SyscallReturnVariant> + Into<usize> {}

impl<T: From<SyscallReturnVariant> + Into<usize>> Variant for T {}

/// An extension of TRD104 that works for 32-bit and 64-bit platforms, and can remap variants.
///
/// On 32-bit platforms using.
/// Using TRD104SyscallReturnVariant on a 32-bit platform, this is exactly TRD104.
/// Using TRD105SyscallReturnVariant on any platform should be TRD1105.
/// Archtiectures not following either of these are free to provide their own mappings.
/// On 64-bit platforms, both 64-bit and usize values are passed as a single register,
/// shifting down register number if that means fewer registers are needed.
/// For usize, there is no change in number of registers between platforms.
/// For explicitly 64-bit arguments, this would require rewriting prototypes for userspace
/// functions between 32 and 64 bit platforms.
/// No usize other than 4 and 8 bytes is supported.
/// CHERI notes:
/// the high part of any capability register is zeroed if any non CapabilityPtr arguments are
/// passed.
/// SuccessPtr is as passed the full CapabilityPtr register.
/// Pointers from allow'd buffers have minimal bounds reattached that cover their length,
/// and the same permissions that were checked at the syscall boundary.
pub fn encode_syscall_return_with_variant<SyscallVariant: Variant>(
    syscall_return: &SyscallReturn,
    a0: &mut MachineRegister,
    a1: &mut MachineRegister,
    a2: &mut MachineRegister,
    a3: &mut MachineRegister,
) {
    // Writes a 64-bit value into either one (64-bit platforms) or two (32-bit platforms) registers
    fn write_64(a: &mut MachineRegister, b: &mut MachineRegister, val: u64) {
        let is_64_bit = core::mem::size_of::<usize>() == 8;
        if !is_64_bit {
            let (msb, lsb) = u64_to_be_u32s(val);
            *a = (lsb as usize).into();
            *b = (msb as usize).into();
        } else {
            *a = (val as usize).into();
        }
    }

    fn variant_to_reg<SyscallVariant: From<SyscallReturnVariant> + Into<usize>>(
        v: SyscallReturnVariant,
    ) -> MachineRegister {
        // First map from
        let lowered_to_abi: SyscallVariant = v.into();
        // Then cast to usize
        let as_usize: usize = lowered_to_abi.into();
        // and pack that into a register
        as_usize.into()
    }

    match *syscall_return {
        SyscallReturn::Failure(e) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::Failure);
            *a1 = (usize::from(e)).into();
        }
        SyscallReturn::FailureU32(e, data0) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::FailureU32);
            *a1 = usize::from(e).into();
            *a2 = (data0 as usize).into();
        }
        SyscallReturn::FailureU32U32(e, data0, data1) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::FailureU32U32);
            *a1 = (usize::from(e)).into();
            *a2 = (data0 as usize).into();
            *a3 = (data1 as usize).into();
        }
        SyscallReturn::FailureU64(e, data0) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::FailureU64);
            *a1 = (usize::from(e)).into();
            write_64(a2, a3, data0)
        }
        SyscallReturn::Success => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::Success);
        }
        SyscallReturn::SuccessU32(data0) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::SuccessU32);
            *a1 = (data0 as usize).into();
        }
        SyscallReturn::SuccessU32U32(data0, data1) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::SuccessU32U32);
            *a1 = (data0 as usize).into();
            *a2 = (data1 as usize).into();
        }
        SyscallReturn::SuccessU32U32U32(data0, data1, data2) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::SuccessU32U32U32);
            *a1 = (data0 as usize).into();
            *a2 = (data1 as usize).into();
            *a3 = (data2 as usize).into();
        }
        SyscallReturn::SuccessU64(data0) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::SuccessU64);
            write_64(a1, a2, data0);
        }
        SyscallReturn::SuccessU32U64(data0, data1) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::SuccessU32U64);
            *a1 = (data0 as usize).into();
            write_64(a2, a3, data1);
        }
        SyscallReturn::AllowReadWriteSuccess(ptr, len) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::SucessPtrUsize);
            // Safety: we previously checked these permissions and this length when this was
            // allowed to us
            *a1 = unsafe {
                CapabilityPtr::new_with_authority(
                    ptr as *const (),
                    ptr as usize,
                    len,
                    CapabilityPtrPermissions::ReadWrite,
                )
                .into()
            };
            *a2 = len.into();
        }
        SyscallReturn::UserspaceReadableAllowSuccess(ptr, len) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::SucessPtrUsize);
            // Safety: we previously checked these permissions and this length when this was
            // allowed to us
            *a1 = unsafe {
                CapabilityPtr::new_with_authority(
                    ptr as *const (),
                    ptr as usize,
                    len,
                    CapabilityPtrPermissions::Read,
                )
                .into()
            };
            *a2 = len.into();
        }
        SyscallReturn::AllowReadWriteFailure(err, ptr, len) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::FailurePtrUsize);
            *a1 = (usize::from(err)).into();
            // Safety: we previously checked these permissions and this length when this was
            // allowed to us
            *a2 = unsafe {
                CapabilityPtr::new_with_authority(
                    ptr as *const (),
                    ptr as usize,
                    len,
                    CapabilityPtrPermissions::ReadWrite,
                )
                .into()
            };
            *a3 = len.into();
        }
        SyscallReturn::UserspaceReadableAllowFailure(err, ptr, len) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::FailurePtrUsize);
            *a1 = (usize::from(err)).into();
            // Safety: we previously checked these permissions and this length when this was
            // allowed to us
            *a2 = unsafe {
                CapabilityPtr::new_with_authority(
                    ptr as *const (),
                    ptr as usize,
                    len,
                    CapabilityPtrPermissions::Read,
                )
                .into()
            };
            *a3 = len.into();
        }
        SyscallReturn::AllowReadOnlySuccess(ptr, len) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::SucessPtrUsize);
            // Safety: we previously checked these permissions and this length when this was
            // allowed to us
            *a1 = unsafe {
                CapabilityPtr::new_with_authority(
                    ptr as *const (),
                    ptr as usize,
                    len,
                    CapabilityPtrPermissions::Read,
                )
                .into()
            };
            *a2 = len.into();
        }
        SyscallReturn::AllowReadOnlyFailure(err, ptr, len) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::FailurePtrUsize);
            *a1 = (usize::from(err)).into();
            *a2 = unsafe {
                CapabilityPtr::new_with_authority(
                    ptr as *const (),
                    ptr as usize,
                    len,
                    CapabilityPtrPermissions::Read,
                )
                .into()
            };
            *a3 = len.into();
        }
        SyscallReturn::SubscribeSuccess(ptr, data) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::SucessPtrPtr);
            *a1 = (ptr as usize).into();
            *a2 = data.into();
        }
        SyscallReturn::SubscribeFailure(err, ptr, data) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::FailurePtrPtr);
            *a1 = (usize::from(err)).into();
            *a2 = (ptr as usize).into();
            *a3 = data.into();
        }
        SyscallReturn::SuccessPtr(metaptr) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::SucessPtr);
            *a1 = metaptr.into();
        }
        SyscallReturn::SuccessAddr(addr) => {
            *a0 = variant_to_reg::<SyscallVariant>(SyscallReturnVariant::SuccessAddr);
            *a1 = addr.into();
        }
        SyscallReturn::YieldWaitFor(data0, data1, data2) => {
            *a0 = data0.into();
            *a1 = data1.into();
            *a2 = data2.into();
        }
    }
}
