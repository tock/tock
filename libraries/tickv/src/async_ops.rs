// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! TicKV can be used asynchronously. This module provides documentation and
//! tests for using it with an async `FlashController` interface.
//!
//! To do this first there are special error values to return from the
//! `FlashController` functions. These are the `ReadNotReady`, `WriteNotReady`
//! and `EraseNotReady` types.
//!
//! ```rust
//! // EXAMPLE ONLY: The `DefaultHasher` is subject to change
//! // and hence is not a good fit.
//! use std::collections::hash_map::DefaultHasher;
//! use core::hash::{Hash, Hasher};
//! use std::cell::{Cell, RefCell};
//! use tickv::{AsyncTicKV, MAIN_KEY};
//! use tickv::error_codes::ErrorCode;
//! use tickv::flash_controller::FlashController;
//!
//! fn get_hashed_key(unhashed_key: &[u8]) -> u64 {
//!     let mut hash_function = DefaultHasher::new();
//!     unhashed_key.hash(&mut hash_function);
//!     hash_function.finish()
//! }
//!
//! struct FlashCtrl {
//!     buf: RefCell<[[u8; 1024]; 64]>,
//!     async_read_region: Cell<usize>,
//!     async_erase_region: Cell<usize>,
//! }
//!
//! impl FlashCtrl {
//!     fn new() -> Self {
//!         Self {
//!             buf: RefCell::new([[0xFF; 1024]; 64]),
//!             async_read_region: Cell::new(10),
//!             async_erase_region: Cell::new(10),
//!         }
//!     }
//! }
//!
//! impl FlashController<1024> for FlashCtrl {
//!     fn read_region(
//!         &self,
//!         region_number: usize,
//!         offset: usize,
//!         buf: &mut [u8; 1024],
//!     ) -> Result<(), ErrorCode> {
//!          // We aren't ready yet, launch the async operation
//!          self.async_read_region.set(region_number);
//!          return Err(ErrorCode::ReadNotReady(region_number));
//!
//!         Ok(())
//!     }
//!
//!     fn write(&self, address: usize, buf: &[u8]) -> Result<(), ErrorCode> {
//!         // Save the write operation to a queue, we don't need to re-call
//!         for (i, d) in buf.iter().enumerate() {
//!             self.buf.borrow_mut()[address / 1024][(address % 1024) + i] = *d;
//!         }
//!         Ok(())
//!     }
//!
//!     fn erase_region(&self, region_number: usize) -> Result<(), ErrorCode> {
//!         if self.async_erase_region.get() != region_number {
//!             // We aren't ready yet, launch the async operation
//!             self.async_erase_region.set(region_number);
//!             return Err(ErrorCode::EraseNotReady(region_number));
//!         }
//!
//!         Ok(())
//!     }
//! }
//!
//! // Create the TicKV instance and loop until everything is done
//! // NOTE in an real implementation you will want to wait on
//! // callbacks/interrupts and make this async.
//!
//! let mut read_buf: [u8; 1024] = [0; 1024];
//! let mut hash_function = DefaultHasher::new();
//! MAIN_KEY.hash(&mut hash_function);
//! let tickv = AsyncTicKV::<FlashCtrl, 1024>::new(FlashCtrl::new(),
//!                   &mut read_buf, 0x1000);
//!
//! let mut ret = tickv.initialise(hash_function.finish());
//! while ret.is_err() {
//!     // There is no actual delay here, in a real implementation wait on some event
//!     ret = tickv.continue_operation().0;
//!
//!     match ret {
//!         Err(ErrorCode::ReadNotReady(reg)) => {
//!             tickv.set_read_buffer(&tickv.tickv.controller.buf.borrow()[reg]);
//!         }
//!         Ok(_) => break,
//!         Err(ErrorCode::WriteNotReady(reg)) => break,
//!         Err(ErrorCode::EraseNotReady(reg)) => {}
//!         _ => unreachable!(),
//!     }
//! }
//!
//! // Then when calling the TicKV function check for the error. For example
//! // when appending a key:
//!
//! // Add a key
//! static mut VALUE: [u8; 32] = [0x23; 32];
//! let ret = unsafe { tickv.append_key(get_hashed_key(b"ONE"), &mut VALUE, 32) };
//!
//! match ret {
//!     Err((_buf, ErrorCode::ReadNotReady(reg))) => {
//!         // There is no actual delay in the test, just continue now
//!         tickv.set_read_buffer(&tickv.tickv.controller.buf.borrow()[reg]);
//!         tickv
//!             .continue_operation().0
//!             .unwrap();
//!     }
//!     Ok(_) => {}
//!     _ => unreachable!(),
//! }
//!
//! ```
//!
//! This will call into the `FlashController` again where the
//! `FlashController` implementation must return the data that is requested.
//! If the data isn't ready (multiple reads might occur) then the `NotReady`
//! error types can still be used.
//!

