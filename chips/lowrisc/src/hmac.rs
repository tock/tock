// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! SHA256 HMAC (Hash-based Message Authentication Code).

use core::cell::Cell;
use core::ops::Index;
use kernel::hil;
use kernel::hil::digest::{self, DigestData, DigestHash};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::leasable_buffer::LeasableBuffer;
use kernel::utilities::leasable_buffer::LeasableBufferDynamic;
use kernel::utilities::leasable_buffer::LeasableMutableBuffer;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

register_structs! {
    pub HmacRegisters {
        (0x00 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x04 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x08 => intr_test: ReadWrite<u32, INTR_TEST::Register>),
        (0x0C => alert_test: ReadWrite<u32>),
        (0x10 => cfg: ReadWrite<u32, CFG::Register>),
        (0x14 => cmd: ReadWrite<u32, CMD::Register>),
        (0x18 => status: ReadOnly<u32, STATUS::Register>),
        (0x1C => err_code: ReadOnly<u32>),
        (0x20 => wipe_secret: WriteOnly<u32>),
        (0x24 => key: [WriteOnly<u32>; 8]),
        (0x44 => digest: [ReadOnly<u32>; 8]),
        (0x64 => msg_length_lower: ReadOnly<u32>),
        (0x68 => msg_length_upper: ReadOnly<u32>),
        (0x6C => _reserved0),
        (0x800 => msg_fifo: WriteOnly<u32>),
        (0x804 => msg_fifo_8: WriteOnly<u8>),
        (0x805 => _reserved1),
        (0x808 => @END),
    }
}

register_bitfields![u32,
    INTR_STATE [
        HMAC_DONE OFFSET(0) NUMBITS(1) [],
        FIFO_EMPTY OFFSET(1) NUMBITS(1) [],
        HMAC_ERR OFFSET(2) NUMBITS(1) []
    ],
    INTR_ENABLE [
        HMAC_DONE OFFSET(0) NUMBITS(1) [],
        FIFO_EMPTY OFFSET(1) NUMBITS(1) [],
        HMAC_ERR OFFSET(2) NUMBITS(1) []
    ],
    INTR_TEST [
        HMAC_DONE OFFSET(0) NUMBITS(1) [],
        FIFO_EMPTY OFFSET(1) NUMBITS(1) [],
        HMAC_ERR OFFSET(2) NUMBITS(1) []
    ],
    CFG [
        HMAC_EN OFFSET(0) NUMBITS(1) [],
        SHA_EN OFFSET(1) NUMBITS(1) [],
        ENDIAN_SWAP OFFSET(2) NUMBITS(1) [],
        DIGEST_SWAP OFFSET(3) NUMBITS(1) []
    ],
    CMD [
        START OFFSET(0) NUMBITS(1) [],
        PROCESS OFFSET(1) NUMBITS(1) []
    ],
    STATUS [
        FIFO_EMPTY OFFSET(0) NUMBITS(1) [],
        FIFO_FULL OFFSET(1) NUMBITS(1) [],
        FIFO_DEPTH OFFSET(4) NUMBITS(5) []
    ]
];

pub struct Hmac<'a> {
    registers: StaticRef<HmacRegisters>,
    client: OptionalCell<&'a dyn hil::digest::Client<32>>,
    data: Cell<Option<LeasableBufferDynamic<'static, u8>>>,
    verify: Cell<bool>,
    digest: Cell<Option<&'static mut [u8; 32]>>,
    cancelled: Cell<bool>,
    busy: Cell<bool>,
}

