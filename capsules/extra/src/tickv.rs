//! Tock TicKV capsule.
//!
//! This capsule implements the TicKV library in Tock. This is done
//! using the TicKV library (libraries/tickv).
//!
//! This capsule interfaces with flash and exposes the Tock `hil::kv_system`
//! interface to others.
//!
//! ```
//! +-----------------------+
//! |                       |
//! |  Capsule using K-V    |
//! |                       |
//! +-----------------------+
//!
//!    hil::kv_store
//!
//! +-----------------------+
//! |                       |
//! |  K-V in Tock          |
//! |                       |
//! +-----------------------+
//!
//!    hil::kv_system
//!
//! +-----------------------+
//! |                       |
//! |  TicKV (this file)    |
//! |                       |
//! +-----------------------+
//!
//!    hil::flash
//! ```

use core::cell::Cell;
use kernel::hil::flash::{self, Flash};
use kernel::hil::hasher::{self, Hasher};
use kernel::hil::kv_system::{self, KVSystem};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::LeasableMutableBuffer;
use kernel::ErrorCode;
use tickv::{self, AsyncTicKV};

#[derive(Clone, Copy, PartialEq)]
enum Operation {
    None,
    Init,
    GetKey,
    AppendKey,
    InvalidateKey,
    GarbageCollect,
}

pub struct TickFSFlashCtrl<'a, F: Flash + 'static> {
    flash: &'a F,
    flash_read_buffer: TakeCell<'static, F::Page>,
    region_offset: usize,
}

impl<'a, F: Flash> TickFSFlashCtrl<'a, F> {
    pub fn new(
        flash: &'a F,
        flash_read_buffer: &'static mut F::Page,
        region_offset: usize,
    ) -> TickFSFlashCtrl<'a, F> {
        Self {
            flash,
            flash_read_buffer: TakeCell::new(flash_read_buffer),
            region_offset,
        }
    }
}

impl<'a, F: Flash> tickv::flash_controller::FlashController<2048> for TickFSFlashCtrl<'a, F> {
    fn read_region(
        &self,
        region_number: usize,
        _offset: usize,
        _buf: &mut [u8; 2048],
    ) -> Result<(), tickv::error_codes::ErrorCode> {
        if self
            .flash
            .read_page(
                self.region_offset + region_number,
                self.flash_read_buffer.take().unwrap(),
            )
            .is_err()
        {
            Err(tickv::error_codes::ErrorCode::ReadFail)
        } else {
            Err(tickv::error_codes::ErrorCode::ReadNotReady(region_number))
        }
    }

    fn write(&self, address: usize, buf: &[u8]) -> Result<(), tickv::error_codes::ErrorCode> {
        let data_buf = self.flash_read_buffer.take().unwrap();

        for (i, d) in buf.iter().enumerate() {
            data_buf.as_mut()[i + (address % 2048)] = *d;
        }

        if self
            .flash
            .write_page(self.region_offset + (address / 2048), data_buf)
            .is_err()
        {
            return Err(tickv::error_codes::ErrorCode::WriteFail);
        }

        Err(tickv::error_codes::ErrorCode::WriteNotReady(address))
    }

    fn erase_region(&self, region_number: usize) -> Result<(), tickv::error_codes::ErrorCode> {
        let _ = self.flash.erase_page(self.region_offset + region_number);

        Err(tickv::error_codes::ErrorCode::EraseNotReady(region_number))
    }
}

pub type TicKVKeyType = [u8; 8];

pub struct TicKVStore<'a, F: Flash + 'static, H: Hasher<'a, 8>> {
    tickv: AsyncTicKV<'a, TickFSFlashCtrl<'a, F>, 2048>,
    hasher: &'a H,
    operation: Cell<Operation>,
    next_operation: Cell<Operation>,

    value_buffer: Cell<Option<&'static mut [u8]>>,
    key_buffer: TakeCell<'static, [u8; 8]>,
    ret_buffer: TakeCell<'static, [u8]>,
    unhashed_key_buf: TakeCell<'static, [u8]>,
    key_buf: TakeCell<'static, [u8; 8]>,

    client: OptionalCell<&'a dyn kv_system::Client<TicKVKeyType>>,
}