use crate::error_codes::ErrorCode;
use crate::flash_controller::FlashController;
use crate::success_codes::SuccessCode;
use crate::tickv::{State, TicKV};
use core::cell::Cell;

/// The return type from the continue operation
type ContinueReturn = (
    // Result
    Result<SuccessCode, ErrorCode>,
    // Buf Buffer
    Option<&'static mut [u8]>,
    // Length of valid data inside of the buffer.
    usize,
);

/// The struct storing all of the TicKV information for the async implementation.
pub struct AsyncTicKV<'a, C: FlashController<S>, const S: usize> {
    /// The main TicKV struct
    pub tickv: TicKV<'a, C, S>,
    key: Cell<Option<u64>>,
    value: Cell<Option<&'static mut [u8]>>,
    value_length: Cell<usize>,
}

impl<'a, C: FlashController<S>, const S: usize> AsyncTicKV<'a, C, S> {
    /// Create a new struct
    ///
    /// `C`: An implementation of the `FlashController` trait
    ///
    /// `controller`: An new struct implementing `FlashController`
    /// `flash_size`: The total size of the flash used for TicKV
    pub fn new(controller: C, read_buffer: &'a mut [u8; S], flash_size: usize) -> Self {
        Self {
            tickv: TicKV::<C, S>::new(controller, read_buffer, flash_size),
            key: Cell::new(None),
            value: Cell::new(None),
            value_length: Cell::new(0),
        }
    }

