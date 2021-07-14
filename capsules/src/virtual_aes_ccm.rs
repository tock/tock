//! Implements and virtualizes AES-CCM* encryption/decryption/authentication using an underlying
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
//! # use capsules::test::aes_ccm::Test;
//! # use capsules::virtual_aes_ccm;
//! # use kernel::common::dynamic_deferred_call::DynamicDeferredCall;
//! # use kernel::hil::symmetric_encryption::{AES128, AES128CCM, AES128_BLOCK_SIZE};
//! # use kernel::static_init;
//! # use sam4l::aes::{Aes, AES};
//! type AESCCMMUX = virtual_aes_ccm::MuxAES128CCM<'static, Aes<'static>>;
//! type AESCCMCLIENT = virtual_aes_ccm::VirtualAES128CCM<'static, AESCCMMUX>;
//! // mux
//! let ccm_mux = static_init!(AESCCMMUX, virtual_aes_ccm::MuxAES128CCM::new(&AES));
//! AES.set_client(ccm_mux);
//! ccm_mux.initialize_callback_handle(
//!     dynamic_deferred_caller
//!         .register(ccm_mux)
//!         .expect("no deferred call slot available for ccm mux"),
//! );
//! const CRYPT_SIZE: usize = 7 * AES128_BLOCK_SIZE;
//! let crypt_buf1 = static_init!([u8; CRYPT_SIZE], [0x00; CRYPT_SIZE]);
//! let ccm_client1 = static_init!(
//!     AESCCMCLIENT,
//!     virtual_aes_ccm::VirtualAES128CCM::new(ccm_mux, crypt_buf1)
//! );
//! ccm_client1.setup();
//! let data1 = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0x00; 4 * AES128_BLOCK_SIZE]);
//! let t1 = static_init!(Test<'static, AESCCMCLIENT>, Test::new(ccm_client1, data1));
//! ccm_client1.set_client(t1);
//! let crypt_buf2 = static_init!([u8; CRYPT_SIZE], [0x00; CRYPT_SIZE]);
//! let ccm_client2 = static_init!(
//!     AESCCMCLIENT,
//!     virtual_aes_ccm::VirtualAES128CCM::new(ccm_mux, crypt_buf2)
//! );
//! ccm_client2.setup();
//! let data2 = static_init!([u8; 4 * AES128_BLOCK_SIZE], [0x00; 4 * AES128_BLOCK_SIZE]);
//! let t2 = static_init!(Test<'static, AESCCMCLIENT>, Test::new(ccm_client2, data2));
//! ccm_client2.set_client(t2);
//! t1.run();
//! t2.run();
//!
//! ```

use core::cell::Cell;

use kernel::collections::list::{List, ListLink, ListNode};
use kernel::debug;
use kernel::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};
use kernel::hil::symmetric_encryption;
use kernel::hil::symmetric_encryption::{
    AES128Ctr, AES128, AES128CBC, AES128_BLOCK_SIZE, AES128_KEY_SIZE, CCM_NONCE_LENGTH,
};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

use crate::net::stream::SResult;
use crate::net::stream::{encode_bytes, encode_u16};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum CCMState {
    Idle,
    Auth,
    Encrypt,
}

// to cache up the function parameters of the crypt() function
struct CryptFunctionParameters {
    buf: &'static mut [u8],
    a_off: usize,
    m_off: usize,
    m_len: usize,
    mic_len: usize,
    confidential: bool,
    encrypting: bool,
}

impl CryptFunctionParameters {
    pub fn new(
        buf: &'static mut [u8],
        a_off: usize,
        m_off: usize,
        m_len: usize,
        mic_len: usize,
        confidential: bool,
        encrypting: bool,
    ) -> CryptFunctionParameters {
        CryptFunctionParameters {
            buf: buf,
            a_off: a_off,
            m_off: m_off,
            m_len: m_len,
            mic_len: mic_len,
            confidential: confidential,
            encrypting: encrypting,
        }
    }
}

