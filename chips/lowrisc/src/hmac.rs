//! SHA256 HMAC (Hash-based Message Authentication Code).

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::leasable_buffer::LeasableBuffer;
use kernel::common::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::hil::digest;
use kernel::ReturnCode;

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
        (0x804 => @END),
    }
}

register_bitfields![u32,
    INTR_STATE [
        HMAC_DONE OFFSET(0) NUMBITS(1) [],
        FIFO_FULL OFFSET(1) NUMBITS(1) [],
        HMAC_ERR OFFSET(2) NUMBITS(1) []
    ],
    INTR_ENABLE [
        HMAC_DONE OFFSET(0) NUMBITS(1) [],
        FIFO_FULL OFFSET(1) NUMBITS(1) [],
        HMAC_ERR OFFSET(2) NUMBITS(1) []
    ],
    INTR_TEST [
        HMAC_DONE OFFSET(0) NUMBITS(1) [],
        FIFO_FULL OFFSET(1) NUMBITS(1) [],
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

    client: OptionalCell<&'a dyn hil::digest::Client<'a, [u8; 32]>>,

    data: Cell<Option<LeasableBuffer<'static, u8>>>,
    data_len: Cell<usize>,
    data_index: Cell<usize>,

    digest: Cell<Option<&'static mut [u8; 32]>>,
}

impl Hmac<'a> {
    pub const fn new(base: StaticRef<HmacRegisters>) -> Hmac<'a> {
        Hmac {
            registers: base,
            client: OptionalCell::empty(),
            data: Cell::new(None),
            data_len: Cell::new(0),
            data_index: Cell::new(0),
            digest: Cell::new(None),
        }
    }

    fn data_progress(&self) {
        let regs = self.registers;
        let idx = self.data_index.get();
        let len = self.data_len.get();

        let slice = self.data.take().unwrap().take();

        if idx < len {
            let data_len = len - idx;

            for i in 0..(data_len / 4) {
                if regs.status.is_set(STATUS::FIFO_FULL) {
                    // Due to: https://github.com/lowRISC/opentitan/issues/1276
                    //   we can't get back to processing the data as there is no
                    //   FIFO not full interrupt.
                    // Let's just keep going and put up with the back pressure.
                    // break;
                }

                let data_idx = i * 4;

                let mut d = (slice[data_idx + 0] as u32) << 0;
                d |= (slice[data_idx + 1] as u32) << 8;
                d |= (slice[data_idx + 2] as u32) << 16;
                d |= (slice[data_idx + 3] as u32) << 24;

                regs.msg_fifo.set(d);
                self.data_index.set(data_idx + 4);
            }

            let idx = self.data_index.get();

            for i in 0..(data_len % 4) {
                let data_idx = idx + i;
                let d = (slice[data_idx]) as u32;

                regs.msg_fifo.set(d);
                self.data_index.set(data_idx + 1)
            }
        }

        self.client.map(move |client| {
            client.add_data_done(Ok(()), slice);
        });
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let intrs = regs.intr_state.extract();

        regs.intr_enable
            .modify(INTR_ENABLE::HMAC_DONE::CLEAR + INTR_ENABLE::HMAC_ERR::CLEAR);

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
        } else if intrs.is_set(INTR_STATE::FIFO_FULL) {
            // FIFO is full, we can't do anything
        } else if intrs.is_set(INTR_STATE::HMAC_ERR) {
            regs.intr_state.modify(INTR_STATE::HMAC_ERR::SET);

            self.client.map(|client| {
                client.hash_done(Err(ReturnCode::FAIL), self.digest.take().unwrap());
            });
        }
    }
}

impl hil::digest::Digest<'a, [u8; 32]> for Hmac<'a> {
    fn set_client(&'a self, client: &'a dyn digest::Client<'a, [u8; 32]>) {
        self.client.set(client);
    }

    fn add_data(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<usize, (ReturnCode, &'static mut [u8])> {
        let regs = self.registers;

        // Ensure the HMAC is setup
        regs.cfg
            .write(CFG::ENDIAN_SWAP::SET + CFG::SHA_EN::SET + CFG::DIGEST_SWAP::SET);

        regs.cmd.modify(CMD::START::SET);

        // Set the length and data index of the data to write
        self.data_len.set(data.len());
        self.data.set(Some(data));
        self.data_index.set(0);

        // Call the process function, this will start an async fill method
        self.data_progress();

        Ok(self.data_len.get())
    }

    fn run(
        &'a self,
        digest: &'static mut [u8; 32],
    ) -> Result<(), (ReturnCode, &'static mut [u8; 32])> {
        let regs = self.registers;

        // Enable interrrupts
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

impl hil::digest::HMACSha256 for Hmac<'a> {
    fn set_mode_hmacsha256(&self, key: &[u8; 32]) -> Result<(), ReturnCode> {
        let regs = self.registers;

        // Ensure the HMAC is setup
        regs.cfg
            .write(CFG::ENDIAN_SWAP::SET + CFG::SHA_EN::SET + CFG::DIGEST_SWAP::SET);

        for i in 0..8 {
            let idx = i * 4;

            let mut k = key[idx + 0] as u32;
            k |= (key[i * 4 + 1] as u32) << 8;
            k |= (key[i * 4 + 2] as u32) << 16;
            k |= (key[i * 4 + 3] as u32) << 24;

            regs.key[i as usize].set(k);
        }

        Ok(())
    }
}