    /// This function setups the flash region to be used as a key-value store.
    /// If the region is already initialised this won't make any changes.
    ///
    /// `hashed_main_key`: The u64 hash of the const string `MAIN_KEY`.
    ///
    /// If the specified region has not already been setup for TicKV
    /// the entire region will be erased.
    ///
    /// On success a `SuccessCode` will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn initialise(&self, hashed_main_key: u64) -> Result<SuccessCode, ErrorCode> {
        self.key.replace(Some(hashed_main_key));
        self.tickv.initialise(hashed_main_key)
    }

    /// Appends the key/value pair to flash storage.
    ///
    /// `hash`: A hashed key. This key will be used in future to retrieve
    ///         or remove the `value`.
    /// `value`: A buffer containing the data to be stored to flash.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn append_key(
        &self,
        hash: u64,
        value: &'static mut [u8],
        length: usize,
    ) -> Result<SuccessCode, (&'static mut [u8], ErrorCode)> {
        match self.tickv.append_key(hash, &value[0..length]) {
            Ok(_code) => {
                // Ok is a problem, since that means no asynchronous operations
                // were called, which means our client will never get a
                // callback. We need to error.
                Err((value, ErrorCode::WriteFail))
            }
            Err(e) => match e {
                ErrorCode::ReadNotReady(_)
                | ErrorCode::EraseNotReady(_)
                | ErrorCode::WriteNotReady(_) => {
                    // This is what we expect, since it means we are going
                    // an asynchronous operation which this interface expects.
                    self.key.replace(Some(hash));
                    self.value.replace(Some(value));
                    self.value_length.set(length);
                    Ok(SuccessCode::Queued)
                }
                _ => {
                    // On any other error we report the error.
                    Err((value, e))
                }
            },
        }
    }

    /// Retrieves the value from flash storage.
    ///
    /// `hash`: A hashed key.
    /// `buf`: A buffer to store the value to.
    ///
    /// On success a `SuccessCode` will be returned.
    /// On error a `ErrorCode` will be returned.
    ///
    /// If a power loss occurs before success is returned the data is
    /// assumed to be lost.
    pub fn get_key(
        &self,
        hash: u64,
        buf: &'static mut [u8],
    ) -> Result<SuccessCode, (&'static mut [u8], ErrorCode)> {
        match self.tickv.get_key(hash, buf) {
            Ok(_code) => {
                // Ok is a problem, since that means no asynchronous operations
                // were called, which means our client will never get a
                // callback. We need to error.
                Err((buf, ErrorCode::ReadFail))
            }
            Err(e) => match e {
                ErrorCode::ReadNotReady(_)
                | ErrorCode::EraseNotReady(_)
                | ErrorCode::WriteNotReady(_) => {
                    self.key.replace(Some(hash));
                    self.value.replace(Some(buf));
                    Ok(SuccessCode::Queued)
                }
                _ => Err((buf, e)),
            },
        }
    }

    /// Invalidates the key in flash storage
    ///
    /// `hash`: A hashed key.
    /// `key`: A unhashed key. This will be hashed internally.
    ///
    /// On success a `SuccessCode` will be returned.
    /// On error a `ErrorCode` will be returned.
    ///
    /// If a power loss occurs before success is returned the data is
    /// assumed to be lost.
    pub fn invalidate_key(&self, hash: u64) -> Result<SuccessCode, ErrorCode> {
        match self.tickv.invalidate_key(hash) {
            Ok(_code) => Err(ErrorCode::WriteFail),
            Err(_e) => {
                self.key.replace(Some(hash));
                Ok(SuccessCode::Queued)
            }
        }
    }

    /// Perform a garbage collection on TicKV
    ///
    /// On success a `SuccessCode` will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn garbage_collect(&self) -> Result<SuccessCode, ErrorCode> {
        match self.tickv.garbage_collect() {
            Ok(_code) => Err(ErrorCode::EraseFail),
            Err(_e) => Ok(SuccessCode::Queued),
        }
    }

    /// Copy data from `read_buffer` argument to the internal read_buffer.
    /// This should be used to copy the data that the implementation wanted
    /// to read when calling `read_region` after the async operation has
    /// completed.
    pub fn set_read_buffer(&self, read_buffer: &[u8]) {
        let buf = self.tickv.read_buffer.take().unwrap();
        buf.copy_from_slice(read_buffer);
        self.tickv.read_buffer.replace(Some(buf));
    }

    /// Continue the last operation after the async operation has completed.
    /// This should be called from a read/erase complete callback.
    /// NOTE: If called from a read callback, `set_read_buffer` should be
    /// called first to update the data.
    ///
    /// `hash_function`: Hash function with no previous state. This is
    ///                  usually a newly created hash.
    ///
    /// Returns a tuple of 3 values
    ///    Result:
    ///        On success a `SuccessCode` will be returned.
    ///        On error a `ErrorCode` will be returned.
    ///    Buf Buffer:
    ///        An option of the buf buffer used
    ///    Length usize:
    ///        The number of valid bytes in the buffer. 0 if Buf is None.
    /// The buffers will only be returned on a non async error or on success.
    pub fn continue_operation(&self) -> ContinueReturn {
        let (ret, length) = match self.tickv.state.get() {
            State::Init(_) => (self.tickv.initialise(self.key.get().unwrap()), 0),
            State::AppendKey(_) => {
                let value = self.value.take().unwrap();
                let value_length = self.value_length.get();
                let ret = self
                    .tickv
                    .append_key(self.key.get().unwrap(), &value[0..value_length]);
                self.value.replace(Some(value));
                (ret, value_length)
            }
            State::GetKey(_) => {
                let buf = self.value.take().unwrap();
                let ret = self.tickv.get_key(self.key.get().unwrap(), buf);
                self.value.replace(Some(buf));
                match ret {
                    Ok((s, len)) => (Ok(s), len),
                    Err(e) => (Err(e), 0),
                }
            }
            State::InvalidateKey(_) => (self.tickv.invalidate_key(self.key.get().unwrap()), 0),
            State::GarbageCollect(_) => match self.tickv.garbage_collect() {
                Ok(bytes_freed) => (Ok(SuccessCode::Complete), bytes_freed),
                Err(e) => (Err(e), 0),
            },
            _ => unreachable!(),
        };

        match ret {
            Ok(_) => {
                self.tickv.state.set(State::None);
                (ret, self.value.take(), length)
            }
            Err(e) => match e {
                ErrorCode::ReadNotReady(_) | ErrorCode::EraseNotReady(_) => (ret, None, 0),
                ErrorCode::WriteNotReady(_) => {
                    self.tickv.state.set(State::None);
                    (ret, None, 0)
                }
                _ => {
                    self.tickv.state.set(State::None);
                    (ret, self.value.take(), length)
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(unsafe_code)]

    /// Tests using a flash controller that can store data
    mod store_flast_ctrl {
        use crate::async_ops::AsyncTicKV;
        use crate::error_codes::ErrorCode;
        use crate::flash_controller::FlashController;
        use crate::success_codes::SuccessCode;
        use crate::tickv::{HASH_OFFSET, LEN_OFFSET, MAIN_KEY, VERSION, VERSION_OFFSET};
        use core::hash::{Hash, Hasher};
        use std::cell::Cell;
        use std::cell::RefCell;
        use std::collections::hash_map::DefaultHasher;

        fn check_region_main(buf: &[u8]) {
            // Check the version
            assert_eq!(buf[VERSION_OFFSET], VERSION);

            // Check the length
            assert_eq!(buf[LEN_OFFSET], 0x80);
            assert_eq!(buf[LEN_OFFSET + 1], 15);

            // Check the hash
            assert_eq!(buf[HASH_OFFSET + 0], 0x7b);
            assert_eq!(buf[HASH_OFFSET + 1], 0xc9);
            assert_eq!(buf[HASH_OFFSET + 2], 0xf7);
            assert_eq!(buf[HASH_OFFSET + 3], 0xff);
            assert_eq!(buf[HASH_OFFSET + 4], 0x4f);
            assert_eq!(buf[HASH_OFFSET + 5], 0x76);
            assert_eq!(buf[HASH_OFFSET + 6], 0xf2);
            assert_eq!(buf[HASH_OFFSET + 7], 0x44);

            // Check the check hash
            assert_eq!(buf[HASH_OFFSET + 8], 0xbb);
            assert_eq!(buf[HASH_OFFSET + 9], 0x32);
            assert_eq!(buf[HASH_OFFSET + 10], 0x74);
            assert_eq!(buf[HASH_OFFSET + 11], 0x1d);
        }

        fn check_region_one(buf: &[u8]) {
            // Check the version
            assert_eq!(buf[VERSION_OFFSET], VERSION);

            // Check the length
            assert_eq!(buf[LEN_OFFSET], 0x80);
            assert_eq!(buf[LEN_OFFSET + 1], 47);

            // Check the hash
            assert_eq!(buf[HASH_OFFSET + 0], 0x81);
            assert_eq!(buf[HASH_OFFSET + 1], 0x13);
            assert_eq!(buf[HASH_OFFSET + 2], 0x7e);
            assert_eq!(buf[HASH_OFFSET + 3], 0x95);
            assert_eq!(buf[HASH_OFFSET + 4], 0x9e);
            assert_eq!(buf[HASH_OFFSET + 5], 0x93);
            assert_eq!(buf[HASH_OFFSET + 6], 0xaa);
            assert_eq!(buf[HASH_OFFSET + 7], 0x3d);

            // Check the value
            assert_eq!(buf[HASH_OFFSET + 8], 0x23);
            assert_eq!(buf[28], 0x23);
            assert_eq!(buf[42], 0x23);

            // Check the check hash
            assert_eq!(buf[43], 0xfd);
            assert_eq!(buf[44], 0x24);
            assert_eq!(buf[45], 0xf0);
            assert_eq!(buf[46], 0x07);
        }

        fn check_region_two(buf: &[u8]) {
            // Check the version
            assert_eq!(buf[VERSION_OFFSET], VERSION);

            // Check the length
            assert_eq!(buf[LEN_OFFSET], 0x80);
            assert_eq!(buf[LEN_OFFSET + 1], 47);

            // Check the hash
            assert_eq!(buf[HASH_OFFSET + 0], 0x9d);
            assert_eq!(buf[HASH_OFFSET + 1], 0xd3);
            assert_eq!(buf[HASH_OFFSET + 2], 0x71);
            assert_eq!(buf[HASH_OFFSET + 3], 0x45);
            assert_eq!(buf[HASH_OFFSET + 4], 0x05);
            assert_eq!(buf[HASH_OFFSET + 5], 0xc2);
            assert_eq!(buf[HASH_OFFSET + 6], 0xf8);
            assert_eq!(buf[HASH_OFFSET + 7], 0x66);

            // Check the value
            assert_eq!(buf[HASH_OFFSET + 8], 0x23);
            assert_eq!(buf[28], 0x23);
            assert_eq!(buf[42], 0x23);

            // Check the check hash
            assert_eq!(buf[43], 0x1b);
            assert_eq!(buf[44], 0x53);
            assert_eq!(buf[45], 0xf9);
            assert_eq!(buf[46], 0x54);
        }

        fn get_hashed_key(unhashed_key: &[u8]) -> u64 {
            let mut hash_function = DefaultHasher::new();
            unhashed_key.hash(&mut hash_function);
            hash_function.finish()
        }

        #[derive(Clone, Copy)]
        enum FlashCtrlAction {
            Idle,
            Read,
            Write,
            Erase,
        }

        // An example FlashCtrl implementation
        struct FlashCtrl<const S: usize> {
            buf: RefCell<[[u8; S]; 64]>,
            run: Cell<u8>,
            async_read_region: Cell<usize>,
            async_erase_region: Cell<usize>,
            waiting_on: Cell<FlashCtrlAction>,
            check_write_contents: bool,
        }

        impl<const S: usize> FlashCtrl<S> {
            fn new(check_write_contents: bool) -> Self {
                Self {
                    buf: RefCell::new([[0xFF; S]; 64]),
                    run: Cell::new(0),
                    async_read_region: Cell::new(100),
                    async_erase_region: Cell::new(100),
                    waiting_on: Cell::new(FlashCtrlAction::Idle),
                    check_write_contents,
                }
            }

            fn get_waiting_action(&self) -> FlashCtrlAction {
                self.waiting_on.get()
            }
        }

        impl<const S: usize> FlashController<S> for FlashCtrl<S> {
            fn read_region(
                &self,
                region_number: usize,
                offset: usize,
                _buf: &mut [u8; S],
            ) -> Result<(), ErrorCode> {
                println!("Read from region: {}, offset: {offset}", region_number);

                // Pretend that we aren't ready
                self.async_read_region.set(region_number);
                println!("  Not ready");

                self.waiting_on.set(FlashCtrlAction::Read);

                Err(ErrorCode::ReadNotReady(region_number))
            }

            fn write(&self, address: usize, buf: &[u8]) -> Result<(), ErrorCode> {
                println!(
                    "Write to address: {:#x}, region: {}",
                    address % S,
                    address / S
                );

                for (i, d) in buf.iter().enumerate() {
                    self.buf.borrow_mut()[address / S][(address % S) + i] = *d;
                }

                // Check to see if we are adding a key
                if buf.len() > 1 && self.check_write_contents {
                    if self.run.get() == 0 {
                        println!("Writing main key: {:#x?}", buf);
                        check_region_main(buf);
                    } else if self.run.get() == 1 {
                        println!("Writing key ONE: {:#x?}", buf);
                        check_region_one(buf);
                    } else if self.run.get() == 2 {
                        println!("Writing key TWO: {:#x?}", buf);
                        check_region_two(buf);
                    }
                }

                self.run.set(self.run.get() + 1);
                self.waiting_on.set(FlashCtrlAction::Write);
                Err(ErrorCode::WriteNotReady(address))
            }

            fn erase_region(&self, region_number: usize) -> Result<(), ErrorCode> {
                println!("Erase region: {}", region_number);

                let mut local_buf = self.buf.borrow_mut()[region_number];

                for d in local_buf.iter_mut() {
                    *d = 0xFF;
                }

                // Pretend that we aren't ready
                self.async_erase_region.set(region_number);

                self.waiting_on.set(FlashCtrlAction::Erase);
                Err(ErrorCode::EraseNotReady(region_number))
            }
        }

        /// This function implements what would happen in the callback function
        /// triggered by the underlying flash hardware. In effect, it is the
        /// callback handler, but since we don't actually have an async flash
        /// implementation, this is just called by each test after starting the
        /// flash operation.
        fn flash_ctrl_callback<const S: usize>(tickv: &AsyncTicKV<FlashCtrl<S>, S>) {
            match tickv.tickv.controller.get_waiting_action() {
                FlashCtrlAction::Read => {
                    // This mimics a read is complete, and we provide the buffer
                    // with the newly read data to the tickv layer.
                    tickv.set_read_buffer(
                        &tickv.tickv.controller.buf.borrow()
                            [tickv.tickv.controller.async_read_region.get()],
                    );
                }
                _ => {
                    // For write an erase all of the operation already occurred
                    // in the original operation, and nothing needs to be done
                    // in the simulated callback.
                }
            }
        }

        #[test]
        fn test_simple_append() {
            let mut read_buf: [u8; 1024] = [0; 1024];
            let mut hash_function = DefaultHasher::new();
            MAIN_KEY.hash(&mut hash_function);

            let tickv = AsyncTicKV::<FlashCtrl<1024>, 1024>::new(
                FlashCtrl::new(true),
                &mut read_buf,
                0x1000,
            );

            let mut ret = tickv.initialise(hash_function.finish());
            while ret.is_err() {
                flash_ctrl_callback(&tickv);

                // There is no actual delay in the test, just continue now
                let (r, _buf, _len) = tickv.continue_operation();
                ret = r;
            }

            static mut VALUE: [u8; 32] = [0x23; 32];

            println!("HASHED KEY {:?}", get_hashed_key(b"ONE"));

            let ret = unsafe { tickv.append_key(get_hashed_key(b"ONE"), &mut VALUE, 32) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);
                    tickv.continue_operation().0.unwrap();
                }
                Err(_) => {}
                _ => unreachable!(),
            }

            let ret = unsafe { tickv.append_key(get_hashed_key(b"TWO"), &mut VALUE, 32) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);
                    tickv.continue_operation().0.unwrap();
                }
                Err(_) => {}
                _ => unreachable!(),
            }
        }

        #[test]
        fn test_double_append() {
            let mut read_buf: [u8; 1024] = [0; 1024];
            let mut hash_function = DefaultHasher::new();
            MAIN_KEY.hash(&mut hash_function);

            let tickv = AsyncTicKV::<FlashCtrl<1024>, 1024>::new(
                FlashCtrl::new(true),
                &mut read_buf,
                0x10000,
            );

            let mut ret = tickv.initialise(hash_function.finish());
            while ret.is_err() {
                flash_ctrl_callback(&tickv);

                // There is no actual delay in the test, just continue now
                let (r, _buf, _len) = tickv.continue_operation();
                ret = r;
            }

            static mut VALUE: [u8; 32] = [0x23; 32];
            static mut BUF: [u8; 32] = [0; 32];

            println!("Add key ONE");
            let ret = unsafe { tickv.append_key(get_hashed_key(b"ONE"), &mut VALUE, 32) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);
                    tickv.continue_operation().0.unwrap();
                }
                Err(_) => {}
                _ => unreachable!(),
            }

            println!("Get key ONE");

            let ret = unsafe { tickv.get_key(get_hashed_key(b"ONE"), &mut BUF) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    flash_ctrl_callback(&tickv);
                    tickv.continue_operation().0.unwrap();
                }
                Err(_) => {}
                _ => unreachable!(),
            }

            println!("Get non-existent key TWO");
            let ret = unsafe { tickv.get_key(get_hashed_key(b"TWO"), &mut BUF) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);
                    assert_eq!(tickv.continue_operation().0, Err(ErrorCode::KeyNotFound));
                }
                Err((_, ErrorCode::KeyNotFound)) => {}
                _ => unreachable!(),
            }

            println!("Add key ONE again");
            let ret = unsafe { tickv.append_key(get_hashed_key(b"ONE"), &mut VALUE, 32) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);
                    assert_eq!(
                        tickv.continue_operation().0,
                        Err(ErrorCode::KeyAlreadyExists)
                    );
                }
                Err((_buf, ErrorCode::KeyAlreadyExists)) => {}
                _ => unreachable!(),
            }

            println!("Add key TWO");
            let ret = unsafe { tickv.append_key(get_hashed_key(b"TWO"), &mut VALUE, 32) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);
                    tickv.continue_operation().0.unwrap();
                }
                Err(_) => {}
                _ => unreachable!(),
            }

            println!("Get key ONE");
            let ret = unsafe { tickv.get_key(get_hashed_key(b"ONE"), &mut BUF) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);
                    tickv.continue_operation().0.unwrap();
                }
                Err(_) => {}
                _ => unreachable!(),
            }

            println!("Get key TWO");
            let ret = unsafe { tickv.get_key(get_hashed_key(b"TWO"), &mut BUF) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);
                    tickv.continue_operation().0.unwrap();
                }
                Err(_) => {}
                _ => unreachable!(),
            }

            println!("Get non-existent key THREE");
            let ret = unsafe { tickv.get_key(get_hashed_key(b"THREE"), &mut BUF) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);
                    assert_eq!(tickv.continue_operation().0, Err(ErrorCode::KeyNotFound));
                }
                _ => unreachable!(),
            }

            let ret = unsafe { tickv.get_key(get_hashed_key(b"THREE"), &mut BUF) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    flash_ctrl_callback(&tickv);
                    assert_eq!(tickv.continue_operation().0, Err(ErrorCode::KeyNotFound));
                }
                Err(_) => {}
                _ => unreachable!(),
            }
        }

        #[test]
        fn test_spread_pages() {
            let mut read_buf: [u8; 64] = [0; 64];
            let mut hash_function = DefaultHasher::new();
            MAIN_KEY.hash(&mut hash_function);

            let tickv =
                AsyncTicKV::<FlashCtrl<64>, 64>::new(FlashCtrl::new(false), &mut read_buf, 64 * 64);

            let mut ret = tickv.initialise(hash_function.finish());
            while ret.is_err() {
                tickv.set_read_buffer(
                    &tickv.tickv.controller.buf.borrow()
                        [tickv.tickv.controller.async_read_region.get()],
                );

                // There is no actual delay in the test, just continue now
                let (r, _buf, _len) = tickv.continue_operation();
                ret = r;
            }

            static mut VALUE: [u8; 32] = [0x23; 32];
            static mut BUF: [u8; 32] = [0; 32];

            println!("Add key 0x1000");
            let ret = unsafe { tickv.append_key(0x1000, &mut VALUE, 32) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);
                    tickv.continue_operation().0.unwrap();
                }
                Err(e) => panic!("Unable to add key 0x100: {e:?}"),
                _ => unreachable!(),
            }

            println!("Add key 0x2000");
            let ret = unsafe { tickv.append_key(0x2000, &mut VALUE, 32) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);

                    assert_eq!(
                        tickv.continue_operation().0,
                        Err(ErrorCode::ReadNotReady(1))
                    );
                    flash_ctrl_callback(&tickv);

                    tickv.continue_operation().0.unwrap();
                }
                Err(e) => panic!("Unable to add key 0x200: {e:?}"),
                _ => unreachable!(),
            }

            println!("Add key 0x3000");
            let ret = unsafe { tickv.append_key(0x3000, &mut VALUE, 32) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);
                }
                Err(e) => panic!("Unable to add key 0x3000: {e:?}"),
                _ => unreachable!(),
            }

            loop {
                let ret = tickv.continue_operation().0;
                match ret {
                    Err(ErrorCode::ReadNotReady(_reg)) => {
                        // There is no actual delay in the test, just continue now
                        flash_ctrl_callback(&tickv);
                    }
                    Err(e) => panic!("Unable to add key 0x3000: {e:?}"),
                    Ok(_) => break,
                }
            }

            println!("Get key 0x1000");
            let ret = unsafe { tickv.get_key(0x1000, &mut BUF) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);
                }
                _ => unreachable!(),
            }

            loop {
                let ret = tickv.continue_operation().0;
                match ret {
                    Err(ErrorCode::ReadNotReady(_reg)) => {
                        // There is no actual delay in the test, just continue now
                        flash_ctrl_callback(&tickv);
                    }
                    Err(e) => panic!("Unable to get key 0x1000: {e:?}"),
                    Ok(_) => break,
                }
            }

            println!("Get key 0x3000");
            let ret = unsafe { tickv.get_key(0x3000, &mut BUF) };
            match ret {
                Ok(_) => flash_ctrl_callback(&tickv),
                Err(_) => unreachable!(),
            }

            loop {
                let ret = tickv.continue_operation().0;
                match ret {
                    Err(ErrorCode::ReadNotReady(reg)) => {
                        // There is no actual delay in the test, just continue now
                        tickv.set_read_buffer(&tickv.tickv.controller.buf.borrow()[reg]);
                    }
                    Err(e) => panic!("Unable to get key 0x1000: {e:?}"),
                    Ok(_) => break,
                }
            }
        }

        #[test]
        fn test_append_and_delete() {
            let mut read_buf: [u8; 1024] = [0; 1024];
            let mut hash_function = DefaultHasher::new();
            MAIN_KEY.hash(&mut hash_function);

            let tickv = AsyncTicKV::<FlashCtrl<1024>, 1024>::new(
                FlashCtrl::new(true),
                &mut read_buf,
                0x10000,
            );

            let mut ret = tickv.initialise(hash_function.finish());
            while ret.is_err() {
                flash_ctrl_callback(&tickv);

                // There is no actual delay in the test, just continue now
                let (r, _buf, _len) = tickv.continue_operation();
                ret = r;
            }

            static mut VALUE: [u8; 32] = [0x23; 32];
            static mut BUF: [u8; 32] = [0; 32];

            println!("Add key ONE");
            let ret = unsafe { tickv.append_key(get_hashed_key(b"ONE"), &mut VALUE, 32) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);
                    tickv.continue_operation().0.unwrap();
                }
                Err(_) => {}
                _ => unreachable!(),
            }

            println!("Get key ONE");
            let ret = unsafe { tickv.get_key(get_hashed_key(b"ONE"), &mut BUF) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);
                    tickv.continue_operation().0.unwrap();
                }
                Err(_) => {}
                _ => unreachable!(),
            }

            println!("Delete Key ONE");
            let ret = tickv.invalidate_key(get_hashed_key(b"ONE"));
            match ret {
                Ok(SuccessCode::Queued) => {
                    flash_ctrl_callback(&tickv);
                    tickv.continue_operation().0.unwrap();
                }
                Err(_) => {}
                _ => unreachable!(),
            }

            println!("Get non-existent key ONE");
            unsafe {
                match tickv.get_key(get_hashed_key(b"ONE"), &mut BUF) {
                    Ok(SuccessCode::Queued) => {
                        flash_ctrl_callback(&tickv);

                        assert_eq!(
                            tickv.continue_operation().0,
                            Err(ErrorCode::ReadNotReady(62))
                        );
                        flash_ctrl_callback(&tickv);

                        match tickv.continue_operation().0 {
                            Err(ErrorCode::ReadNotReady(reg)) => {
                                panic!("Searching too far for keys: {reg}");
                            }
                            Err(ErrorCode::KeyNotFound) => {}
                            e => {
                                panic!("Expected ErrorCode::KeyNotFound, got {e:?}");
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }

            println!("Try to delete Key ONE Again");
            match tickv.invalidate_key(get_hashed_key(b"ONE")) {
                Ok(SuccessCode::Queued) => {
                    let reg = tickv.tickv.controller.async_read_region.get();

                    flash_ctrl_callback(&tickv);
                    assert_eq!(
                        tickv.continue_operation().0,
                        Err(ErrorCode::ReadNotReady(reg + 1))
                    );

                    // In normal operation we will read region `reg`, determine
                    // that it isn't full and stop looking for the key
                    //
                    // The following test is a hack to continue testing.
                    // We don't fill the read buffer with new data. So
                    // the read buffer will continue to provide the data from
                    // `reg`, which means TicKV will continue searching for
                    // an empty region.
                    //
                    // In normal operation this isn't correct, but for the test
                    // case it's a good check to test region searching
                    assert_eq!(
                        tickv.continue_operation().0,
                        Err(ErrorCode::ReadNotReady(reg - 1))
                    );

                    assert_eq!(
                        tickv.continue_operation().0,
                        Err(ErrorCode::ReadNotReady(reg + 2))
                    );

                    assert_eq!(
                        tickv.continue_operation().0,
                        Err(ErrorCode::ReadNotReady(reg - 2))
                    );

                    assert_eq!(
                        tickv.continue_operation().0,
                        Err(ErrorCode::ReadNotReady(reg - 3))
                    );

                    // Now set the read buffer and end the search
                    tickv.set_read_buffer(&tickv.tickv.controller.buf.borrow()[reg - 1]);

                    match tickv.continue_operation().0 {
                        Err(ErrorCode::ReadNotReady(reg)) => {
                            panic!("Searching too far for keys: {reg}");
                        }
                        Err(ErrorCode::KeyNotFound) => {}
                        e => {
                            panic!("Expected ErrorCode::KeyNotFound, got {e:?}");
                        }
                    }
                }
                e => {
                    panic!("Expected ErrorCode::KeyNotFound, got {e:?}");
                }
            }
        }

        #[test]
        fn test_garbage_collect() {
            let mut read_buf: [u8; 1024] = [0; 1024];
            let mut hash_function = DefaultHasher::new();
            MAIN_KEY.hash(&mut hash_function);

            let tickv = AsyncTicKV::<FlashCtrl<1024>, 1024>::new(
                FlashCtrl::new(true),
                &mut read_buf,
                0x10000,
            );

            let mut ret = tickv.initialise(hash_function.finish());
            while ret.is_err() {
                flash_ctrl_callback(&tickv);

                // There is no actual delay in the test, just continue now
                let (r, _buf, _len) = tickv.continue_operation();
                ret = r;
            }

            static mut VALUE: [u8; 32] = [0x23; 32];
            static mut BUF: [u8; 32] = [0; 32];

            println!("Garbage collect empty flash");
            let ret = tickv.garbage_collect();
            match ret {
                Ok(SuccessCode::Queued) => loop {
                    flash_ctrl_callback(&tickv);
                    let (res, _buf, len) = tickv.continue_operation();
                    if res.is_ok() {
                        assert_eq!(len, 0);
                        break;
                    }
                },
                Ok(_) => {}
                _ => unreachable!(),
            }

            println!("Add key ONE");
            let ret = unsafe { tickv.append_key(get_hashed_key(b"ONE"), &mut VALUE, 32) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);
                    tickv.continue_operation().0.unwrap();
                }
                Ok(_) => {}
                _ => unreachable!(),
            }

            println!("Garbage collect flash with valid key");
            let ret = tickv.garbage_collect();
            match ret {
                Ok(SuccessCode::Queued) => loop {
                    flash_ctrl_callback(&tickv);
                    let (res, _buf, len) = tickv.continue_operation();
                    if res.is_ok() {
                        assert_eq!(len, 0);
                        break;
                    }
                },
                Ok(_) => {}
                _ => unreachable!(),
            }

            println!("Delete Key ONE");
            let ret = tickv.invalidate_key(get_hashed_key(b"ONE"));
            match ret {
                Ok(SuccessCode::Queued) => {
                    flash_ctrl_callback(&tickv);
                    tickv.continue_operation().0.unwrap();
                }
                Err(_) => {}
                _ => unreachable!(),
            }

            println!("Garbage collect flash with deleted key");
            let ret = tickv.garbage_collect();
            match ret {
                Ok(SuccessCode::Queued) => loop {
                    flash_ctrl_callback(&tickv);
                    let (res, _buf, len) = tickv.continue_operation();
                    if res.is_ok() {
                        assert_eq!(len, 1024);
                        break;
                    }
                },
                Ok(_) => {}
                _ => unreachable!(),
            }

            println!("Get non-existent key ONE");
            match unsafe { tickv.get_key(get_hashed_key(b"ONE"), &mut BUF) } {
                Ok(SuccessCode::Queued) => {
                    flash_ctrl_callback(&tickv);
                    assert_eq!(
                        tickv.continue_operation().0,
                        Err(ErrorCode::ReadNotReady(62))
                    );
                    flash_ctrl_callback(&tickv);
                    assert_eq!(tickv.continue_operation().0, Err(ErrorCode::KeyNotFound));
                }
                _ => unreachable!(),
            }

            println!("Add Key ONE");
            let ret = unsafe { tickv.append_key(get_hashed_key(b"ONE"), &mut VALUE, 32) };
            match ret {
                Ok(SuccessCode::Queued) => {
                    // There is no actual delay in the test, just continue now
                    flash_ctrl_callback(&tickv);
                    tickv.continue_operation().0.unwrap();
                }
                _ => unreachable!("ret: {:?}", ret),
            }
        }
    }
}
