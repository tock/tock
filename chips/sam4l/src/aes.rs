//! Implementation of the AESA peripheral on the SAM4L.
//!
//! Authors:
//!
//! - Daniel Giffin  <daniel@beech-grove.net>
//! - Hubert Teo <hubert.teo.hk@gmail.com>
//! - Brad Campbell <bradjc5@gmail.com>
//!
//! Converted to new register abstraction by Philip Levis <pal@cs.stanford.edu>

use crate::pm;
use crate::scif;
use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::debug;
use kernel::hil;
use kernel::hil::symmetric_encryption::{AES128_BLOCK_SIZE, AES128_KEY_SIZE};
use kernel::ReturnCode;

#[allow(dead_code)]
#[derive(Copy, Clone)]
enum ConfidentialityMode {
    ECB = 0,
    CBC = 1,
    CFB = 2,
    OFB = 3,
    CTR = 4,
}

/// The registers used to interface with the hardware
#[repr(C)]
struct AesRegisters {
    ctrl: ReadWrite<u32, Control::Register>,         //   0x00
    mode: ReadWrite<u32, Mode::Register>,            //   0x04
    databufptr: ReadWrite<u32, DataBuf::Register>,   //   0x08
    sr: ReadOnly<u32, Status::Register>,             //   0x0c
    ier: WriteOnly<u32, Interrupt::Register>,        //   0x10
    idr: WriteOnly<u32, Interrupt::Register>,        //   0x14
    imr: ReadOnly<u32, Interrupt::Register>,         //   0x18
    _reserved0: [u32; 1],                            //   0x1c
    key0: WriteOnly<u32, Key::Register>,             //   0x20
    key1: WriteOnly<u32, Key::Register>,             //   0x24
    key2: WriteOnly<u32, Key::Register>,             //   0x28
    key3: WriteOnly<u32, Key::Register>,             //   0x2c
    key4: WriteOnly<u32, Key::Register>,             //   0x30
    key5: WriteOnly<u32, Key::Register>,             //   0x34
    key6: WriteOnly<u32, Key::Register>,             //   0x38
    key7: WriteOnly<u32, Key::Register>,             //   0x3c
    initvect0: WriteOnly<u32, InitVector::Register>, //   0x40
    initvect1: WriteOnly<u32, InitVector::Register>, //   0x44
    initvect2: WriteOnly<u32, InitVector::Register>, //   0x48
    initvect3: WriteOnly<u32, InitVector::Register>, //   0x4c
    idata: WriteOnly<u32, Data::Register>,           //   0x50
    _reserved1: [u32; 3],                            //          0x54 - 0x5c
    odata: ReadOnly<u32, Data::Register>,            //   0x60
    _reserved2: [u32; 3],                            //          0x64 - 0x6c
    drngseed: WriteOnly<u32, DrngSeed::Register>,    //   0x70
    parameter: ReadOnly<u32, Parameter::Register>,   //   0x70
    version: ReadOnly<u32, Version::Register>,       //   0x70
}

register_bitfields![u32,
    Control [
        ENABLE 0,
        DKEYGEN 1,
        NEWMSG 2,
        SWSRT 8
    ],
    Mode [
        CTYPE4  OFFSET(19) NUMBITS(1) [],
        CTYPE3  OFFSET(18) NUMBITS(1) [],
        CTYPE2  OFFSET(17) NUMBITS(1) [],
        CTYPE1  OFFSET(16) NUMBITS(1) [],
        CFBS    OFFSET(8)  NUMBITS(3) [
            Bits128 = 0,
            Bits64  = 1,
            Bits32  = 2,
            Bits16  = 3,
            Bits8   = 4
        ],
        OPMODE  OFFSET(4)  NUMBITS(3) [
            ECB     = 0,
            CBC     = 1,
            CFB     = 2,
            OFB     = 3,
            CTR     = 4
        ],
        DMA     OFFSET(3)  NUMBITS(1) [],
        ENCRYPT OFFSET(0)  NUMBITS(1) []
    ],
    DataBuf [
        ODATAW OFFSET(4)  NUMBITS(2) [],
        IDATAW OFFSET(0)  NUMBITS(2) []
    ],
    Status [
        IBUFRDY 16,
        ODATARDY 0
    ],
    Interrupt [
        IBUFRDY 16,
        ODATARDY 0
    ],
    Key [
        KEY OFFSET(0)  NUMBITS(32) []
    ],
    InitVector [
        VECTOR OFFSET(0)  NUMBITS(32) []
    ],
    Data [
        DATA OFFSET(0)  NUMBITS(32) []
    ],
    DrngSeed [
        SEED OFFSET(0)  NUMBITS(32) []
    ],
    Parameter [
        CTRMEAS OFFSET(8)  NUMBITS(1) [
            Implemented = 0,
            NotImplemented = 1
        ],
        OPMODE  OFFSET(2)  NUMBITS(3) [
            ECB = 0,
            ECB_CBC = 1,
            ECB_CBC_CFB = 2,
            ECB_CBC_CFB_OFB = 3,
            ECB_CBC_CFB_OFB_CTR = 4
        ],
        MAXKEYSIZE OFFSET(0)  NUMBITS(2) [
            Bits128 = 0,
            Bits192 = 1,
            Bits256 = 2
        ]
    ],
    Version [
        VARIANT  OFFSET(16)  NUMBITS(4),
        VERSION  OFFSET(0)   NUMBITS(12)
    ]
];

