//! AES128 driver, nRF5X-family
//!
//! Provides a simple driverto encrypt and decrypt
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
//! * ECB_DATA must be a static mut \[u8\] and can't be located in the struct
//! * PAYLOAD size is restricted to 128 bytes
//!
//! Authors
//! --------
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Fredrik Nilsson <frednils@student.chalmers.se>
//! * Date: April 21, 2017

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::registers::{register_bitfields, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::symmetric_encryption;
use kernel::ReturnCode;

// DMA buffer that the aes chip will mutate during encryption
// Byte 0-15   - Key
// Byte 16-32  - Payload
// Byte 33-47  - Ciphertext
static mut ECB_DATA: [u8; 48] = [0; 48];

#[allow(dead_code)]
const KEY_START: usize = 0;
#[allow(dead_code)]
const KEY_END: usize = 15;
const PLAINTEXT_START: usize = 16;
const PLAINTEXT_END: usize = 32;
#[allow(dead_code)]
const CIPHERTEXT_START: usize = 33;
#[allow(dead_code)]
const CIPHERTEXT_END: usize = 47;
const MAX_LENGTH: usize = 128;

const AESECB_BASE: StaticRef<AesEcbRegisters> =
    unsafe { StaticRef::new(0x4000E000 as *const AesEcbRegisters) };

#[repr(C)]
struct AesEcbRegisters {
    /// Start ECB block encrypt
    /// - Address 0x000 - 0x004
    task_startecb: WriteOnly<u32, Task::Register>,
    /// Abort a possible executing ECB operation
    /// - Address: 0x004 - 0x008
    task_stopecb: WriteOnly<u32, Task::Register>,
    /// Reserved
    _reserved1: [u32; 62],
    /// ECB block encrypt complete
    /// - Address: 0x100 - 0x104
    event_endecb: ReadWrite<u32, Event::Register>,
    /// ECB block encrypt aborted because of a STOPECB task or due to an error
    /// - Address: 0x104 - 0x108
    event_errorecb: ReadWrite<u32, Event::Register>,
    /// Reserved
    _reserved2: [u32; 127],
    /// Enable interrupt
    /// - Address: 0x304 - 0x308
    intenset: ReadWrite<u32, Intenset::Register>,
    /// Disable interrupt
    /// - Address: 0x308 - 0x30c
    intenclr: ReadWrite<u32, Intenclr::Register>,
    /// Reserved
    _reserved3: [u32; 126],
    /// ECB block encrypt memory pointers
    /// - Address: 0x504 - 0x508
    ecbdataptr: ReadWrite<u32, EcbDataPointer::Register>,
}

register_bitfields! [u32,
    /// Start task
    Task [
        ENABLE OFFSET(0) NUMBITS(1)
    ],

    /// Read event
    Event [
        READY OFFSET(0) NUMBITS(1)
    ],

    /// Enabled interrupt
    Intenset [
        ENDECB OFFSET(0) NUMBITS(1),
        ERRORECB OFFSET(1) NUMBITS(1)
    ],

    /// Disable interrupt
    Intenclr [
        ENDECB OFFSET(0) NUMBITS(1),
        ERRORECB OFFSET(1) NUMBITS(1)
    ],

    /// ECB block encrypt memory pointers
    EcbDataPointer [
        POINTER OFFSET(0) NUMBITS(32)
    ]
];

pub struct AesECB<'a> {
    registers: StaticRef<AesEcbRegisters>,
    client: OptionalCell<&'a dyn kernel::hil::symmetric_encryption::Client<'a>>,
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

impl AesECB<'a> {
    const fn new() -> AesECB<'a> {
        AesECB {
            registers: AESECB_BASE,
            client: OptionalCell::empty(),
            input: TakeCell::empty(),
            output: TakeCell::empty(),
            keystream: Cell::new([0; MAX_LENGTH]),
            current_idx: Cell::new(0),
            start_idx: Cell::new(0),
            end_idx: Cell::new(0),
        }
    }

    fn set_dma(&self) {
        let regs = &*self.registers;
        unsafe {
            regs.ecbdataptr.set(ECB_DATA.as_ptr() as u32);
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
        let regs = &*self.registers;

        regs.event_endecb.write(Event::READY::CLEAR);
        regs.task_startecb.set(1);

        self.enable_interrupts();
    }

    /// AesEcb Interrupt handler
    pub fn handle_interrupt(&self) {
        let regs = &*self.registers;

        // disable interrupts
        self.disable_interrupts();

        if regs.event_endecb.get() == 1 {
            let current_idx = self.current_idx.get();
            let end_idx = self.end_idx.get();

            // Get the number of bytes to be used in the keystream/block
            let take = match end_idx.checked_sub(current_idx) {
                Some(v) if v > symmetric_encryption::AES128_BLOCK_SIZE => {
                    symmetric_encryption::AES128_BLOCK_SIZE
                }
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

                        for ((i, out), inp) in buf.as_mut()[start..end]
                            .iter_mut()
                            .enumerate()
                            .zip(slice.as_ref()[0..len].iter())
                        {
                            *out = ks[i] ^ *inp;
                        }

                        self.client
                            .map(move |client| client.crypt_done(Some(slice), buf));
                    });
                });
            }

            self.keystream.set(ks);
        }
    }

    fn enable_interrupts(&self) {
        let regs = &*self.registers;
        regs.intenset
            .write(Intenset::ENDECB::SET + Intenset::ERRORECB::SET);
    }

    fn disable_interrupts(&self) {
        let regs = &*self.registers;
        regs.intenclr
            .write(Intenclr::ENDECB::SET + Intenclr::ERRORECB::SET);
    }
}

