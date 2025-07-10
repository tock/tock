use core::cell::Cell;
use kernel::{
    collections::list::{List, ListLink, ListNode},
    hil::symmetric_encryption::{self, AES128_BLOCK_SIZE, AES128_KEY_SIZE},
    utilities::{
        cells::{MapCell, TakeCell},
        leasable_buffer::SubSliceMut,
    },
    ErrorCode,
};

use crate::aes;

// Wrapper for IV buffer.
//
// The IV buffer is needed for AES modes besides
// ECB. `IVBuf` denotes if the buffer has been
// passed to the pending AES work object.
enum IvBuf<'a> {
    NotProvided(TakeCell<'a, [u8; AES128_KEY_SIZE]>),
    Provided(TakeCell<'a, [u8; AES128_KEY_SIZE]>),
}

impl<'a> IvBuf<'a> {
    fn replace_buffer(&self, new_buf: &[u8]) -> Option<Self> {
        // get buffer length, we will pad with zeros if needed
        let new_buf_len = new_buf.len();
        let padded_buf = &mut [0u8; AES128_BLOCK_SIZE];
        padded_buf[..new_buf_len].copy_from_slice(new_buf);

        let buf = match self {
            IvBuf::NotProvided(take_cell) => take_cell.take(),
            IvBuf::Provided(take_cell) => take_cell.take(),
        }?;

        Some(IvBuf::Provided(TakeCell::new(buf)))
    }
}

// Work object for AES HW virtualizer.
// Contains all state needed to perform
// an AES operation on the underlying HW.
//
// This allows for crypto operations to
// be enqueued and executed later when
// the HW is available. The work object
// contains static buffers for the iv/key
// (copied into from caller) and receives
// static buffers from the caller for src/dest.
struct WorkAesHw<'a, T> {
    key: TakeCell<'a, [u8; AES128_KEY_SIZE]>,
    mode: MapCell<SetModeFn<'a, T>>,
    iv: IvBuf<'a>,
    buffers: MapCell<(Option<SubSliceMut<'static, u8>>, SubSliceMut<'static, u8>)>,
    client: MapCell<&'a dyn symmetric_encryption::Client<'a>>,
    ready: Cell<bool>,
}

impl<'a, T> WorkAesHw<'a, T> {
    fn new(key_buf: &'a mut [u8; AES128_KEY_SIZE], iv_buf: &'a mut [u8; AES128_KEY_SIZE]) -> Self {
        WorkAesHw {
            key: TakeCell::new(key_buf),
            mode: MapCell::empty(),
            iv: IvBuf::NotProvided(TakeCell::new(iv_buf)),
            buffers: MapCell::empty(),
            client: MapCell::empty(),
            ready: Cell::new(false),
        }
    }
}

impl<'a, A: symmetric_encryption::AES128<'a>> ListNode<'a, AesVirtualHw<'a, A>>
    for AesVirtualHw<'a, A>
{
    fn next(&'a self) -> &'a ListLink<'a, AesVirtualHw<'a, A>> {
        &self.next
    }
}

/// Virtualizer for AES HW implementations.
///
/// This provides an abstraction to for multiple clients
/// to use and share a single underlying AES hardware. This
/// virtualizer is not specific to a given AES mode.
pub struct AesVirtualHw<'a, A: symmetric_encryption::AES128<'a>> {
    aes_hw_mux: &'a AesHwMux<'a, A>,
    next: ListLink<'a, Self>,
    work: WorkAesHw<'a, A>,
}

impl<'a, A: symmetric_encryption::AES128<'a>> AesVirtualHw<'a, A> {
    pub fn new(
        aes_hw_mux: &'a AesHwMux<'a, A>,
        key_buf: &'a mut [u8; AES128_KEY_SIZE],
        iv_buf: &'a mut [u8; AES128_KEY_SIZE],
    ) -> Self {
        AesVirtualHw {
            aes_hw_mux,
            next: ListLink::empty(),
            work: WorkAesHw::new(key_buf, iv_buf),
        }
    }

    fn setup_work_config(
        &self,
        key: &[u8; AES128_KEY_SIZE],
        iv: Option<&[u8]>,
        fn_ptr: SetModeFn<'a, A>,
    ) -> Result<(), ErrorCode> {
        let work = &self.work;

        work.key.map_or_else(
            || Err(ErrorCode::BUSY),
            |worker_key| {
                worker_key.copy_from_slice(key);
                Ok(())
            },
        )?;

        work.mode.replace(fn_ptr);

        if let Some(iv) = iv {
            work.iv
                .replace_buffer(iv)
                .map_or(Err(ErrorCode::BUSY), |_| Ok(()))
        } else {
            Ok(())
        }
    }

    fn setup_work_buffers(
        &self,
        src: Option<SubSliceMut<'static, u8>>,
        dest: SubSliceMut<'static, u8>,
    ) {
        let work = &self.work;

        work.buffers.replace((src, dest));
    }
}

