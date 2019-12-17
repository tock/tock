//! Implements AES-CCM* encryption/decryption/authentication using an underlying
//! AES-CBC and AES-CTR implementation.
//!
//! IEEE 802.15.4-2015: Appendix B.4.1, CCM* transformations. CCM* is
//! defined so that both encryption and decryption can be done by preparing two
//! fields: the AuthData and either the PlaintextData or the CiphertextData.
//! Then, two passes of AES are performed with one block of overlap.
//!
//! ```text
//! crypt_buf: [ -------- AuthData -------- | -------- PData/CData -------- ]
//! aes_cbc:    \__________________________/
//! aes_ctr:                        \ 1 blk | _____________________________/
//! ```
//!
//! The overlapping block is then the encrypted authentication tag U. For
//! encryption, we append U to the data as a message integrity code (MIC).
//! For decryption, we compare U with the provided MIC.
//
//! This is true only if data confidentiality is not needed. If it is, then
//! the AuthData includes the PlaintextData. At encryption, we perform CBC over
//! both fields, then copy the last block to just before the PData. Then,
//! CTR mode is performed over the same overlapping region, forming the encrypted
//! authentication tag U.
//!
//! ```text
//! crypt_buf: [ -------- AuthData -------- | -------- PData/CData -------- ]
//! aes_cbc:    \__________________________________________________________/
//! aes_ctr:                        \ 1 blk | _____________________________/
//! ```
//!
//! At decryption, there is no choice but the reverse the order of operations.
//! First, we zero out the overlapping block and perform ctr over it and the
//! PlaintextData. This produces Enc(Key, A_i), which we save in saved_tag.
//! Then, we restore the previous value of the last block of AuthData and re-pad
//! PlaintextData before running CBC over both fields. The last step is to
//! combine saved_tag and the unencrypted tag to form the encrypted tag and
//! verify its correctness.
//!
//! Usage
//! -----
//!
//! ```rust
//! # use kernel::static_init;
//! # use kernel::hil::symmetric_encryption;
//!
//! const CRYPT_SIZE: usize = 3 * symmetric_encryption::AES128_BLOCK_SIZE + radio::MAX_BUF_SIZE;
//! static mut CRYPT_BUF: [u8; CRYPT_SIZE] = [0x00; CRYPT_SIZE];
//!
//! let aes_ccm = static_init!(
//!     capsules::aes_ccm::AES128CCM<'static, sam4l::aes::Aes<'static>>,
//!     capsules::aes_ccm::AES128CCM::new(&sam4l::aes::AES, &mut CRYPT_BUF)
//! );
//! sam4l::aes::AES.set_client(aes_ccm);
//! sam4l::aes::AES.enable();
//! ```

use crate::net::stream::SResult;
use crate::net::stream::{encode_bytes, encode_u16};
use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::symmetric_encryption;
use kernel::hil::symmetric_encryption::{
    AES128Ctr, AES128, AES128CBC, AES128_BLOCK_SIZE, AES128_KEY_SIZE, CCM_NONCE_LENGTH,
};
use kernel::ReturnCode;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum CCMState {
    Idle,
    Auth,
    Encrypt,
}

pub struct AES128CCM<'a, A: AES128<'a> + AES128Ctr + AES128CBC> {
    aes: &'a A,
    crypt_buf: TakeCell<'a, [u8]>,
    crypt_auth_len: Cell<usize>,
    crypt_enc_len: Cell<usize>,
    crypt_client: OptionalCell<&'a dyn symmetric_encryption::CCMClient>,

    state: Cell<CCMState>,
    confidential: Cell<bool>,
    encrypting: Cell<bool>,

    buf: TakeCell<'static, [u8]>,
    pos: Cell<(usize, usize, usize, usize)>,
    key: Cell<[u8; AES128_KEY_SIZE]>,
    nonce: Cell<[u8; CCM_NONCE_LENGTH]>,
    saved_tag: Cell<[u8; AES128_BLOCK_SIZE]>,
}

