//! TickFS can be used asynchronously. This module provides documentation and
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
//! use std::cell::{Cell, RefCell};
//! use tickfs::AsyncTickFS;
//! use tickfs::error_codes::ErrorCode;
//! use tickfs::flash_controller::FlashController;
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
//!         if self.async_read_region.get() != region_number {
//!             // We aren't ready yet, launch the async operation
//!             self.async_read_region.set(region_number);
//!             return Err(ErrorCode::ReadNotReady(region_number));
//!         }
//!
//!         for (i, b) in buf.iter_mut().enumerate() {
//!             *b = self.buf.borrow()[region_number][offset + i]
//!         }
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
//! // Create the TickFS instance and loop until everything is done
//! // NOTE in an real implementation you will want to wait on
//! // callbacks/interrupts and make this async.
//!
//! let mut read_buf: [u8; 1024] = [0; 1024];
//! let tickfs = AsyncTickFS::<FlashCtrl, DefaultHasher, 1024>::new(FlashCtrl::new(),
//!                   &mut read_buf, 0x1000);
//!
//! let mut ret = tickfs.initalise((&mut DefaultHasher::new(), &mut DefaultHasher::new()));
//! while ret.is_err() {
//!     // There is no actual delay here, in a real implementation wait on some event
//!     ret = tickfs.continue_initalise(
//!         (&mut DefaultHasher::new(), &mut DefaultHasher::new()),
//!     );
//! }
//!
//! // Then when calling the TickFS function check for the error. For example
//! // when appending a key:
//!
//! // Add a key
//! let value: [u8; 32] = [0x23; 32];
//! let ret = tickfs.append_key(&mut DefaultHasher::new(), b"ONE", &value);
//!
//! match ret {
//!     Err(ErrorCode::ReadNotReady(reg)) => {
//!         // Then once the read operation is ready call the `continue_operation()`
//!         // function.
//!
//!         let ret = tickfs.
//!                       continue_operation(Some(&mut DefaultHasher::new()), Some(b"ONE"), Some(&value), None);
//!     }
//!     Ok(_) => {},
//!     _ => panic!("Other error"),
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
use crate::tickfs::{State, TickFS};
use core::hash::Hasher;

/// The struct storing all of the TickFS information for the async implementation.
pub struct AsyncTickFS<'a, C: FlashController<S>, H: Hasher, const S: usize> {
    /// The main tickFS struct
    pub tickfs: TickFS<'a, C, H, S>,
}

impl<'a, C: FlashController<S>, H: Hasher, const S: usize> AsyncTickFS<'a, C, H, S> {
    /// Create a new struct
    ///
    /// `C`: An implementation of the `FlashController` trait
    ///
    /// `controller`: An new struct implementing `FlashController`
    /// `flash_size`: The total size of the flash used for TickFS
    pub fn new(controller: C, read_buffer: &'a mut [u8; S], flash_size: usize) -> Self {
        Self {
            tickfs: TickFS::<C, H, S>::new(controller, read_buffer, flash_size),
        }
    }