impl Hmac<'_> {
    pub fn new(base: StaticRef<HmacRegisters>) -> Self {
        Hmac {
            registers: base,
            client: OptionalCell::empty(),
            data: Cell::new(None),
            verify: Cell::new(false),
            digest: Cell::new(None),
            cancelled: Cell::new(false),
            busy: Cell::new(false),
        }
    }

    fn process(&self, data: &dyn Index<usize, Output = u8>, count: usize) -> usize {
        let regs = self.registers;
        for i in 0..(count / 4) {
            if regs.status.is_set(STATUS::FIFO_FULL) {
                return i * 4;
            }

            let data_idx = i * 4;

            let mut d = (data[data_idx + 3] as u32) << 0;
            d |= (data[data_idx + 2] as u32) << 8;
            d |= (data[data_idx + 1] as u32) << 16;
            d |= (data[data_idx + 0] as u32) << 24;

            regs.msg_fifo.set(d);
        }

        if (count % 4) != 0 {
            for i in 0..(count % 4) {
                let data_idx = (count - (count % 4)) + i;
                regs.msg_fifo_8.set(data[data_idx]);
            }
        }
        count
    }

    // Return true if processing more data, false if the buffer
    // is completely processed.
    fn data_progress(&self) -> bool {
        self.data.take().map_or(false, |buf| match buf {
            LeasableBufferDynamic::Immutable(mut b) => {
                if b.len() == 0 {
                    self.data.set(Some(LeasableBufferDynamic::Immutable(b)));
                    false
                } else {
                    let count = self.process(&b, b.len());
                    b.slice(count..);
                    self.data.set(Some(LeasableBufferDynamic::Immutable(b)));
                    true
                }
            }
            LeasableBufferDynamic::Mutable(mut b) => {
                if b.len() == 0 {
                    self.data.set(Some(LeasableBufferDynamic::Mutable(b)));
                    false
                } else {
                    let count = self.process(&b, b.len());
                    b.slice(count..);
                    self.data.set(Some(LeasableBufferDynamic::Mutable(b)));
                    true
                }
            }
        })
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let intrs = regs.intr_state.extract();
        regs.intr_enable.modify(
            INTR_ENABLE::HMAC_DONE::CLEAR
                + INTR_ENABLE::FIFO_EMPTY::CLEAR
                + INTR_ENABLE::HMAC_ERR::CLEAR,
        );
        self.busy.set(false);
        if intrs.is_set(INTR_STATE::HMAC_DONE) {
            self.client.map(|client| {
                let digest = self.digest.take().unwrap();

                regs.intr_state.modify(INTR_STATE::HMAC_DONE::SET);

                if self.verify.get() {
                    let mut equal = true;

                    for i in 0..8 {
                        let d = regs.digest[i].get().to_ne_bytes();

                        let idx = i * 4;

                        if digest[idx + 0] != d[0]
                            || digest[idx + 1] != d[1]
                            || digest[idx + 2] != d[2]
                            || digest[idx + 3] != d[3]
                        {
                            equal = false;
                        }
                    }

                    if self.cancelled.get() {
                        self.clear_data();
                        self.cancelled.set(false);
                        client.verification_done(Err(ErrorCode::CANCEL), digest);
                    } else {
                        self.clear_data();
                        self.cancelled.set(false);
                        client.verification_done(Ok(equal), digest);
                    }
                } else {
                    for i in 0..8 {
                        let d = regs.digest[i].get().to_ne_bytes();

                        let idx = i * 4;

                        digest[idx + 0] = d[0];
                        digest[idx + 1] = d[1];
                        digest[idx + 2] = d[2];
                        digest[idx + 3] = d[3];
                    }
                    if self.cancelled.get() {
                        self.clear_data();
                        self.cancelled.set(false);
                        client.hash_done(Err(ErrorCode::CANCEL), digest);
                    } else {
                        self.clear_data();
                        self.cancelled.set(false);
                        client.hash_done(Ok(()), digest);
                    }
                }
            });
        } else if intrs.is_set(INTR_STATE::FIFO_EMPTY) {
            // Clear the FIFO empty interrupt
            regs.intr_state.modify(INTR_STATE::FIFO_EMPTY::SET);
            let rval = if self.cancelled.get() {
                self.cancelled.set(false);
                Err(ErrorCode::CANCEL)
            } else {
                Ok(())
            };
            if self.data_progress() == false {
                // False means we are done
                self.client.map(move |client| {
                    self.data.take().map(|buf| match buf {
                        LeasableBufferDynamic::Mutable(b) => client.add_mut_data_done(rval, b),
                        LeasableBufferDynamic::Immutable(b) => client.add_data_done(rval, b),
                    })
                });
                // Make sure we don't get any more FIFO empty interrupts
                regs.intr_enable.modify(INTR_ENABLE::FIFO_EMPTY::CLEAR);
            } else {
                // Processing more data
                // Enable interrupts
                regs.intr_enable.modify(INTR_ENABLE::FIFO_EMPTY::SET);
            }
        } else if intrs.is_set(INTR_STATE::HMAC_ERR) {
            regs.intr_state.modify(INTR_STATE::HMAC_ERR::SET);

            self.client.map(|client| {
                let errval = if self.cancelled.get() {
                    self.cancelled.set(false);
                    ErrorCode::CANCEL
                } else {
                    ErrorCode::FAIL
                };
                if self.verify.get() {
                    client.hash_done(Err(errval), self.digest.take().unwrap());
                } else {
                    client.hash_done(Err(errval), self.digest.take().unwrap());
                }
            });
        }
    }
}

