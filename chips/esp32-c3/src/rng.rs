// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::entropy::{self, Client32, Continue, Entropy32};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::Readable;
use kernel::utilities::registers::{register_structs, ReadOnly};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

const RNG_DATA_REG: StaticRef<RngRegister> =
    unsafe { StaticRef::new(0x6002_60B0 as *const RngRegister) };

register_structs! {
    pub RngRegister {
        (0x0 => data: ReadOnly<u32>),
        (0x4 => @END),
    }
}

pub struct Rng<'a> {
    register: StaticRef<RngRegister>,
    client: OptionalCell<&'a dyn entropy::Client32>,
    value: OptionalCell<u32>,
    deferred_call: DeferredCall,
}

impl<'a> Rng<'a> {
    pub fn new() -> Rng<'a> {
        Rng {
            register: RNG_DATA_REG,
            client: OptionalCell::empty(),
            value: OptionalCell::empty(),
            deferred_call: DeferredCall::new(),
        }
    }
}

impl DeferredCallClient for Rng<'_> {
    fn register(&'static self) {
        self.deferred_call.register(self);
    }

    fn handle_deferred_call(&self) {
        self.value.set(self.register.data.get());
        self.client.map(|client| {
            if let Continue::More = client.entropy_available(&mut RngIter(self), Ok(())) {
                self.deferred_call.set();
            };
        });
    }
}

impl<'a> Entropy32<'a> for Rng<'a> {
    fn get(&self) -> Result<(), ErrorCode> {
        self.deferred_call.set();
        Ok(())
    }

    fn cancel(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn set_client(&self, client: &'a dyn Client32) {
        self.client.set(client);
    }
}

struct RngIter<'a, 'b: 'a>(&'a Rng<'b>);

impl Iterator for RngIter<'_, '_> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        self.0.value.take()
    }
}