// Section 7.1 of datasheet
const AES_BASE: StaticRef<AesRegisters> =
    unsafe { StaticRef::new(0x400B0000 as *const AesRegisters) };

pub struct Aes<'a> {
    registers: StaticRef<AesRegisters>,

    client: OptionalCell<&'a hil::symmetric_encryption::Client<'a>>,
    source: TakeCell<'a, [u8]>,
    dest: TakeCell<'a, [u8]>,

    // An index into `source` (or `dest` if that does not exist),
    // marking how much data has been written to the AESA
    write_index: Cell<usize>,

    // An index into `dest`, marking how much data has been read back from the AESA
    read_index: Cell<usize>,

    // The index just after the last byte of `dest` that should receive encrypted output
    stop_index: Cell<usize>,
}

impl Aes<'a> {
    const fn new() -> Aes<'a> {
        Aes {
            registers: AES_BASE,
            client: OptionalCell::empty(),
            source: TakeCell::empty(),
            dest: TakeCell::empty(),
            write_index: Cell::new(0),
            read_index: Cell::new(0),
            stop_index: Cell::new(0),
        }
    }

    fn enable_clock(&self) {
        pm::enable_clock(pm::Clock::HSB(pm::HSBClock::AESA));
        scif::generic_clock_enable_divided(
            scif::GenericClock::GCLK4,
            scif::ClockSource::CLK_CPU,
            1,
        );
        scif::generic_clock_enable(scif::GenericClock::GCLK4, scif::ClockSource::CLK_CPU);
    }

    fn disable_clock(&self) {
        scif::generic_clock_disable(scif::GenericClock::GCLK4);
        pm::disable_clock(pm::Clock::HSB(pm::HSBClock::AESA));
    }

    fn enable_interrupts(&self) {
        let regs: &AesRegisters = &*self.registers;
        regs.ier
            .write(Interrupt::IBUFRDY.val(1) + Interrupt::ODATARDY.val(1));
    }

    fn disable_interrupts(&self) {
        let regs: &AesRegisters = &*self.registers;
        regs.idr
            .write(Interrupt::IBUFRDY.val(1) + Interrupt::ODATARDY.val(1));
    }

    fn disable_input_interrupt(&self) {
        let regs: &AesRegisters = &*self.registers;
        // Tell the AESA not to send an interrupt looking for more input
        regs.idr.write(Interrupt::IBUFRDY.val(1));
    }

    fn busy(&self) -> bool {
        let regs: &AesRegisters = &*self.registers;
        // Are any interrupts set, meaning an encryption operation
        // is in progress?
        (regs.imr.read(Interrupt::IBUFRDY) | regs.imr.read(Interrupt::ODATARDY)) != 0
    }

    fn set_mode(&self, encrypting: bool, mode: ConfidentialityMode) {
        let regs: &AesRegisters = &*self.registers;
        let encrypt = if encrypting { 1 } else { 0 };
        let dma = 0;
        regs.mode.write(
            Mode::ENCRYPT.val(encrypt)
                + Mode::DMA.val(dma)
                + Mode::OPMODE.val(mode as u32)
                + Mode::CTYPE4.val(1)
                + Mode::CTYPE3.val(1)
                + Mode::CTYPE2.val(1)
                + Mode::CTYPE1.val(1),
        );
    }

    fn input_buffer_ready(&self) -> bool {
        let regs: &AesRegisters = &*self.registers;
        regs.sr.read(Status::IBUFRDY) != 0
    }

    fn output_data_ready(&self) -> bool {
        let regs: &AesRegisters = &*self.registers;
        regs.sr.read(Status::ODATARDY) != 0
    }

    fn try_set_indices(&self, start_index: usize, stop_index: usize) -> bool {
        stop_index.checked_sub(start_index).map_or(false, |sublen| {
            sublen % AES128_BLOCK_SIZE == 0 && {
                self.source.map_or_else(
                    || {
                        // The destination buffer is also the input
                        if self.dest.map_or(false, |dest| stop_index <= dest.len()) {
                            self.write_index.set(start_index);
                            self.read_index.set(start_index);
                            self.stop_index.set(stop_index);
                            true
                        } else {
                            false
                        }
                    },
                    |source| {
                        if sublen == source.len()
                            && self.dest.map_or(false, |dest| stop_index <= dest.len())
                        {
                            // We will start writing to the AES from the beginning of `source`,
                            // and end at its end
                            self.write_index.set(0);

                            // We will start reading from the AES into `dest` at `start_index`,
                            // and continue until `stop_index`
                            self.read_index.set(start_index);
                            self.stop_index.set(stop_index);
                            true
                        } else {
                            false
                        }
                    },
                )
            }
        })
    }

    // Copy a block from the request buffer to the AESA input register,
    // if there is a block left in the buffer.  Either way, this function
    // returns true if more blocks remain to send.
    fn write_block(&self) -> bool {
        let regs: &AesRegisters = &*self.registers;
        self.source.map_or_else(
            || {
                // The source and destination are the same buffer
                self.dest.map_or_else(
                    || {
                        debug!("Called write_block() with no data");
                        false
                    },
                    |dest| {
                        let index = self.write_index.get();
                        let more = index + AES128_BLOCK_SIZE <= self.stop_index.get();
                        if !more {
                            return false;
                        }
                        for i in 0..4 {
                            let mut v = dest[index + (i * 4) + 0] as usize;
                            v |= (dest[index + (i * 4) + 1] as usize) << 8;
                            v |= (dest[index + (i * 4) + 2] as usize) << 16;
                            v |= (dest[index + (i * 4) + 3] as usize) << 24;
                            regs.idata.set(v as u32);
                        }
                        self.write_index.set(index + AES128_BLOCK_SIZE);

                        let more =
                            self.write_index.get() + AES128_BLOCK_SIZE <= self.stop_index.get();
                        more
                    },
                )
            },
            |source| {
                let index = self.write_index.get();

                let more = index + AES128_BLOCK_SIZE <= source.len();
                if !more {
                    return false;
                }

                for i in 0..4 {
                    let mut v = source[index + (i * 4) + 0] as usize;
                    v |= (source[index + (i * 4) + 1] as usize) << 8;
                    v |= (source[index + (i * 4) + 2] as usize) << 16;
                    v |= (source[index + (i * 4) + 3] as usize) << 24;
                    regs.idata.set(v as u32);
                }

                self.write_index.set(index + AES128_BLOCK_SIZE);

                let more = self.write_index.get() + AES128_BLOCK_SIZE <= source.len();
                more
            },
        )
    }

    // Copy a block from the AESA output register back into the request buffer
    // if there is any room left.  Return true if we are still waiting for more
    // blocks after this
    fn read_block(&self) -> bool {
        let regs: &AesRegisters = &*self.registers;
        self.dest.map_or_else(
            || {
                debug!("Called read_block() with no data");
                false
            },
            |dest| {
                let index = self.read_index.get();
                let more = index + AES128_BLOCK_SIZE <= self.stop_index.get();
                if !more {
                    return false;
                }

                for i in 0..4 {
                    let v = regs.odata.get();
                    dest[index + (i * 4) + 0] = (v >> 0) as u8;
                    dest[index + (i * 4) + 1] = (v >> 8) as u8;
                    dest[index + (i * 4) + 2] = (v >> 16) as u8;
                    dest[index + (i * 4) + 3] = (v >> 24) as u8;
                }

                self.read_index.set(index + AES128_BLOCK_SIZE);

                let more = self.read_index.get() + AES128_BLOCK_SIZE <= self.stop_index.get();
                more
            },
        )
    }

    /// Handle an interrupt, which will indicate either that the AESA's input
    /// buffer is ready for more data, or that it has completed a block of output
    /// for us to consume
    pub fn handle_interrupt(&self) {
        if !self.busy() {
            // Ignore errant interrupts, in case it's possible for the AES interrupt flag
            // to be set again while we are in this handler.
            return;
        }

        if self.input_buffer_ready() {
            // The AESA says it is ready to receive another block

            if !self.write_block() {
                // We've now written the entirety of the request buffer,
                // so unsubscribe from input interrupts
                self.disable_input_interrupt();
            }
        }

        if self.output_data_ready() {
            // The AESA says it has a completed block to give us

            if !self.read_block() {
                // We've read back all the blocks, so unsubscribe from
                // all interrupts
                self.disable_interrupts();

                // Alert the client of the completion
                self.client.map(|client| {
                    client.crypt_done(self.source.take(), self.dest.take().unwrap());
                });
            }
        }
    }
}

