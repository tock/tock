//! Implementation of the AESA peripheral on the SAM4L.

use core::cell::Cell;
use kernel::common::VolatileCell;
use kernel::common::take_cell::TakeCell;
use kernel::hil;
use pm;
use scif;

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

pub struct Aes {
    registers: *mut AesRegisters,
    client: Cell<Option<&'static hil::symmetric_encryption::Client>>,
    data: TakeCell<'static, [u8]>,
    iv: TakeCell<'static, [u8]>,
    data_index: Cell<usize>,
    remaining_length: Cell<usize>,
}

pub static mut AES: Aes = Aes::new();

impl Aes {
    pub const fn new() -> Aes {
        Aes {
            registers: AES_BASE as *mut AesRegisters,
            client: Cell::new(None),
            data: TakeCell::empty(),
            iv: TakeCell::empty(),
            data_index: Cell::new(0),
            remaining_length: Cell::new(0),
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

    pub fn enable(&self) {
        let regs: &AesRegisters = unsafe { &*self.registers };

        self.enable_clock();
        regs.ctrl.set(0x01);
    }

    pub fn disable(&self) {
        let regs: &AesRegisters = unsafe { &*self.registers };

        regs.ctrl.set(0x00);
        self.disable_clock();
    }

    fn enable_ctr_mode(&self) {
        let regs: &AesRegisters = unsafe { &*self.registers };

        //         encrypt    dma        mode       cmeasure
        let mode = (1 << 0) | (0 << 3) | (4 << 4) | (0xF << 16);
        regs.mode.set(mode);
    }

    fn enable_interrupts(&self) {
        let regs: &AesRegisters = unsafe { &*self.registers };

        // We want both interrupts.
        regs.ier.set((1 << 16) | (1 << 0));
    }

    fn disable_interrupts(&self) {
        let regs: &AesRegisters = unsafe { &*self.registers };
        regs.idr.set((1 << 16) | (1 << 0));
    }

    fn notify_new_message(&self) {
        let regs: &AesRegisters = unsafe { &*self.registers };

        // Notify of a new message.
        regs.ctrl.set((1 << 2) | (1 << 0));
    }

    fn write_block(&self) {
        let regs: &AesRegisters = unsafe { &*self.registers };

        self.data.map(|data| {
            let index = self.data_index.get();
            for i in 0..4 {
                let mut v = data[index + (i * 4) + 0] as usize;
                v |= (data[index + (i * 4) + 1] as usize) << 8;
                v |= (data[index + (i * 4) + 2] as usize) << 16;
                v |= (data[index + (i * 4) + 3] as usize) << 24;
                regs.idata.set(v as u32);
            }
            self.data_index.set(index + 16);
            self.remaining_length.set(self.remaining_length.get() - 16);
        });
    }

    pub fn handle_interrupt(&self) {
        let regs: &AesRegisters = unsafe { &*self.registers };

        let status = regs.sr.get();

        if status & 0x01 == 0x01 {
            // Incoming data ready. Copy it into the same buffer we got
            // data from.

            self.data.take().map(|data| {
                // We need to go back to overwrite the previous 16 bytes.
                let index = self.data_index.get() - 16;
                for i in 0..4 {
                    let v = regs.odata.get();
                    data[index + (i * 4) + 0] = (v >> 0) as u8;
                    data[index + (i * 4) + 1] = (v >> 8) as u8;
                    data[index + (i * 4) + 2] = (v >> 16) as u8;
                    data[index + (i * 4) + 3] = (v >> 24) as u8;
                }
                // Check if we processed all of the data.
                if self.remaining_length.get() == 0 {
                    self.disable_interrupts();
                    self.iv.take().map(|iv| {
                        self.client.get().map(move |client| {
                            client.crypt_done(data, iv, index + 16);
                        });
                    });
                } else {
                    // Need to put the data buffer back
                    self.data.replace(data);
                }
            });
        }

        if status & (1 << 16) == (1 << 16) {
            // Check if we have more data to send.
            if self.remaining_length.get() > 0 {
                self.write_block();
            }
        }
    }
}

impl hil::symmetric_encryption::SymmetricEncryption for Aes {
    fn set_client(&self, client: &'static hil::symmetric_encryption::Client) {
        self.client.set(Some(client));
    }

    fn init(&self) {}

    fn set_key(&self, key: &'static mut [u8], len: usize) -> &'static mut [u8] {
        let regs: &AesRegisters = unsafe { &*self.registers };
        self.enable();

        if len == 16 {
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
        }
        key
    }

    fn aes128_crypt_ctr(&self, data: &'static mut [u8], init_ctr: &'static mut [u8], len: usize) {
        let regs: &AesRegisters = unsafe { &*self.registers };
        self.enable();
        self.enable_interrupts();
        self.enable_ctr_mode();
        self.notify_new_message();

        // Set the CTR value from the array.
        for i in 0..4 {
            let mut c = init_ctr[i * 4 + 0] as usize;
            c |= (init_ctr[i * 4 + 1] as usize) << 8;
            c |= (init_ctr[i * 4 + 2] as usize) << 16;
            c |= (init_ctr[i * 4 + 3] as usize) << 24;
            match i {
                0 => regs.initvect0.set(c as u32),
                1 => regs.initvect1.set(c as u32),
                2 => regs.initvect2.set(c as u32),
                3 => regs.initvect3.set(c as u32),
                _ => {}
            }
        }
        self.iv.replace(init_ctr);

        self.data.replace(data);
        self.remaining_length.set(len);
        self.data_index.set(0);
        self.write_block();
    }
}