impl<A: AES128<'a> + AES128Ctr + AES128CBC> AES128CCM<'a, A> {
    pub fn new(aes: &'a A, crypt_buf: &'static mut [u8]) -> AES128CCM<'a, A> {
        AES128CCM {
            aes: aes,
            crypt_buf: TakeCell::new(crypt_buf),
            crypt_auth_len: Cell::new(0),
            crypt_enc_len: Cell::new(0),
            crypt_client: OptionalCell::empty(),
            state: Cell::new(CCMState::Idle),
            confidential: Cell::new(false),
            encrypting: Cell::new(false),
            buf: TakeCell::empty(),
            pos: Cell::new((0, 0, 0, 0)),
            key: Cell::new(Default::default()),
            nonce: Cell::new(Default::default()),
            saved_tag: Cell::new(Default::default()),
        }
    }

    /// Prepares crypt_buf with the input for the CCM* authentication and
    /// encryption/decryption transformations. Returns ENOMEM if crypt_buf is
    /// not present or if it is not long enough.
    fn prepare_ccm_buffer(
        &self,
        nonce: &[u8; CCM_NONCE_LENGTH],
        mic_len: usize,
        a_data: &[u8],
        m_data: &[u8],
    ) -> ReturnCode {
        self.crypt_buf.map_or(ReturnCode::ENOMEM, |cbuf| {
            let (auth_len, enc_len) =
                match Self::encode_ccm_buffer(cbuf, nonce, mic_len, a_data, m_data) {
                    SResult::Done(_, out) => out,
                    SResult::Needed(_) => {
                        return ReturnCode::ENOMEM;
                    }
                    SResult::Error(_) => {
                        return ReturnCode::FAIL;
                    }
                };
            // debug!("auth: ({})", auth_len);
            // for i in 0..auth_len {
            //     debug!("{:02x}", cbuf[i]);
            // }
            // debug!("enc: ({})", enc_len);
            // for i in auth_len..enc_len {
            //     debug!("{:02x}", cbuf[i]);
            // }

            self.crypt_auth_len.set(auth_len);
            self.crypt_enc_len.set(enc_len);
            ReturnCode::SUCCESS
        })
    }

    /// This function encodes AuthData (a_data) and PData/CData (m_data) into a
    /// buffer, along with the prerequisite metadata/padding bytes. On success,
    /// `auth_len` (the length of the AuthData field) and `enc_len` (the
    /// combined length of AuthData and PData/CData) are returned. `auth_len` is
    /// guaranteed to be >= AES128_BLOCK_SIZE
    fn encode_ccm_buffer(
        buf: &mut [u8],
        nonce: &[u8; CCM_NONCE_LENGTH],
        mic_len: usize,
        a_data: &[u8],
        m_data: &[u8],
    ) -> SResult<(usize, usize)> {
        // IEEE 802.15.4-2015: Appendix B.4.1.2, CCM* authentication
        // The authentication tag T is computed with AES128-CBC-MAC on
        // B_0 | AuthData, where
        //   B_0 = Flags (1 byte) | nonce (13 bytes) | m length (2 bytes)
        //   Flags = 0 | A data present? (1 bit) | M (3 bits) | L (3 bits)
        //   AuthData = AddAuthData | PlaintextData
        //   AddAuthData = L(a) (encoding of a_data.len()) | a_data
        //   PlaintextData = m_data
        //   Both AddAuthData and PlaintextData are 0-padded to 16-byte blocks.
        // The following code places B_0 | AuthData into crypt_buf.

        // flags = reserved | Adata | (M - 2) / 2 | (L - 1)
        let mut flags: u8 = 0;
        if a_data.len() != 0 {
            flags |= 1 << 6;
        }
        if mic_len != 0 {
            flags |= (((mic_len - 2) / 2) as u8) << 3;
        }
        flags |= 1;

        stream_len_cond!(buf, AES128_BLOCK_SIZE);
        // The first block is flags | nonce | m length
        buf[0] = flags;
        buf[1..14].copy_from_slice(nonce.as_ref());
        let mut off = enc_consume!(buf, 14; encode_u16,
                                            (m_data.len() as u16).to_le());

        // After that comes L(a) | a, where L(a) is the following
        // encoding of a_len:
        if a_data.len() == 0 {
            // L(a) is empty, and the Adata flag is zero
        } else if a_data.len() < 0xff00 as usize {
            // L(a) is l(a) in 2 bytes of little-endian
            off = enc_consume!(buf, off; encode_u16,
                                         (a_data.len() as u16).to_le());
        } else {
            // These length encoding branches are defined in the specification
            // but should never be reached because our MTU is 127.
            stream_err!(());
        }

        // Append the auth data and 0-pad to a multiple of 16 bytes
        off = enc_consume!(buf, off; encode_bytes, a_data);
        let auth_len = ((off + AES128_BLOCK_SIZE - 1) / AES128_BLOCK_SIZE) * AES128_BLOCK_SIZE;
        stream_len_cond!(buf, auth_len);
        buf[off..auth_len].iter_mut().for_each(|b| *b = 0);
        off = auth_len;

        // Append plaintext data and 0-pad to a multiple of 16 bytes
        off = enc_consume!(buf, off; encode_bytes, m_data);
        let enc_len = ((off + AES128_BLOCK_SIZE - 1) / AES128_BLOCK_SIZE) * AES128_BLOCK_SIZE;
        stream_len_cond!(buf, enc_len);
        buf[off..enc_len].iter_mut().for_each(|b| *b = 0);
        off = enc_len;

        stream_done!(off, (auth_len, enc_len));
    }

    fn reversed(&self) -> bool {
        self.confidential.get() && !self.encrypting.get()
    }

    // Assumes that the state is Idle, which means that crypt_buf must be
    // present. Panics if this is not the case.
    fn start_ccm_auth(&self) -> ReturnCode {
        if !(self.state.get() == CCMState::Idle)
            && !(self.state.get() == CCMState::Encrypt && self.reversed())
        {
            panic!("Called start_ccm_auth when not idle");
        }

        let iv = [0u8; AES128_BLOCK_SIZE];
        let res = self.aes.set_iv(&iv);
        if res != ReturnCode::SUCCESS {
            return res;
        }
        let res = self.aes.set_key(&self.key.get());
        if res != ReturnCode::SUCCESS {
            return res;
        }

        let crypt_buf = match self.crypt_buf.take() {
            None => panic!("Cannot perform CCM* auth because crypt_buf is not present."),
            Some(buf) => buf,
        };

        // If confidentiality is needed, authenticate over message data.
        let auth_end = if self.confidential.get() {
            self.crypt_enc_len.get()
        } else {
            self.crypt_auth_len.get()
        };

        // We are performing CBC-MAC, so always encrypting.
        self.aes.set_mode_aes128cbc(true);
        self.aes.start_message();
        match self.aes.crypt(None, crypt_buf, 0, auth_end) {
            None => {
                self.state.set(CCMState::Auth);
                ReturnCode::SUCCESS
            }
            Some((res, _, crypt_buf)) => {
                // Request failed
                self.crypt_buf.replace(crypt_buf);
                res
            }
        }
    }

    fn start_ccm_encrypt(&self) -> ReturnCode {
        if !(self.state.get() == CCMState::Auth)
            && !(self.state.get() == CCMState::Idle && self.reversed())
        {
            return ReturnCode::FAIL;
        }
        self.state.set(CCMState::Idle); // default to fail

        // debug!("after auth:");
        // self.crypt_buf.map(|buf| {
        //     for i in 0..self.crypt_auth_len.get() {
        //         debug!("{:02x}", buf[i]);
        //     }
        // });

        let mut iv = [0u8; AES128_BLOCK_SIZE];
        // flags = reserved | reserved | 0 | (L - 1)
        // Since L = 2, flags = 1.
        iv[0] = 1;
        iv[1..1 + CCM_NONCE_LENGTH].copy_from_slice(&self.nonce.get());
        let res = self.aes.set_iv(&iv);
        if res != ReturnCode::SUCCESS {
            return res;
        }

        self.aes.set_mode_aes128ctr(self.encrypting.get());
        self.aes.start_message();
        let crypt_buf = match self.crypt_buf.take() {
            None => panic!("Cannot perform CCM* encrypt because crypt_buf is not present."),
            Some(buf) => buf,
        };

        match self.aes.crypt(
            None,
            crypt_buf,
            self.crypt_auth_len.get() - AES128_BLOCK_SIZE,
            self.crypt_enc_len.get(),
        ) {
            None => {
                self.state.set(CCMState::Encrypt);
                ReturnCode::SUCCESS
            }
            Some((res, _, crypt_buf)) => {
                self.crypt_buf.replace(crypt_buf);
                res
            }
        }
    }

    fn end_ccm(&self) {
        let tag_valid = self.buf.map_or(false, |buf| {
            self.crypt_buf.map_or_else(
                || {
                    panic!("We lost track of crypt_buf!");
                },
                |cbuf| {
                    // Copy the encrypted/decrypted message data
                    let (_, m_off, m_len, mic_len) = self.pos.get();
                    let auth_len = self.crypt_auth_len.get();
                    buf[m_off..m_off + m_len].copy_from_slice(&cbuf[auth_len..auth_len + m_len]);

                    let m_end = m_off + m_len;
                    let tag_off = auth_len - AES128_BLOCK_SIZE;
                    if self.encrypting.get() {
                        // Copy the encrypted tag to the end of the message
                        buf[m_end..m_end + mic_len]
                            .copy_from_slice(&cbuf[tag_off..tag_off + mic_len]);
                        true
                    } else {
                        // Compare the computed encrypted tag to the received
                        // encrypted tag
                        buf[m_end..m_end + mic_len]
                            .iter()
                            .zip(cbuf[tag_off..tag_off + mic_len].iter())
                            .all(|(a, b)| *a == *b)
                    }
                },
            )
        });

        self.state.set(CCMState::Idle);
        self.crypt_client.map(|client| {
            self.buf.take().map(|buf| {
                client.crypt_done(buf, ReturnCode::SUCCESS, tag_valid);
            });
        });
    }

    fn reverse_end_ccm(&self) {
        // Finalize CCM process only in the case where we did CTR before CBC
        let tag_valid = self.buf.map_or(false, |buf| {
            self.crypt_buf.map_or_else(
                || {
                    panic!("We lost track of crypt_buf!");
                },
                |cbuf| {
                    let (_, m_off, m_len, mic_len) = self.pos.get();

                    // Combine unencrypted tag at end of crypt_buf with saved
                    // CTR-encrypted block to obtain encrypted tag
                    let tag_off = self.crypt_enc_len.get() - AES128_BLOCK_SIZE;
                    self.saved_tag.get()[..mic_len]
                        .iter()
                        .zip(cbuf[tag_off..tag_off + mic_len].iter_mut())
                        .for_each(|(a, b)| *b ^= *a);

                    // Compare the computed encrypted tag to the received
                    // encrypted tag
                    buf[m_off + m_len..m_off + m_len + mic_len]
                        .iter()
                        .zip(cbuf[tag_off..tag_off + mic_len].iter())
                        .all(|(a, b)| *a == *b)
                },
            )
        });

        self.state.set(CCMState::Idle);
        self.crypt_client.map(|client| {
            self.buf.take().map(|buf| {
                client.crypt_done(buf, ReturnCode::SUCCESS, tag_valid);
            });
        });
    }

    fn save_tag_block(&self) {
        // Copies [auth_len - AES128_BLOCK_SIZE..auth_len] to saved_tag
        // and zeroes it out
        let auth_len = self.crypt_auth_len.get();
        self.crypt_buf.map(|cbuf| {
            let mut cbuf_block = [0u8; AES128_BLOCK_SIZE];
            cbuf_block.copy_from_slice(&cbuf[auth_len - AES128_BLOCK_SIZE..auth_len]);
            self.saved_tag.set(cbuf_block);
            cbuf[auth_len - AES128_BLOCK_SIZE..auth_len]
                .iter_mut()
                .for_each(|b| *b = 0);
        });
    }

    fn swap_tag_block(&self) {
        // Swaps [auth_len - AES128_BLOCK_SIZE..auth_len] with
        // the value in saved_tag
        let auth_len = self.crypt_auth_len.get();
        self.crypt_buf.map(|cbuf| {
            let mut cbuf_block = [0u8; AES128_BLOCK_SIZE];
            cbuf_block.copy_from_slice(&cbuf[auth_len - AES128_BLOCK_SIZE..auth_len]);
            cbuf[auth_len - AES128_BLOCK_SIZE..auth_len].copy_from_slice(&self.saved_tag.get());
            self.saved_tag.set(cbuf_block);
        });
    }
}

