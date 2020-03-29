//! AES 128/192/256-bit implementation in software
//!
//! This implementation of the Advanced Encryption Standard is
//! designed to be very similar to the reference implementation
//! described in the [official
//! standard](https://doi.org/10.6028/NIST.FIPS.197).  It is heavily
//! inspired by [tiny-AES-c](https://github.com/kokke/tiny-AES-c).
//!
//! *WARNING:* This implementation is susceptible to side-channel
//! attacks. In addition to that, the authors make no claims of it
//! being secure, bug-free or safe to use in any environment. Use for
//! development purposes exclusively!
//!
//!
//! Usage
//! -------------------
//!
//! // TODO: Update usage example
//! ```rust
//! static mut AES_KEYEXPANSION_BUFFER: [[u8; AES_WORDSIZE]; 4 * AES_256_EXPANDED_KEYS]
//!     = [[0; AES_WORDSIZE]; 4 * AES_256_EXPANDED_KEYS];
//!
//! let aes = static_init!(
//!     SoftAES<'static>,
//!     SoftAES::new(
//!         &mut AES_KEYEXPANSION_BUFFER
//!     )
//! );
//! aes.expand_key(&AESKey::K128([0; 16]));
//! ```
//!
//! Authors
//! -------------------
//! * Leon Schuermann <leon.git@is.currently.online>
//! * Daniel Rutz <info@danielrutz.com>
//! * March 29, 2020

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};
use kernel::ReturnCode;

use crate::aes::{
    AESBlock, AESBlockClient, AESBlockEngine, AESClient, AESEngine, AESKey, AESWord,
    AES_256_EXPANDED_KEYS, AES_BLOCKSIZE, AES_WORDSIZE, AES_WORDS_IN_BLOCK,
};

mod aes_constants;
use self::aes_constants::{RCON, RSBOX, SBOX};

enum DeferredCall {
    Default,
    BlockReady(AESBlock),
    ExpandedKeyReady,
}
impl Default for DeferredCall {
    fn default() -> DeferredCall {
        DeferredCall::Default
    }
}

pub struct SoftAES<'a> {
    key_expansion_buffer: TakeCell<'static, [AESWord; 4 * AES_256_EXPANDED_KEYS]>,
    key_rounds: OptionalCell<usize>,
    aes_client: OptionalCell<&'a dyn AESClient>,
    block_client: OptionalCell<&'a dyn AESBlockClient>,
    deferred_call: Cell<DeferredCall>,
    deferred_call_mux: &'a DynamicDeferredCall,
    deferred_call_handle: OptionalCell<DeferredCallHandle>,
}

