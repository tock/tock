//! Support for the AES hardware block on OpenTitan
//!
//! <https://docs.opentitan.org/hw/ip/aes/doc/>

use core::cell::Cell;
use kernel::common::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};
use kernel::hil;
use kernel::hil::symmetric_encryption;
use kernel::hil::symmetric_encryption::{AES128_BLOCK_SIZE, AES128_KEY_SIZE};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

const MAX_LENGTH: usize = 128;

register_structs! {
    pub AesRegisters {
        (0x00 => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x04 => key_share0_0: WriteOnly<u32>),
        (0x08 => key_share0_1: WriteOnly<u32>),
        (0x0C => key_share0_2: WriteOnly<u32>),
        (0x10 => key_share0_3: WriteOnly<u32>),
        (0x14 => key_share0_4: WriteOnly<u32>),
        (0x18 => key_share0_5: WriteOnly<u32>),
        (0x1C => key_share0_6: WriteOnly<u32>),
        (0x20 => key_share0_7: WriteOnly<u32>),
        (0x24 => key_share1_0: WriteOnly<u32>),
        (0x28 => key_share1_1: WriteOnly<u32>),
        (0x2C => key_share1_2: WriteOnly<u32>),
        (0x30 => key_share1_3: WriteOnly<u32>),
        (0x34 => key_share1_4: WriteOnly<u32>),
        (0x38 => key_share1_5: WriteOnly<u32>),
        (0x3C => key_share1_6: WriteOnly<u32>),
        (0x40 => key_share1_7: WriteOnly<u32>),
        (0x44 => iv_0: WriteOnly<u32>),
        (0x48 => iv_1: WriteOnly<u32>),
        (0x4C => iv_2: WriteOnly<u32>),
        (0x50 => iv_3: WriteOnly<u32>),
        (0x54 => data_in0: WriteOnly<u32>),
        (0x58 => data_in1: WriteOnly<u32>),
        (0x5C => data_in2: WriteOnly<u32>),
        (0x60 => data_in3: WriteOnly<u32>),
        (0x64 => data_out0: ReadOnly<u32>),
        (0x68 => data_out1: ReadOnly<u32>),
        (0x6C => data_out2: ReadOnly<u32>),
        (0x70 => data_out3: ReadOnly<u32>),
        (0x74 => ctrl: ReadWrite<u32, CTRL::Register>),
        (0x78 => trigger: WriteOnly<u32, TRIGGER::Register>),
        (0x7C => status: ReadOnly<u32, STATUS::Register>),
        (0x80 => @END),
    }
}

register_bitfields![u32,
    ALERT_TEST [
        RECOV_CTRL_UPDATE_ERR OFFSET(0) NUMBITS(1) [],
        FATAL_FAULT OFFSET(1) NUMBITS(1) [],
    ],
    CTRL [
        OPERATION OFFSET(0) NUMBITS(1) [
            Encrypting = 0,
            Decrypting = 1,
        ],
        MODE OFFSET(1) NUMBITS(6) [
            AES_ECB = 1,
            AES_CBC = 2,
            AES_CFB = 4,
            AES_OFB = 8,
            AES_CTR = 16,
            AES_NONE = 32,
        ],
        KEY_LEN OFFSET(7) NUMBITS(3) [
            Key128 = 1,
            Key192 = 2,
            Key256 = 4,
        ],
        MANUAL_OPERATION OFFSET(10) NUMBITS(1) [],
        FORCE_ZERO_MASKS OFFSET(11) NUMBITS(1) [],
    ],
    TRIGGER [
        START OFFSET(0) NUMBITS(1) [],
        KEY_IV_DATA_IN_CLEAR OFFSET(1) NUMBITS(1) [],
        DATA_OUT_CLEAR OFFSET(2) NUMBITS(1) [],
        PRNG_RESEED OFFSET(3) NUMBITS(1) [],
    ],
    STATUS [
        IDLE OFFSET(0) NUMBITS(1) [],
        STALL OFFSET(1) NUMBITS(1) [],
        OUTPUT_LOST OFFSET(2) NUMBITS(1) [],
        OUTPUT_VALID OFFSET(3) NUMBITS(1) [],
        INPUT_READY OFFSET(4) NUMBITS(1) [],
        ALERT_RECOV_CTRL_UPDATE_ERR OFFSET(5) NUMBITS(1) [],
        ALERT_FATAL_FAULT OFFSET(6) NUMBITS(1) [],
    ]
];