    /// This function setups the flash region to be used as a key-value store.
    /// If the region is already initalised this won't make any changes.
    ///
    /// `H`: An implementation of a `core::hash::Hasher` trait. This MUST
    ///      always return the same hash for the same input. That is the
    ///      implementation can NOT change over time.
    ///
    /// If the specified region has not already been setup for TickFS
    /// the entire region will be erased.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn initalise(&self, hash_function: (&mut H, &mut H)) -> Result<SuccessCode, ErrorCode> {
        self.tickfs.initalise(hash_function)
    }

    /// Appends the key/value pair to flash storage.
    ///
    /// `hash_function`: Hash function with no previous state. This is
    ///                  usually a newly created hash.
    /// `key`: A unhashed key. This will be hashed internally. This key
    ///        will be used in future to retrieve or remove the `value`.
    /// `value`: A buffer containing the data to be stored to flash.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn append_key(
        &self,
        hash_function: &mut H,
        key: &[u8],
        value: &[u8],
    ) -> Result<SuccessCode, ErrorCode> {
        self.tickfs.append_key(hash_function, key, value)
    }

    /// Retrieves the value from flash storage.
    ///
    /// `hash_function`: Hash function with no previous state. This is
    ///                  usually a newly created hash.
    /// `key`: A unhashed key. This will be hashed internally.
    /// `buf`: A buffer to store the value to.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    ///
    /// If a power loss occurs before success is returned the data is
    /// assumed to be lost.
    pub fn get_key(
        &self,
        hash_function: &mut H,
        key: &[u8],
        buf: &mut [u8],
    ) -> Result<SuccessCode, ErrorCode> {
        self.tickfs.get_key(hash_function, key, buf)
    }

    /// Invalidates the key in flash storage
    ///
    /// `hash_function`: Hash function with no previous state. This is
    ///                  usually a newly created hash.
    /// `key`: A unhashed key. This will be hashed internally.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    ///
    /// If a power loss occurs before success is returned the data is
    /// assumed to be lost.
    pub fn invalidate_key(
        &self,
        hash_function: &mut H,
        key: &[u8],
    ) -> Result<SuccessCode, ErrorCode> {
        self.tickfs.invalidate_key(hash_function, key)
    }

    /// Perform a garbage collection on TickFS
    ///
    /// On success the number of bytes freed will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn garbage_collect(&self) -> Result<usize, ErrorCode> {
        self.tickfs.garbage_collect()
    }

    /// Continue the initalise process after an async access.
    ///
    /// `H`: An implementation of a `core::hash::Hasher` trait. This MUST
    ///      always return the same hash for the same input. That is the
    ///      implementation can NOT change over time.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn continue_initalise(
        &self,
        hash_function: (&mut H, &mut H),
    ) -> Result<SuccessCode, ErrorCode> {
        let ret = self.tickfs.initalise(hash_function);

        match ret {
            Ok(_) => self.tickfs.state.set(State::None),
            Err(e) => match e {
                ErrorCode::ReadNotReady(_) | ErrorCode::EraseNotReady(_) => {}
                _ => self.tickfs.state.set(State::None),
            },
        }

        ret
    }

    /// Continue a previous key operation after an async access.
    ///
    /// `hash_function`: Hash function with no previous state. This is
    ///                  usually a newly created hash.
    /// `key`: A unhashed key. This will be hashed internally. This key
    ///        will be used in future to retrieve or remove the `value`.
    /// `value`: A buffer containing the data to be stored to flash.
    /// `buf`: A buffer to store the value to.
    ///
    /// On success nothing will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn continue_operation(
        &self,
        hash_function: Option<&mut H>,
        key: Option<&[u8]>,
        value: Option<&[u8]>,
        buf: Option<&mut [u8]>,
    ) -> Result<SuccessCode, ErrorCode> {
        let ret = match self.tickfs.state.get() {
            State::AppendKey(_) => {
                self.tickfs
                    .append_key(hash_function.unwrap(), key.unwrap(), value.unwrap())
            }
            State::GetKey(_) => {
                self.tickfs
                    .get_key(hash_function.unwrap(), key.unwrap(), buf.unwrap())
            }
            State::InvalidateKey(_) => self
                .tickfs
                .invalidate_key(hash_function.unwrap(), key.unwrap()),
            _ => unreachable!(),
        };

        match ret {
            Ok(_) => self.tickfs.state.set(State::None),
            Err(e) => match e {
                ErrorCode::ReadNotReady(_) | ErrorCode::EraseNotReady(_) => {}
                _ => self.tickfs.state.set(State::None),
            },
        }

        ret
    }

    /// Continue the garbage collection process after an async access.
    ///
    /// On success the number of bytes freed will be returned.
    /// On error a `ErrorCode` will be returned.
    pub fn continue_garbage_collection(&self) -> Result<usize, ErrorCode> {
        let ret = self.tickfs.garbage_collect();

        match ret {
            Ok(_) => self.tickfs.state.set(State::None),
            Err(e) => match e {
                ErrorCode::ReadNotReady(_) | ErrorCode::EraseNotReady(_) => {}
                _ => self.tickfs.state.set(State::None),
            },
        }

        ret
    }
}

/// Tests using a flash controller that can store data
#[cfg(test)]
mod store_flast_ctrl {
    use crate::async_ops::AsyncTickFS;
    use crate::error_codes::ErrorCode;
    use crate::flash_controller::FlashController;
    use crate::tickfs::{HASH_OFFSET, LEN_OFFSET, VERSION, VERSION_OFFSET};
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::collections::hash_map::DefaultHasher;

