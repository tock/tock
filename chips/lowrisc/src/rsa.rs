// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! RSA Implemented on top of the OTBN

use crate::virtual_otbn::VirtualMuxAccel;
use kernel::hil::public_key_crypto::rsa_math::{Client, ClientMut, RsaCryptoBase};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::mut_imut_buffer::MutImutBuffer;
use kernel::ErrorCode;

pub struct AppAddresses {
    pub imem_start: usize,
    pub imem_size: usize,
    pub dmem_start: usize,
    pub dmem_size: usize,
}

pub struct OtbnRsa<'a> {
    otbn: &'a VirtualMuxAccel<'a>,
    client: OptionalCell<&'a dyn Client<'a>>,
    client_mut: OptionalCell<&'a dyn ClientMut<'a>>,

    internal: TakeCell<'static, [u8]>,

    message: TakeCell<'static, [u8]>,
    modulus: OptionalCell<MutImutBuffer<'static, u8>>,
    exponent: OptionalCell<MutImutBuffer<'static, u8>>,

    rsa: AppAddresses,
}

impl<'a> OtbnRsa<'a> {
    pub fn new(
        otbn: &'a VirtualMuxAccel<'a>,
        rsa: AppAddresses,
        internal_buffer: &'static mut [u8],
    ) -> Self {
        OtbnRsa {
            otbn,
            client: OptionalCell::empty(),
            client_mut: OptionalCell::empty(),
            internal: TakeCell::new(internal_buffer),
            message: TakeCell::empty(),
            modulus: OptionalCell::empty(),
            exponent: OptionalCell::empty(),
            rsa,
        }
    }

    fn report_error(&self, error: ErrorCode, result: &'static mut [u8]) {
        match self.exponent.take().unwrap() {
            MutImutBuffer::Mutable(exponent) => {
                self.client_mut.map(|client| {
                    match self.modulus.take().unwrap() {
                        MutImutBuffer::Mutable(modulus) => {
                            client.mod_exponent_done(
                                Err(error),
                                self.message.take().unwrap(),
                                modulus,
                                exponent,
                                result,
                            );
                        }
                        MutImutBuffer::Immutable(_) => unreachable!(),
                    };
                });
            }
            MutImutBuffer::Immutable(exponent) => {
                match self.modulus.take().unwrap() {
                    MutImutBuffer::Immutable(modulus) => {
                        self.client.map(|client| {
                            client.mod_exponent_done(
                                Err(error),
                                self.message.take().unwrap(),
                                modulus,
                                exponent,
                                result,
                            );
                        });
                    }
                    MutImutBuffer::Mutable(_) => unreachable!(),
                };
            }
        }
    }
}

impl<'a> crate::otbn::Client<'a> for OtbnRsa<'a> {
    fn op_done(&'a self, result: Result<(), ErrorCode>, output: &'static mut [u8]) {
        if let Err(e) = result {
            self.report_error(e, output);
            return;
        }

        // We want to return BE data
        output.reverse();

        match self.exponent.take().unwrap() {
            MutImutBuffer::Mutable(exponent) => {
                self.client_mut.map(|client| {
                    match self.modulus.take().unwrap() {
                        MutImutBuffer::Mutable(modulus) => {
                            client.mod_exponent_done(
                                Ok(true),
                                self.message.take().unwrap(),
                                modulus,
                                exponent,
                                output,
                            );
                        }
                        MutImutBuffer::Immutable(_) => unreachable!(),
                    };
                });
            }
            MutImutBuffer::Immutable(exponent) => {
                match self.modulus.take().unwrap() {
                    MutImutBuffer::Immutable(modulus) => {
                        self.client.map(|client| {
                            client.mod_exponent_done(
                                Ok(true),
                                self.message.take().unwrap(),
                                modulus,
                                exponent,
                                output,
                            );
                        });
                    }
                    MutImutBuffer::Mutable(_) => unreachable!(),
                };
            }
        }
    }
}

impl<'a> RsaCryptoBase<'a> for OtbnRsa<'a> {
    fn set_client(&'a self, client: &'a dyn Client<'a>) {
        self.client.set(client);
    }

    fn clear_data(&self) {
        self.otbn.clear_data();
    }

    fn mod_exponent(
        &self,
        message: &'static mut [u8],
        modulus: &'static [u8],
        exponent: &'static [u8],
        result: &'static mut [u8],
    ) -> Result<
        (),
        (
            ErrorCode,
            &'static mut [u8],
            &'static [u8],
            &'static [u8],
            &'static mut [u8],
        ),
    > {
        // Check that the lengths match our expectations
        let op_len = modulus.len();

        if result.len() < op_len {
            return Err((ErrorCode::SIZE, message, modulus, exponent, result));
        }

        let slice = unsafe {
            core::slice::from_raw_parts(self.rsa.imem_start as *mut u8, self.rsa.imem_size)
        };
        if let Err(e) = self.otbn.load_binary(slice) {
            return Err((e, message, modulus, exponent, result));
        }

        let slice = unsafe {
            core::slice::from_raw_parts(self.rsa.dmem_start as *mut u8, self.rsa.dmem_size)
        };
        if let Err(e) = self.otbn.load_data(0, slice) {
            return Err((e, message, modulus, exponent, result));
        }

        // Set the mode to decryption
        if let Some(data) = self.internal.take() {
            data[0] = 2;
            data[1] = 0;
            data[2] = 0;
            data[3] = 0;
            // Set the RSA mode
            // The address is the offset of `mode` in the RSA elf
            if let Err(e) = self.otbn.load_data(0, &data[0..4]) {
                return Err((e, message, modulus, exponent, result));
            }

            data[0] = (op_len / 32) as u8;
            data[1] = 0;
            data[2] = 0;
            data[3] = 0;
            // Set the RSA length
            // The address is the offset of `n_limbs` in the RSA elf
            if let Err(e) = self.otbn.load_data(4, &data[0..4]) {
                return Err((e, message, modulus, exponent, result));
            }

            data[0..op_len].copy_from_slice(modulus);
            // We were passed BE data and the OTBN expects LE
            // so reverse the order.
            data[0..op_len].reverse();
            // Set the RSA modulus
            // The address is the offset of `modulus` in the RSA elf
            if let Err(e) = self.otbn.load_data(0x20, &data[0..op_len]) {
                return Err((e, message, modulus, exponent, result));
            }

            let len = exponent.len().min(op_len);
            data[0..len].copy_from_slice(exponent);
            // We were passed BE data and the OTBN expects LE
            // so reverse the order.
            data[0..len].reverse();

            // Set the RSA exponent
            // The address is the offset of `exp` in the RSA elf
            if let Err(e) = self.otbn.load_data(0x220, &data[0..len]) {
                return Err((e, message, modulus, exponent, result));
            }

            self.internal.replace(data);
        } else {
            return Err((ErrorCode::NOMEM, message, modulus, exponent, result));
        }

        // Set the data in
        // The address is the offset of `inout` in the RSA elf
        if let Err(e) = self.otbn.load_data(0x420, message) {
            return Err((e, message, modulus, exponent, result));
        }

        self.message.replace(message);
        self.modulus.replace(MutImutBuffer::Immutable(modulus));
        self.exponent.replace(MutImutBuffer::Immutable(exponent));

        // Get the data out
        // The address is the offset of `inout` in the RSA elf
        if let Err(e) = self.otbn.run(0x420, result) {
            return Err((e.0, self.message.take().unwrap(), modulus, exponent, e.1));
        }

        Ok(())
    }
}
