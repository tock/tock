//! AES kernel abstraction layer
//!
//! Generic collection of traits, representing possible combinations of
//! AES key sizes and modi.
//!
//! In general, the AES key size and mode can be viewed seperately.
//! Therefore, the following keysize traits exist:
//!
//! - AESEngine (supporting 128, 192 and 256bit keys,
//!              automatically implementing AES128Engine + AES192Engine +
//!              AES256Engine)
//! - AES128Engine (supporting 128bit keys)
//! - AES192Engine (supporting 192bit keys)
//! - AES256Engine (supporting 256bit keys)
//!
//! For the modi, a simple AESBlockEngine trait for standalone block
//! operations, as well as specific mode traits exist:
//!
//! - AESBlockEngine, AESBlockClient
//! - AESECBMode, AESECBClient
//!
//!
//! Authors
//! -------------------
//! * Leon Schuermann <leon@is.currently.online>
//! * March 29, 2019

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::ReturnCode;

pub enum AESImplementationType {
    /// Implementation of the primitives and mode fully done in
    /// hardware
    Hardware,
    /// Some part of the implementation uses hardware acceleration,
    /// while other parts are handled in software
    HardwareAccelerated,
    /// Implementation done purely in software
    Software,
}

/// Set the AES operation direction
#[derive(Debug, Clone, Copy)]
pub enum AESOperation {
    Encrypt,
    Decrypt,
}

pub const AES_WORDSIZE: usize = 4;
pub const AES_BLOCKSIZE: usize = 16;
pub const AES_WORDS_IN_BLOCK: usize = AES_BLOCKSIZE / AES_WORDSIZE;
pub const AES_ROUND_KEYSIZE: usize = AES_BLOCKSIZE;

pub const AES_128_KEYSIZE: usize = 16;
pub const AES_192_KEYSIZE: usize = 24;
pub const AES_256_KEYSIZE: usize = 32;

pub const AES_128_ROUNDS: usize = 10;
pub const AES_128_EXPANDED_KEYS: usize = AES_128_ROUNDS + 1;
pub const AES_192_ROUNDS: usize = 12;
pub const AES_192_EXPANDED_KEYS: usize = AES_192_ROUNDS + 1;
pub const AES_256_ROUNDS: usize = 14;
pub const AES_256_EXPANDED_KEYS: usize = AES_256_ROUNDS + 1;

pub type AESWord = [u8; AES_WORDSIZE];
pub type AESBlock = [u8; AES_BLOCKSIZE];

pub type AES128Key = [u8; AES_128_KEYSIZE];
pub type AES192Key = [u8; AES_192_KEYSIZE];
pub type AES256Key = [u8; AES_256_KEYSIZE];

pub fn xor_blocks(a: &mut AESBlock, b: &AESBlock) {
    a.iter_mut()
        .zip(b.iter())
        .for_each(|(byte_a, byte_b)| *byte_a = *byte_a ^ *byte_b);
}

/// An enumeration over all possible key lengths
///
/// This is as large as the longest possible key, plus the enum
/// variant indicator
pub enum AESKey {
    K128(AES128Key),
    K192(AES192Key),
    K256(AES256Key),
}

impl AESKey {
    /// Retrieve the AES key as an array of bytes
    pub fn raw_key(&self) -> &[u8] {
        match self {
            &AESKey::K128(ref k) => k,
            &AESKey::K192(ref k) => k,
            &AESKey::K256(ref k) => k,
        }
    }

    /// Retrieve the key length in bytes
    pub fn key_size(&self) -> usize {
        match self {
            &AESKey::K128(k) => k.len(),
            &AESKey::K192(k) => k.len(),
            &AESKey::K256(k) => k.len(),
        }
    }

    /// Retrieve the key length in words
    pub fn key_words(&self) -> usize {
        self.key_size() / AES_WORDSIZE
    }

    /// Retrieve the required AES rounds depending on the key size
    pub fn rounds(&self) -> usize {
        match self {
            &AESKey::K128(_k) => AES_128_ROUNDS,
            &AESKey::K192(_k) => AES_192_ROUNDS,
            &AESKey::K256(_k) => AES_256_ROUNDS,
        }
    }
}

// ----- AES ENGINE (KEY) TRAITS -----

/// An AES engine which supports all possible key sizes (128/192/256)
///
/// Implementing this trait will also implement the
/// - AES128Engine
/// - AES192Engine
/// - AES256Engine
/// traits
pub trait AESEngine<'a> {
    fn expand_key(&self, key: &AESKey) -> Result<(), ReturnCode>;
    fn invalidate_key(&self) -> Result<(), ReturnCode>;
    fn set_client(&'a self, client: &'a dyn AESClient);
}