impl<'a, A: symmetric_encryption::AES128<'a>> kernel::hil::symmetric_encryption::Client<'a>
    for AesVirtualHw<'a, A>
{
    fn crypt_done(
        &'a self,
        source: Option<SubSliceMut<'static, u8>>,
        dest: Result<SubSliceMut<'static, u8>, (ErrorCode, SubSliceMut<'static, u8>)>,
    ) {
        self.work.client.map(|client| {
            client.crypt_done(source, dest);
        });
    }
}

// (todo) we potentially have unbounded linked list?
pub struct AesHwMux<'a, A: symmetric_encryption::AES128<'a>> {
    aes_hw: &'a A,
    virtual_aes_hw_list: List<'a, AesVirtualHw<'a, A>>,
    current_virtualhw_client: MapCell<&'a dyn symmetric_encryption::Client<'a>>,
}

impl<'a, A: symmetric_encryption::AES128<'a>> symmetric_encryption::Client<'a> for AesHwMux<'a, A> {
    fn crypt_done(
        &'a self,
        source: Option<SubSliceMut<'static, u8>>,
        dest: Result<SubSliceMut<'static, u8>, (ErrorCode, SubSliceMut<'static, u8>)>,
    ) {
        // We set the client of our AesHwMux to be the current AesVirtualHw that
        // is using the HW.
        self.current_virtualhw_client.map(|client| {
            client.crypt_done(source, dest);
        });

        // start next operation.
        self.do_next_op();
    }
}

// Type to represent a function pointer to configure the
// specific AES HW mode.
type SetModeFn<'a, T> = fn(&T) -> Result<(), ErrorCode>;

impl<'a, A: symmetric_encryption::AES128<'a>> AesHwMux<'a, A> {
    pub fn new(aes_hw: &'a A) -> Self {
        AesHwMux {
            aes_hw: aes_hw,
            virtual_aes_hw_list: List::new(),
            current_virtualhw_client: MapCell::empty(),
        }
    }

    fn do_next_op(&self) {
        // (todo) Potentially, a greedy first node in the list starves
        // others. This can be mitigated, but I think we can assume
        // that clients will be well-behaved.

        // Iterate through list and find first ready work item.
        self.virtual_aes_hw_list
            .iter()
            .find(|aes_virtual_hw| aes_virtual_hw.work.ready.get())
            .and_then(|aes_virtual_hw| {
                let work = &aes_virtual_hw.work;

                // Mark as not ready (we are servicing it now).
                work.ready.set(false);

                // Configure for needed HW mode (set hw into proper mode and set key/iv).
                work.key.map(|key| {
                    self.aes_hw.set_key(key);
                })?;

                // Take the buffers to be encrypted/decrypted.
                let (src, dest) = work.buffers.take()?;

                if let IvBuf::Provided(iv) = &work.iv {
                    if let Err(code) = iv.map(|iv| self.aes_hw.set_iv(iv))? {
                        // notify client of failure
                        work.client.map(|client| {
                            client.crypt_done(src, Err((code, dest)));
                        });

                        return None;
                    }
                }

                // Set AES HW into proper mode.
                let mode = work.mode.get()?;

                if let Err(err) = (mode)(self.aes_hw) {
                    // notify client of failure
                    work.client.map(|client| {
                        client.crypt_done(src, Err((err, dest)));
                    });

                    return None;
                }

                self.current_virtualhw_client.replace(aes_virtual_hw);

                // Start the operation.
                self.aes_hw.crypt(src, dest).map_or_else(
                    |(err, src, dest)| {
                        // notify client of failure
                        work.client.map(|client| {
                            client.crypt_done(src, Err((err, dest)));
                        });

                        None
                    },
                    |_| Some(()),
                )
            })
            .map_or_else(
                || {
                    // None indicates something went wrong (e.g. buffer not provided) or
                    // the crypt operation returned an error.
                    // (todo) error handling here
                    self.do_next_op();
                },
                |_| (),
            );
    }
}

// Interface for ECB Device
impl<'a, A: symmetric_encryption::AES128ECB<'a>> aes::Aes128Ecb<'a> for AesVirtualHw<'a, A> {
    fn crypt(
        &self,
        source: Option<SubSliceMut<'static, u8>>,
        dest: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<SubSliceMut<'static, u8>>,
            SubSliceMut<'static, u8>,
        ),
    > {
        self.setup_work_buffers(source, dest);
        self.work.ready.set(true);
        self.aes_hw_mux.do_next_op();
        Ok(())
    }

