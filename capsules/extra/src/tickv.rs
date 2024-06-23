// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock TicKV capsule.
//!
//! This capsule implements the TicKV library in Tock. This is done using the
//! TicKV library (libraries/tickv).
//!
//! This capsule interfaces with flash and exposes the Tock `tickv::kv_system`
//! interface to others.
//!
//! ```text
//! +-----------------------+
//! |  Capsule using K-V    |
//! +-----------------------+
//!
//!    hil::kv::KV
//!
//! +-----------------------+
//! |  TickVKVStore         |
//! +-----------------------+
//!
//!    capsules::tickv::KVSystem
//!
//! +-----------------------+
//! |  TicKV (this file)    |
//! +-----------------------+
//!       |             |
//!   hil::flash        |
//!               +-----------------+
//!               | libraries/tickv |
//!               +-----------------+
//! ```

use core::cell::Cell;
use kernel::hil::flash::{self, Flash};
use kernel::hil::hasher::{self, Hasher};
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::{SubSlice, SubSliceMut};
use kernel::ErrorCode;
use tickv::AsyncTicKV;

/// The type of keys, this should define the output size of the digest
/// operations.
pub trait KeyType: Eq + Copy + Clone + Sized + AsRef<[u8]> + AsMut<[u8]> {}

impl KeyType for [u8; 8] {}

/// Implement this trait and use `set_client()` in order to receive callbacks.
pub trait KVSystemClient<K: KeyType> {
    /// This callback is called when the append_key operation completes.
    ///
    /// - `result`: Nothing on success, 'ErrorCode' on error
    /// - `unhashed_key`: The unhashed_key buffer
    /// - `key_buf`: The key_buf buffer
    fn generate_key_complete(
        &self,
        result: Result<(), ErrorCode>,
        unhashed_key: SubSliceMut<'static, u8>,
        key_buf: &'static mut K,
    );

    /// This callback is called when the append_key operation completes.
    ///
    /// - `result`: Nothing on success, 'ErrorCode' on error
    /// - `key`: The key buffer
    /// - `value`: The value buffer
    fn append_key_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut K,
        value: SubSliceMut<'static, u8>,
    );

    /// This callback is called when the get_value operation completes.
    ///
    /// - `result`: Nothing on success, 'ErrorCode' on error
    /// - `key`: The key buffer
    /// - `ret_buf`: The ret_buf buffer
    fn get_value_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut K,
        ret_buf: SubSliceMut<'static, u8>,
    );

    /// This callback is called when the invalidate_key operation completes.
    ///
    /// - `result`: Nothing on success, 'ErrorCode' on error
    /// - `key`: The key buffer
    fn invalidate_key_complete(&self, result: Result<(), ErrorCode>, key: &'static mut K);

    /// This callback is called when the garbage_collect operation completes.
    ///
    /// - `result`: Nothing on success, 'ErrorCode' on error
    fn garbage_collect_complete(&self, result: Result<(), ErrorCode>);
}

