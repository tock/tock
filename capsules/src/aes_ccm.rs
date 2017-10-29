// IEEE 802.15.4-2015: Appendix B.4.1, CCM* transformations. CCM* is
// defined so that both encryption and decryption can be done by preparing two
// fields: the AuthData and either the PlaintextData or the CiphertextData.
// Then, two passes of AES are performed with one block of overlap.
//
// crypt_buf: [ -------- AuthData -------- | -------- PData/CData -------- ]
// aes_cbc:    \__________________________/
// aes_ccm:                        \ 1 blk | _____________________________/
//
// The overlapping block is then the encrypted authentication tag U. For
// encryption, we append U to the data as a message integrity code (MIC).
// For decryption, we compare U with the provided MIC.

use core::cell::Cell;
use kernel::ReturnCode;
use kernel::common::take_cell::TakeCell;
use kernel::hil::symmetric_encryption;
use kernel::hil::symmetric_encryption::{AES128, AES128Ctr, AES128CBC};
use net::stream::{encode_u16, encode_bytes};
use net::stream::SResult;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum CCMState {
    Idle,
    Auth,
    Encrypt,
}

pub const BLOCK_LENGTH: usize = 16;
pub const NONCE_LENGTH: usize = 13;

pub trait Client {
    /// `res` is SUCCESS if the encryption/decryption process succeeded. This
    /// does not mean that the message has been verified in the case of
    /// decryption.
    /// If we are encrypting: `tag_is_valid` is `true` iff `res` is SUCCESS.
    /// If we are decrypting: `tag_is_valid` is `true` iff `res` is SUCCESS and the
    /// message authentication tag is valid.
    fn crypt_done(&self, buf: &'static mut [u8], res: ReturnCode, tag_is_valid: bool);
}

pub struct AES128CCM<'a, A: AES128<'a> + AES128Ctr + AES128CBC + 'a> {
    aes: &'a A,
    crypt_buf: TakeCell<'a, [u8]>,
    crypt_auth_len: Cell<usize>,
    crypt_enc_len: Cell<usize>,
    crypt_client: Cell<Option<&'a Client>>,

    state: Cell<CCMState>,
    encrypting: Cell<bool>,

    buf: TakeCell<'static, [u8]>,
    pos: Cell<(usize, usize, usize, usize)>,
    key: Cell<[u8; BLOCK_LENGTH]>,
    nonce: Cell<[u8; NONCE_LENGTH]>,
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC + 'a> AES128CCM<'a, A> {
    pub fn new(aes: &'a A, crypt_buf: &'static mut [u8]) -> AES128CCM<'a, A> {
        AES128CCM {
            aes: aes,
            crypt_buf: TakeCell::new(crypt_buf),
            crypt_auth_len: Cell::new(0),
            crypt_enc_len: Cell::new(0),
            crypt_client: Cell::new(None),
            state: Cell::new(CCMState::Idle),
            encrypting: Cell::new(false),
            buf: TakeCell::empty(),
            pos: Cell::new((0, 0, 0, 0)),
            key: Cell::new(Default::default()),
            nonce: Cell::new(Default::default()),
        }
    }

    pub fn set_client(&self, client: &'a Client) {
        self.crypt_client.set(Some(client));
    }

    pub fn set_key(&self, key: &[u8]) -> ReturnCode {
        if key.len() < BLOCK_LENGTH {
            ReturnCode::EINVAL
        } else {
            let mut new_key = [0u8; BLOCK_LENGTH];
            new_key.copy_from_slice(key);
            self.key.set(new_key);
            ReturnCode::SUCCESS
        }
    }

    pub fn set_nonce(&self, nonce: &[u8]) -> ReturnCode {
        if nonce.len() < NONCE_LENGTH {
            ReturnCode::EINVAL
        } else {
            let mut new_nonce = [0u8; NONCE_LENGTH];
            new_nonce.copy_from_slice(nonce);
            self.nonce.set(new_nonce);
            ReturnCode::SUCCESS
        }
    }