impl<'a, F: Flash, H: Hasher<'a, 8>> TicKVStore<'a, F, H> {
    pub fn new(
        flash: &'a F,
        hasher: &'a H,
        tickfs_read_buf: &'static mut [u8; 2048],
        flash_read_buffer: &'static mut F::Page,
        region_offset: usize,
        flash_size: usize,
    ) -> TicKVStore<'a, F, H> {
        let tickv = AsyncTicKV::<TickFSFlashCtrl<F>, 2048>::new(
            TickFSFlashCtrl::new(flash, flash_read_buffer, region_offset),
            tickfs_read_buf,
            flash_size,
        );

        Self {
            tickv,
            hasher,
            operation: Cell::new(Operation::None),
            next_operation: Cell::new(Operation::None),
            value_buffer: Cell::new(None),
            key_buffer: TakeCell::empty(),
            ret_buffer: TakeCell::empty(),
            unhashed_key_buf: TakeCell::empty(),
            key_buf: TakeCell::empty(),
            client: OptionalCell::empty(),
        }
    }

    pub fn initialise(&self) {
        let _ret = self.tickv.initialise(0x7bc9f7ff4f76f244);
        self.operation.set(Operation::Init);
    }

    fn complete_init(&self) {
        self.operation.set(Operation::None);
        match self.next_operation.get() {
            Operation::None | Operation::Init => {}
            Operation::AppendKey => {
                match self.append_key(
                    self.key_buffer.take().unwrap(),
                    self.value_buffer.take().unwrap(),
                ) {
                    Err((key, value, error)) => {
                        self.client.map(move |cb| {
                            cb.append_key_complete(error, key, value);
                        });
                    }
                    _ => {}
                }
            }
            Operation::GetKey => {
                match self.get_value(
                    self.key_buffer.take().unwrap(),
                    self.ret_buffer.take().unwrap(),
                ) {
                    Err((key, ret_buf, error)) => {
                        self.client.map(move |cb| {
                            cb.get_value_complete(error, key, ret_buf);
                        });
                    }
                    _ => {}
                }
            }
            Operation::InvalidateKey => {
                match self.invalidate_key(self.key_buffer.take().unwrap()) {
                    Err((key, error)) => {
                        self.client.map(move |cb| {
                            cb.invalidate_key_complete(error, key);
                        });
                    }
                    _ => {}
                }
            }
            Operation::GarbageCollect => match self.garbage_collect() {
                Err(error) => {
                    self.client.map(move |cb| {
                        cb.garbage_collect_complete(error);
                    });
                }
                _ => {}
            },
        }
        self.next_operation.set(Operation::None);
    }
}

impl<'a, F: Flash, H: Hasher<'a, 8>> hasher::Client<8> for TicKVStore<'a, F, H> {
    fn add_mut_data_done(&self, _result: Result<(), ErrorCode>, data: &'static mut [u8]) {
        self.unhashed_key_buf.replace(data);
        self.hasher.run(self.key_buf.take().unwrap()).unwrap();
    }

    fn add_data_done(&self, _result: Result<(), ErrorCode>, _data: &'static [u8]) {}

    fn hash_done(&self, _result: Result<(), ErrorCode>, digest: &'static mut [u8; 8]) {
        self.client.map(move |cb| {
            cb.generate_key_complete(Ok(()), self.unhashed_key_buf.take().unwrap(), digest);
        });

        self.hasher.clear_data();
    }
}

