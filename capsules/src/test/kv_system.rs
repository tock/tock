//! Test for Tock KV System capsules.
//!
//! This capsule implements the tests for KV system libraies in Tock.
//! This is originally written to test TicKV.
//!
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
//!
//! The tests can be enabled by adding this line to the `main()`
//!
//! ```rust
//! tickv_test::run_tickv_tests(kvstore)
//! ```
//!
//! You should then see the following output
//!
//! ```
//! ---Starting TicKV Tests---
//! Key: [18, 52, 86, 120, 154, 188, 222, 240] with value [16, 32, 48] was added
//! Now retriving the key
//! Key: [18, 52, 86, 120, 154, 188, 222, 240] with value [16, 32, 48, 0] was retrived
//! Removed Key: [18, 52, 86, 120, 154, 188, 222, 240]
//! Try to read removed key: [18, 52, 86, 120, 154, 188, 222, 240]
//! Unable to find key: [18, 52, 86, 120, 154, 188, 222, 240]
//! Let's start a garbage collection
//! Finished garbage collection
//! ---Finished TicKV Tests---
//! ```

use core::cell::Cell;
use core::marker::PhantomData;
use kernel::debug;
use kernel::hil::kv_system::{self, KVSystem, KeyType};
use kernel::utilities::cells::TakeCell;
use kernel::ErrorCode;

#[derive(Clone, Copy, PartialEq)]
enum CurrentState {
    Normal,
    ExpectGetValueFail,
}

pub struct KVSystemTest<'a, S: KVSystem<'static>, T: KeyType> {
    kv_system: &'a S,
    phantom: PhantomData<&'a T>,
    ret_buffer: TakeCell<'static, [u8]>,
    state: Cell<CurrentState>,
}

impl<'a, S: KVSystem<'static>, T: KeyType> KVSystemTest<'a, S, T> {
    pub fn new(kv_system: &'a S, static_buf: &'static mut [u8; 4]) -> KVSystemTest<'a, S, T> {
        debug!("---Starting TicKV Tests---");

        Self {
            kv_system: kv_system,
            phantom: PhantomData,
            ret_buffer: TakeCell::new(static_buf),
            state: Cell::new(CurrentState::Normal),
        }
    }
}

impl<'a, S: KVSystem<'static, K = T>, T: KeyType + core::fmt::Debug> kv_system::Client<T>
    for KVSystemTest<'a, S, T>
{
    fn generate_key_complete(
        &self,
        _result: Result<(), ErrorCode>,
        _unhashed_key: &'static [u8],
        _key_buf: &'static T,
    ) {
        unimplemented!()
    }

    fn append_key_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut T,
        value: &'static [u8],
    ) {
        match result {
            Ok(()) => {
                debug!("Key: {:?} with value {:?} was added", key, value);
                debug!("Now retriving the key");
                self.kv_system
                    .get_value(key, self.ret_buffer.take().unwrap())
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
        ret_buf: &'static mut [u8],
    ) {
        match result {
            Ok(()) => {
                debug!("Key: {:?} with value {:?} was retrived", key, ret_buf);
                self.ret_buffer.replace(ret_buf);
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
                    .get_value(key, self.ret_buffer.take().unwrap())
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