impl<'a> SoftAES<'a> {
    pub fn new(
        key_expansion_buffer: &'static mut [AESWord; 4 * AES_256_EXPANDED_KEYS],
        deferred_call_mux: &'a DynamicDeferredCall,
    ) -> SoftAES<'a> {
        SoftAES {
            key_expansion_buffer: TakeCell::new(key_expansion_buffer),
            key_rounds: OptionalCell::empty(),
            aes_client: OptionalCell::empty(),
            block_client: OptionalCell::empty(),
            deferred_call: Cell::default(),
            deferred_call_mux: deferred_call_mux,
            deferred_call_handle: OptionalCell::empty(),
        }
    }

    pub fn set_deferred_handle(&self, handle: DeferredCallHandle) {
        self.deferred_call_handle.set(handle);
    }

    fn rot_word(word: &mut AESWord) -> () {
        let tmp: u8 = word[0];
        word[0] = word[1];
        word[1] = word[2];
        word[2] = word[3];
        word[3] = tmp;
    }

    fn sub_word(word: &mut AESWord) -> () {
        for b in word.iter_mut() {
            *b = SBOX[*b as usize];
        }
    }

    fn add_round_key(
        round: usize,
        key_schedule: &[AESWord; 4 * AES_256_EXPANDED_KEYS],
        state: &mut [[u8; AES_WORDSIZE]; AES_WORDS_IN_BLOCK],
    ) {
        state
            .iter_mut()
            .zip(key_schedule.iter().skip(round * AES_WORDS_IN_BLOCK))
            .for_each(|(state_column, key_expansion_column)| {
                state_column
                    .iter_mut()
                    .zip(key_expansion_column.iter())
                    .for_each(|(state_byte, round_key_byte)| *state_byte ^= round_key_byte);
            });
    }

    fn sub_bytes(buf: &mut [[u8; AES_WORDSIZE]; AES_WORDS_IN_BLOCK]) {
        for row in buf.iter_mut() {
            for byte in row.iter_mut() {
                *byte = SBOX[*byte as usize];
            }
        }
    }

    fn inv_sub_bytes(buf: &mut [[u8; AES_WORDSIZE]; AES_WORDS_IN_BLOCK]) {
        for row in buf.iter_mut() {
            for byte in row.iter_mut() {
                *byte = RSBOX[*byte as usize];
            }
        }
    }

    fn shift_rows(buf: &mut [[u8; AES_WORDSIZE]; AES_WORDS_IN_BLOCK]) {
        let mut temp: u8;

        temp = buf[0][1];
        buf[0][1] = buf[1][1];
        buf[1][1] = buf[2][1];
        buf[2][1] = buf[3][1];
        buf[3][1] = temp;

        temp = buf[0][2];
        buf[0][2] = buf[2][2];
        buf[2][2] = temp;
        temp = buf[1][2];
        buf[1][2] = buf[3][2];
        buf[3][2] = temp;

        temp = buf[0][3];
        buf[0][3] = buf[3][3];
        buf[3][3] = buf[2][3];
        buf[2][3] = buf[1][3];
        buf[1][3] = temp;
    }

    fn inv_shift_rows(buf: &mut [[u8; AES_WORDSIZE]; AES_WORDS_IN_BLOCK]) {
        let mut temp: u8;

        temp = buf[3][1];
        buf[3][1] = buf[2][1];
        buf[2][1] = buf[1][1];
        buf[1][1] = buf[0][1];
        buf[0][1] = temp;

        temp = buf[0][2];
        buf[0][2] = buf[2][2];
        buf[2][2] = temp;
        temp = buf[1][2];
        buf[1][2] = buf[3][2];
        buf[3][2] = temp;

        temp = buf[0][3];
        buf[0][3] = buf[1][3];
        buf[1][3] = buf[2][3];
        buf[2][3] = buf[3][3];
        buf[3][3] = temp;
    }

    fn mix_columns(state: &mut [[u8; AES_WORDSIZE]; AES_WORDS_IN_BLOCK]) {
        fn xtime(x: u8) -> u8 {
            // 0x1b represents the irreducible polynomial
            (x << 1) ^ (((x >> 7) & 1) * 0x1b)
        }

        for column in state.iter_mut() {
            let t: u8 = column[0];
            let tmp: u8 = column[0] ^ column[1] ^ column[2] ^ column[3];

            column[0] ^= xtime(column[0] ^ column[1]) ^ tmp;
            column[1] ^= xtime(column[1] ^ column[2]) ^ tmp;
            column[2] ^= xtime(column[2] ^ column[3]) ^ tmp;
            column[3] ^= xtime(column[3] ^ t) ^ tmp;
        }
    }

    fn inv_mix_columns(state: &mut [[u8; AES_WORDSIZE]; AES_WORDS_IN_BLOCK]) {
        fn xtime(x: u8) -> u8 {
            // 0x1b represents the irreducible polynomial
            (x << 1) ^ (((x >> 7) & 1) * 0x1b)
        }

        fn multiply(x: u8, y: u8) -> u8 {
            ((y & 1) * x)
                ^ ((y >> 1 & 1) * xtime(x))
                ^ ((y >> 2 & 1) * xtime(xtime(x)))
                ^ ((y >> 3 & 1) * xtime(xtime(xtime(x))))
                ^ ((y >> 4 & 1) * xtime(xtime(xtime(xtime(x)))))
        }

        for column in state.iter_mut() {
            let tmp = column.clone();

            column[0] = multiply(tmp[0], 0x0e)
                ^ multiply(tmp[1], 0x0b)
                ^ multiply(tmp[2], 0x0d)
                ^ multiply(tmp[3], 0x09);
            column[1] = multiply(tmp[0], 0x09)
                ^ multiply(tmp[1], 0x0e)
                ^ multiply(tmp[2], 0x0b)
                ^ multiply(tmp[3], 0x0d);
            column[2] = multiply(tmp[0], 0x0d)
                ^ multiply(tmp[1], 0x09)
                ^ multiply(tmp[2], 0x0e)
                ^ multiply(tmp[3], 0x0b);
            column[3] = multiply(tmp[0], 0x0b)
                ^ multiply(tmp[1], 0x0d)
                ^ multiply(tmp[2], 0x09)
                ^ multiply(tmp[3], 0x0e);
        }
    }

    fn cipher(
        rounds: usize,
        key_schedule: &[AESWord; 4 * AES_256_EXPANDED_KEYS],
        state: &mut [AESWord; AES_WORDS_IN_BLOCK],
    ) {
        Self::add_round_key(0, key_schedule, state);

        for round in 1..rounds {
            Self::sub_bytes(state);
            Self::shift_rows(state);
            Self::mix_columns(state);
            Self::add_round_key(round, key_schedule, state);
        }

        Self::sub_bytes(state);
        Self::shift_rows(state);
        Self::add_round_key(rounds, key_schedule, state);
    }

    fn inv_cipher(
        rounds: usize,
        key_schedule: &[AESWord; 4 * AES_256_EXPANDED_KEYS],
        state: &mut [AESWord; AES_WORDS_IN_BLOCK],
    ) {
        Self::add_round_key(rounds, key_schedule, state);

        for round in (1..rounds).rev() {
            Self::inv_shift_rows(state);
            Self::inv_sub_bytes(state);
            Self::add_round_key(round, key_schedule, state);
            Self::inv_mix_columns(state);
        }

        Self::inv_shift_rows(state);
        Self::inv_sub_bytes(state);
        Self::add_round_key(0, key_schedule, state);
    }
}