pub struct MuxAES128CCM<'a, A: AES128<'a> + AES128Ctr + AES128CBC> {
    aes: &'a A,
    clients: List<'a, VirtualAES128CCM<'a, A>>,
    inflight: OptionalCell<&'a VirtualAES128CCM<'a, A>>,
    deferred_caller: &'a DynamicDeferredCall,
    handle: OptionalCell<DeferredCallHandle>,
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC> MuxAES128CCM<'a, A> {
    pub fn new(aes: &'a A, deferred_caller: &'a DynamicDeferredCall) -> MuxAES128CCM<'a, A> {
        aes.enable(); // enable the hardware, in case it's forgotten elsewhere
        MuxAES128CCM {
            aes: aes,
            clients: List::new(),
            inflight: OptionalCell::empty(),
            deferred_caller: deferred_caller,
            handle: OptionalCell::empty(),
        }
    }

    /// manually re-enable the hardware
    pub fn enable(&self) {
        self.aes.enable();
    }

    /// disable the underlying hardware
    pub fn disable(&self) {
        self.aes.disable();
    }

    /// inorder to receive callbacks correctly, please call
    /// ```rust
    /// mux.initialize_callback_handle(
    ///     dynamic_deferred_caller.register(mux)
    ///     .expect("no deferred call slot available for ccm mux")
    /// );
    /// ```
    /// after the creation of the mux
    pub fn initialize_callback_handle(&self, handle: DeferredCallHandle) {
        self.handle.replace(handle);
    }

    /// Asynchronously executes the next operation, if any. Used by calls
    /// to trigger do_next_op such that it will execute after the call
    /// returns.
    /// See virtual_uart::MuxUart<'a>::do_next_op_async
    fn do_next_op_async(&self) {
        self.handle.map(|handle| self.deferred_caller.set(*handle));
    }

    fn do_next_op(&self) {
        if self.inflight.is_none() {
            let mnode = self.clients.iter().find(|node| node.queued_up.is_some());
            mnode.map(|node| {
                self.inflight.set(node);
                let parameters: CryptFunctionParameters = node.queued_up.take().unwrap();
                // now, eat the parameters
                let _ = node.crypt_r(parameters).map_err(|(ecode, _)| {
                    // notice that we didn't put the parameters back...
                    // because it's already eaten
                    if node.crypt_client.is_none() {
                        debug!(
                            "virtual_aes_ccm: no crypt_client is registered in VirtualAES128CCM"
                        );
                    }
                    if node.buf.is_none() {
                        debug!("virtual_aes_ccm: no buffer is binded with VirtualAES128CCM");
                    }
                    // notify the client that there's a failure
                    node.buf.take().map(|buf| {
                        node.crypt_client.map(move |client| {
                            client.crypt_done(buf, Err(ecode), false);
                        });
                    });
                    // if it fails to trigger encryption, remove it and perform the next
                    node.remove_from_queue();
                    self.do_next_op();
                });
                // otherwise, wait for crypt_done
            });
        }
    }
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC> DynamicDeferredCallClient for MuxAES128CCM<'a, A> {
    fn call(&self, _handle: DeferredCallHandle) {
        self.do_next_op();
    }
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC> symmetric_encryption::Client<'a>
    for MuxAES128CCM<'a, A>
{
    fn crypt_done(&'a self, source: Option<&'a mut [u8]>, dest: &'a mut [u8]) {
        if self.inflight.is_none() {
            panic!("MuxAES128CCM: crypt_done is called but inflight is none!");
        }
        self.inflight.map(move |vaes_ccm| {
            // vaes_ccm.crypt_done might call additional start_ccm_crypt / start_ccm_auth
            // when the encryption is *really* done, inflight will be cleared by remove_from_queue
            // and it will call do_next_op to perform the next operation
            // self.do_next_op() will be called when the encryption is failed or is really done
            // search for self.crypt_client
            vaes_ccm.crypt_done(source, dest);
        });
    }
}