pub trait KVSystem<'a> {
    /// The type of the hashed key. For example `[u8; 8]`.
    type K: KeyType;

    /// Set the client.
    fn set_client(&self, client: &'a dyn KVSystemClient<Self::K>);

    /// Generate key.
    ///
    /// - `unhashed_key`: A unhashed key that should be hashed.
    /// - `key_buf`: A buffer to store the hashed key output.
    ///
    /// On success returns nothing.
    /// On error the unhashed_key, key_buf and `Result<(), ErrorCode>` will be returned.
    fn generate_key(
        &self,
        unhashed_key: SubSliceMut<'static, u8>,
        key_buf: &'static mut Self::K,
    ) -> Result<(), (SubSliceMut<'static, u8>, &'static mut Self::K, ErrorCode)>;

    /// Appends the key/value pair.
    ///
    /// If the key already exists in the store and has not been invalidated then
    /// the append operation will fail. To update an existing key to a new value
    /// the key must first be invalidated.
    ///
    /// - `key`: A hashed key. This key will be used in future to retrieve
    ///          or remove the `value`.
    /// - `value`: A buffer containing the data to be stored to flash.
    ///
    /// On success nothing will be returned.
    /// On error the key, value and a `Result<(), ErrorCode>` will be returned.
    ///
    /// The possible `Result<(), ErrorCode>`s are:
    /// - `BUSY`: An operation is already in progress
    /// - `INVAL`: An invalid parameter was passed
    /// - `NODEVICE`: No KV store was setup
    /// - `NOSUPPORT`: The key could not be added due to a collision.
    /// - `NOMEM`: The key could not be added due to no more space.
    fn append_key(
        &self,
        key: &'static mut Self::K,
        value: SubSliceMut<'static, u8>,
    ) -> Result<(), (&'static mut Self::K, SubSliceMut<'static, u8>, ErrorCode)>;

    /// Retrieves the value from a specified key.
    ///
    /// - `key`: A hashed key. This key will be used to retrieve the `value`.
    /// - `ret_buf`: A buffer to store the value to.
    ///
    /// On success nothing will be returned.
    /// On error the key, ret_buf and a `Result<(), ErrorCode>` will be returned.
    ///
    /// The possible `Result<(), ErrorCode>`s are:
    /// - `BUSY`: An operation is already in progress
    /// - `INVAL`: An invalid parameter was passed
    /// - `NODEVICE`: No KV store was setup
    /// - `ENOSUPPORT`: The key could not be found.
    /// - `SIZE`: The value is longer than the provided buffer.
    fn get_value(
        &self,
        key: &'static mut Self::K,
        ret_buf: SubSliceMut<'static, u8>,
    ) -> Result<(), (&'static mut Self::K, SubSliceMut<'static, u8>, ErrorCode)>;

    /// Invalidates the key in flash storage.
    ///
    /// - `key`: A hashed key. This key will be used to remove the `value`.
    ///
    /// On success nothing will be returned.
    /// On error the key and a `Result<(), ErrorCode>` will be returned.
    ///
    /// The possible `Result<(), ErrorCode>`s are:
    /// - `BUSY`: An operation is already in progress
    /// - `INVAL`: An invalid parameter was passed
    /// - `NODEVICE`: No KV store was setup
    /// - `ENOSUPPORT`: The key could not be found.
    fn invalidate_key(
        &self,
        key: &'static mut Self::K,
    ) -> Result<(), (&'static mut Self::K, ErrorCode)>;

    /// Perform a garbage collection on the KV Store.
    ///
    /// For implementations that don't require garbage collecting this should
    /// return `Err(ErrorCode::ALREADY)`.
    ///
    /// On success nothing will be returned.
    /// On error a `Result<(), ErrorCode>` will be returned.
    ///
    /// The possible `ErrorCode`s are:
    /// - `BUSY`: An operation is already in progress.
    /// - `ALREADY`: Nothing to be done. Callback will not trigger.
    /// - `INVAL`: An invalid parameter was passed.
    /// - `NODEVICE`: No KV store was setup.
    fn garbage_collect(&self) -> Result<(), ErrorCode>;
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Operation {
    None,
    Init,
    GetKey,
    AppendKey,
    InvalidateKey,
    GarbageCollect,
}

/// Wrapper object that provides the flash interface TicKV expects using the
/// Tock flash HIL.
///
/// Note, TicKV expects a synchronous flash implementation, but the Tock flash
/// HIL is asynchronous. To mediate this, this wrapper starts a flash
/// read/write/erase, but returns without the requested operation having
/// completed. To signal TicKV that this is what happened, this implementation
/// returns `NotReady` errors. When the underlying flash operation has completed
/// the `TicKVSystem` object will get the callback and then notify TicKV that
/// the requested operation is now ready.
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

impl<'a, F: Flash, const PAGE_SIZE: usize> tickv::flash_controller::FlashController<PAGE_SIZE>
    for TickFSFlashCtrl<'a, F>
{
    fn read_region(
        &self,
        region_number: usize,
        _offset: usize,
        _buf: &mut [u8; PAGE_SIZE],
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
            data_buf.as_mut()[i + (address % PAGE_SIZE)] = *d;
        }

        if self
            .flash
            .write_page(self.region_offset + (address / PAGE_SIZE), data_buf)
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

/// `TicKVSystem` implements `KVSystem` using the TicKV library.
pub struct TicKVSystem<'a, F: Flash + 'static, H: Hasher<'a, 8>, const PAGE_SIZE: usize> {
    /// Underlying asynchronous TicKV implementation.
    tickv: AsyncTicKV<'a, TickFSFlashCtrl<'a, F>, PAGE_SIZE>,
    /// Hash engine that converts key strings to 8 byte keys.
    hasher: &'a H,
    /// Track our internal asynchronous state machine.
    operation: Cell<Operation>,
    /// The operation to run _after_ initialization has completed.
    next_operation: Cell<Operation>,
    /// Holder for the key string passed from the caller until the operation
    /// completes.
    unhashed_key_buffer: MapCell<SubSliceMut<'static, u8>>,
    /// Holder for the hashed key used in the given operation.
    key_buffer: TakeCell<'static, [u8; 8]>,
    /// Holder for a buffer containing a value being read from or written to the
    /// key-value store.
    value_buffer: MapCell<SubSliceMut<'static, u8>>,
    /// Callback client when the `KVSystem` operation completes.
    client: OptionalCell<&'a dyn KVSystemClient<TicKVKeyType>>,
}

impl<'a, F: Flash, H: Hasher<'a, 8>, const PAGE_SIZE: usize> TicKVSystem<'a, F, H, PAGE_SIZE> {
    pub fn new(
        flash: &'a F,
        hasher: &'a H,
        tickfs_read_buf: &'static mut [u8; PAGE_SIZE],
        flash_read_buffer: &'static mut F::Page,
        region_offset: usize,
        flash_size: usize,
    ) -> TicKVSystem<'a, F, H, PAGE_SIZE> {
        let tickv = AsyncTicKV::<TickFSFlashCtrl<F>, PAGE_SIZE>::new(
            TickFSFlashCtrl::new(flash, flash_read_buffer, region_offset),
            tickfs_read_buf,
            flash_size,
        );

        Self {
            tickv,
            hasher,
            operation: Cell::new(Operation::None),
            next_operation: Cell::new(Operation::None),
            unhashed_key_buffer: MapCell::empty(),
            key_buffer: TakeCell::empty(),
            value_buffer: MapCell::empty(),
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
            Operation::GetKey => {
                match self.get_value(
                    self.key_buffer.take().unwrap(),
                    self.value_buffer.take().unwrap(),
                ) {
                    Err((key, value, error)) => {
                        self.client.map(move |cb| {
                            cb.get_value_complete(Err(error), key, value);
                        });
                    }
                    _ => {}
                }
            }
            Operation::AppendKey => {
                match self.append_key(
                    self.key_buffer.take().unwrap(),
                    self.value_buffer.take().unwrap(),
                ) {
                    Err((key, value, error)) => {
                        self.client.map(move |cb| {
                            cb.append_key_complete(Err(error), key, value);
                        });
                    }
                    _ => {}
                }
            }
            Operation::InvalidateKey => {
                match self.invalidate_key(self.key_buffer.take().unwrap()) {
                    Err((key, error)) => {
                        self.client.map(move |cb| {
                            cb.invalidate_key_complete(Err(error), key);
                        });
                    }
                    _ => {}
                }
            }
            Operation::GarbageCollect => match self.garbage_collect() {
                Err(error) => {
                    self.client.map(move |cb| {
                        cb.garbage_collect_complete(Err(error));
                    });
                }
                _ => {}
            },
        }
        self.next_operation.set(Operation::None);
    }
}