/// An AES engine which supports 128bit keys
pub trait AES128Engine<'a> {
    fn expand_key(&self, key: &AES128Key) -> Result<(), ReturnCode>;
    fn invalidate_key(&self) -> Result<(), ReturnCode>;
    fn set_client(&'a self, client: &'a dyn AESClient);
}

/// An AES engine which supports 192bit keys
pub trait AES192Engine<'a> {
    fn expand_key(&self, key: &AES192Key) -> Result<(), ReturnCode>;
    fn invalidate_key(&self) -> Result<(), ReturnCode>;
    fn set_client(&'a self, client: &'a dyn AESClient);
}

/// An AES engine which supports 256bit keys
pub trait AES256Engine<'a> {
    fn expand_key(&self, key: &AES256Key) -> Result<(), ReturnCode>;
    fn invalidate_key(&self) -> Result<(), ReturnCode>;
    fn set_client(&'a self, client: &'a dyn AESClient);
}

/// Client for an AES engine
///
/// This must be registered with the AES engine before any of the
/// following methods is called:
/// - expand_key()
/// - invalidate_key()
///
/// The operations are only finished when the respective client method
/// has been called
pub trait AESClient {
    fn expanded_key_ready(&self);
    fn key_invalidated(&self);
}

// Automatically implement an AES128Engine for an AESEngine
impl<'a, T> AES128Engine<'a> for T
where
    T: AESEngine<'a>,
{
    fn expand_key(&self, key: &AES128Key) -> Result<(), ReturnCode> {
        AESEngine::expand_key(self, &AESKey::K128(*key))
    }

    fn invalidate_key(&self) -> Result<(), ReturnCode> {
        AESEngine::invalidate_key(self)
    }

    fn set_client(&'a self, client: &'a dyn AESClient) {
        AESEngine::set_client(self, client)
    }
}

// Automatically implement an AES192Engine for an AESEngine
impl<T> AES192Engine<'a> for T
where
    T: AESEngine<'a>,
{
    fn expand_key(&self, key: &AES192Key) -> Result<(), ReturnCode> {
        AESEngine::expand_key(self, &AESKey::K192(*key))
    }

    fn invalidate_key(&self) -> Result<(), ReturnCode> {
        AESEngine::invalidate_key(self)
    }

    fn set_client(&'a self, client: &'a dyn AESClient) {
        AESEngine::set_client(self, client)
    }
}

// Automatically implement an AES256Engine for an AESEngine
impl<T> AES256Engine<'a> for T
where
    T: AESEngine<'a>,
{
    fn expand_key(&self, key: &AES256Key) -> Result<(), ReturnCode> {
        AESEngine::expand_key(self, &AESKey::K256(*key))
    }

    fn invalidate_key(&self) -> Result<(), ReturnCode> {
        AESEngine::invalidate_key(self)
    }

    fn set_client(&'a self, client: &'a dyn AESClient) {
        AESEngine::set_client(self, client)
    }
}

// ----- AES BLOCK ENGINE -----

/// Client for the AESBlockEngine
pub trait AESBlockClient {
    fn block_ready(&self, block: [u8; 16]);
}

/// AES engine supporting a basic single-block operation
///
/// This is a particularly simple implementation of the AES algorithm,
/// which supports processing only a single block at a time. This
/// performs the raw AES algorithm, like ECB on a single block.
///
/// This trait is useful for software AES implementations or very
/// simple hardware accelerators (like special AES block instructions)
///
/// All other AES modes can be derived from this trait
pub trait AESBlockEngine<'a> {
    fn set_client(&'a self, client: &'a dyn AESBlockClient);
    fn get_implementation_type(&self) -> AESImplementationType;

    fn encrypt(&self, src: &AESBlock) -> Result<(), ReturnCode>;
    fn decrypt(&self, src: &AESBlock) -> Result<(), ReturnCode>;
}

// ----- AES MODES -----

/// Client for the AESECBMode
pub trait AESECBClient<'buffer> {
    fn buffer_ready(&self, data: &'buffer mut [u8], processed_bytes: usize);
}