impl hil::symmetric_encryption::AES128<'a> for Aes<'a> {
    fn enable(&self) {
        let regs: &AesRegisters = &*self.registers;
        self.enable_clock();
        regs.ctrl.write(Control::ENABLE.val(1));
    }

    fn disable(&self) {
        let regs: &AesRegisters = &*self.registers;
        regs.ctrl.set(0);
        self.disable_clock();
    }

    fn set_client(&'a self, client: &'a hil::symmetric_encryption::Client<'a>) {
        self.client.set(client);
    }

    fn set_key(&self, key: &[u8]) -> ReturnCode {
        let regs: &AesRegisters = &*self.registers;
        if key.len() != AES128_KEY_SIZE {
            return ReturnCode::EINVAL;
        }

        for i in 0..4 {
            let mut k = key[i * 4 + 0] as usize;
            k |= (key[i * 4 + 1] as usize) << 8;
            k |= (key[i * 4 + 2] as usize) << 16;
            k |= (key[i * 4 + 3] as usize) << 24;
            match i {
                0 => regs.key0.set(k as u32),
                1 => regs.key1.set(k as u32),
                2 => regs.key2.set(k as u32),
                3 => regs.key3.set(k as u32),
                _ => {}
            }
        }

        ReturnCode::SUCCESS
    }