impl<'a, F: Flash, H: Hasher<'a, 8>> flash::Client<F> for TicKVStore<'a, F, H> {
    fn read_complete(&self, pagebuffer: &'static mut F::Page, _error: flash::Error) {
        self.tickv.set_read_buffer(pagebuffer.as_mut());
        self.tickv
            .tickv
            .controller
            .flash_read_buffer
            .replace(pagebuffer);
        let (ret, buf_buffer) = self.tickv.continue_operation();

        buf_buffer.map(|buf| {
            self.ret_buffer.replace(buf);
        });

        match self.operation.get() {
            Operation::Init => match ret {
                Ok(tickv::success_codes::SuccessCode::Complete)
                | Ok(tickv::success_codes::SuccessCode::Written) => {
                    self.operation.set(Operation::None)
                }
                _ => {}
            },
            Operation::GetKey => match ret {
                Ok(tickv::success_codes::SuccessCode::Complete)
                | Ok(tickv::success_codes::SuccessCode::Written) => {
                    self.operation.set(Operation::None);
                    self.client.map(|cb| {
                        cb.get_value_complete(
                            Ok(()),
                            self.key_buffer.take().unwrap(),
                            self.ret_buffer.take().unwrap(),
                        );
                    });
                }
                Err(tickv::error_codes::ErrorCode::EraseNotReady(_)) | Ok(_) => {}
                _ => {
                    self.operation.set(Operation::None);
                    self.client.map(|cb| {
                        cb.get_value_complete(
                            Err(ErrorCode::FAIL),
                            self.key_buffer.take().unwrap(),
                            self.ret_buffer.take().unwrap(),
                        );
                    });
                }
            },
            Operation::AppendKey => match ret {
                Ok(tickv::success_codes::SuccessCode::Complete)
                | Ok(tickv::success_codes::SuccessCode::Written) => {
                    self.operation.set(Operation::None);
                }
                _ => {}
            },
            Operation::InvalidateKey => match ret {
                Ok(tickv::success_codes::SuccessCode::Complete)
                | Ok(tickv::success_codes::SuccessCode::Written) => {
                    self.operation.set(Operation::None);
                }
                _ => {}
            },
            Operation::GarbageCollect => match ret {
                Ok(tickv::success_codes::SuccessCode::Complete)
                | Ok(tickv::success_codes::SuccessCode::Written) => {
                    self.operation.set(Operation::None);
                    self.client.map(|cb| {
                        cb.garbage_collect_complete(Ok(()));
                    });
                }
                _ => {}
            },
            _ => unreachable!(),
        }
    }

    fn write_complete(&self, pagebuffer: &'static mut F::Page, _error: flash::Error) {
        self.tickv
            .tickv
            .controller
            .flash_read_buffer
            .replace(pagebuffer);

        match self.operation.get() {
            Operation::Init => {
                self.complete_init();
            }
            Operation::AppendKey => {
                self.operation.set(Operation::None);
                self.client.map(|cb| {
                    cb.append_key_complete(
                        Ok(()),
                        self.key_buffer.take().unwrap(),
                        self.tickv.get_stored_value_buffer().unwrap(),
                    );
                });
            }
            Operation::InvalidateKey => {
                self.operation.set(Operation::None);
                self.client.map(|cb| {
                    cb.invalidate_key_complete(Ok(()), self.key_buffer.take().unwrap());
                });
            }
            _ => unreachable!(),
        }
    }

    fn erase_complete(&self, _error: flash::Error) {
        let (ret, buf_buffer) = self.tickv.continue_operation();

        buf_buffer.map(|buf| {
            self.ret_buffer.replace(buf);
        });

        match self.operation.get() {
            Operation::Init => match ret {
                Ok(tickv::success_codes::SuccessCode::Complete)
                | Ok(tickv::success_codes::SuccessCode::Written) => {
                    self.complete_init();
                }
                _ => {}
            },
            Operation::GarbageCollect => match ret {
                Ok(tickv::success_codes::SuccessCode::Complete)
                | Ok(tickv::success_codes::SuccessCode::Written) => {
                    self.operation.set(Operation::None);
                    self.client.map(|cb| {
                        cb.garbage_collect_complete(Ok(()));
                    });
                }
                _ => {}
            },
            _ => unreachable!(),
        }
    }
}

impl<'a, F: Flash, H: Hasher<'a, 8>> KVSystem<'a> for TicKVStore<'a, F, H> {
    type K = TicKVKeyType;

    fn set_client(&self, client: &'a dyn kv_system::Client<Self::K>) {
        self.client.set(client);
    }

    fn generate_key(
        &self,
        unhashed_key: &'static mut [u8],
        key_buf: &'static mut Self::K,
    ) -> Result<
        (),
        (
            &'static mut [u8],
            &'static mut Self::K,
            Result<(), ErrorCode>,
        ),
    > {
        if let Err((e, buf)) = self
            .hasher
            .add_mut_data(LeasableMutableBuffer::new(unhashed_key))
        {
            return Err((buf, key_buf, Err(e)));
        }

        self.key_buf.replace(key_buf);

        Ok(())
    }