impl<'a> DynamicDeferredCallClient for SoftAES<'a> {
    fn call(&self, _handle: DeferredCallHandle) {
        match self.deferred_call.replace(DeferredCall::Default) {
            DeferredCall::Default => (),
            DeferredCall::ExpandedKeyReady => {
                self.aes_client.map(|c| c.expanded_key_ready());
            }
            DeferredCall::BlockReady(block) => {
                self.block_client.map(|c| c.block_ready(block));
            }
        }
    }
}

impl<'a> AESEngine<'a> for SoftAES<'a> {
    fn set_client(&'a self, client: &'a dyn AESClient) {
        self.aes_client.set(client);
    }

    fn invalidate_key(&self) -> Result<(), ReturnCode> {
        self.key_rounds.clear();

        Ok(())
    }

    fn expand_key(&self, key: &AESKey) -> Result<(), ReturnCode> {
        self.key_expansion_buffer.map_or(
            Err(ReturnCode::EBUSY),
            |key_expansion_buffer| -> Result<(), ReturnCode> {
                let key_is_256 = match key {
                    AESKey::K256(_) => true,
                    _ => false,
                };

                for i in 0..key.key_words() {
                    key_expansion_buffer[i] = [
                        key.raw_key()[i * AES_WORDSIZE + 0],
                        key.raw_key()[i * AES_WORDSIZE + 1],
                        key.raw_key()[i * AES_WORDSIZE + 2],
                        key.raw_key()[i * AES_WORDSIZE + 3],
                    ];
                }

                for i in key.key_words()..(AES_WORDS_IN_BLOCK * (key.rounds() + 1)) {
                    let mut temp_word: AESWord = key_expansion_buffer[i - 1];

                    if i % key.key_words() == 0 {
                        Self::rot_word(&mut temp_word);
                        Self::sub_word(&mut temp_word);

                        temp_word[0] = temp_word[0] ^ RCON[i / key.key_words()];
                    } else if key_is_256 && i % key.key_words() == 4 {
                        // TODO: Magic number?
                        Self::sub_word(&mut temp_word);
                    }

                    for byte_in_word in 0..4 {
                        key_expansion_buffer[i][byte_in_word] = key_expansion_buffer
                            [i - key.key_words()][byte_in_word]
                            ^ temp_word[byte_in_word];
                    }
                }

                self.key_rounds.set(key.rounds());

                Ok(())
            },
        )?;

        self.deferred_call.set(DeferredCall::ExpandedKeyReady);
        self.deferred_call_handle
            .map(|h| self.deferred_call_mux.set(*h));

        Ok(())
    }
}

