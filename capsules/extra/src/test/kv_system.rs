// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Test for Tock KV System capsules.
//!
//! This capsule implements the tests for KV system libraries in Tock.
//! This is originally written to test TicKV.
//!
//!    hil::flash
//!
//! The tests can be enabled by adding this line to the `main()`
//!
//! ```rust,ignore
//! tickv_test::run_tickv_tests(kvstore)
//! ```
//!
//! You should then see the following output
//!
//! ```text
//! ---Starting TicKV Tests---
//! Key: [18, 52, 86, 120, 154, 188, 222, 240] with value [16, 32, 48] was added
//! Now retrieving the key
//! Key: [18, 52, 86, 120, 154, 188, 222, 240] with value [16, 32, 48, 0] was retrieved
//! Removed Key: [18, 52, 86, 120, 154, 188, 222, 240]
//! Try to read removed key: [18, 52, 86, 120, 154, 188, 222, 240]
//! Unable to find key: [18, 52, 86, 120, 154, 188, 222, 240]
//! Let's start a garbage collection
//! Finished garbage collection
//! ---Finished TicKV Tests---
//! ```

use crate::tickv::{KVSystem, KVSystemClient, KeyType};
use core::cell::Cell;
use core::marker::PhantomData;
use kernel::debug;
use kernel::utilities::cells::{MapCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

#[derive(Clone, Copy, PartialEq)]
enum CurrentState {
    Normal,
    ExpectGetValueFail,
}

pub struct KVSystemTest<'a, S: KVSystem<'static>, T: KeyType> {
    kv_system: &'a S,
    phantom: PhantomData<&'a T>,
    value: MapCell<SubSliceMut<'static, u8>>,
    ret_buffer: TakeCell<'static, [u8]>,
    state: Cell<CurrentState>,
}

impl<'a, S: KVSystem<'static>, T: KeyType> KVSystemTest<'a, S, T> {
    pub fn new(
        kv_system: &'a S,
        value: SubSliceMut<'static, u8>,
        static_buf: &'static mut [u8; 4],
    ) -> KVSystemTest<'a, S, T> {
        debug!("---Starting TicKV Tests---");

        Self {
            kv_system: kv_system,
            phantom: PhantomData,
            value: MapCell::new(value),
            ret_buffer: TakeCell::new(static_buf),
            state: Cell::new(CurrentState::Normal),
        }
    }
}

impl<'a, S: KVSystem<'static, K = T>, T: KeyType + core::fmt::Debug> KVSystemClient<T>
    for KVSystemTest<'a, S, T>
{
    fn generate_key_complete(
        &self,
        result: Result<(), ErrorCode>,
        _unhashed_key: SubSliceMut<'static, u8>,
        key_buf: &'static mut T,
    ) {
        match result {
            Ok(()) => {
                debug!("Generated key: {:?}", key_buf);
                debug!("Now appending the key");
                self.kv_system
                    .append_key(key_buf, self.value.take().unwrap())
                    .unwrap();
            }
            Err(e) => {
                panic!("Error adding key: {:?}", e);
            }
        }
    }

    fn append_key_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut T,
        value: SubSliceMut<'static, u8>,
    ) {
        match result {
            Ok(()) => {
                debug!("Key: {:?} with value {:?} was added", key, value);
                debug!("Now retrieving the key");
                self.kv_system
                    .get_value(key, SubSliceMut::new(self.ret_buffer.take().unwrap()))
                    .unwrap();
            }
            Err(e) => {
                panic!("Error adding key: {:?}", e);
            }
        }
    }

    fn get_value_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut T,
        ret_buf: SubSliceMut<'static, u8>,
    ) {
        match result {
            Ok(()) => {
                debug!("Key: {:?} with value {:?} was retrieved", key, ret_buf);
                self.ret_buffer.replace(ret_buf.take());
                self.kv_system.invalidate_key(key).unwrap();
            }
            Err(e) => {
                if self.state.get() == CurrentState::ExpectGetValueFail {
                    // We expected this failure
                    debug!("Unable to find key: {:?}", key);
                    self.state.set(CurrentState::Normal);

                    debug!("Let's start a garbage collection");
                    self.kv_system.garbage_collect().unwrap();
                } else {
                    panic!("Error finding key: {:?}", e);
                }
            }
        }
    }

    fn invalidate_key_complete(&self, result: Result<(), ErrorCode>, key: &'static mut T) {
        match result {
            Ok(()) => {
                debug!("Removed Key: {:?}", key);

                debug!("Try to read removed key: {:?}", key);
                self.state.set(CurrentState::ExpectGetValueFail);
                self.kv_system
                    .get_value(key, SubSliceMut::new(self.ret_buffer.take().unwrap()))
                    .unwrap();
            }
            Err(e) => {
                panic!("Error invalidating key: {:?}", e);
            }
        }
    }

    fn garbage_collect_complete(&self, result: Result<(), ErrorCode>) {
        match result {
            Ok(()) => {
                debug!("Finished garbage collection");
                debug!("---Finished TicKV Tests---");
            }
            Err(e) => {
                panic!("Error running garbage collection: {:?}", e);
            }
        }
    }
}
