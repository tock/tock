//! Tock SipHash capsule.
//!
//! This is a async implementation of the SipHash.
//!
//! This capsule was originally written to be used as part of Tocks
//! key/value store. SipHash was used as it is generally fast, while also
//! being resilient against DOS attacks from userspace
//! (unlike <https://github.com/servo/rust-fnv>).
//!
//! Read <https://github.com/veorq/SipHash/blob/master/README.md> for more
//! details on SipHash.
//!
//! The implementation is based on the Rust implementation from
//! rust-core, avaliable here: <https://github.com/jedisct1/rust-siphash>
//!
//! Copyright 2012-2016 The Rust Project Developers.
//! Copyright 2016-2021 Frank Denis.
//! Copyright 2021 Western Digital
//!
//! Licensed under the Apache License, Version 2.0 LICENSE-APACHE or
//! <http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
//! LICENSE-MIT or <http://opensource.org/licenses/MIT>, at your
//! option.

use core::cell::Cell;
use core::convert::TryInto;
use core::{cmp, mem};
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::hasher::{Client, Hasher, SipHash};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::LeasableBuffer;
use kernel::utilities::leasable_buffer::LeasableBufferDynamic;
use kernel::utilities::leasable_buffer::LeasableMutableBuffer;
use kernel::ErrorCode;

pub struct SipHasher24<'a> {
    client: OptionalCell<&'a dyn Client<8>>,

    hasher: Cell<SipHasher>,

    add_data_deferred_call: Cell<bool>,
    complete_deferred_call: Cell<bool>,
    deferred_call: DeferredCall,

    data_buffer: Cell<Option<LeasableBufferDynamic<'static, u8>>>,
    out_buffer: TakeCell<'static, [u8; 8]>,
}

#[derive(Debug, Clone, Copy)]
struct SipHasher {
    k0: u64,
    k1: u64,
    length: usize, // how many bytes we've processed
    state: State,  // hash State
    tail: u64,     // unprocessed bytes le
    ntail: usize,  // how many bytes in tail are valid
}

#[derive(Debug, Clone, Copy)]
struct State {
    // v0, v2 and v1, v3 show up in pairs in the algorithm,
    // and simd implementations of SipHash will use vectors
    // of v02 and v13. By placing them in this order in the struct,
    // the compiler can pick up on just a few simd optimizations by itself.
    v0: u64,
    v2: u64,
    v1: u64,
    v3: u64,
}

impl<'a> SipHasher24<'a> {
    pub fn new() -> Self {
        let hasher = SipHasher {
            k0: 0,
            k1: 0,
            length: 0,
            state: State {
                v0: 0x736f6d6570736575,
                v1: 0x646f72616e646f6d,
                v2: 0x6c7967656e657261,
                v3: 0x7465646279746573,
            },
            ntail: 0,
            tail: 0,
        };

        Self {
            client: OptionalCell::empty(),
            hasher: Cell::new(hasher),
            add_data_deferred_call: Cell::new(false),
            complete_deferred_call: Cell::new(false),
            deferred_call: DeferredCall::new(),
            data_buffer: Cell::new(None),
            out_buffer: TakeCell::empty(),
        }
    }

    pub fn new_with_keys(k0: u64, k1: u64) -> Self {
        let hasher = SipHasher {
            k0,
            k1,
            length: 0,
            state: State {
                v0: 0x736f6d6570736575,
                v1: 0x646f72616e646f6d,
                v2: 0x6c7967656e657261,
                v3: 0x7465646279746573,
            },
            ntail: 0,
            tail: 0,
        };

        Self {
            client: OptionalCell::empty(),
            hasher: Cell::new(hasher),
            add_data_deferred_call: Cell::new(false),
            complete_deferred_call: Cell::new(false),
            deferred_call: DeferredCall::new(),
            data_buffer: Cell::new(None),
            out_buffer: TakeCell::empty(),
        }
    }
}