#[derive(Clone, Copy)]
enum Mode {
    IDLE,
    AES128CTR,
    AES128CBC,
    AES128ECB,
}

// https://docs.opentitan.org/hw/top_earlgrey/doc/
const AES_BASE: StaticRef<AesRegisters> =
    unsafe { StaticRef::new(0x4110_0000 as *const AesRegisters) };

pub struct Aes<'a> {
    registers: StaticRef<AesRegisters>,

    client: OptionalCell<&'a dyn hil::symmetric_encryption::Client<'a>>,
    source: TakeCell<'a, [u8]>,
    dest: TakeCell<'a, [u8]>,
    mode: Cell<Mode>,

    deferred_call: Cell<bool>,
    deferred_caller: &'static DynamicDeferredCall,
    deferred_handle: OptionalCell<DeferredCallHandle>,
}

impl<'a> Aes<'a> {
    pub const fn new(deferred_caller: &'static DynamicDeferredCall) -> Aes<'a> {
        Aes {
            registers: AES_BASE,
            client: OptionalCell::empty(),
            source: TakeCell::empty(),
            dest: TakeCell::empty(),
            mode: Cell::new(Mode::IDLE),
            deferred_call: Cell::new(false),
            deferred_caller,
            deferred_handle: OptionalCell::empty(),
        }
    }

    pub fn initialise(&self, deferred_call_handle: DeferredCallHandle) {
        self.deferred_handle.set(deferred_call_handle);
    }

    fn idle(&self) -> bool {
        self.registers.status.is_set(STATUS::IDLE)
    }

    fn input_ready(&self) -> bool {
        self.registers.status.is_set(STATUS::INPUT_READY)
    }

    /// Wait for the input to be ready, return an error if it takes too long
    fn wait_for_input_ready(&self) -> Result<(), ErrorCode> {
        let mut j = 0;

        while !self.input_ready() {
            j += 1;
            if j > 10000 {
                return Err(ErrorCode::FAIL);
            }
        }

        Ok(())
    }

    fn output_valid(&self) -> bool {
        self.registers.status.is_set(STATUS::OUTPUT_VALID)
    }

    /// Wait for the output to be valid, return an error if it takes too long
    fn wait_for_output_valid(&self) -> Result<(), ErrorCode> {
        let mut j = 0;

        while !self.output_valid() {
            j += 1;
            if j > 10000 {
                return Err(ErrorCode::FAIL);
            }
        }

        Ok(())
    }

    fn read_block(&self, blocknum: usize) -> Result<(), ErrorCode> {
        let blocknum = blocknum * AES128_BLOCK_SIZE;

        self.dest.map_or(Err(ErrorCode::NOMEM), |dest| {
            for i in 0..4 {
                // we work off an array of u8 so we need to assemble those
                // back into a u32
                let mut v = 0;
                match i {
                    0 => v = self.registers.data_out0.get(),
                    1 => v = self.registers.data_out1.get(),
                    2 => v = self.registers.data_out2.get(),
                    3 => v = self.registers.data_out3.get(),
                    _ => {}
                }
                dest[blocknum + (i * 4) + 0] = (v >> 0) as u8;
                dest[blocknum + (i * 4) + 1] = (v >> 8) as u8;
                dest[blocknum + (i * 4) + 2] = (v >> 16) as u8;
                dest[blocknum + (i * 4) + 3] = (v >> 24) as u8;
            }
            Ok(())
        })
    }

    fn write_block(&self, blocknum: usize) -> Result<(), ErrorCode> {
        self.source.map_or_else(
            || {
                // This is the case that dest = source
                self.dest.map_or(Err(ErrorCode::NOMEM), |dest| {
                    for i in 0..4 {
                        let mut v = dest[blocknum + (i * 4) + 0] as usize;
                        v |= (dest[blocknum + (i * 4) + 1] as usize) << 8;
                        v |= (dest[blocknum + (i * 4) + 2] as usize) << 16;
                        v |= (dest[blocknum + (i * 4) + 3] as usize) << 24;
                        match i {
                            0 => self.registers.data_in0.set(v as u32),
                            1 => self.registers.data_in1.set(v as u32),
                            2 => self.registers.data_in2.set(v as u32),
                            3 => self.registers.data_in3.set(v as u32),
                            _ => {}
                        }
                    }
                    Ok(())
                })
            },
            |source| {
                for i in 0..4 {
                    // we work off an array of u8 so we need to assemble
                    // those back into a u32
                    let mut v = source[blocknum + (i * 4) + 0] as usize;
                    v |= (source[blocknum + (i * 4) + 1] as usize) << 8;
                    v |= (source[blocknum + (i * 4) + 2] as usize) << 16;
                    v |= (source[blocknum + (i * 4) + 3] as usize) << 24;
                    match i {
                        0 => self.registers.data_in0.set(v as u32),
                        1 => self.registers.data_in1.set(v as u32),
                        2 => self.registers.data_in2.set(v as u32),
                        3 => self.registers.data_in3.set(v as u32),
                        _ => {}
                    }
                }
                Ok(())
            },
        )
    }

    fn do_crypt(
        &self,
        start_index: usize,
        stop_index: usize,
        mut write_block: usize,
    ) -> Result<(), ErrorCode> {
        let start_block = start_index / AES128_BLOCK_SIZE;
        let end_block = stop_index / AES128_BLOCK_SIZE;

        for i in start_block..end_block {
            self.wait_for_input_ready()?;
            self.write_block(write_block)?;

            self.wait_for_output_valid()?;
            self.read_block(i)?;
            write_block = write_block + AES128_BLOCK_SIZE;
        }

        Ok(())
    }
}

impl<'a> hil::symmetric_encryption::AES128<'a> for Aes<'a> {
    fn enable(&self) {
        self.registers.trigger.write(
            TRIGGER::KEY_IV_DATA_IN_CLEAR::SET
                + TRIGGER::DATA_OUT_CLEAR::SET
                + TRIGGER::PRNG_RESEED::SET,
        );
    }