    fn append_key(
        &self,
        key: &'static mut Self::K,
        value: &'static mut [u8],
    ) -> Result<
        (),
        (
            &'static mut [u8; 8],
            &'static mut [u8],
            Result<(), kernel::ErrorCode>,
        ),
    > {
        match self.operation.get() {
            Operation::None => {
                self.operation.set(Operation::AppendKey);

                match self.tickv.append_key(u64::from_le_bytes(*key), value) {
                    Ok(_ret) => {
                        self.key_buffer.replace(key);
                        Ok(())
                    }
                    Err((buf, e)) => match e {
                        tickv::error_codes::ErrorCode::ReadNotReady(_)
                        | tickv::error_codes::ErrorCode::WriteNotReady(_) => {
                            self.key_buffer.replace(key);
                            Ok(())
                        }
                        _ => Err((key, buf.unwrap(), Err(ErrorCode::FAIL))),
                    },
                }
            }
            Operation::Init => {
                // The init process is still occurring.
                // We can save this request and start it after init
                self.next_operation.set(Operation::AppendKey);
                self.key_buffer.replace(key);
                self.value_buffer.replace(Some(value));
                Ok(())
            }
            _ => {
                // An operation is already in process.
                Err((key, value, Err(ErrorCode::BUSY)))
            }
        }
    }

    fn get_value(
        &self,
        key: &'static mut Self::K,
        ret_buf: &'static mut [u8],
    ) -> Result<
        (),
        (
            &'static mut Self::K,
            &'static mut [u8],
            Result<(), ErrorCode>,
        ),
    > {
        match self.operation.get() {
            Operation::None => {
                self.operation.set(Operation::GetKey);

                match self.tickv.get_key(u64::from_le_bytes(*key), ret_buf) {
                    Ok(_ret) => {
                        self.key_buffer.replace(key);
                        Ok(())
                    }
                    Err((buf, e)) => match e {
                        tickv::error_codes::ErrorCode::ReadNotReady(_)
                        | tickv::error_codes::ErrorCode::WriteNotReady(_) => {
                            self.key_buffer.replace(key);
                            Ok(())
                        }
                        _ => Err((key, buf.unwrap(), Err(ErrorCode::FAIL))),
                    },
                }
            }
            Operation::Init => {
                // The init process is still occurring.
                // We can save this request and start it after init
                self.next_operation.set(Operation::GetKey);
                self.key_buffer.replace(key);
                self.ret_buffer.replace(ret_buf);
                Ok(())
            }
            _ => {
                // An operation is already in process.
                Err((key, ret_buf, Err(ErrorCode::BUSY)))
            }
        }
    }

    fn invalidate_key(
        &self,
        key: &'static mut Self::K,
    ) -> Result<(), (&'static mut Self::K, Result<(), ErrorCode>)> {
        match self.operation.get() {
            Operation::None => {
                self.operation.set(Operation::InvalidateKey);

                match self.tickv.invalidate_key(u64::from_le_bytes(*key)) {
                    Ok(_ret) => {
                        self.key_buffer.replace(key);
                        Ok(())
                    }
                    Err(e) => match e {
                        tickv::error_codes::ErrorCode::ReadNotReady(_)
                        | tickv::error_codes::ErrorCode::WriteNotReady(_) => {
                            self.key_buffer.replace(key);
                            Ok(())
                        }
                        _ => Err((key, Err(ErrorCode::FAIL))),
                    },
                }
            }
            Operation::Init => {
                // The init process is still occurring.
                // We can save this request and start it after init
                self.next_operation.set(Operation::InvalidateKey);
                self.key_buffer.replace(key);
                Ok(())
            }
            _ => {
                // An operation is already in process.
                Err((key, Err(ErrorCode::BUSY)))
            }
        }
    }

    fn garbage_collect(&self) -> Result<usize, Result<(), ErrorCode>> {
        match self.operation.get() {
            Operation::None => {
                self.operation.set(Operation::GarbageCollect);

                match self.tickv.garbage_collect() {
                    Ok(freed) => Ok(freed),
                    Err(e) => match e {
                        tickv::error_codes::ErrorCode::ReadNotReady(_)
                        | tickv::error_codes::ErrorCode::WriteNotReady(_) => Ok(0),
                        _ => Err(Err(ErrorCode::FAIL)),
                    },
                }
            }
            Operation::Init => {
                // The init process is still occurring.
                // We can save this request and start it after init
                self.next_operation.set(Operation::GarbageCollect);
                Ok(0)
            }
            _ => {
                // An operation is already in process.
                Err(Err(ErrorCode::BUSY))
            }
        }
    }
}