    fn check_region_main(buf: &[u8]) {
        // Check the version
        assert_eq!(buf[VERSION_OFFSET], VERSION);

        // Check the length
        assert_eq!(buf[LEN_OFFSET], 0x80);
        assert_eq!(buf[LEN_OFFSET + 1], 19);

        // Check the hash
        assert_eq!(buf[HASH_OFFSET + 0], 0x13);
        assert_eq!(buf[HASH_OFFSET + 1], 0x67);
        assert_eq!(buf[HASH_OFFSET + 2], 0xd3);
        assert_eq!(buf[HASH_OFFSET + 3], 0xe4);
        assert_eq!(buf[HASH_OFFSET + 4], 0xe0);
        assert_eq!(buf[HASH_OFFSET + 5], 0x9b);
        assert_eq!(buf[HASH_OFFSET + 6], 0xf7);
        assert_eq!(buf[HASH_OFFSET + 7], 0x6e);

        // Check the check hash
        assert_eq!(buf[HASH_OFFSET + 8], 0xdb);
        assert_eq!(buf[HASH_OFFSET + 9], 0x6d);
        assert_eq!(buf[HASH_OFFSET + 10], 0x81);
        assert_eq!(buf[HASH_OFFSET + 11], 0xc6);
        assert_eq!(buf[HASH_OFFSET + 12], 0x6b);
        assert_eq!(buf[HASH_OFFSET + 13], 0x95);
        assert_eq!(buf[HASH_OFFSET + 14], 0x50);
        assert_eq!(buf[HASH_OFFSET + 15], 0xdc);
    }

    fn check_region_one(buf: &[u8]) {
        // Check the version
        assert_eq!(buf[VERSION_OFFSET], VERSION);

        // Check the length
        assert_eq!(buf[LEN_OFFSET], 0x80);
        assert_eq!(buf[LEN_OFFSET + 1], 51);

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
        assert_eq!(buf[43], 0x08);
        assert_eq!(buf[44], 0x05);
        assert_eq!(buf[45], 0x89);
        assert_eq!(buf[46], 0xef);
        assert_eq!(buf[47], 0x5d);
        assert_eq!(buf[48], 0x42);
        assert_eq!(buf[49], 0x42);
        assert_eq!(buf[50], 0xdc);
    }

    fn check_region_two(buf: &[u8]) {
        // Check the version
        assert_eq!(buf[VERSION_OFFSET], VERSION);

        // Check the length
        assert_eq!(buf[LEN_OFFSET], 0x80);
        assert_eq!(buf[LEN_OFFSET + 1], 51);

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
        assert_eq!(buf[43], 0xdb);
        assert_eq!(buf[44], 0x1d);
        assert_eq!(buf[45], 0xd4);
        assert_eq!(buf[46], 0x8a);
        assert_eq!(buf[47], 0x7b);
        assert_eq!(buf[48], 0x39);
        assert_eq!(buf[49], 0x53);
        assert_eq!(buf[50], 0x8f);
    }

    // An example FlashCtrl implementation
    struct FlashCtrl {
        buf: RefCell<[[u8; 1024]; 64]>,
        run: Cell<u8>,
        async_read_region: Cell<usize>,
        async_erase_region: Cell<usize>,
    }

    impl FlashCtrl {
        fn new() -> Self {
            Self {
                buf: RefCell::new([[0xFF; 1024]; 64]),
                run: Cell::new(0),
                async_read_region: Cell::new(100),
                async_erase_region: Cell::new(100),
            }
        }
    }

    impl FlashController<1024> for FlashCtrl {
        fn read_region(
            &self,
            region_number: usize,
            offset: usize,
            buf: &mut [u8; 1024],
        ) -> Result<(), ErrorCode> {
            println!("Read from region: {}", region_number);

            if self.async_read_region.get() != region_number {
                // Pretend that we aren't ready
                self.async_read_region.set(region_number);
                println!("  Not ready");
                return Err(ErrorCode::ReadNotReady(region_number));
            }

            for (i, b) in buf.iter_mut().enumerate() {
                *b = self.buf.borrow()[region_number][offset + i]
            }

            // println!("  buf: {:#x?}", self.buf.borrow()[region_number]);

            Ok(())
        }

