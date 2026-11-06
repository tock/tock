// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! The CryptoCell gateway: the non-secure callable entry points of
//! `nrf5340::cryptocell::gateway`, and the checked operations behind them.
//!
//! Each entry point is a secure gate (SG) and a branch, 8 bytes, in the window
//! the SPU makes non-secure callable. The operation it reaches checks that
//! every pointer it is given is non-secure memory of the kernel, copies the
//! arguments in, runs the operation on the CryptoCell, and copies the results
//! out. The return path then clears the argument registers and flags so no
//! secure value leaks, and returns to non-secure state.

use core::ops::Range;
use nrf5340::chip::{CC312_BASE, CRYPTOCELL_BASE};
use nrf5340::cryptocell::gateway::{AesRequest, INVAL, OK};
use nrf5340::cryptocell::{AesMode, Cc312, Sha256State};

/// The non-secure callable window, the last 32 bytes of this stage's flash.
pub const WINDOW: Range<usize> = 0x3FE0..0x4000;

/// RAM the kernel may pass to read and write. The first 8 KB holds this
/// stage's stack and stays secure.
pub const KERNEL_RAM: Range<usize> = 0x2000_2000..0x2008_0000;
/// Flash the kernel may pass to read: its own and its apps'.
const KERNEL_FLASH: Range<usize> = crate::KERNEL..crate::FLASH_END;

#[cfg(all(target_arch = "arm", target_os = "none"))]
core::arch::global_asm!(
    // `.inst.w 0xe97fe97f` is SG, which the assembler accepts only with the
    // security extension enabled.
    ".section .nsc, \"ax\"",
    ".thumb",
    ".inst.w 0xe97fe97f",
    "b.w 1f",
    ".inst.w 0xe97fe97f",
    "b.w 2f",
    ".inst.w 0xe97fe97f",
    "b.w 3f",
    ".inst.w 0xe97fe97f",
    "b.w 4f",
    ".section .text.gateway, \"ax\"",
    ".thumb",
    // R4 keeps the stack 8-byte aligned.
    "1: push {{r4, lr}}",
    "bl {aes128}",
    "b 5f",
    "2: push {{r4, lr}}",
    "bl {sha256_blocks}",
    "b 5f",
    "3: push {{r4, lr}}",
    "bl {sha256_finish}",
    "b 5f",
    "4: push {{r4, lr}}",
    "bl {trng_read}",
    // The operations restore R4 to R11. `.inst.n 0x4774` is BXNS LR.
    "5: pop {{r4, lr}}",
    "movs r1, #0",
    "mov r2, r1",
    "mov r3, r1",
    "mov r12, r1",
    "msr apsr_nzcvq, r1",
    ".inst.n 0x4774",
    aes128 = sym aes128,
    sha256_blocks = sym sha256_blocks,
    sha256_finish = sym sha256_finish,
    trng_read = sym trng_read,
);

fn cc312() -> Cc312 {
    Cc312::new(CRYPTOCELL_BASE, CC312_BASE)
}

/// Whether `len` bytes at `address` lie in `memory`.
fn within(memory: &Range<usize>, address: usize, len: usize) -> bool {
    address >= memory.start
        && address
            .checked_add(len)
            .is_some_and(|end| end <= memory.end)
}

/// Whether a `T` at `pointer` is aligned and in the kernel's RAM.
fn in_ram<T>(pointer: *const T) -> bool {
    pointer.is_aligned() && within(&KERNEL_RAM, pointer as usize, size_of::<T>())
}

/// `len` bytes at `address` in the kernel's RAM, to read and write.
fn ram_bytes<'a>(address: usize, len: usize) -> Option<&'a mut [u8]> {
    // ### Safety
    //
    // The bytes are in the kernel's RAM, non-secure memory that no secure code
    // uses, and the kernel waits in this call. The network core may also
    // write them, which changes only the result.
    within(&KERNEL_RAM, address, len)
        .then(|| unsafe { core::slice::from_raw_parts_mut(address as *mut u8, len) })
}

/// `len` bytes at `address` in the kernel's RAM or flash, to read.
fn kernel_bytes<'a>(address: usize, len: usize) -> Option<&'a [u8]> {
    if within(&KERNEL_FLASH, address, len) {
        // ### Safety
        //
        // The bytes are in the kernel's flash, which nothing writes, as the
        // NVMC is secure.
        Some(unsafe { core::slice::from_raw_parts(address as *const u8, len) })
    } else {
        ram_bytes(address, len).map(|bytes| &*bytes)
    }
}

extern "C" fn aes128(request: *mut AesRequest) -> u32 {
    if !in_ram(request) {
        return INVAL;
    }
    // ### Safety
    //
    // `in_ram` checked the request is the kernel's to pass. It is copied
    // before its fields are checked, so they cannot change after.
    let AesRequest {
        mode,
        encrypt,
        key,
        iv,
        data,
        len,
    } = unsafe { request.read() };
    let (Ok(mode), true, Some(data)) = (
        AesMode::try_from(mode),
        len.is_multiple_of(16),
        ram_bytes(data as usize, len),
    ) else {
        return INVAL;
    };
    let next = cc312().aes128(mode, encrypt != 0, &key, &iv, data);
    // ### Safety
    //
    // As above.
    unsafe { (*request).iv = next };
    OK
}

extern "C" fn sha256_blocks(state: *mut Sha256State, data: *const u8, len: usize) -> u32 {
    let (true, true, Some(data)) = (
        in_ram(state),
        len.is_multiple_of(64),
        kernel_bytes(data as usize, len),
    ) else {
        return INVAL;
    };
    // ### Safety
    //
    // `in_ram` checked the state is the kernel's to pass. A copy is hashed, as
    // `data` may overlap it.
    let mut copy = unsafe { state.read() };
    cc312().sha256_blocks(&mut copy, data);
    // ### Safety
    //
    // As above.
    unsafe { state.write(copy) };
    OK
}

extern "C" fn sha256_finish(
    state: *const Sha256State,
    tail: *const u8,
    len: usize,
    digest: *mut [u8; 32],
) -> u32 {
    let (true, true, true, Some(tail)) = (
        in_ram(state),
        in_ram(digest),
        len <= 64,
        kernel_bytes(tail as usize, len),
    ) else {
        return INVAL;
    };
    // ### Safety
    //
    // `in_ram` checked the state and digest are the kernel's to pass.
    unsafe { digest.write(cc312().sha256_finish(&state.read(), tail)) };
    OK
}

extern "C" fn trng_read(bits: *mut [u32; 6]) -> u32 {
    if !in_ram(bits) {
        return INVAL;
    }
    let cc = cc312();
    cc.trng_start();
    // Drop the first sample after a start, as the secure driver does.
    let mut warm = false;
    let sample = loop {
        if let Some(sample) = cc.trng_take() {
            if warm {
                break sample;
            }
            warm = true;
        }
    };
    cc.trng_stop();
    // ### Safety
    //
    // `in_ram` checked the bits are the kernel's to pass.
    unsafe { bits.write(sample) };
    OK
}
