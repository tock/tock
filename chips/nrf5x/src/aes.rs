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
use kernel::ReturnCode;
use kernel::common::take_cell::TakeCell;
use kernel::hil::symmetric_encryption;
use peripheral_registers;


// DMA buffer that the aes chip will mutate during encryption
// Byte 0-15   - Key
// Byte 16-32  - Payload
// Byte 33-47  - Ciphertext
static mut ECB_DATA: [u8; 48] = [0; 48];

#[allow(dead_code)]
const KEY_START: usize = 0;
#[allow(dead_code)]
const KEY_END: usize = 15;
#[allow(dead_code)]
const PLAINTEXT_START: usize = 16;
#[allow(dead_code)]
const PLAINTEXT_END: usize = 32;
#[allow(dead_code)]
const CIPHERTEXT_START: usize = 33;
#[allow(dead_code)]
const CIPHERTEXT_END: usize = 47;
#[allow(dead_code)]
const MAX_LENGTH: usize = 128;

const NRF_INTR_ENDECB: u32 = 0;
const NRF_INTR_ERRORECB: u32 = 1;

pub struct AesECB<'a> {
    regs: *const peripheral_registers::AESECB_REGS,
    client: Cell<Option<&'a kernel::hil::symmetric_encryption::Client<'a>>>,
    /// Input either plaintext or ciphertext to be encrypted or decrypted.
    input: TakeCell<'a, [u8]>,
    output: TakeCell<'a, [u8]>,
    /// Keystream to be XOR'ed with the input.
    keystream: Cell<[u8; MAX_LENGTH]>,
    current_idx: Cell<usize>,
    start_idx: Cell<usize>,
    end_idx: Cell<usize>,
}

pub static mut AESECB: AesECB = AesECB::new();

impl<'a> AesECB<'a> {
    const fn new() -> AesECB<'a> {
        AesECB {
            regs: peripheral_registers::AESECB_BASE as *const peripheral_registers::AESECB_REGS,
            client: Cell::new(None),
            input: TakeCell::empty(),
            output: TakeCell::empty(),
            keystream: Cell::new([0; MAX_LENGTH]),
            current_idx: Cell::new(0),
            start_idx: Cell::new(0),
            end_idx: Cell::new(0),
        }
    }

    fn init_dma(&self) {
        let regs = unsafe { &*self.regs };
        unsafe {
            regs.ecbdataptr.set((&ECB_DATA as *const u8) as u32);
        }
    }

    // FIXME: should this be performed in constant time i.e. skip the break part
    // and always loop 16 times?
    fn update_ctr(&self) {
        for i in (PLAINTEXT_START..PLAINTEXT_END).rev() {
            unsafe {
                ECB_DATA[i] += 1;
                if ECB_DATA[i] != 0 {
                    break;
                }
            }
        }
    }

    fn crypt(&self) {
        let regs = unsafe { &*self.regs };

        regs.event_endecb.set(0);
        regs.task_startecb.set(1);

        self.enable_interrupts();
    }

    pub fn handle_interrupt(&self) {
        let regs = unsafe { &*self.regs };

        // disable interrupts
        self.disable_interrupts();

        if regs.event_endecb.get() == 1 {

            let current_idx = self.current_idx.get();
            let end_idx = self.end_idx.get();
            
            // get number of bytes to be used in the keystream/block
            let take = match end_idx.checked_sub(current_idx) {
                Some(v) if v > 16 => 16,
                Some(v) => v,
                None => 0,
            };

            let mut ks = self.keystream.get();

            // Append keystream to the KEYSTREAM array
            if take > 0 {
                for i in current_idx..current_idx + take {
                    ks[i] = unsafe { ECB_DATA[i - current_idx + PLAINTEXT_END] }
                }
                self.current_idx.set(current_idx + take);
                self.update_ctr();
            }

            // More bytes to encrypt!!!
            if self.current_idx.get() < self.end_idx.get() {
                self.crypt();
            }
            
            // Entire keystream generated we are done!
            // XOR keystream the input
            else if self.input.is_some() && self.output.is_some() {
                self.input.take().map(|slice| {
                    self.output.take().map(|buf| {

                        let start = self.start_idx.get();
                        let end = self.end_idx.get();
                        let len = end - start;

                        for ((i,out), inp) in buf.as_mut()[start..end]
                            .iter_mut().enumerate()
                            .zip(slice.as_ref()[0..len].iter())
                        {
                            *out = ks[i] ^ *inp;
                        }

                        self.client.get().map(move |client| {
                            client.crypt_done(Some(slice), buf)
                        });

                    });

                });
            }
            // FIXME: else ERROR encrypt error do nothing
            else {
                debug!("error empty TakeCell");
            }

            self.keystream.set(ks);
        }
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
}

impl<'a> kernel::hil::symmetric_encryption::AES128<'a> for AesECB<'a> {
    fn enable(&self) {
        self.init_dma();
    }

    fn disable(&self) {
        let regs = unsafe { &*self.regs };
        regs.task_stopecb.set(1);
        self.disable_interrupts();
    }

    fn set_client(&'a self, client: &'a symmetric_encryption::Client<'a>) {
        self.client.set(Some(client));
    }

    fn set_key(&self, key: &[u8]) -> ReturnCode {
        if key.len() != 16 {
            ReturnCode::EINVAL
        } else {
            for (i, c) in key.iter().enumerate() {
                unsafe {
                    ECB_DATA[i] = *c;
                }
            }
            ReturnCode::SUCCESS
        }
    }

    fn set_iv(&self, iv: &[u8]) -> ReturnCode {
        if iv.len() != 16 {
            ReturnCode::EINVAL
        } else {
            for (i, c) in iv.iter().enumerate() {
                unsafe {
                    ECB_DATA[i + 16] = *c;
                }
            }
            ReturnCode::SUCCESS
        }
    }

    // not needed by NRF5x
    fn start_message(&self) {
        ()
    }

    // start_index and stop_index not used!!!
    // assuming that
    fn crypt(
        &'a self,
        source: Option<&'a mut [u8]>,
        dest: &'a mut [u8],
        start_index: usize,
        stop_index: usize,
    ) -> Option<(ReturnCode, Option<&'a mut [u8]>, &'a mut [u8])> {

        match source {
            None => Some((ReturnCode::EINVAL, source, dest)),
            Some(src) => {
                if stop_index - start_index <= MAX_LENGTH {
                    // replace buffers
                    self.input.replace(src);
                    self.output.replace(dest);
                    
                    // configure buffer offsets
                    self.current_idx.set(0);
                    self.start_idx.set(start_index);
                    self.end_idx.set(stop_index);
                    
                    // start crypt
                    self.crypt();
                    None
                } else {
                    Some((ReturnCode::ESIZE, Some(src), dest))
                }
            }
        }
    }
}

impl<'a> kernel::hil::symmetric_encryption::AES128Ctr for AesECB<'a> {
    // not needed by NRF5x (the configuration is the same for encryption and decryption)
    fn set_mode_aes128ctr(&self, _encrypting: bool) {
        ()
    }
}

// FIXME: implemented inorder to avoid modify the trait bounds on capsules/test/aes.rs
impl<'a> kernel::hil::symmetric_encryption::AES128CBC for AesECB<'a> {
    // the mode is not supported and will be a runtime error
    fn set_mode_aes128cbc(&self, _encrypting: bool) {
        unimplemented!()
    }
}