    fn set_iv(&self, iv: &[u8]) -> ReturnCode {
        let regs: &AesRegisters = &*self.registers;
        if iv.len() != AES128_BLOCK_SIZE {
            return ReturnCode::EINVAL;
        }

        // Set the initial value from the array.
        for i in 0..4 {
            let mut c = iv[i * 4 + 0] as usize;
            c |= (iv[i * 4 + 1] as usize) << 8;
            c |= (iv[i * 4 + 2] as usize) << 16;
            c |= (iv[i * 4 + 3] as usize) << 24;
            match i {
                0 => regs.initvect0.set(c as u32),
                1 => regs.initvect1.set(c as u32),
                2 => regs.initvect2.set(c as u32),
                3 => regs.initvect3.set(c as u32),
                _ => {}
            }
        }

        ReturnCode::SUCCESS
    }

    fn start_message(&self) {
        if self.busy() {
            return;
        }
        let regs: &AesRegisters = &*self.registers;
        regs.ctrl
            .write(Control::NEWMSG.val(1) + Control::ENABLE.val(1));
    }

    fn crypt(
        &'a self,
        source: Option<&'a mut [u8]>,
        dest: &'a mut [u8],
        start_index: usize,
        stop_index: usize,
    ) -> Option<(ReturnCode, Option<&'a mut [u8]>, &'a mut [u8])> {
        if self.busy() {
            Some((ReturnCode::EBUSY, source, dest))
        } else {
            self.source.put(source);
            self.dest.replace(dest);
            if self.try_set_indices(start_index, stop_index) {
                self.enable_interrupts();
                None
            } else {
                Some((
                    ReturnCode::EINVAL,
                    self.source.take(),
                    self.dest.take().unwrap(),
                ))
            }
        }
    }
}

impl hil::symmetric_encryption::AES128Ctr for Aes<'a> {
    fn set_mode_aes128ctr(&self, encrypting: bool) {
        self.set_mode(encrypting, ConfidentialityMode::CTR);
    }
}

impl hil::symmetric_encryption::AES128CBC for Aes<'a> {
    fn set_mode_aes128cbc(&self, encrypting: bool) {
        self.set_mode(encrypting, ConfidentialityMode::CBC);
    }
}

pub static mut AES: Aes<'static> = Aes::new();
