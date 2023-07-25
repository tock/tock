// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

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
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::{SubSlice, SubSliceMut};
use kernel::ErrorCode;
use tickv::{self, AsyncTicKV};

#[derive(Clone, Copy, PartialEq, Debug)]
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

pub struct TicKVStore<'a, F: Flash + 'static, H: Hasher<'a, 8>, const PAGE_SIZE: usize> {
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
    client: OptionalCell<&'a dyn kv_system::Client<TicKVKeyType>>,
}

impl<'a, F: Flash, H: Hasher<'a, 8>, const PAGE_SIZE: usize> TicKVStore<'a, F, H, PAGE_SIZE> {
    pub fn new(
        flash: &'a F,
        hasher: &'a H,
        tickfs_read_buf: &'static mut [u8; PAGE_SIZE],
        flash_read_buffer: &'static mut F::Page,
        region_offset: usize,
        flash_size: usize,
    ) -> TicKVStore<'a, F, H, PAGE_SIZE> {
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
                            cb.get_value_complete(error, key, value);
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
                            cb.append_key_complete(error, key, value);
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
    for TicKVStore<'a, F, H, PAGE_SIZE>
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
    for TicKVStore<'a, F, H, PAGE_SIZE>
{
    fn read_complete(&self, pagebuffer: &'static mut F::Page, _error: flash::Error) {
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

    fn erase_complete(&self, _error: flash::Error) {
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
    for TicKVStore<'a, F, H, PAGE_SIZE>
{
    type K = TicKVKeyType;

    fn set_client(&self, client: &'a dyn kv_system::Client<Self::K>) {
        self.client.set(client);
    }

    fn generate_key(
        &self,
        unhashed_key: SubSliceMut<'static, u8>,
        key: &'static mut Self::K,
    ) -> Result<
        (),
        (
            SubSliceMut<'static, u8>,
            &'static mut Self::K,
            Result<(), ErrorCode>,
        ),
    > {
        match self.hasher.add_mut_data(unhashed_key) {
            Ok(_) => {
                self.key_buffer.replace(key);
                Ok(())
            }
            Err((e, buf)) => Err((buf, key, Err(e))),
        }
    }

    fn append_key(
        &self,
        key: &'static mut Self::K,
        value: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            &'static mut [u8; 8],
            SubSliceMut<'static, u8>,
            Result<(), kernel::ErrorCode>,
        ),
    > {
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
                    Err((buf, _e)) => Err((key, SubSliceMut::new(buf), Err(ErrorCode::FAIL))),
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
                Err((key, value, Err(ErrorCode::BUSY)))
            }
        }
    }

    fn get_value(
        &self,
        key: &'static mut Self::K,
        value: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            &'static mut [u8; 8],
            SubSliceMut<'static, u8>,
            Result<(), kernel::ErrorCode>,
        ),
    > {
        if value.is_sliced() {
            return Err((key, value, Err(ErrorCode::SIZE)));
        }
        match self.operation.get() {
            Operation::None => {
                self.operation.set(Operation::GetKey);

                match self.tickv.get_key(u64::from_be_bytes(*key), value.take()) {
                    Ok(_ret) => {
                        self.key_buffer.replace(key);
                        Ok(())
                    }
                    Err((buf, _e)) => Err((key, SubSliceMut::new(buf), Err(ErrorCode::FAIL))),
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
                Err((key, value, Err(ErrorCode::BUSY)))
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

                match self.tickv.invalidate_key(u64::from_be_bytes(*key)) {
                    Ok(_ret) => {
                        self.key_buffer.replace(key);
                        Ok(())
                    }
                    Err(_e) => Err((key, Err(ErrorCode::FAIL))),
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
                Err((key, Err(ErrorCode::BUSY)))
            }
        }
    }

    fn garbage_collect(&self) -> Result<(), ErrorCode> {
        match self.operation.get() {
            Operation::None => {
                self.operation.set(Operation::GarbageCollect);
                self.tickv.garbage_collect().or(Err(ErrorCode::FAIL))
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