macro_rules! compress {
    ($state:expr) => {{
        compress!($state.v0, $state.v1, $state.v2, $state.v3)
    }};
    ($v0:expr, $v1:expr, $v2:expr, $v3:expr) => {{
        $v0 = $v0.wrapping_add($v1);
        $v1 = $v1.rotate_left(13);
        $v1 ^= $v0;
        $v0 = $v0.rotate_left(32);
        $v2 = $v2.wrapping_add($v3);
        $v3 = $v3.rotate_left(16);
        $v3 ^= $v2;
        $v0 = $v0.wrapping_add($v3);
        $v3 = $v3.rotate_left(21);
        $v3 ^= $v0;
        $v2 = $v2.wrapping_add($v1);
        $v1 = $v1.rotate_left(17);
        $v1 ^= $v2;
        $v2 = $v2.rotate_left(32);
    }};
}

fn read_le_u64(input: &[u8]) -> u64 {
    let (int_bytes, _rest) = input.split_at(mem::size_of::<u64>());
    u64::from_le_bytes(int_bytes.try_into().unwrap())
}

fn read_le_u16(input: &[u8]) -> u16 {
    let (int_bytes, _rest) = input.split_at(mem::size_of::<u16>());
    u16::from_le_bytes(int_bytes.try_into().unwrap())
}

#[inline]
fn u8to64_le(buf: &[u8], start: usize, len: usize) -> u64 {
    debug_assert!(len < 8);
    let mut i = 0; // current byte index (from LSB) in the output u64
    let mut out = 0;
    if i + 3 < len {
        out = read_le_u64(&buf[start + i..]);
        i += 4;
    }
    if i + 1 < len {
        out |= (read_le_u16(&buf[start + i..]) as u64) << (i * 8);
        i += 2
    }
    if i < len {
        out |= (*buf.get(start + i).unwrap() as u64) << (i * 8);
        i += 1;
    }
    debug_assert_eq!(i, len);
    out
}

