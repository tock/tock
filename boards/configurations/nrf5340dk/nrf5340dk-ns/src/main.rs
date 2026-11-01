// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Tock kernel for the Nordic Semiconductor nRF5340 development kit, running
//! in non-secure state. The secure stage in `secure/` starts it.

#![no_std]
#![no_main]
#![deny(missing_docs)]

use kernel::ErrorCode;
use kernel::{capabilities, create_capability};
use nrf5340::cryptocell::gateway::{self, AesRequest};
use nrf5340::cryptocell::{AesMode, SecureCryptoCell, Sha256State};

/// The secure stage's CryptoCell gateway, whose entry points are the last 32
/// bytes of its flash.
struct SecureStage;

impl SecureStage {
    /// The entry point for `operation`, as a Thumb function address.
    const fn entry(operation: usize) -> usize {
        (0x3FE0 + 8 * operation) | 1
    }

    fn result(status: u32) -> Result<(), ErrorCode> {
        match status {
            gateway::OK => Ok(()),
            _ => Err(ErrorCode::INVAL),
        }
    }
}

// ### Safety
//
// Each transmute below makes a function pointer of the secure stage's entry
// point for that operation, which `nrf5340::cryptocell::gateway` defines with
// these arguments. The secure stage checks the pointers it is given.
impl SecureCryptoCell for SecureStage {
    fn aes128(
        &self,
        mode: AesMode,
        encrypt: bool,
        key: &[u8; 16],
        iv: &[u8; 16],
        data: &mut [u8],
    ) -> Result<[u8; 16], ErrorCode> {
        let mut request = AesRequest {
            mode: mode as u32,
            encrypt: encrypt.into(),
            key: *key,
            iv: *iv,
            data: data.as_mut_ptr(),
            len: data.len(),
        };
        let call: extern "C" fn(*mut AesRequest) -> u32 =
            unsafe { core::mem::transmute(Self::entry(gateway::AES128)) };
        Self::result(call(&mut request)).map(|()| request.iv)
    }

    fn sha256_blocks(&self, state: &mut Sha256State, blocks: &[u8]) -> Result<(), ErrorCode> {
        let call: extern "C" fn(*mut Sha256State, *const u8, usize) -> u32 =
            unsafe { core::mem::transmute(Self::entry(gateway::SHA256_BLOCKS)) };
        Self::result(call(state, blocks.as_ptr(), blocks.len()))
    }

    fn sha256_finish(&self, state: &Sha256State, tail: &[u8]) -> Result<[u8; 32], ErrorCode> {
        let mut digest = [0; 32];
        let call: extern "C" fn(*const Sha256State, *const u8, usize, *mut [u8; 32]) -> u32 =
            unsafe { core::mem::transmute(Self::entry(gateway::SHA256_FINISH)) };
        Self::result(call(state, tail.as_ptr(), tail.len(), &mut digest)).map(|()| digest)
    }

    fn trng_read(&self) -> Result<[u32; 6], ErrorCode> {
        let mut bits = [0; 6];
        let call: extern "C" fn(*mut [u32; 6]) -> u32 =
            unsafe { core::mem::transmute(Self::entry(gateway::TRNG_READ)) };
        Self::result(call(&mut bits)).map(|()| bits)
    }
}

/// Main function called after RAM initialized.
///
/// # Safety
///
/// ## `no_mangle`
///
/// We use `main` as a symbol in the arch crate's initialization routine to jump
/// here. We guarantee no other symbol has the same name.
///
/// Note: this requires a global/uniqueness guarantee.
/// See <https://github.com/tock/tock/issues/5250>.
#[unsafe(no_mangle)]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, platform, chip) = nrf5340dk_lib::start(Some(&SecureStage));
    board_kernel.kernel_loop(
        &platform,
        chip,
        None::<&kernel::ipc::IPC<0>>,
        &main_loop_capability,
    );
}