impl<'a> AESBlockEngine<'a> for SoftAES<'a> {
    fn set_client(&'a self, client: &'a dyn AESBlockClient) {
        self.block_client.set(client);
    }

    fn encrypt(&self, src: &[u8; 16]) -> Result<(), ReturnCode> {
        // Let's first try to access the key expansion buffer before checking
        // the key rounds. If we return ERESERVE because of no expanded key
        // ready first, the client is likely to try expanding a key, which will
        // still not work as the buffer is in use.
        self.key_expansion_buffer.map_or(
            Err(ReturnCode::EBUSY),
            |key_schedule| -> Result<(), ReturnCode> {
                // Abort early with error in case key isn't expanded yet
                // This should be replaced by a self.key_rounds.get()
                let key_rounds = if self.key_rounds.is_some() {
                    self.key_rounds.expect("is_some")
                } else {
                    return Err(ReturnCode::ERESERVE);
                };

                let state_src = src;

                let mut state: [[u8; AES_WORDSIZE]; AES_WORDS_IN_BLOCK] = [
                    [state_src[0], state_src[1], state_src[2], state_src[3]],
                    [state_src[4], state_src[5], state_src[6], state_src[7]],
                    [state_src[8], state_src[9], state_src[10], state_src[11]],
                    [state_src[12], state_src[13], state_src[14], state_src[15]],
                ];

                Self::cipher(key_rounds, key_schedule, &mut state);

                let mut block: AESBlock = [0; AES_BLOCKSIZE];
                state.iter().enumerate().for_each(|(c, col)| {
                    col.iter()
                        .enumerate()
                        .for_each(|(b, byte)| block[c * AES_WORDSIZE + b] = *byte)
                });

                self.deferred_call.set(DeferredCall::BlockReady(block));
                self.deferred_call_handle
                    .map(|h| self.deferred_call_mux.set(*h));

                Ok(())
            },
        )
    }

    fn decrypt(&self, src: &[u8; 16]) -> Result<(), ReturnCode> {
        // Let's first try to access the key expansion buffer before checking
        // the key rounds. If we return ERESERVE because of no expanded key
        // ready first, the client is likely to try expanding a key, which will
        // still not work as the buffer is in use.
        self.key_expansion_buffer.map_or(
            Err(ReturnCode::EBUSY),
            |key_schedule| -> Result<(), ReturnCode> {
                // Abort early with error in case key isn't expanded yet
                // This should be replaced by a self.key_rounds.get()
                let key_rounds = if self.key_rounds.is_some() {
                    self.key_rounds.expect("is_some")
                } else {
                    return Err(ReturnCode::ERESERVE);
                };

                let state_src = src;

                let mut state: [[u8; AES_WORDSIZE]; AES_WORDS_IN_BLOCK] = [
                    [state_src[0], state_src[1], state_src[2], state_src[3]],
                    [state_src[4], state_src[5], state_src[6], state_src[7]],
                    [state_src[8], state_src[9], state_src[10], state_src[11]],
                    [state_src[12], state_src[13], state_src[14], state_src[15]],
                ];

                Self::inv_cipher(key_rounds, key_schedule, &mut state);

                let mut block: AESBlock = [0; AES_BLOCKSIZE];
                state.iter().enumerate().for_each(|(c, col)| {
                    col.iter()
                        .enumerate()
                        .for_each(|(b, byte)| block[c * AES_WORDSIZE + b] = *byte)
                });

                self.deferred_call.set(DeferredCall::BlockReady(block));
                self.deferred_call_handle
                    .map(|h| self.deferred_call_mux.set(*h));

                Ok(())
            },
        )
    }
}