impl<'a> hil::digest::DigestData<'a, 32> for Hmac<'a> {
    fn add_data(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<(), (ErrorCode, LeasableBuffer<'static, u8>)> {
        if self.busy.get() {
            Err((ErrorCode::BUSY, data))
        } else {
            self.busy.set(true);
            self.data.set(Some(LeasableBufferDynamic::Immutable(data)));

            let regs = self.registers;
            regs.cmd.modify(CMD::START::SET);
            // Clear the FIFO empty interrupt
            regs.intr_state.modify(INTR_STATE::FIFO_EMPTY::SET);
            // Enable interrupts
            regs.intr_enable.modify(INTR_ENABLE::FIFO_EMPTY::SET);
            let ret = self.data_progress();

            if ret {
                regs.intr_test.modify(INTR_TEST::FIFO_EMPTY::SET);
            }

            Ok(())
        }
    }

    fn add_mut_data(
        &self,
        data: LeasableMutableBuffer<'static, u8>,
    ) -> Result<(), (ErrorCode, LeasableMutableBuffer<'static, u8>)> {
        if self.busy.get() {
            Err((ErrorCode::BUSY, data))
        } else {
            self.busy.set(true);
            self.data.set(Some(LeasableBufferDynamic::Mutable(data)));

            let regs = self.registers;
            regs.cmd.modify(CMD::START::SET);
            // Clear the FIFO empty interrupt
            regs.intr_state.modify(INTR_STATE::FIFO_EMPTY::SET);
            // Enable interrupts
            regs.intr_enable.modify(INTR_ENABLE::FIFO_EMPTY::SET);
            let ret = self.data_progress();

            if ret {
                regs.intr_test.modify(INTR_TEST::FIFO_EMPTY::SET);
            }

            Ok(())
        }
    }

    fn clear_data(&self) {
        let regs = self.registers;
        regs.cmd.modify(CMD::START::CLEAR);
        regs.wipe_secret.set(1 as u32);
        self.cancelled.set(true);
    }

    fn set_data_client(&'a self, _client: &'a (dyn digest::ClientData<32> + 'a)) {}
}

impl<'a> hil::digest::DigestHash<'a, 32> for Hmac<'a> {
    fn run(
        &'a self,
        digest: &'static mut [u8; 32],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 32])> {
        let regs = self.registers;

        // Enable interrupts
        regs.intr_state
            .modify(INTR_STATE::HMAC_DONE::SET + INTR_STATE::HMAC_ERR::SET);
        regs.intr_enable
            .modify(INTR_ENABLE::HMAC_DONE::SET + INTR_ENABLE::HMAC_ERR::SET);

        // Start the process
        regs.cmd.modify(CMD::PROCESS::SET);
        self.busy.set(true);
        self.digest.set(Some(digest));

        Ok(())
    }

    fn set_hash_client(&'a self, _client: &'a (dyn digest::ClientHash<32> + 'a)) {}
}

impl<'a> hil::digest::DigestVerify<'a, 32> for Hmac<'a> {
    fn verify(
        &'a self,
        compare: &'static mut [u8; 32],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 32])> {
        self.verify.set(true);

        self.run(compare)
    }

    fn set_verify_client(&'a self, _client: &'a (dyn digest::ClientVerify<32> + 'a)) {}
}

impl<'a> hil::digest::Digest<'a, 32> for Hmac<'a> {
    fn set_client(&'a self, client: &'a dyn digest::Client<32>) {
        self.client.set(client);
    }
}

impl hil::digest::HmacSha256 for Hmac<'_> {
    fn set_mode_hmacsha256(&self, key: &[u8]) -> Result<(), ErrorCode> {
        if self.busy.get() {
            return Err(ErrorCode::BUSY);
        }
        let regs = self.registers;
        let mut key_idx = 0;

        if key.len() > 32 {
            return Err(ErrorCode::NOSUPPORT);
        }

        // Ensure the HMAC is setup
        regs.cfg.write(
            CFG::HMAC_EN::SET + CFG::SHA_EN::SET + CFG::ENDIAN_SWAP::CLEAR + CFG::DIGEST_SWAP::SET,
        );

        for i in 0..(key.len() / 4) {
            let idx = i * 4;

            let mut k = *key.get(idx + 3).ok_or(ErrorCode::INVAL)? as u32;
            k |= (*key.get(i * 4 + 2).ok_or(ErrorCode::INVAL)? as u32) << 8;
            k |= (*key.get(i * 4 + 1).ok_or(ErrorCode::INVAL)? as u32) << 16;
            k |= (*key.get(i * 4 + 0).ok_or(ErrorCode::INVAL)? as u32) << 24;

            regs.key.get(i).ok_or(ErrorCode::INVAL)?.set(k);
            key_idx = i + 1;
        }

        if (key.len() % 4) != 0 {
            let mut k = 0;

            for i in 0..(key.len() % 4) {
                k = k
                    | ((*key.get(key_idx * 4 + 1).ok_or(ErrorCode::INVAL)? as u32)
                        << (8 * (3 - i)));
            }

            regs.key.get(key_idx).ok_or(ErrorCode::INVAL)?.set(k);
            key_idx = key_idx + 1;
        }

        for i in key_idx..8 {
            regs.key.get(i).ok_or(ErrorCode::INVAL)?.set(0);
        }

        Ok(())
    }
}

impl hil::digest::HmacSha384 for Hmac<'_> {
    fn set_mode_hmacsha384(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl hil::digest::HmacSha512 for Hmac<'_> {
    fn set_mode_hmacsha512(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl hil::digest::Sha256 for Hmac<'_> {
    fn set_mode_sha256(&self) -> Result<(), ErrorCode> {
        if self.busy.get() {
            return Err(ErrorCode::BUSY);
        }
        let regs = self.registers;

        // Ensure the SHA is setup
        regs.cfg.write(
            CFG::HMAC_EN::CLEAR
                + CFG::SHA_EN::SET
                + CFG::ENDIAN_SWAP::CLEAR
                + CFG::DIGEST_SWAP::SET,
        );

        Ok(())
    }
}

impl hil::digest::Sha384 for Hmac<'_> {
    fn set_mode_sha384(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl hil::digest::Sha512 for Hmac<'_> {
    fn set_mode_sha512(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}
