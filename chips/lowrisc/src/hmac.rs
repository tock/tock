//! SHA256 HMAC (Hash-based Message Authentication Code).

use core::cell::Cell;
use kernel::common::leasable_buffer::LeasableBuffer;
use kernel::hil;
use kernel::hil::digest;
use kernel::utilities::cells::OptionalCell;
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
        (0x0C => cfg: ReadWrite<u32, CFG::Register>),
        (0x10 => cmd: ReadWrite<u32, CMD::Register>),
        (0x14 => status: ReadOnly<u32, STATUS::Register>),
        (0x18 => err_code: ReadOnly<u32>),
        (0x1C => wipe_secret: WriteOnly<u32>),
        (0x20 => key: [WriteOnly<u32>; 8]),
        (0x40 => digest: [ReadOnly<u32>; 8]),
        (0x60 => msg_length_lower: ReadOnly<u32>),
        (0x64 => msg_length_upper: ReadOnly<u32>),
        (0x68 => _reserved0),
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

    client: OptionalCell<&'a dyn hil::digest::Client<'a, 32>>,

    data: Cell<Option<LeasableBuffer<'static, u8>>>,
    data_len: Cell<usize>,
    data_index: Cell<usize>,

    digest: Cell<Option<&'static mut [u8; 32]>>,
}

impl Hmac<'_> {
    pub const fn new(base: StaticRef<HmacRegisters>) -> Self {
        Hmac {
            registers: base,
            client: OptionalCell::empty(),
            data: Cell::new(None),
            data_len: Cell::new(0),
            data_index: Cell::new(0),
            digest: Cell::new(None),
        }
    }

    fn data_progress(&self) -> bool {
        let regs = self.registers;
        let idx = self.data_index.get();
        let len = self.data_len.get();

        self.data.take().map_or(false, |buf| {
            let slice = buf.take();

            if idx < len {
                let data_len = len - idx;

                for i in 0..(data_len / 4) {
                    if regs.status.is_set(STATUS::FIFO_FULL) {
                        self.data.set(Some(LeasableBuffer::new(slice)));
                        return false;
                    }

                    let data_idx = idx + i * 4;

                    let mut d = (slice[data_idx + 3] as u32) << 0;
                    d |= (slice[data_idx + 2] as u32) << 8;
                    d |= (slice[data_idx + 1] as u32) << 16;
                    d |= (slice[data_idx + 0] as u32) << 24;

                    regs.msg_fifo.set(d);
                    self.data_index.set(data_idx + 4);
                }

                if (data_len % 4) != 0 {
                    let idx = self.data_index.get();

                    for i in 0..(data_len % 4) {
                        let data_idx = idx + i;

                        regs.msg_fifo_8.set(slice[data_idx]);
                        self.data_index.set(data_idx + 1)
                    }
                }
            }
            self.data.set(Some(LeasableBuffer::new(slice)));
            true
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

        if intrs.is_set(INTR_STATE::HMAC_DONE) {
            self.client.map(|client| {
                let digest = self.digest.take().unwrap();

                for i in 0..8 {
                    let d = regs.digest[i].get().to_ne_bytes();

                    let idx = i * 4;

                    digest[idx + 0] = d[0];
                    digest[idx + 1] = d[1];
                    digest[idx + 2] = d[2];
                    digest[idx + 3] = d[3];
                }

                regs.intr_state.modify(INTR_STATE::HMAC_DONE::SET);

                client.hash_done(Ok(()), digest);
            });
        } else if intrs.is_set(INTR_STATE::FIFO_EMPTY) {
            // Clear the FIFO empty interrupt
            regs.intr_state.modify(INTR_STATE::FIFO_EMPTY::SET);

            if self.data_progress() {
                self.client.map(move |client| {
                    self.data.take().map(|buf| {
                        let slice = buf.take();
                        client.add_data_done(Ok(()), slice);
                    })
                });

                // Make sure we don't get any more FIFO empty interrupts
                regs.intr_enable.modify(INTR_ENABLE::FIFO_EMPTY::CLEAR);
            } else {
                // Enable interrupts
                regs.intr_enable.modify(INTR_ENABLE::FIFO_EMPTY::SET);
            }
        } else if intrs.is_set(INTR_STATE::HMAC_ERR) {
            regs.intr_state.modify(INTR_STATE::HMAC_ERR::SET);

            self.client.map(|client| {
                client.hash_done(Err(ErrorCode::FAIL), self.digest.take().unwrap());
            });
        }
    }
}

impl<'a> hil::digest::Digest<'a, 32> for Hmac<'a> {
    fn set_client(&'a self, client: &'a dyn digest::Client<'a, 32>) {
        self.client.set(client);
    }

    fn add_data(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<usize, (ErrorCode, &'static mut [u8])> {
        let regs = self.registers;

        regs.cmd.modify(CMD::START::SET);

        // Clear the FIFO empty interrupt
        regs.intr_state.modify(INTR_STATE::FIFO_EMPTY::SET);

        // Enable interrupts
        regs.intr_enable.modify(INTR_ENABLE::FIFO_EMPTY::SET);

        // Set the length and data index of the data to write
        self.data_len.set(data.len());
        self.data.set(Some(data));
        self.data_index.set(0);

        // Call the process function, this will start an async fill method
        let ret = self.data_progress();

        if ret {
            regs.intr_test.modify(INTR_TEST::FIFO_EMPTY::SET);
        }

        Ok(self.data_len.get())
    }

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

        self.digest.set(Some(digest));

        Ok(())
    }

    fn clear_data(&self) {
        let regs = self.registers;

        regs.cmd.modify(CMD::START::CLEAR);
        regs.wipe_secret.set(1 as u32);
    }
}

impl hil::digest::HMACSha256 for Hmac<'_> {
    fn set_mode_hmacsha256(&self, key: &[u8]) -> Result<(), ErrorCode> {
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

            let mut k = key[idx + 3] as u32;
            k |= (key[i * 4 + 2] as u32) << 8;
            k |= (key[i * 4 + 1] as u32) << 16;
            k |= (key[i * 4 + 0] as u32) << 24;

            regs.key[i as usize].set(k);
            key_idx = i + 1;
        }

        if (key.len() % 4) != 0 {
            let mut k = 0;

            for i in 0..(key.len() % 4) {
                k = k | (key[key_idx * 4 + i] as u32) << (8 * (3 - i));
            }

            regs.key[key_idx].set(k);
            key_idx = key_idx + 1;
        }

        for i in key_idx..8 {
            regs.key[i as usize].set(0);
        }

        Ok(())
    }
}

impl hil::digest::HMACSha384 for Hmac<'_> {
    fn set_mode_hmacsha384(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl hil::digest::HMACSha512 for Hmac<'_> {
    fn set_mode_hmacsha512(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl hil::digest::Sha256 for Hmac<'_> {
    fn set_mode_sha256(&self) -> Result<(), ErrorCode> {
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