pub struct VirtualAES128CCM<'a, A: AES128<'a> + AES128Ctr + AES128CBC> {
    mux: &'a MuxAES128CCM<'a, A>,
    aes: &'a A,
    next: ListLink<'a, VirtualAES128CCM<'a, A>>,

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
    queued_up: OptionalCell<CryptFunctionParameters>,
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC> VirtualAES128CCM<'a, A> {
    pub fn new(
        mux: &'a MuxAES128CCM<'a, A>,
        crypt_buf: &'static mut [u8],
    ) -> VirtualAES128CCM<'a, A> {
        VirtualAES128CCM {
            mux: mux,
            aes: &mux.aes,
            next: ListLink::empty(),
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
            queued_up: OptionalCell::empty(),
        }
    }

    /// bind itself to self.mux, should be called after static_init!
    pub fn setup(&'a self) {
        self.mux.clients.push_head(self);
    }

    /// Prepares crypt_buf with the input for the CCM* authentication and
    /// encryption/decryption transformations. Returns NOMEM if crypt_buf is
    /// not present or if it is not long enough.
    fn prepare_ccm_buffer(
        &self,
        nonce: &[u8; CCM_NONCE_LENGTH],
        mic_len: usize,
        a_data: &[u8],
        m_data: &[u8],
    ) -> Result<(), ErrorCode> {
        self.crypt_buf.map_or(Err(ErrorCode::NOMEM), |cbuf| {
            let (auth_len, enc_len) =
                match Self::encode_ccm_buffer(cbuf, nonce, mic_len, a_data, m_data) {
                    SResult::Done(_, out) => out,
                    SResult::Needed(_) => {
                        return Err(ErrorCode::NOMEM);
                    }
                    SResult::Error(_) => {
                        return Err(ErrorCode::FAIL);
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
            Ok(())
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
    fn start_ccm_auth(&self) -> Result<(), ErrorCode> {
        if !(self.state.get() == CCMState::Idle)
            && !(self.state.get() == CCMState::Encrypt && self.reversed())
        {
            panic!("Called start_ccm_auth when not idle");
        }

        let iv = [0u8; AES128_BLOCK_SIZE];
        let res = self.aes.set_iv(&iv);
        if res != Ok(()) {
            return res;
        }
        let res = self.aes.set_key(&self.key.get());
        if res != Ok(()) {
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
        self.aes.set_mode_aes128cbc(true)?;
        self.aes.start_message();
        match self.aes.crypt(None, crypt_buf, 0, auth_end) {
            None => {
                self.state.set(CCMState::Auth);
                Ok(())
            }
            Some((res, _, crypt_buf)) => {
                // Request failed
                self.crypt_buf.replace(crypt_buf);
                res
            }
        }
    }

    fn start_ccm_encrypt(&self) -> Result<(), ErrorCode> {
        if !(self.state.get() == CCMState::Auth)
            && !(self.state.get() == CCMState::Idle && self.reversed())
        {
            return Err(ErrorCode::FAIL);
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
        if res != Ok(()) {
            return res;
        }

        self.aes.set_mode_aes128ctr(self.encrypting.get())?;
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
                Ok(())
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
        // encryption is successful
        self.state.set(CCMState::Idle);
        self.remove_from_queue();
        self.mux.do_next_op();
        self.crypt_client.map(|client| {
            self.buf.take().map(|buf| {
                client.crypt_done(buf, Ok(()), tag_valid);
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
        // encryption is successful
        self.state.set(CCMState::Idle);
        self.remove_from_queue();
        self.mux.do_next_op();
        self.crypt_client.map(|client| {
            self.buf.take().map(|buf| {
                client.crypt_done(buf, Ok(()), tag_valid);
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

    fn crypt_r(
        &self,
        parameter: CryptFunctionParameters,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // just expanding the parameters......
        let buf: &'static mut [u8] = parameter.buf;
        let a_off: usize = parameter.a_off;
        let m_off: usize = parameter.m_off;
        let m_len: usize = parameter.m_len;
        let mic_len: usize = parameter.mic_len;
        let confidential: bool = parameter.confidential;
        let encrypting: bool = parameter.encrypting;
        //
        if self.state.get() != CCMState::Idle {
            return Err((ErrorCode::BUSY, buf));
        }
        if !(a_off <= m_off && m_off + m_len + mic_len <= buf.len()) {
            return Err((ErrorCode::INVAL, buf));
        }

        self.confidential.set(confidential);
        self.encrypting.set(encrypting);

        let res = self.prepare_ccm_buffer(
            &self.nonce.get(),
            mic_len,
            &buf[a_off..m_off],
            &buf[m_off..m_off + m_len],
        );
        if res != Ok(()) {
            return Err((res.unwrap_err(), buf));
        }

        let res = if !confidential || encrypting {
            // Perform CBC before CTR
            self.start_ccm_auth()
        } else {
            // Perform CTR before CBC
            self.save_tag_block();
            self.start_ccm_encrypt()
        };

        if res != Ok(()) {
            Err((res.unwrap_err(), buf))
        } else {
            self.buf.replace(buf);
            self.pos.set((a_off, m_off, m_len, mic_len));
            Ok(())
        }
    }

    fn remove_from_queue(&self) {
        self.queued_up.clear();
        self.mux.inflight.clear();
    }
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC> symmetric_encryption::AES128CCM<'a>
    for VirtualAES128CCM<'a, A>
{
    fn set_client(&self, client: &'a dyn symmetric_encryption::CCMClient) {
        self.crypt_client.set(client);
    }

    fn set_key(&self, key: &[u8]) -> Result<(), ErrorCode> {
        if key.len() < AES128_KEY_SIZE {
            Err(ErrorCode::INVAL)
        } else {
            let mut new_key = [0u8; AES128_KEY_SIZE];
            new_key.copy_from_slice(key);
            self.key.set(new_key);
            Ok(())
        }
    }

    fn set_nonce(&self, nonce: &[u8]) -> Result<(), ErrorCode> {
        if nonce.len() < CCM_NONCE_LENGTH {
            Err(ErrorCode::INVAL)
        } else {
            let mut new_nonce = [0u8; CCM_NONCE_LENGTH];
            new_nonce.copy_from_slice(nonce);
            self.nonce.set(new_nonce);
            Ok(())
        }
    }
    /// Try to begin the encryption/decryption process
    fn crypt(
        &self,
        buf: &'static mut [u8],
        a_off: usize,
        m_off: usize,
        m_len: usize,
        mic_len: usize,
        confidential: bool,
        encrypting: bool,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.queued_up.is_some() {
            return Err((ErrorCode::BUSY, buf));
        }
        if self.state.get() != CCMState::Idle {
            return Err((ErrorCode::BUSY, buf));
        }
        if !(a_off <= m_off && m_off + m_len + mic_len <= buf.len()) {
            return Err((ErrorCode::INVAL, buf));
        }

        self.queued_up.set(CryptFunctionParameters::new(
            buf,
            a_off,
            m_off,
            m_len,
            mic_len,
            confidential,
            encrypting,
        ));
        self.mux.do_next_op_async();
        Ok(())
    }
}

impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC> symmetric_encryption::Client<'a>
    for VirtualAES128CCM<'a, A>
{
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
                    if res != Ok(()) {
                        // Return client buffer to client
                        self.buf.take().map(|buf| {
                            self.crypt_client.map(move |client| {
                                client.crypt_done(buf, res, false);
                            });
                        });
                        // The operation fails, immediately remove the request and perform the next operation
                        self.state.set(CCMState::Idle);
                        self.remove_from_queue();
                        self.mux.do_next_op();
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
                    if res != Ok(()) {
                        // Return client buffer to client
                        self.buf.take().map(|buf| {
                            self.crypt_client.map(move |client| {
                                client.crypt_done(buf, res, false);
                            });
                        });
                        // The operation fails, immediately remove the request and perform the next operation
                        self.state.set(CCMState::Idle);
                        self.remove_from_queue();
                        self.mux.do_next_op();
                    }
                }
            }
        }
    }
}

// Fit in the linked list
impl<'a, A: AES128<'a> + AES128Ctr + AES128CBC> ListNode<'a, VirtualAES128CCM<'a, A>>
    for VirtualAES128CCM<'a, A>
{
    fn next(&'a self) -> &'a ListLink<'a, VirtualAES128CCM<'a, A>> {
        &self.next
    }
}