impl<'a, F: Flash, H: Hasher<'a, 8>, const PAGE_SIZE: usize> hasher::Client<8>
    for TicKVSystem<'a, F, H, PAGE_SIZE>
{
    fn add_mut_data_done(&self, _result: Result<(), ErrorCode>, data: SubSliceMut<'static, u8>) {
        self.unhashed_key_buffer.replace(data);
        self.hasher.run(self.key_buffer.take().unwrap()).unwrap();
    }

    fn add_data_done(&self, _result: Result<(), ErrorCode>, _data: SubSlice<'static, u8>) {}

    fn hash_done(&self, _result: Result<(), ErrorCode>, digest: &'static mut [u8; 8]) {
        self.client.map(move |cb| {
            cb.generate_key_complete(Ok(()), self.unhashed_key_buffer.take().unwrap(), digest);
        });

        self.hasher.clear_data();
    }
}

impl<'a, F: Flash, H: Hasher<'a, 8>, const PAGE_SIZE: usize> flash::Client<F>
    for TicKVSystem<'a, F, H, PAGE_SIZE>
{
    fn read_complete(&self, pagebuffer: &'static mut F::Page, _result: Result<(), flash::Error>) {
        self.tickv.set_read_buffer(pagebuffer.as_mut());
        self.tickv
            .tickv
            .controller
            .flash_read_buffer
            .replace(pagebuffer);
        let (ret, tickv_buf, tickv_buf_len) = self.tickv.continue_operation();

        // If we got the buffer back from TicKV then store it.
        tickv_buf.map(|buf| {
            let mut val_buf = SubSliceMut::new(buf);
            if tickv_buf_len > 0 {
                // Length of zero means nothing was inserted into the buffer so
                // no need to slice it.
                val_buf.slice(0..tickv_buf_len);
            }
            self.value_buffer.replace(val_buf);
        });

        match self.operation.get() {
            Operation::Init => match ret {
                Ok(tickv::success_codes::SuccessCode::Complete)
                | Ok(tickv::success_codes::SuccessCode::Written) => {
                    self.complete_init();
                }
                _ => {}
            },
            Operation::GetKey => {
                match ret {
                    Ok(tickv::success_codes::SuccessCode::Complete)
                    | Ok(tickv::success_codes::SuccessCode::Written) => {
                        // We successfully got the key-value object and we can
                        // call the callback with the retrieved value.
                        self.operation.set(Operation::None);
                        self.client.map(|cb| {
                            cb.get_value_complete(
                                Ok(()),
                                self.key_buffer.take().unwrap(),
                                self.value_buffer.take().unwrap(),
                            );
                        });
                    }
                    Err(tickv::error_codes::ErrorCode::BufferTooSmall(_)) => {
                        // Notify the upper layer using the `SIZE` error that
                        // the entire value was not read into the buffer as
                        // there was not enough room to store the entire value.
                        // The buffer still contains the portion of the value
                        // that would fit.
                        self.operation.set(Operation::None);
                        self.client.map(|cb| {
                            cb.get_value_complete(
                                Err(ErrorCode::SIZE),
                                self.key_buffer.take().unwrap(),
                                self.value_buffer.take().unwrap(),
                            );
                        });
                    }
                    Err(tickv::error_codes::ErrorCode::ReadNotReady(_)) => {
                        // Need to do another flash read.
                        //
                        // `self.operation` will still be `GetKey`, so this will automatically
                        // be retried by the primary state machine.
                    }
                    Err(tickv::error_codes::ErrorCode::EraseNotReady(_)) | Ok(_) => {}
                    Err(e) => {
                        let get_tock_err = match e {
                            tickv::error_codes::ErrorCode::KeyNotFound => ErrorCode::NOSUPPORT,
                            _ => ErrorCode::FAIL,
                        };
                        self.operation.set(Operation::None);
                        self.client.map(|cb| {
                            cb.get_value_complete(
                                Err(get_tock_err),
                                self.key_buffer.take().unwrap(),
                                self.value_buffer.take().unwrap(),
                            );
                        });
                    }
                }
            }
            Operation::AppendKey => {
                match ret {
                    Ok(tickv::success_codes::SuccessCode::Complete)
                    | Ok(tickv::success_codes::SuccessCode::Written) => {
                        // Nothing to do at this point as we need to wait
                        // for the flash write to complete.
                        self.operation.set(Operation::None);
                    }
                    Ok(tickv::success_codes::SuccessCode::Queued) => {}
                    Err(tickv::error_codes::ErrorCode::ReadNotReady(_))
                    | Err(tickv::error_codes::ErrorCode::WriteNotReady(_))
                    | Err(tickv::error_codes::ErrorCode::EraseNotReady(_)) => {
                        // Need to do another flash operation.
                    }
                    Err(e) => {
                        self.operation.set(Operation::None);

                        let tock_hil_error = match e {
                            tickv::error_codes::ErrorCode::KeyAlreadyExists => ErrorCode::NOSUPPORT,
                            tickv::error_codes::ErrorCode::RegionFull => ErrorCode::NOMEM,
                            tickv::error_codes::ErrorCode::FlashFull => ErrorCode::NOMEM,
                            _ => ErrorCode::FAIL,
                        };
                        self.client.map(|cb| {
                            cb.append_key_complete(
                                Err(tock_hil_error),
                                self.key_buffer.take().unwrap(),
                                self.value_buffer.take().unwrap(),
                            );
                        });
                    }
                }
            }
            Operation::InvalidateKey => match ret {
                Ok(tickv::success_codes::SuccessCode::Complete)
                | Ok(tickv::success_codes::SuccessCode::Written) => {
                    // Need to wait for flash write to complete.
                    self.operation.set(Operation::None);
                }
                Ok(tickv::success_codes::SuccessCode::Queued) => {}
                Err(tickv::error_codes::ErrorCode::ReadNotReady(_))
                | Err(tickv::error_codes::ErrorCode::WriteNotReady(_))
                | Err(tickv::error_codes::ErrorCode::EraseNotReady(_)) => {
                    // Need to do another flash operation.
                }
                Err(e) => {
                    self.operation.set(Operation::None);

                    let tock_hil_error = match e {
                        tickv::error_codes::ErrorCode::KeyNotFound => ErrorCode::NOSUPPORT,
                        _ => ErrorCode::FAIL,
                    };
                    self.client.map(|cb| {
                        cb.invalidate_key_complete(
                            Err(tock_hil_error),
                            self.key_buffer.take().unwrap(),
                        );
                    });
                }
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

    fn write_complete(&self, pagebuffer: &'static mut F::Page, _result: Result<(), flash::Error>) {
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
                        self.value_buffer.take().unwrap(),
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

    fn erase_complete(&self, _result: Result<(), flash::Error>) {
        let (ret, tickv_buf, tickv_buf_len) = self.tickv.continue_operation();

        // If we got the buffer back from TicKV then store it.
        tickv_buf.map(|buf| {
            let mut val_buf = SubSliceMut::new(buf);
            if tickv_buf_len > 0 {
                // Length of zero means nothing was inserted into the buffer so
                // no need to slice it.
                val_buf.slice(0..tickv_buf_len);
            }
            self.value_buffer.replace(val_buf);
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

impl<'a, F: Flash, H: Hasher<'a, 8>, const PAGE_SIZE: usize> KVSystem<'a>
    for TicKVSystem<'a, F, H, PAGE_SIZE>
{
    type K = TicKVKeyType;

    fn set_client(&self, client: &'a dyn KVSystemClient<Self::K>) {
        self.client.set(client);
    }

    fn generate_key(
        &self,
        unhashed_key: SubSliceMut<'static, u8>,
        key: &'static mut Self::K,
    ) -> Result<(), (SubSliceMut<'static, u8>, &'static mut Self::K, ErrorCode)> {
        match self.hasher.add_mut_data(unhashed_key) {
            Ok(_) => {
                self.key_buffer.replace(key);
                Ok(())
            }
            Err((e, buf)) => Err((buf, key, e)),
        }
    }

    fn append_key(
        &self,
        key: &'static mut Self::K,
        value: SubSliceMut<'static, u8>,
    ) -> Result<(), (&'static mut [u8; 8], SubSliceMut<'static, u8>, ErrorCode)> {
        match self.operation.get() {
            Operation::None => {
                self.operation.set(Operation::AppendKey);

                let length = value.len();
                match self
                    .tickv
                    .append_key(u64::from_be_bytes(*key), value.take(), length)
                {
                    Ok(_ret) => {
                        self.key_buffer.replace(key);
                        Ok(())
                    }
                    Err((buf, e)) => {
                        let tock_error = match e {
                            tickv::error_codes::ErrorCode::ObjectTooLarge => ErrorCode::SIZE,
                            _ => ErrorCode::FAIL,
                        };
                        Err((key, SubSliceMut::new(buf), tock_error))
                    }
                }
            }
            Operation::Init => {
                // The init process is still occurring.
                // We can save this request and start it after init
                self.next_operation.set(Operation::AppendKey);
                self.key_buffer.replace(key);
                self.value_buffer.replace(value);
                Ok(())
            }
            _ => {
                // An operation is already in process.
                Err((key, value, ErrorCode::BUSY))
            }
        }
    }

    fn get_value(
        &self,
        key: &'static mut Self::K,
        value: SubSliceMut<'static, u8>,
    ) -> Result<(), (&'static mut [u8; 8], SubSliceMut<'static, u8>, ErrorCode)> {
        if value.is_sliced() {
            return Err((key, value, ErrorCode::SIZE));
        }
        match self.operation.get() {
            Operation::None => {
                self.operation.set(Operation::GetKey);

                match self.tickv.get_key(u64::from_be_bytes(*key), value.take()) {
                    Ok(_ret) => {
                        self.key_buffer.replace(key);
                        Ok(())
                    }
                    Err((buf, _e)) => Err((key, SubSliceMut::new(buf), ErrorCode::FAIL)),
                }
            }
            Operation::Init => {
                // The init process is still occurring.
                // We can save this request and start it after init
                self.next_operation.set(Operation::GetKey);
                self.key_buffer.replace(key);
                self.value_buffer.replace(value);
                Ok(())
            }
            _ => {
                // An operation is already in process.
                Err((key, value, ErrorCode::BUSY))
            }
        }
    }

    fn invalidate_key(
        &self,
        key: &'static mut Self::K,
    ) -> Result<(), (&'static mut Self::K, ErrorCode)> {
        match self.operation.get() {
            Operation::None => {
                self.operation.set(Operation::InvalidateKey);

                match self.tickv.invalidate_key(u64::from_be_bytes(*key)) {
                    Ok(_ret) => {
                        self.key_buffer.replace(key);
                        Ok(())
                    }
                    Err(_e) => Err((key, ErrorCode::FAIL)),
                }
            }
            Operation::Init => {
                // The init process is still occurring.
                // We can save this request and start it after init.
                self.next_operation.set(Operation::InvalidateKey);
                self.key_buffer.replace(key);
                Ok(())
            }
            _ => {
                // An operation is already in process.
                Err((key, ErrorCode::BUSY))
            }
        }
    }

    fn garbage_collect(&self) -> Result<(), ErrorCode> {
        match self.operation.get() {
            Operation::None => {
                self.operation.set(Operation::GarbageCollect);
                self.tickv
                    .garbage_collect()
                    .and(Ok(()))
                    .or(Err(ErrorCode::FAIL))
            }
            Operation::Init => {
                // The init process is still occurring.
                // We can save this request and start it after init.
                self.next_operation.set(Operation::GarbageCollect);
                Ok(())
            }
            _ => {
                // An operation is already in process.
                Err(ErrorCode::BUSY)
            }
        }
    }
}