    pub fn crypt(&self,
                 buf: &'static mut [u8],
                 a_off: usize,
                 m_off: usize,
                 m_len: usize,
                 mic_len: usize,
                 encrypting: bool) -> (ReturnCode, Option<&'static [u8]>) {
        if self.state.get() != CCMState::Idle {
            return (ReturnCode::EBUSY, Some(buf));
        }
        if !(a_off <= m_off && m_off + m_len + mic_len <= buf.len()) {
            return (ReturnCode::EINVAL, Some(buf));
        }
        let res = self.prepare_ccm_buffer(&self.nonce.get(),
                                          mic_len,
                                          &buf[a_off..m_off],
                                          &buf[m_off..m_off + m_len]);
        if res != ReturnCode::SUCCESS {
            return (res, Some(buf));
        }

        let res = self.start_ccm_auth();
        if res != ReturnCode::SUCCESS {
            (res, Some(buf))
        } else {
            self.encrypting.set(encrypting);
            self.buf.replace(buf);
            self.pos.set((a_off, m_off, m_len, mic_len));
            (ReturnCode::SUCCESS, None)
        }
    }

    /// Prepares crypt_buf with the input for the CCM* authentication and
    /// encryption/decryption transformations. Returns ENOMEM if crypt_buf is
    /// not present or if it is not long enough.
    fn prepare_ccm_buffer(&self,
                          nonce: &[u8; NONCE_LENGTH],
                          mic_len: usize,
                          a_data: &[u8],
                          m_data: &[u8]) -> ReturnCode {
        self.crypt_buf.map_or(ReturnCode::ENOMEM, |cbuf| {
            let (auth_len, enc_len) = match Self::encode_ccm_buffer(
                cbuf, nonce, mic_len, a_data, m_data) {
                SResult::Done(_, out) => out,
                SResult::Needed(_) => { return ReturnCode::ENOMEM; }
                SResult::Error(_) => { return ReturnCode::FAIL; }
            };

            self.crypt_auth_len.set(auth_len);
            self.crypt_enc_len.set(enc_len);
            ReturnCode::SUCCESS
        })
    }

    /// This function encudes AuthData (a_data) and PData/CData (m_data) into a
    /// buffer, along with the prerequisite metadata/padding bytes. On success,
    /// `auth_len` (the length of the AuthData field) and `enc_len` (the
    /// combined length of AuthData and PData/CData) are returned. `auth_len` is
    /// guaranteed to be >= BLOCK_LENGTH
    fn encode_ccm_buffer(buf: &mut [u8],
                         nonce: &[u8; NONCE_LENGTH],
                         mic_len: usize,
                         a_data: &[u8],
                         m_data: &[u8]) -> SResult<(usize, usize)> {
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

        stream_len_cond!(buf, BLOCK_LENGTH);
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
        let auth_len = ((off + BLOCK_LENGTH - 1) / BLOCK_LENGTH) * BLOCK_LENGTH;
        stream_len_cond!(buf, auth_len);
        buf[off..auth_len].iter_mut().for_each(|b| *b = 0);
        off = auth_len;

        // Append plaintext data and 0-pad to a multiple of 16 bytes
        off = enc_consume!(buf, off; encode_bytes, m_data);
        let enc_len = ((off + BLOCK_LENGTH - 1) / BLOCK_LENGTH) * BLOCK_LENGTH;
        stream_len_cond!(buf, enc_len);
        buf[off..auth_len].iter_mut().for_each(|b| *b = 0);
        off = enc_len;

        stream_done!(off, (auth_len, enc_len));
    }

    fn retrieve_crypt_buf(&self) {
        match self.aes.take_dest() {
            Ok(Some(buf)) => self.crypt_buf.replace(buf),
            Ok(None) => {
                panic!("UM WHAT DOES THIS MEAN?");
            }
            Err(_) => {
                panic!("Could not get crypt_buf back from AES");
            }
        };
    }