impl<'a> Hasher<'a, 8> for SipHasher24<'a> {
    fn set_client(&'a self, client: &'a dyn Client<8>) {
        self.client.set(client);
    }

    fn add_data(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<usize, (ErrorCode, &'static [u8])> {
        let length = data.len();
        let msg = data.take();
        let mut hasher = self.hasher.get();

        hasher.length += length;

        let mut needed = 0;

        if hasher.ntail != 0 {
            needed = 8 - hasher.ntail;
            hasher.tail |= u8to64_le(msg, 0, cmp::min(length, needed)) << (8 * hasher.ntail);
            if length < needed {
                hasher.ntail += length;
                return Ok(length);
            } else {
                hasher.state.v3 ^= hasher.tail;
                compress!(&mut hasher.state);
                compress!(&mut hasher.state);
                hasher.state.v0 ^= hasher.tail;
                hasher.ntail = 0;
            }
        }

        // Buffered tail is now flushed, process new input.
        let len = length - needed;
        let left = len & 0x7;

        let mut i = needed;
        while i < len - left {
            let mi = read_le_u64(&msg[i..]);

            hasher.state.v3 ^= mi;
            compress!(&mut hasher.state);
            compress!(&mut hasher.state);
            hasher.state.v0 ^= mi;

            i += 8;
        }

        hasher.tail = u8to64_le(msg, i, left);
        hasher.ntail = left;

        self.hasher.set(hasher);
        self.data_buffer
            .set(Some(LeasableBufferDynamic::Immutable(LeasableBuffer::new(
                msg,
            ))));

        self.add_data_deferred_call.set(true);
        self.deferred_call.set();

        Ok(length)
    }

    fn add_mut_data(
        &self,
        data: LeasableMutableBuffer<'static, u8>,
    ) -> Result<usize, (ErrorCode, &'static mut [u8])> {
        let length = data.len();
        let msg = data.take();
        let mut hasher = self.hasher.get();

        hasher.length += length;

        let mut needed = 0;

        if hasher.ntail != 0 {
            needed = 8 - hasher.ntail;
            hasher.tail |= u8to64_le(msg, 0, cmp::min(length, needed)) << (8 * hasher.ntail);
            if length < needed {
                hasher.ntail += length;
                return Ok(length);
            } else {
                hasher.state.v3 ^= hasher.tail;
                compress!(&mut hasher.state);
                compress!(&mut hasher.state);
                hasher.state.v0 ^= hasher.tail;
                hasher.ntail = 0;
            }
        }

        // Buffered tail is now flushed, process new input.
        let len = length - needed;
        let left = len & 0x7;

        let mut i = needed;
        while i < len - left {
            let mi = read_le_u64(&msg[i..]);

            hasher.state.v3 ^= mi;
            compress!(&mut hasher.state);
            compress!(&mut hasher.state);
            hasher.state.v0 ^= mi;

            i += 8;
        }

        hasher.tail = u8to64_le(msg, i, left);
        hasher.ntail = left;

        self.hasher.set(hasher);
        self.data_buffer.set(Some(LeasableBufferDynamic::Mutable(
            LeasableMutableBuffer::new(msg),
        )));

        self.add_data_deferred_call.set(true);
        self.deferred_call.set();

        Ok(length)
    }

    fn run(
        &'a self,
        digest: &'static mut [u8; 8],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 8])> {
        let mut hasher = self.hasher.get();

        let b: u64 = ((hasher.length as u64 & 0xff) << 56) | hasher.tail;

        hasher.state.v3 ^= b;
        compress!(&mut hasher.state);
        compress!(&mut hasher.state);
        hasher.state.v0 ^= b;

        hasher.state.v2 ^= 0xff;
        compress!(&mut hasher.state);
        compress!(&mut hasher.state);
        compress!(&mut hasher.state);
        compress!(&mut hasher.state);

        self.hasher.set(hasher);
        self.out_buffer.replace(digest);

        self.complete_deferred_call.set(true);
        self.deferred_call.set();

        Ok(())
    }

    fn clear_data(&self) {
        let mut hasher = self.hasher.get();

        hasher.length = 0;
        hasher.state.v0 = hasher.k0 ^ 0x736f6d6570736575;
        hasher.state.v1 = hasher.k1 ^ 0x646f72616e646f6d;
        hasher.state.v2 = hasher.k0 ^ 0x6c7967656e657261;
        hasher.state.v3 = hasher.k1 ^ 0x7465646279746573;
        hasher.ntail = 0;

        self.hasher.set(hasher);
    }
}

impl<'a> SipHash for SipHasher24<'a> {
    fn set_keys(&self, k0: u64, k1: u64) -> Result<(), ErrorCode> {
        let mut hasher = self.hasher.get();

        hasher.k0 = k0;
        hasher.k1 = k1;

        self.hasher.set(hasher);
        self.clear_data();

        Ok(())
    }
}

impl<'a> DeferredCallClient for SipHasher24<'a> {
    fn handle_deferred_call(&self) {
        if self.add_data_deferred_call.get() {
            self.add_data_deferred_call.set(false);

            self.client.map(|client| {
                self.data_buffer.take().map(|buffer| match buffer {
                    LeasableBufferDynamic::Immutable(b) => client.add_data_done(Ok(()), b.take()),
                    LeasableBufferDynamic::Mutable(b) => client.add_mut_data_done(Ok(()), b.take()),
                });
            });
        }

        if self.complete_deferred_call.get() {
            self.complete_deferred_call.set(false);

            self.client.map(|client| {
                self.out_buffer.take().map(|buffer| {
                    let state = self.hasher.get().state;

                    let result = state.v0 ^ state.v1 ^ state.v2 ^ state.v3;
                    buffer.copy_from_slice(&result.to_le_bytes());

                    client.hash_done(Ok(()), buffer);
                });
            });
        }
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