impl<A: AES128<'a> + AES128Ctr + AES128CBC> symmetric_encryption::AES128CCM<'a>
    for AES128CCM<'a, A>
{
    fn set_client(&self, client: &'a dyn symmetric_encryption::CCMClient) {
        self.crypt_client.set(client);
    }

    fn set_key(&self, key: &[u8]) -> ReturnCode {
        if key.len() < AES128_KEY_SIZE {
            ReturnCode::EINVAL
        } else {
            let mut new_key = [0u8; AES128_KEY_SIZE];
            new_key.copy_from_slice(key);
            self.key.set(new_key);
            ReturnCode::SUCCESS
        }
    }

    fn set_nonce(&self, nonce: &[u8]) -> ReturnCode {
        if nonce.len() < CCM_NONCE_LENGTH {
            ReturnCode::EINVAL
        } else {
            let mut new_nonce = [0u8; CCM_NONCE_LENGTH];
            new_nonce.copy_from_slice(nonce);
            self.nonce.set(new_nonce);
            ReturnCode::SUCCESS
        }
    }

    fn crypt(
        &self,
        buf: &'static mut [u8],
        a_off: usize,
        m_off: usize,
        m_len: usize,
        mic_len: usize,
        confidential: bool,
        encrypting: bool,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if self.state.get() != CCMState::Idle {
            return (ReturnCode::EBUSY, Some(buf));
        }
        if !(a_off <= m_off && m_off + m_len + mic_len <= buf.len()) {
            return (ReturnCode::EINVAL, Some(buf));
        }

        self.confidential.set(confidential);
        self.encrypting.set(encrypting);

        let res = self.prepare_ccm_buffer(
            &self.nonce.get(),
            mic_len,
            &buf[a_off..m_off],
            &buf[m_off..m_off + m_len],
        );
        if res != ReturnCode::SUCCESS {
            return (res, Some(buf));
        }

        let res = if !confidential || encrypting {
            // Perform CBC before CTR
            self.start_ccm_auth()
        } else {
            // Perform CTR before CBC
            self.save_tag_block();
            self.start_ccm_encrypt()
        };

        if res != ReturnCode::SUCCESS {
            (res, Some(buf))
        } else {
            self.buf.replace(buf);
            self.pos.set((a_off, m_off, m_len, mic_len));
            (ReturnCode::SUCCESS, None)
        }
    }
}