    fn set_client(&self, client: &'a dyn symmetric_encryption::Client<'a>) {
        self.work.client.replace(client);
    }

    fn setup_cipher(
        &self,
        key: &[u8; symmetric_encryption::AES128_KEY_SIZE],
    ) -> Result<(), ErrorCode> {
        self.setup_work_config(key, None, A::set_mode_aes128ecb)
    }
}

impl<'a, A: symmetric_encryption::AES128CBC<'a>> aes::Aes128Cbc<'a> for AesVirtualHw<'a, A> {
    fn crypt(
        &self,
        source: Option<SubSliceMut<'static, u8>>,
        dest: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<SubSliceMut<'static, u8>>,
            SubSliceMut<'static, u8>,
        ),
    > {
        self.setup_work_buffers(source, dest);
        self.work.ready.set(true);
        self.aes_hw_mux.do_next_op();
        Ok(())
    }

    fn set_client(&self, client: &'a dyn symmetric_encryption::Client<'a>) {
        self.work.client.replace(client);
    }

    fn setup_cipher(
        &self,
        key: &[u8; symmetric_encryption::AES128_KEY_SIZE],
        _iv: &[u8; symmetric_encryption::AES128_BLOCK_SIZE],
    ) -> Result<(), ErrorCode> {
        self.setup_work_config(key, None, A::set_mode_aes128cbc)
    }
}

impl<'a, A: symmetric_encryption::AES128Ctr<'a>> aes::Aes128Ctr<'a> for AesVirtualHw<'a, A> {
    fn crypt(
        &self,
        source: Option<SubSliceMut<'static, u8>>,
        dest: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<SubSliceMut<'static, u8>>,
            SubSliceMut<'static, u8>,
        ),
    > {
        self.setup_work_buffers(source, dest);
        self.work.ready.set(true);
        self.aes_hw_mux.do_next_op();
        Ok(())
    }

    fn set_client(&self, client: &'a dyn symmetric_encryption::Client<'a>) {
        self.work.client.replace(client);
    }

    fn setup_cipher(
        &self,
        key: &[u8; symmetric_encryption::AES128_KEY_SIZE],
        iv: &[u8],
    ) -> Result<(), ErrorCode> {
        // Check that IV is less than 16 and greater than [..] (TODO)
        let iv_len = iv.len();
        if iv_len < 13 || iv_len > 16 {
            return Err(ErrorCode::INVAL);
        }

        self.setup_work_config(key, Some(iv), A::set_mode_aes128ctr)
    }
}

impl<'a, A: symmetric_encryption::AES128CCM<'a>> aes::Aes128Ccm<'a> for AesVirtualHw<'a, A> {
    fn crypt(
        &self,
        source: Option<SubSliceMut<'static, u8>>,
        dest: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<SubSliceMut<'static, u8>>,
            SubSliceMut<'static, u8>,
        ),
    > {
        self.setup_work_buffers(source, dest);
        self.work.ready.set(true);
        self.aes_hw_mux.do_next_op();
        Ok(())
    }

    fn set_client(&self, client: &'a dyn symmetric_encryption::Client<'a>) {
        self.work.client.replace(client);
    }

    fn setup_cipher(
        &self,
        key: &[u8; symmetric_encryption::AES128_KEY_SIZE],
        iv: &[u8],
    ) -> Result<(), ErrorCode> {
        // Check that 7 <= iv_len <= 13
        let iv_len = iv.len();
        if iv_len < 7 || iv_len > 13 {
            return Err(ErrorCode::INVAL);
        }

        self.setup_work_config(key, Some(iv), A::set_mode_aes128ccm)
    }
}

impl<'a, A: symmetric_encryption::AES128GCM<'a>> aes::Aes128Gcm<'a> for AesVirtualHw<'a, A> {
    fn crypt(
        &self,
        source: Option<SubSliceMut<'static, u8>>,
        dest: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<SubSliceMut<'static, u8>>,
            SubSliceMut<'static, u8>,
        ),
    > {
        self.setup_work_buffers(source, dest);
        self.work.ready.set(true);
        self.aes_hw_mux.do_next_op();
        Ok(())
    }

    fn set_client(&self, client: &'a dyn symmetric_encryption::Client<'a>) {
        self.work.client.replace(client);
    }

    fn setup_cipher(&self, key: &[u8; 16], iv: &[u8; 12]) -> Result<(), ErrorCode> {
        self.setup_work_config(key, Some(iv), A::set_mode_aes128gcm)
    }
}
