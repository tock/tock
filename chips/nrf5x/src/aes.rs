//! aes128 driver, nRF5X-family
//!
//! Provides a simple driver for userspace applications to encrypt and decrypt
//! messages using aes128-ctr mode on top of aes128-ecb.
//!
//! Roughly, the module three buffers with the following content:
//!
//! * Key
//! * Initial counter
//! * Payload, to be encrypted or decrypted
//!
//! ### Key
//! The key is used for getting a key and configure it in the AES chip
//!
//! ### Initial Counter
//! Counter to be used for aes-ctr and it is entered into AES to generate the
//! the keystream. After each encryption the initial counter is incremented
//!
//! ### Payload
//! Data to be encrypted or decrypted it is XOR:ed with the generated keystream
//!
//! ### Things to highlight that can be improved:
//!
//! * ECB_DATA must be a static mut [u8] and can't be located in the struct
//! * PAYLOAD size is restricted to 128 bytes
//!
//! Authors
//! --------
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Fredrik Nilsson <frednils@student.chalmers.se>
//! * Date: April 21, 2017

use core::cell::Cell;
use kernel;
use kernel::common::take_cell::TakeCell;
use peripheral_registers;

// array that the aes chip will mutate during encryption
// Byte 0-15   - The key
// Byte 16-32  - Payload
// Byte 33-47  - Ciphertext
static mut ECB_DATA: [u8; 48] = [0; 48];
// data to replace TakeCell initial counter in the capsule
static mut INIT_CTR: [u8; 16] = [0; 16];

const NRF_INTR_ENDECB: u32 = 0;
const NRF_INTR_ERRORECB: u32 = 1;

pub struct AesECB {
    regs: *const peripheral_registers::AESECB_REGS,
    client: Cell<Option<&'static kernel::hil::symmetric_encryption::Client>>,
    ctr: Cell<[u8; 16]>,
    /// Input either plaintext or ciphertext to be encrypted or decrypted.
    input: TakeCell<'static, [u8]>,
    /// Keystream to be XOR'ed with the input.
    keystream: Cell<[u8; 128]>,
    remaining: Cell<usize>,
    len: Cell<usize>,
    offset: Cell<usize>,
}

pub static mut AESECB: AesECB = AesECB::new();

impl AesECB {
    const fn new() -> AesECB {
        AesECB {
            regs: peripheral_registers::AESECB_BASE as *const peripheral_registers::AESECB_REGS,
            client: Cell::new(None),
            ctr: Cell::new([0; 16]),
            input: TakeCell::empty(),
            keystream: Cell::new([0; 128]),
            remaining: Cell::new(0),
            len: Cell::new(0),
            offset: Cell::new(0),
        }
    }

    // This Function is called once Tock is booted
    pub fn ecb_init(&self) {
        let regs = unsafe { &*self.regs };
        unsafe {
            regs.ecbdataptr.set((&ECB_DATA as *const u8) as u32);
        }
    }

    // FIXME: should this be performed in constant time i.e. skip the break part
    // and always loop 16 times?
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

        regs.event_endecb.set(0);
        regs.task_startecb.set(1);

        self.enable_interrupts();
    }


    pub fn handle_interrupt(&self) {
        let regs = unsafe { &*self.regs };

        // disable interrupts
        self.disable_interrupts();

        if regs.event_endecb.get() == 1 {

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
            // Entire keystream generated now XOR with the data
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
        // FIXME: else ERROR encrypt error do nothing
    }

    fn enable_interrupts(&self) {
        // set ENDECB bit and ERROR bit
        let regs = unsafe { &*self.regs };
        regs.intenset.set(NRF_INTR_ENDECB | NRF_INTR_ERRORECB);
    }

    fn disable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenclr.set(NRF_INTR_ENDECB | NRF_INTR_ERRORECB);
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

impl kernel::hil::symmetric_encryption::SymmetricEncryption for AesECB {
    fn set_client(&self, client: &'static kernel::hil::symmetric_encryption::Client) {
        self.client.set(Some(client));
    }

    fn init(&self) {}

    // capsule ensures that the key is 16 bytes
    // precondition: key_len = 16 || 24 || 32
    fn set_key(&self, key: &'static mut [u8], len: usize) -> &'static mut [u8] {
        for (i, c) in key.as_ref()[0..len].iter().enumerate() {
            unsafe {
                ECB_DATA[i] = *c;
            }
        }
        key
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