    fn disable(&self) {
        self.registers.ctrl.write(CTRL::MANUAL_OPERATION::SET);
        self.registers.ctrl.write(CTRL::MANUAL_OPERATION::SET);

        self.registers.ctrl.write(CTRL::MANUAL_OPERATION::CLEAR);
        self.registers.ctrl.write(CTRL::MANUAL_OPERATION::CLEAR);
    }

    fn set_client(&'a self, client: &'a dyn symmetric_encryption::Client<'a>) {
        self.client.set(client);
    }

    fn set_iv(&self, iv: &[u8]) -> Result<(), ErrorCode> {
        if !self.idle() {
            return Err(ErrorCode::BUSY);
        }

        if iv.len() != AES128_BLOCK_SIZE {
            return Err(ErrorCode::INVAL);
        }

        for i in 0..(AES128_BLOCK_SIZE / 4) {
            let mut k = iv[i * 4 + 0] as u32;
            k |= (iv[i * 4 + 1] as u32) << 8;
            k |= (iv[i * 4 + 2] as u32) << 16;
            k |= (iv[i * 4 + 3] as u32) << 24;
            match i {
                0 => self.registers.iv_0.set(k),
                1 => self.registers.iv_1.set(k),
                2 => self.registers.iv_2.set(k),
                3 => self.registers.iv_3.set(k),
                _ => {
                    unreachable!()
                }
            }
        }

        Ok(())
    }

    fn set_key(&self, key: &[u8]) -> Result<(), ErrorCode> {
        if !self.idle() {
            return Err(ErrorCode::BUSY);
        }

        if key.len() != AES128_KEY_SIZE {
            return Err(ErrorCode::INVAL);
        }

        for i in 0..(AES128_KEY_SIZE / 4) {
            let mut k = key[i * 4 + 0] as u32;
            k |= (key[i * 4 + 1] as u32) << 8;
            k |= (key[i * 4 + 2] as u32) << 16;
            k |= (key[i * 4 + 3] as u32) << 24;
            match i {
                0 => {
                    self.registers.key_share0_0.set(k);
                    self.registers.key_share1_0.set(0);
                }
                1 => {
                    self.registers.key_share0_1.set(k);
                    self.registers.key_share1_1.set(0);
                }
                2 => {
                    self.registers.key_share0_2.set(k);
                    self.registers.key_share1_2.set(0);
                }
                3 => {
                    self.registers.key_share0_3.set(k);
                    self.registers.key_share1_3.set(0);
                }
                _ => {
                    unreachable!()
                }
            }
        }

        // We must write the rest of the registers as well
        // This should be written with random data, for now this will do
        self.registers.key_share0_4.set(0x12);
        self.registers.key_share0_5.set(0x34);
        self.registers.key_share0_6.set(0x56);
        self.registers.key_share0_7.set(0x78);

        self.registers.key_share1_4.set(0xAB);
        self.registers.key_share1_5.set(0xCD);
        self.registers.key_share1_6.set(0xEF);
        self.registers.key_share1_7.set(0x00);

        Ok(())
    }

