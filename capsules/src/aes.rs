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
//! * Leon Schuermann <leon.git@is.currently.online>
//! * March 29, 2019

use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::ReturnCode;

/// Set the AES operation direction
#[derive(Debug, Clone, Copy)]
pub enum AESOperation {
    Encrypt,
    Decrypt,
}

/// Indicator whether the AES implementation can process more data
/// immediately or the client has to wait
#[derive(Debug, Clone, Copy)]
pub enum Continue {
    More,
    Stop,
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

    fn encrypt(&self, src: &AESBlock) -> Result<(), ReturnCode>;
    fn decrypt(&self, src: &AESBlock) -> Result<(), ReturnCode>;
}

// ----- AES MODES -----

/// Client for the AESECBMode
pub trait AESECBClient {
    fn data_available(&self, data: &[u8], cont: Continue);
    fn more_data(&self);
}

/// AES engine supporting *Electronic Code Book* mode encryption and decryption
///
/// The AESECBMode works similarly to the AESBlockClient, however it
/// can queue Blocks to be processed. This is useful for
/// implementations employing DMA or similar techniques.
pub trait AESECBMode<'a> {
    fn set_operation(&self, mode: AESOperation);

    fn input_block(&self, block: &AESBlock) -> Result<Continue, ReturnCode>;

    fn set_client(&'a self, client: &'a dyn AESECBClient);
}

// ----- SOFTWARE MODE IMPLEMENTATIONS -----

/// A software implementation of the AESECBMode for an AESBlockEngine
pub struct SoftAESECB<'a, T: 'a>
where
    T: AESBlockEngine<'a>,
{
    mode: Cell<AESOperation>,
    aes_block_engine: &'a T,
    client: OptionalCell<&'a dyn AESECBClient>,
}

impl<'a, T: 'a> SoftAESECB<'a, T>
where
    T: AESBlockEngine<'a>,
{
    pub fn new(block_engine: &'a T) -> Self {
        SoftAESECB {
            mode: Cell::new(AESOperation::Encrypt), // Dummy value
            aes_block_engine: block_engine,
            client: OptionalCell::empty(),
        }
    }
}

impl<'a, T: 'a> AESECBMode<'a> for SoftAESECB<'a, T>
where
    T: AESBlockEngine<'a>,
{
    fn set_operation(&self, mode: AESOperation) {
        self.mode.set(mode);
    }

    fn input_block(&self, block: &AESBlock) -> Result<Continue, ReturnCode> {
        match self.mode.get() {
            AESOperation::Encrypt => self.aes_block_engine.encrypt(block),
            AESOperation::Decrypt => self.aes_block_engine.decrypt(block),
        }?;

        Ok(Continue::Stop)
    }

    fn set_client(&'a self, client: &'a dyn AESECBClient) {
        self.client.set(client);
    }
}

impl<'a, T: 'a> AESBlockClient for SoftAESECB<'a, T>
where
    T: AESBlockEngine<'a>,
{
    fn block_ready(&self, block: [u8; AES_BLOCKSIZE]) {
        self.client
            .map(|c| c.data_available(&block, Continue::More));
    }
}
