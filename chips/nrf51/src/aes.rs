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
//!     - maybe move some stuff to capsule instead
//!     - INIT_CTR can be replaced with TakeCell
//!     - ECB_DATA must be a static mut [u8]
//!       and can't be located in the struct
//!     - PAYLOAD size is restricted to 128 bytes
//!
//! Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! Author: Fredrik Nilsson <frednils@student.chalmers.se>
//! Date: April 21, 2017


use chip;
use core::cell::Cell;
use kernel::common::take_cell::TakeCell;
use kernel::hil::symmetric_encryption::{SymmetricEncryptionDriver, Client};
use nvic;
use peripheral_interrupts::NvicIdx;
use peripheral_registers::{AESECB_REGS, AESECB_BASE};

// array that the AES-CHIP will mutate during AES-ECB
// key 0-15     cleartext 16-32     ciphertext 32-47
static mut ECB_DATA: [u8; 48] = [0; 48];

// data to replace TakeCell initial counter in the capsule
static mut INIT_CTR: [u8; 16] = [0; 16];

pub struct AesECB {
    regs: *mut AESECB_REGS,
    client: Cell<Option<&'static Client>>,
    ctr: Cell<[u8; 16]>,
    // input either plaintext or ciphertext to be encrypted or decrypted
    input: TakeCell<'static, [u8]>,
    // keystream to be XOR:ed with the input
    keystream: Cell<[u8; 128]>,
    remaining: Cell<usize>,
    len: Cell<usize>,
    offset: Cell<usize>,
}

pub static mut AESECB: AesECB = AesECB::new();

impl AesECB {
    const fn new() -> AesECB {
        AesECB {
            regs: AESECB_BASE as *mut AESECB_REGS,
            client: Cell::new(None),
            ctr: Cell::new([0; 16]),
            input: TakeCell::empty(),
            keystream: Cell::new([0; 128]),
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

        self.client
            .get()
            .map(|client| unsafe { client.set_key_done(&mut INIT_CTR[0..16]) });
    }

    pub fn handle_interrupt(&self) {
        let regs = unsafe { &*self.regs };

        // disable interrupts
        self.disable_nvic();
        self.disable_interrupts();
        nvic::clear_pending(NvicIdx::ECB);

        if regs.ENDECB.get() == 1 {

            let rem = self.remaining.get();
            let offset = self.offset.get();
            let take = if rem >= 16 { 16 } else { rem };
            let mut ks = self.keystream.get();

            // Append keystream to the KEYSTREAM array
            if take > 0 {
                for i in offset..offset + take {
                    ks[i] = unsafe { ECB_DATA[i - offset + 32] }
                }
                self.offset.set(offset + take);
                self.remaining.set(rem - take);
                self.update_ctr();
            }

            // More bytes to encrypt!!!
            if self.remaining.get() > 0 {
                self.crypt();
            }
            // Entire Keystream generate now XOR with the date
            else if self.input.is_some() {
                self.input
                    .take()
                    .map(|buf| {
                        for (i, c) in buf.as_mut()[0..self.len.get()].iter_mut().enumerate() {
                            *c = ks[i] ^ *c;
                        }
                        // ugly work-around to replace buffers in the capsule;
                        self.client
                            .get()
                            .map(move |client| unsafe {
                                     client.crypt_done(buf, &mut INIT_CTR, self.len.get())
                                 });
                    });

            }
            self.keystream.set(ks);
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
        self.remaining.set(len);
        self.len.set(len);
        self.offset.set(0);
        self.input.replace(data);
        self.set_initial_ctr(iv);
        self.crypt();
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn ECB_Handler() {
    use kernel::common::Queue;
    nvic::disable(NvicIdx::ECB);
    chip::INTERRUPT_QUEUE
        .as_mut()
        .unwrap()
        .enqueue(NvicIdx::ECB);
}