    fn start_message(&self) {}

    fn crypt(
        &'a self,
        source: Option<&'a mut [u8]>,
        dest: &'a mut [u8],
        start_index: usize,
        stop_index: usize,
    ) -> Option<(Result<(), ErrorCode>, Option<&'a mut [u8]>, &'a mut [u8])> {
        match stop_index.checked_sub(start_index) {
            None => return Some((Err(ErrorCode::INVAL), source, dest)),
            Some(s) => {
                if s > MAX_LENGTH {
                    return Some((Err(ErrorCode::INVAL), source, dest));
                }
                if s % AES128_BLOCK_SIZE != 0 {
                    return Some((Err(ErrorCode::INVAL), source, dest));
                }
            }
        }

        if self.deferred_call.get() {
            return Some((
                Err(ErrorCode::BUSY),
                self.source.take(),
                self.dest.take().unwrap(),
            ));
        }

        let ret;
        self.dest.replace(dest);
        match source {
            None => {
                ret = self.do_crypt(start_index, stop_index, start_index);
            }
            Some(src) => {
                self.source.replace(src);
                ret = self.do_crypt(start_index, stop_index, 0);
            }
        }

        if ret.is_ok() {
            // Schedule a deferred call
            self.deferred_call.set(true);
            self.deferred_handle
                .map(|handle| self.deferred_caller.set(*handle));
            None
        } else {
            Some((ret, self.source.take(), self.dest.take().unwrap()))
        }
    }
}

impl kernel::hil::symmetric_encryption::AES128Ctr for Aes<'_> {
    fn set_mode_aes128ctr(&self, encrypting: bool) -> Result<(), ErrorCode> {
        if !self.idle() {
            return Err(ErrorCode::BUSY);
        }

        self.mode.set(Mode::AES128CTR);

        let mut ctrl = if encrypting {
            CTRL::OPERATION::Encrypting
        } else {
            CTRL::OPERATION::Decrypting
        };
        ctrl += CTRL::MODE::AES_CTR;
        // Tock only supports 128-bit keys
        ctrl += CTRL::KEY_LEN::Key128;
        ctrl += CTRL::MANUAL_OPERATION::CLEAR;

        // We need to set the control register twice as it's shadowed
        self.registers.ctrl.write(ctrl);
        self.registers.ctrl.write(ctrl);

        Ok(())
    }
}

impl kernel::hil::symmetric_encryption::AES128ECB for Aes<'_> {
    fn set_mode_aes128ecb(&self, encrypting: bool) -> Result<(), ErrorCode> {
        if !self.idle() {
            return Err(ErrorCode::BUSY);
        }

        self.mode.set(Mode::AES128ECB);

        let mut ctrl = if encrypting {
            CTRL::OPERATION::Encrypting
        } else {
            CTRL::OPERATION::Decrypting
        };
        ctrl += CTRL::MODE::AES_ECB;
        // Tock only supports 128-bit keys
        ctrl += CTRL::KEY_LEN::Key128;
        ctrl += CTRL::MANUAL_OPERATION::CLEAR;

        // We need to set the control register twice as it's shadowed
        self.registers.ctrl.write(ctrl);
        self.registers.ctrl.write(ctrl);

        Ok(())
    }
}

impl kernel::hil::symmetric_encryption::AES128CBC for Aes<'_> {
    fn set_mode_aes128cbc(&self, encrypting: bool) -> Result<(), ErrorCode> {
        if !self.idle() {
            return Err(ErrorCode::BUSY);
        }

        self.mode.set(Mode::AES128CBC);

        let mut ctrl = if encrypting {
            CTRL::OPERATION::Encrypting
        } else {
            CTRL::OPERATION::Decrypting
        };
        ctrl += CTRL::MODE::AES_CBC;
        // Tock only supports 128-bit keys
        ctrl += CTRL::KEY_LEN::Key128;
        ctrl += CTRL::MANUAL_OPERATION::CLEAR;

        // We need to set the control register twice as it's shadowed
        self.registers.ctrl.write(ctrl);
        self.registers.ctrl.write(ctrl);

        Ok(())
    }
}

impl<'a> DynamicDeferredCallClient for Aes<'_> {
    fn call(&self, _handle: DeferredCallHandle) {
        // Are we currently in a TX or RX transaction?
        if self.deferred_call.get() {
            self.deferred_call.set(false);

            self.client.map(|client| {
                client.crypt_done(self.source.take(), self.dest.take().unwrap());
            });
        }
    }
}
