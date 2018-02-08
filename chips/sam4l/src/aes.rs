//! Implementation of the AESA peripheral on the SAM4L

use core::cell::Cell;
use core::mem;
use kernel::common::VolatileCell;
use kernel::common::take_cell::TakeCell;
use kernel::hil;
use kernel::hil::symmetric_encryption::AES128_BLOCK_SIZE;
use kernel::returncode::ReturnCode;
use pm;
use scif;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum ConfidentialityMode {
    ECB = 0,
    CBC,
    CFB,
    OFB,
    Ctr,
}

/// The registers used to interface with the hardware
#[repr(C)]
struct AesRegisters {
    ctrl: VolatileCell<u32>,       //       0x00
    mode: VolatileCell<u32>,       //       0x04
    databufptr: VolatileCell<u32>, // 0x08
    sr: VolatileCell<u32>,         //         0x0C
    ier: VolatileCell<u32>,        //        0x10
    idr: VolatileCell<u32>,        //        0x14
    imr: VolatileCell<u32>,        //        0x18
    _reserved0: VolatileCell<u32>, // 0x1C
    key0: VolatileCell<u32>,       //       0x20
    key1: VolatileCell<u32>,       //       0x24
    key2: VolatileCell<u32>,       //       0x28
    key3: VolatileCell<u32>,       //       0x2c
    key4: VolatileCell<u32>,       //       0x30
    key5: VolatileCell<u32>,       //       0x34
    key6: VolatileCell<u32>,       //       0x38
    key7: VolatileCell<u32>,       //       0x3c
    initvect0: VolatileCell<u32>,  //  0x40
    initvect1: VolatileCell<u32>,  //  0x44
    initvect2: VolatileCell<u32>,  //  0x48
    initvect3: VolatileCell<u32>,  //  0x4c
    idata: VolatileCell<u32>,      //      0x50
    _reserved1: [u32; 3],          //          0x54 - 0x5c
    odata: VolatileCell<u32>,      //      0x60
    _reserved2: [u32; 3],          //          0x64 - 0x6c
    drngseed: VolatileCell<u32>,   //   0x70
}

// Section 7.1 of datasheet
const AES_BASE: u32 = 0x400B0000;

const IBUFRDY: u32 = 1 << 16;
const ODATARDY: u32 = 1 << 0;

pub struct Aes<'a> {
    registers: *mut AesRegisters,

    client: Cell<Option<&'a hil::symmetric_encryption::Client<'a>>>,
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

impl<'a> Aes<'a> {
    pub const fn new() -> Aes<'a> {
        Aes {
            registers: AES_BASE as *mut AesRegisters,
            client: Cell::new(None),
            source: TakeCell::empty(),
            dest: TakeCell::empty(),
            write_index: Cell::new(0),
            read_index: Cell::new(0),
            stop_index: Cell::new(0),
        }
    }

    fn enable_clock(&self) {
        unsafe {
            pm::enable_clock(pm::Clock::HSB(pm::HSBClock::AESA));
            scif::generic_clock_enable_divided(
                scif::GenericClock::GCLK4,
                scif::ClockSource::CLK_CPU,
                1,
            );
            scif::generic_clock_enable(scif::GenericClock::GCLK4, scif::ClockSource::CLK_CPU);
        }
    }

    fn disable_clock(&self) {
        unsafe {
            scif::generic_clock_disable(scif::GenericClock::GCLK4);
            pm::disable_clock(pm::Clock::HSB(pm::HSBClock::AESA));
        }
    }

    fn enable_interrupts(&self) {
        let regs: &AesRegisters = unsafe { &*self.registers };
        // We want both interrupts.
        regs.ier.set(IBUFRDY | ODATARDY);
    }

    fn disable_interrupts(&self) {
        let regs: &AesRegisters = unsafe { &*self.registers };

        // Disable both interrupts
        regs.idr.set(IBUFRDY | ODATARDY);
    }

    fn disable_input_interrupt(&self) {
        let regs: &AesRegisters = unsafe { &*self.registers };

        // Tell the AESA not to send an interrupt looking for more input
        regs.idr.set(IBUFRDY);
    }

    fn busy(&self) -> bool {
        let regs: &AesRegisters = unsafe { &*self.registers };

        // Are any interrupts set, meaning an encryption operation is in progress?
        regs.imr.get() & (IBUFRDY | ODATARDY) != 0
    }

    fn set_mode(&self, encrypting: bool, mode: ConfidentialityMode) {
        let regs: &AesRegisters = unsafe { &*self.registers };

        let encrypt = if encrypting { 1 } else { 0 };
        let dma = 0;
        let cmeasure = 0xF;
        regs.mode
            .set(encrypt << 0 | dma << 3 | (mode as u32) << 4 | cmeasure << 16);
    }

    fn input_buffer_ready(&self) -> bool {
        let regs: &AesRegisters = unsafe { &*self.registers };
        let status = regs.sr.get();

        status & (1 << 16) != 0
    }

    fn output_data_ready(&self) -> bool {
        let regs: &AesRegisters = unsafe { &*self.registers };
        let status = regs.sr.get();

        status & (1 << 0) != 0
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
                        let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };
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

                let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };
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

                let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };
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
                if let Some(client) = self.client.get() {
                    client.crypt_done(self.source.take(), self.dest.take().unwrap());
                }
            }
        }
    }
}

impl<'a> hil::symmetric_encryption::AES128<'a> for Aes<'a> {
    fn enable(&self) {
        let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };

        self.enable_clock();
        regs.ctrl.set(0x01);
    }

    fn disable(&self) {
        let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };

        regs.ctrl.set(0x00);
        self.disable_clock();
    }

    fn set_client(&'a self, client: &'a hil::symmetric_encryption::Client<'a>) {
        self.client.set(Some(client));
    }

    fn set_key(&self, key: &[u8]) -> ReturnCode {
        if key.len() != AES128_BLOCK_SIZE {
            return ReturnCode::EINVAL;
        }

        let regs: &AesRegisters = unsafe { &*self.registers };

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
        if iv.len() != AES128_BLOCK_SIZE {
            return ReturnCode::EINVAL;
        }

        let regs: &AesRegisters = unsafe { &*self.registers };

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

        let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };

        regs.ctrl.set((1 << 2) | (1 << 0));
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

impl<'a> hil::symmetric_encryption::AES128Ctr for Aes<'a> {
    fn set_mode_aes128ctr(&self, encrypting: bool) {
        self.set_mode(encrypting, ConfidentialityMode::Ctr);
    }
}

impl<'a> hil::symmetric_encryption::AES128CBC for Aes<'a> {
    fn set_mode_aes128cbc(&self, encrypting: bool) {
        self.set_mode(encrypting, ConfidentialityMode::CBC);
    }
}

pub static mut AES: Aes<'static> = Aes::new();