impl<A: AES128<'a> + AES128Ctr + AES128CBC> symmetric_encryption::Client<'a> for AES128CCM<'a, A> {
    fn crypt_done(&self, _: Option<&'a mut [u8]>, crypt_buf: &'a mut [u8]) {
        self.crypt_buf.replace(crypt_buf);
        match self.state.get() {
            CCMState::Idle => {}
            CCMState::Auth => {
                if !self.reversed() {
                    if self.confidential.get() {
                        let (_, m_off, m_len, _) = self.pos.get();
                        let auth_len = self.crypt_auth_len.get();
                        let enc_len = self.crypt_enc_len.get();
                        self.crypt_buf.map(|cbuf| {
                            // If we authenticated over the plaintext, copy the last
                            // block over to the beginning again so that it becomes
                            // the encrypted tag after ctr mode
                            let auth_last = auth_len - AES128_BLOCK_SIZE;
                            let enc_last = enc_len - AES128_BLOCK_SIZE;
                            for i in 0..AES128_BLOCK_SIZE {
                                cbuf[auth_last + i] = cbuf[enc_last + i];
                            }

                            // Then repopulate the plaintext data field
                            self.buf.map(|buf| {
                                cbuf[auth_len..auth_len + m_len]
                                    .copy_from_slice(&buf[m_off..m_off + m_len]);
                            });
                            cbuf[auth_len + m_len..enc_len]
                                .iter_mut()
                                .for_each(|b| *b = 0);
                        });
                    }

                    let res = self.start_ccm_encrypt();
                    if res != ReturnCode::SUCCESS {
                        // Return client buffer to client
                        self.buf.take().map(|buf| {
                            self.crypt_client.map(move |client| {
                                client.crypt_done(buf, res, false);
                            });
                        });
                        self.state.set(CCMState::Idle);
                    }
                } else {
                    self.reverse_end_ccm();
                }
            }
            CCMState::Encrypt => {
                if !self.reversed() {
                    self.end_ccm();
                } else {
                    self.swap_tag_block();
                    self.crypt_buf.map(|cbuf| {
                        // Copy the encrypted/decrypted message data
                        let (_, m_off, m_len, _) = self.pos.get();
                        let auth_len = self.crypt_auth_len.get();
                        self.buf.map(|buf| {
                            buf[m_off..m_off + m_len]
                                .copy_from_slice(&cbuf[auth_len..auth_len + m_len]);
                        });

                        // Reset the rest of the padding
                        cbuf[self.crypt_auth_len.get() + m_len..self.crypt_enc_len.get()]
                            .iter_mut()
                            .for_each(|b| *b = 0);
                    });
                    let res = self.start_ccm_auth();
                    if res != ReturnCode::SUCCESS {
                        // Return client buffer to client
                        self.buf.take().map(|buf| {
                            self.crypt_client.map(move |client| {
                                client.crypt_done(buf, res, false);
                            });
                        });
                        self.state.set(CCMState::Idle);
                    }
                }
            }
        }
    }
}