        fn write(&self, address: usize, buf: &[u8]) -> Result<(), ErrorCode> {
            println!(
                "Write to address: {:#x}, region: {}",
                address,
                address / 1024
            );

            for (i, d) in buf.iter().enumerate() {
                self.buf.borrow_mut()[address / 1024][(address % 1024) + i] = *d;
            }

            // Check to see if we are adding a key
            if buf.len() > 1 {
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

            Ok(())
        }

        fn erase_region(&self, region_number: usize) -> Result<(), ErrorCode> {
            println!("Erase region: {}", region_number);

            if self.async_erase_region.get() != region_number {
                // Pretend that we aren't ready
                self.async_erase_region.set(region_number);
                return Err(ErrorCode::EraseNotReady(region_number));
            }

            let mut local_buf = self.buf.borrow_mut()[region_number];

            for d in local_buf.iter_mut() {
                *d = 0xFF;
            }

            Ok(())
        }
    }

    #[test]
    fn test_simple_append() {
        let mut read_buf: [u8; 1024] = [0; 1024];
        let tickfs = AsyncTickFS::<FlashCtrl, DefaultHasher, 1024>::new(
            FlashCtrl::new(),
            &mut read_buf,
            0x1000,
        );

        let mut ret = tickfs.initalise((&mut DefaultHasher::new(), &mut DefaultHasher::new()));
        while ret.is_err() {
            // There is no actual delay in the test, just continue now
            ret = tickfs.continue_initalise((&mut DefaultHasher::new(), &mut DefaultHasher::new()));
        }

        let value: [u8; 32] = [0x23; 32];

        let ret = tickfs.append_key(&mut DefaultHasher::new(), b"ONE", &value);
        match ret {
            Err(ErrorCode::ReadNotReady(_reg)) => {
                // There is no actual delay in the test, just continue now
                tickfs
                    .continue_operation(
                        Some(&mut DefaultHasher::new()),
                        Some(b"ONE"),
                        Some(&value),
                        None,
                    )
                    .unwrap();
            }
            Ok(_) => {}
            _ => unreachable!(),
        }

        let ret = tickfs.append_key(&mut DefaultHasher::new(), b"TWO", &value);
        match ret {
            Err(ErrorCode::ReadNotReady(_reg)) => {
                // There is no actual delay in the test, just continue now
                tickfs
                    .continue_operation(
                        Some(&mut DefaultHasher::new()),
                        Some(b"TWO"),
                        Some(&value),
                        None,
                    )
                    .unwrap();
            }
            Ok(_) => {}
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_double_append() {
        let mut read_buf: [u8; 1024] = [0; 1024];
        let tickfs = AsyncTickFS::<FlashCtrl, DefaultHasher, 1024>::new(
            FlashCtrl::new(),
            &mut read_buf,
            0x10000,
        );

        let mut ret = tickfs.initalise((&mut DefaultHasher::new(), &mut DefaultHasher::new()));
        while ret.is_err() {
            // There is no actual delay in the test, just continue now
            ret = tickfs.continue_initalise((&mut DefaultHasher::new(), &mut DefaultHasher::new()));
        }

        let value: [u8; 32] = [0x23; 32];
        let mut buf: [u8; 32] = [0; 32];

        println!("Add key ONE");
        let ret = tickfs.append_key(&mut DefaultHasher::new(), b"ONE", &value);
        match ret {
            Err(ErrorCode::ReadNotReady(_reg)) => {
                // There is no actual delay in the test, just continue now
                tickfs
                    .continue_operation(
                        Some(&mut DefaultHasher::new()),
                        Some(b"ONE"),
                        Some(&value),
                        None,
                    )
                    .unwrap();
            }
            Ok(_) => {}
            _ => unreachable!(),
        }

        println!("Get key ONE");
        tickfs
            .get_key(&mut DefaultHasher::new(), b"ONE", &mut buf)
            .unwrap();

        println!("Get non-existant key TWO");
        let ret = tickfs.get_key(&mut DefaultHasher::new(), b"TWO", &mut buf);
        match ret {
            Err(ErrorCode::ReadNotReady(_reg)) => {
                // There is no actual delay in the test, just continue now
                assert_eq!(
                    tickfs.continue_operation(
                        Some(&mut DefaultHasher::new()),
                        Some(b"ONE"),
                        None,
                        Some(&mut buf),
                    ),
                    Err(ErrorCode::KeyNotFound)
                );
            }
            Err(ErrorCode::KeyNotFound) => {}
            _ => unreachable!(),
        }

        println!("Add key ONE again");
        let ret = tickfs.append_key(&mut DefaultHasher::new(), b"ONE", &value);
        match ret {
            Err(ErrorCode::ReadNotReady(_reg)) => {
                // There is no actual delay in the test, just continue now
                assert_eq!(
                    tickfs.continue_operation(
                        Some(&mut DefaultHasher::new()),
                        Some(b"ONE"),
                        Some(&value),
                        None,
                    ),
                    Err(ErrorCode::KeyAlreadyExists)
                );
            }
            Err(ErrorCode::KeyAlreadyExists) => {}
            _ => unreachable!(),
        }

        println!("Add key TWO");
        let ret = tickfs.append_key(&mut DefaultHasher::new(), b"TWO", &value);
        match ret {
            Err(ErrorCode::ReadNotReady(_reg)) => {
                // There is no actual delay in the test, just continue now
                tickfs
                    .continue_operation(
                        Some(&mut DefaultHasher::new()),
                        Some(b"TWO"),
                        Some(&value),
                        None,
                    )
                    .unwrap();
            }
            Ok(_) => {}
            _ => unreachable!(),
        }

        println!("Get key ONE");
        let ret = tickfs.get_key(&mut DefaultHasher::new(), b"ONE", &mut buf);
        match ret {
            Err(ErrorCode::ReadNotReady(_reg)) => {
                // There is no actual delay in the test, just continue now
                tickfs
                    .continue_operation(
                        Some(&mut DefaultHasher::new()),
                        Some(b"ONE"),
                        None,
                        Some(&mut buf),
                    )
                    .unwrap();
            }
            Ok(_) => {}
            _ => unreachable!(),
        }

        println!("Get key TWO");
        let ret = tickfs.get_key(&mut DefaultHasher::new(), b"TWO", &mut buf);
        match ret {
            Err(ErrorCode::ReadNotReady(_reg)) => {
                // There is no actual delay in the test, just continue now
                tickfs
                    .continue_operation(
                        Some(&mut DefaultHasher::new()),
                        Some(b"TWO"),
                        None,
                        Some(&mut buf),
                    )
                    .unwrap();
            }
            Ok(_) => {}
            _ => unreachable!(),
        }

        println!("Get non-existant key THREE");
        let ret = tickfs.get_key(&mut DefaultHasher::new(), b"THREE", &mut buf);
        match ret {
            Err(ErrorCode::ReadNotReady(_reg)) => {
                // There is no actual delay in the test, just continue now
                assert_eq!(
                    tickfs.continue_operation(
                        Some(&mut DefaultHasher::new()),
                        Some(b"TWO"),
                        None,
                        Some(&mut buf),
                    ),
                    Err(ErrorCode::KeyNotFound)
                );
            }
            _ => unreachable!(),
        }

        assert_eq!(
            tickfs.get_key(&mut DefaultHasher::new(), b"THREE", &mut buf),
            Err(ErrorCode::KeyNotFound)
        );
    }

    #[test]
    fn test_append_and_delete() {
        let mut read_buf: [u8; 1024] = [0; 1024];
        let tickfs = AsyncTickFS::<FlashCtrl, DefaultHasher, 1024>::new(
            FlashCtrl::new(),
            &mut read_buf,
            0x10000,
        );

        let mut ret = tickfs.initalise((&mut DefaultHasher::new(), &mut DefaultHasher::new()));
        while ret.is_err() {
            // There is no actual delay in the test, just continue now
            ret = tickfs.continue_initalise((&mut DefaultHasher::new(), &mut DefaultHasher::new()));
        }

        let value: [u8; 32] = [0x23; 32];
        let mut buf: [u8; 32] = [0; 32];

        println!("Add key ONE");
        let ret = tickfs.append_key(&mut DefaultHasher::new(), b"ONE", &value);
        match ret {
            Err(ErrorCode::ReadNotReady(_reg)) => {
                // There is no actual delay in the test, just continue now
                tickfs
                    .continue_operation(
                        Some(&mut DefaultHasher::new()),
                        Some(b"ONE"),
                        Some(&value),
                        None,
                    )
                    .unwrap();
            }
            Ok(_) => {}
            _ => unreachable!(),
        }

        println!("Get key ONE");
        tickfs
            .get_key(&mut DefaultHasher::new(), b"ONE", &mut buf)
            .unwrap();

        println!("Delete Key ONE");
        tickfs
            .invalidate_key(&mut DefaultHasher::new(), b"ONE")
            .unwrap();

        println!("Get non-existant key ONE");
        assert_eq!(
            tickfs.get_key(&mut DefaultHasher::new(), b"ONE", &mut buf),
            Err(ErrorCode::KeyNotFound)
        );

        println!("Try to delete Key ONE Again");
        assert_eq!(
            tickfs.invalidate_key(&mut DefaultHasher::new(), b"ONE"),
            Err(ErrorCode::KeyNotFound)
        );
    }

    #[test]
    fn test_garbage_collect() {
        let mut read_buf: [u8; 1024] = [0; 1024];
        let tickfs = AsyncTickFS::<FlashCtrl, DefaultHasher, 1024>::new(
            FlashCtrl::new(),
            &mut read_buf,
            0x10000,
        );

        let mut ret = tickfs.initalise((&mut DefaultHasher::new(), &mut DefaultHasher::new()));
        while ret.is_err() {
            // There is no actual delay in the test, just continue now
            ret = tickfs.continue_initalise((&mut DefaultHasher::new(), &mut DefaultHasher::new()));
        }

        let value: [u8; 32] = [0x23; 32];
        let mut buf: [u8; 32] = [0; 32];

        println!("Garbage collect empty flash");
        let mut ret = tickfs.garbage_collect();
        while ret.is_err() {
            // There is no actual delay in the test, just continue now
            ret = tickfs.continue_garbage_collection();
        }

        println!("Add key ONE");
        let ret = tickfs.append_key(&mut DefaultHasher::new(), b"ONE", &value);
        match ret {
            Err(ErrorCode::ReadNotReady(_reg)) => {
                // There is no actual delay in the test, just continue now
                tickfs
                    .continue_operation(
                        Some(&mut DefaultHasher::new()),
                        Some(b"ONE"),
                        Some(&value),
                        None,
                    )
                    .unwrap();
            }
            Ok(_) => {}
            _ => unreachable!(),
        }

        println!("Garbage collect flash with valid key");
        let mut ret = tickfs.garbage_collect();
        while ret.is_err() {
            match ret {
                Err(ErrorCode::ReadNotReady(_reg)) => {
                    // There is no actual delay in the test, just continue now
                    ret = tickfs.continue_garbage_collection();
                }
                Ok(num) => {
                    assert_eq!(num, 0);
                }
                _ => unreachable!(),
            }
        }

        println!("Delete Key ONE");
        let ret = tickfs.invalidate_key(&mut DefaultHasher::new(), b"ONE");
        match ret {
            Err(ErrorCode::ReadNotReady(_reg)) => {
                // There is no actual delay in the test, just continue now
                tickfs
                    .continue_operation(Some(&mut DefaultHasher::new()), Some(b"ONE"), None, None)
                    .unwrap();
            }
            Ok(_) => {}
            _ => unreachable!("ret: {:?}", ret),
        }

        println!("Garbage collect flash with deleted key");
        let mut ret = tickfs.garbage_collect();
        while ret.is_err() {
            match ret {
                Err(ErrorCode::ReadNotReady(_reg)) => {
                    // There is no actual delay in the test, just continue now
                    ret = tickfs.continue_garbage_collection();
                }
                Err(ErrorCode::EraseNotReady(_reg)) => {
                    // There is no actual delay in the test, just continue now
                    ret = tickfs.continue_garbage_collection();
                }
                Ok(num) => {
                    assert_eq!(num, 1024);
                }
                _ => unreachable!("ret: {:?}", ret),
            }
        }

        println!("Get non-existant key ONE");
        let ret = tickfs.get_key(&mut DefaultHasher::new(), b"ONE", &mut buf);
        match ret {
            Err(ErrorCode::ReadNotReady(_reg)) => {
                // There is no actual delay in the test, just continue now
                assert_eq!(
                    tickfs.continue_operation(
                        Some(&mut DefaultHasher::new()),
                        Some(b"ONE"),
                        None,
                        Some(&mut buf),
                    ),
                    Err(ErrorCode::KeyNotFound)
                );
            }
            Err(ErrorCode::KeyNotFound) => {}
            _ => unreachable!("ret: {:?}", ret),
        }

        println!("Add Key ONE");
        tickfs
            .append_key(&mut DefaultHasher::new(), b"ONE", &value)
            .unwrap();
    }
}