/// AES engine supporting *Electronic Code Book* mode encryption and decryption
///
/// The AESECBMode works similarly to the AESBlockClient, however it
/// can queue Blocks to be processed. This is useful for
/// implementations employing DMA or similar techniques.
pub trait AESECBMode<'a, 'buffer> {
    fn get_implementation_type(&self) -> AESImplementationType;
    fn set_client(&'a self, client: &'a dyn AESECBClient<'buffer>);
    fn set_operation(&self, mode: AESOperation);

    fn input_buffer(&self, data: &'buffer mut [u8]) -> Result<(), ReturnCode>;
}

/// Client for the AESCBCMODE
pub trait AESCBCClient<'buffer> {
    fn iv_set(&self);
    fn buffer_ready(&self, data: &'buffer mut [u8], processed_bytes: usize);
}

/// AES engine supporting *Cipher Block Chaining* mode encryption and
/// decryption
///
/// The AESCBCMode takes an initialization vector and reuses xors
/// either that or the last ciphertext with the current plaintext to
/// make the plaintext depend on the previous ciphertext.
pub trait AESCBCMode<'a, 'buffer> {
    fn get_implementation_type(&self) -> AESImplementationType;
    fn set_client(&self, client: &'a dyn AESCBCClient<'buffer>);
    fn set_operation(&self, mode: AESOperation);
    fn set_iv(&self, initialization_vector: Option<&AESBlock>) -> bool;

    fn input_buffer(&self, block: &'buffer mut [u8]) -> Result<(), ReturnCode>;
}

// ----- SOFTWARE MODE IMPLEMENTATIONS -----

/// A software implementation of the AESECBMode for an AESBlockEngine
pub struct SoftAESECB<'a, 'buffer, T: 'a>
where
    T: AESBlockEngine<'a>,
{
    mode: Cell<AESOperation>,
    aes_block_engine: &'a T,
    client: OptionalCell<&'a dyn AESECBClient<'buffer>>,
    client_buffer: TakeCell<'buffer, [u8]>,
}

impl<'a, 'buffer, T: 'a> SoftAESECB<'a, 'buffer, T>
where
    T: AESBlockEngine<'a>,
{
    pub fn new(block_engine: &'a T) -> Self {
        SoftAESECB {
            mode: Cell::new(AESOperation::Encrypt), // Dummy value
            aes_block_engine: block_engine,
            client: OptionalCell::empty(),
            client_buffer: TakeCell::empty(),
        }
    }
}

impl<'a, 'buffer, T: 'a> AESECBMode<'a, 'buffer> for SoftAESECB<'a, 'buffer, T>
where
    T: AESBlockEngine<'a>,
{
    fn get_implementation_type(&self) -> AESImplementationType {
        use AESImplementationType as AIT;

        match self.aes_block_engine.get_implementation_type() {
            AIT::Hardware => AIT::HardwareAccelerated,
            AIT::HardwareAccelerated => AIT::HardwareAccelerated,
            AIT::Software => AIT::Software,
        }
    }

    fn set_operation(&self, mode: AESOperation) {
        self.mode.set(mode);
    }

    fn input_buffer(&self, data: &'buffer mut [u8]) -> Result<(), ReturnCode> {
        if data.len() < AES_BLOCKSIZE {
            // We don't support half blocks in CBC
            return Err(ReturnCode::EINVAL);
        }

        if self.client_buffer.is_some() {
            // Already running an operation
            return Err(ReturnCode::EBUSY);
        }

        // Missing slice_as_array support for [0..16] to work
        let data_block: [u8; AES_BLOCKSIZE] = [
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
            data[9], data[10], data[11], data[12], data[13], data[14], data[15],
        ];

        match self.mode.get() {
            AESOperation::Encrypt => self.aes_block_engine.encrypt(&data_block),
            AESOperation::Decrypt => self.aes_block_engine.decrypt(&data_block),
        }?;

        self.client_buffer.replace(data);

        Ok(())
    }

    fn set_client(&'a self, client: &'a dyn AESECBClient<'buffer>) {
        self.client.set(client);
    }
}

impl<'a, 'buffer, T: 'a> AESBlockClient for SoftAESECB<'a, 'buffer, T>
where
    T: AESBlockEngine<'a>,
{
    fn block_ready(&self, block: [u8; AES_BLOCKSIZE]) {
        let data = self
            .client_buffer
            .take()
            .expect("client buffer not present");

        data.iter_mut()
            .zip(block.iter())
            .for_each(|(dst, src)| *dst = *src);

        self.client
            .map(move |c| c.buffer_ready(data, AES_BLOCKSIZE));
    }
}

/// A software implementation of the AESCBCMode for an AESBlockEngine
pub struct SoftAESCBC<'a, 'buffer, T: 'a>
where
    T: AESBlockEngine<'a>,
{
    mode: Cell<AESOperation>,
    aes_block_engine: &'a T,
    client: OptionalCell<&'a dyn AESCBCClient<'buffer>>,
    feedback0: OptionalCell<AESBlock>,
    feedback1: OptionalCell<AESBlock>,
    client_buffer: TakeCell<'buffer, [u8]>,
}

impl<'a, 'buffer, T: 'a> SoftAESCBC<'a, 'buffer, T>
where
    T: AESBlockEngine<'a>,
{
    pub fn new(block_engine: &'a T) -> Self {
        SoftAESCBC {
            mode: Cell::new(AESOperation::Encrypt), // Dummy value
            aes_block_engine: block_engine,
            client: OptionalCell::empty(),
            feedback0: OptionalCell::empty(),
            feedback1: OptionalCell::empty(),
            client_buffer: TakeCell::empty(),
        }
    }
}

impl<'a, 'buffer, T: 'a> AESCBCMode<'a, 'buffer> for SoftAESCBC<'a, 'buffer, T>
where
    T: AESBlockEngine<'a>,
{
    fn get_implementation_type(&self) -> AESImplementationType {
        use AESImplementationType as AIT;

        match self.aes_block_engine.get_implementation_type() {
            AIT::Hardware => AIT::HardwareAccelerated,
            AIT::HardwareAccelerated => AIT::HardwareAccelerated,
            AIT::Software => AIT::Software,
        }
    }

    fn set_operation(&self, mode: AESOperation) {
        self.mode.set(mode);

        // Since encryption and decryption use feedback buffers
        // differently, clear them and require a reset of the IV
        self.feedback0.clear();
        self.feedback1.clear();
    }

    fn set_iv(&self, initialization_vector: Option<&AESBlock>) -> bool {
        if let Some(iv) = initialization_vector {
            // Set IV
            self.feedback0.set(*iv);
        } else {
            // Clear IV
            self.feedback0.clear();
            self.feedback1.clear();
        }

        // We don't require waiting for a callback
        false
    }

    fn input_buffer(&self, data: &'buffer mut [u8]) -> Result<(), ReturnCode> {
        if data.len() < AES_BLOCKSIZE {
            // We don't support half blocks in CBC
            return Err(ReturnCode::EINVAL);
        }

        if self.client_buffer.is_some() {
            // Already running an operation
            return Err(ReturnCode::EBUSY);
        }

        // Missing slice_as_array support for [0..16] to work
        let mut data_block: [u8; AES_BLOCKSIZE] = [
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
            data[9], data[10], data[11], data[12], data[13], data[14], data[15],
        ];

        if let AESOperation::Encrypt = self.mode.get() {
            // Encryption, so we only need one feedback block (feedback0)
            // XOR it with the data
            let enc_feedback_block = self.feedback0.take().expect("feedback block not present");
            xor_blocks(&mut data_block, &enc_feedback_block);
        } else {
            // Decryption, so store the current ciphertext as
            // feedback1 to be applied in the next round
            self.feedback1.replace(data_block);
        }

        // Store the client buffer for the callback
        self.client_buffer.replace(data);

        match self.mode.get() {
            AESOperation::Encrypt => self.aes_block_engine.encrypt(&data_block),
            AESOperation::Decrypt => self.aes_block_engine.decrypt(&data_block),
        }?;

        Ok(())
    }

    fn set_client(&self, client: &'a dyn AESCBCClient<'buffer>) {
        self.client.set(client);
    }
}

impl<'a, 'buffer, T: 'a> AESBlockClient for SoftAESCBC<'a, 'buffer, T>
where
    T: AESBlockEngine<'a>,
{
    fn block_ready(&self, mut block: [u8; AES_BLOCKSIZE]) {
        let data = self
            .client_buffer
            .take()
            .expect("client buffer not present");

        if let AESOperation::Decrypt = self.mode.get() {
            // XOR feedback0 to the result (is the initialization
            // vector in the first round), then move feedback1 to
            // feedback0
            let current_feedback = self.feedback0.expect("cbc decrypt feedback not present");
            self.feedback0.insert(self.feedback1.take());

            xor_blocks(&mut block, &current_feedback);
        } else {
            // Use the block as feedback for the next operation
            self.feedback0.set(block);
        }

        // Copy the processed data back to the buffer
        data.iter_mut()
            .zip(block.iter())
            .for_each(|(dst, src)| *dst = *src);

        // We only processed one block, so report it that way
        self.client
            .map(move |c| c.buffer_ready(data, AES_BLOCKSIZE));
    }
}
