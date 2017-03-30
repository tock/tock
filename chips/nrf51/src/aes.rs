//! AES128-CTR Driver
//!
//! Provides a simple driver for userspace applications to encrypt and decrypt messages
//! using aes128-ctr mode on top of aes128-ecb
//!
//! The initial counter configred according to the counter received from the user application.
//! The capsule is invoked as follows:
//!     - the key has been configured
//!     - the entire buffer has been encrypted
//!     - the entire buffer has been decrypted
//!
//! The buffer is also sliced in chips at the moment and some un-necessary
//! static mut...
//!
//! FIXME:
//!     - replace static mut with TakeCell or something similar
//!     (I had problem to use because it can only be used one with take() )
//!     - maybe move some stuff to capsule instead
//!
//! Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! Author: Fredrik Nilsson <frednils@student.chalmers.se>
//! Date: March 31, 2017


use chip;
use core::cell::Cell;
// use kernel::common::take_cell::TakeCell;
use kernel::hil::symmetric_encryption::{SymmetricEncryptionDriver, Client};
use nvic;
use peripheral_interrupts::NvicIdx;
use peripheral_registers::{AESECB_REGS, AESECB_BASE};

#[deny(no_mangle_const_items)]

static mut ECB_DATA: [u8; 48] = [0; 48];
// key 0-15
// cleartext 16-32
// ciphertext 32-47

static mut BUF: [u8; 128] = [0; 128];
static mut DATA: [u8; 128] = [0; 128];
static mut DMY: [u8; 16] = [0; 16];

#[no_mangle]
pub struct AesECB {
    regs: *mut AESECB_REGS,
    client: Cell<Option<&'static Client>>,
    ctr: Cell<[u8; 16]>,
    // data: TakeCell<'static, [u8]>,
    remaining: Cell<u8>,
    len: Cell<u8>,
    offset: Cell<u8>,
}

pub static mut AESECB: AesECB = AesECB::new();

impl AesECB {
    const fn new() -> AesECB {
        AesECB {
            regs: AESECB_BASE as *mut AESECB_REGS,
            client: Cell::new(None),
            ctr: Cell::new([0; 16]),
            // data: TakeCell::empty(),
            remaining: Cell::new(0),
            len: Cell::new(0),
            offset: Cell::new(0),
        }
    }

    pub fn ecb_init(&self) {
        let regs = unsafe { &*self.regs };
        unsafe {
            regs.ECBDATAPTR.set((&ECB_DATA as *const u8) as u32);
        }
    }

    // FIXME: should this be performed in constant time i.e. skip the break part and always loop 16
    // times?
    fn update_ctr(&self) {
        let mut ctr = self.ctr.get();
        for i in (0..16).rev() {
            ctr[i] += 1;
            if ctr[i] != 0 {
                break;
            }
        }
        self.ctr.set(ctr);
    }

    fn crypt(&self) {
        let regs = unsafe { &*self.regs };

        let ctr = self.ctr.get();

        for i in 0..16 {
            unsafe {
                ECB_DATA[i + 16] = ctr[i];
            }
        }

        regs.ENDECB.set(0);
        regs.STARTECB.set(1);

        self.enable_nvic();
        self.enable_interrupts();
    }

    // precondition: key_len = 16 || 24 || 32
    fn set_key(&self, key: &'static mut [u8], _: usize) {
        for (i, c) in key.as_ref()[0..16].iter().enumerate() {
            unsafe {
                ECB_DATA[i] = *c;
            }
        }

        unsafe {
            self.client.get().map(|client| client.set_key_done(&mut BUF[0..16], 16));
        }

    }

    pub fn handle_interrupt(&self) {
        let regs = unsafe { &*self.regs };

        // disable interrupts
        self.disable_nvic();
        self.disable_interrupts();
        nvic::clear_pending(NvicIdx::ECB);

        if regs.ENDECB.get() == 1 {

            let rem = self.remaining.get() as usize;
            let offset = self.offset.get() as usize;
            let take = if rem >= 16 { 16 } else { rem };


            // THIS DON'T WORK FOR MORE THAN 1 BLOCK BECAUSE TAKE EATS UP THE ENTIRE BUF
            // ------------------------------------------------------------------------------------
            // guard that more bytes exist
            // if self.data.is_some() && take > 0 {
            //     self.data.take().map(|buf| {
            //         if buf.len() >= offset + take {
            //             // take at most 16 bytes and XOR with the keystream
            //             for (i, c) in buf.as_ref()[offset .. offset+take].iter().enumerate() {
            //                 // m XOR ECB(k || ctr)
            //                 unsafe { BUF[i] = ECB_DATA[i-offset+32] ^ *c; }
            //                 debug!("{}\r\n", i);
            //             }
            //             self.offset.set( (offset + take) as u8 );
            //             self.remaining.set( (rem - take) as u8 );
            //         }
            //     });
            // }

            // TEMP solution
            if take > 0 {
                for i in offset..offset + take {
                    unsafe {
                        BUF[i] = ECB_DATA[i - offset + 32] ^ DATA[i];
                    }
                }
                self.offset.set((offset + take) as u8);
                self.remaining.set((rem - take) as u8);
                self.update_ctr();
            }

            // USE THIS PRINT TO TEST THAT THE CTR UPDATES ACCORDINGLY
            // debug!("ctr {:?}\r\n", self.ctr.get());

            // More bytes to encrypt!!!
            if self.remaining.get() > 0 {
                self.crypt();
            }
            // DONE
            else {
                // ugly work-around to replace buffers in the capsule;
                unsafe {
                    self.client.get().map(|client| {
                        client.crypt_done(&mut BUF[0..self.len.get() as usize],
                                          &mut DMY,
                                          self.len.get())
                    });
                }
            }
        }
        // else ERROR encrypt error do nothing
    }

    fn enable_interrupts(&self) {
        // set ENDECB bit and ERROR bit
        let regs = unsafe { &*self.regs };
        regs.INTENSET.set(1 | 1 << 1);
    }

    fn disable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.INTENCLR.set(1 | 1 << 1);
    }

    fn enable_nvic(&self) {
        nvic::enable(NvicIdx::ECB);
    }

    fn disable_nvic(&self) {
        nvic::disable(NvicIdx::ECB);
    }

    pub fn set_client<C: Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }

    pub fn set_initial_ctr(&self, iv: &'static mut [u8]) {
        // read bytes as big-endian
        let mut ctr: [u8; 16] = [0; 16];
        for (i, c) in iv.as_ref()[0..16].iter().enumerate() {
            ctr[i] = *c;
        }
        self.ctr.set(ctr);
    }
}

impl SymmetricEncryptionDriver for AesECB {
    // This Function is called once Tock is booted
    fn init(&self) {
        self.ecb_init();
    }

    // capsule ensures that the key is 16 bytes
    fn set_key(&self, key: &'static mut [u8], len: usize) {
        self.set_key(key, len)
    }

    fn aes128_crypt_ctr(&self, data: &'static mut [u8], iv: &'static mut [u8], len: usize) {
        self.remaining.set(len as u8);
        self.len.set(len as u8);
        self.offset.set(0);
        // self.data.replace(data);

        // append data to "enc/dec" to a the global buf
        for (i, c) in data.as_ref()[0..len as usize].iter().enumerate() {
            unsafe {
                DATA[i] = *c;
            }
        }
        self.set_initial_ctr(iv);
        self.crypt();
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn ECB_Handler() {
    use kernel::common::Queue;
    nvic::disable(NvicIdx::ECB);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(NvicIdx::ECB);
}