    // Assumes that the state is Idle, which means that crypt_buf must be
    // present. Panics if this is not the case.
    fn start_ccm_auth(&self) -> ReturnCode {
        if self.state.get() != CCMState::Idle {
            panic!("Called start_ccm_auth when not idle");
        }

        let iv = [0u8; BLOCK_LENGTH];
        let res = self.aes.set_iv(&iv);
        if res != ReturnCode::SUCCESS { return res; }
        let res = self.aes.set_key(&self.key.get());
        if res != ReturnCode::SUCCESS { return res; }

        let crypt_buf = match self.crypt_buf.take() {
            None => panic!("Cannot perform CCM* auth because crypt_buf is not present."),
            Some(buf) => buf,
        };

        // XXX: Suppose put_dest returns EBUSY because it was still busy.  We
        // have already lost mutable access to crypt_buf, and are hence dead.
        let res = self.aes.put_dest(Some(crypt_buf));
        if res != ReturnCode::SUCCESS {
            self.retrieve_crypt_buf();
            return res;
        }

        self.aes.set_mode_aes128cbc(self.encrypting.get());
        self.aes.start_message();
        let res = self.aes.crypt(0, self.crypt_auth_len.get());
        if res != ReturnCode::SUCCESS {
            self.retrieve_crypt_buf();
            return res;
        }

        self.state.set(CCMState::Auth);
        ReturnCode::SUCCESS
    }

    fn start_ccm_encrypt(&self) -> ReturnCode {
        if self.state.get() != CCMState::Auth {
            return ReturnCode::FAIL;
        }
        self.state.set(CCMState::Idle); // default to fail

        let mut iv = [0u8; BLOCK_LENGTH];
        // flags = reserved | reserved | 0 | (L - 1)
        // Since L = 2, flags = 1.
        iv[0] = 1;
        iv[1..1 + NONCE_LENGTH].copy_from_slice(&self.nonce.get());
        let res = self.aes.set_iv(&iv);
        if res != ReturnCode::SUCCESS { return res; }

        self.aes.set_mode_aes128ctr(self.encrypting.get());
        self.aes.start_message();
        let res = self.aes.crypt(self.crypt_auth_len.get() - BLOCK_LENGTH,
                                 self.crypt_enc_len.get());
        if res != ReturnCode::SUCCESS {
            self.retrieve_crypt_buf();
            return res;
        }

        self.state.set(CCMState::Encrypt);
        ReturnCode::SUCCESS
    }

    fn end_ccm(&self) {
        self.retrieve_crypt_buf();

        // :( this is bad.
        let tag_valid = self.buf.map_or(false, |buf| {
            self.crypt_buf.map_or(false, |cbuf| {
                // Copy the encrypted/decrypted message data
                let (_, m_off, m_len, mic_len) = self.pos.get();
                let auth_len = self.crypt_auth_len.get();
                buf[m_off..m_off + m_len].copy_from_slice(
                    &cbuf[auth_len..auth_len + m_len]);

                let m_end = m_off + m_len;
                let tag_off = auth_len - BLOCK_LENGTH;
                if self.encrypting.get() {
                    // Copy the encrypted tag to the end of the message
                    buf[m_end..m_end + mic_len].copy_from_slice(
                        &cbuf[tag_off..tag_off + mic_len]);
                    true
                } else {
                    // Compare the computed encrypted tag to the received
                    // encrypted tag
                    buf[m_end..m_end + mic_len]
                        .iter()
                        .zip(cbuf[tag_off..tag_off + mic_len].iter())
                        .all(|(a, b)| *a == *b)
                }
            })
        });

        if let Some(client) = self.crypt_client.get() {
            self.buf.take().map(|buf| {
                client.crypt_done(buf, ReturnCode::SUCCESS, tag_valid);
            });
        }
        self.state.set(CCMState::Idle);
    }
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC> symmetric_encryption::Client for AES128CCM<'a, A> {
    fn crypt_done(&self) {
        match self.state.get() {
            CCMState::Idle => {},
            CCMState::Auth => {
                let res = self.start_ccm_encrypt();
                if res != ReturnCode::SUCCESS {
                    // Return client buffer to client
                    self.buf.take().map(|buf| {
                        if let Some(client) = self.crypt_client.get() {
                            client.crypt_done(buf, res, false);
                        }
                    });
                    // Retrieve crypt_buf from AES
                    self.retrieve_crypt_buf();
                    self.state.set(CCMState::Idle);
                }
            },
            CCMState::Encrypt => {
                self.end_ccm();
            },
        }
    }
}