impl kernel::hil::symmetric_encryption::AES128<'a> for AesECB<'a> {
    fn enable(&self) {
        self.set_dma();
    }

    fn disable(&self) {
        let regs = &*self.registers;
        regs.task_stopecb.write(Task::ENABLE::CLEAR);
        self.disable_interrupts();
    }

    fn set_client(&'a self, client: &'a dyn symmetric_encryption::Client<'a>) {
        self.client.set(client);
    }

    fn set_key(&self, key: &[u8]) -> ReturnCode {
        if key.len() != symmetric_encryption::AES128_KEY_SIZE {
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
        if iv.len() != symmetric_encryption::AES128_BLOCK_SIZE {
            ReturnCode::EINVAL
        } else {
            for (i, c) in iv.iter().enumerate() {
                unsafe {
                    ECB_DATA[i + PLAINTEXT_START] = *c;
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

impl kernel::hil::symmetric_encryption::AES128Ctr for AesECB<'a> {
    // not needed by NRF5x (the configuration is the same for encryption and decryption)
    fn set_mode_aes128ctr(&self, _encrypting: bool) {
        ()
    }
}

impl kernel::hil::symmetric_encryption::AES128CBC for AesECB<'a> {
    fn set_mode_aes128cbc(&self, _encrypting: bool) {
        ()
    }
}
//TODO: replace this placeholder with a proper implementation of the AES system
impl kernel::hil::symmetric_encryption::AES128CCM<'a> for AesECB<'a> {
    /// Set the client instance which will receive `crypt_done()` callbacks
    fn set_client(&'a self, _client: &'a dyn kernel::hil::symmetric_encryption::CCMClient) {}

    /// Set the key to be used for CCM encryption
    fn set_key(&self, _key: &[u8]) -> ReturnCode {
        ReturnCode::SUCCESS
    }

    /// Set the nonce (length NONCE_LENGTH) to be used for CCM encryption
    fn set_nonce(&self, _nonce: &[u8]) -> ReturnCode {
        ReturnCode::SUCCESS
    }

    /// Try to begin the encryption/decryption process
    fn crypt(
        &self,
        _buf: &'static mut [u8],
        _a_off: usize,
        _m_off: usize,
        _m_len: usize,
        _mic_len: usize,
        _confidential: bool,
        _encrypting: bool,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        (ReturnCode::SUCCESS, None)
    }
}
