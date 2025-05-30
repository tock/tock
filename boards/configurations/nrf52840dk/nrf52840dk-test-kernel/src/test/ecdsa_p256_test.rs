// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! This tests a software ECDSA P256 implementation. To run this test,
//! add this line to the boot sequence:
//! ```
//! test::ecdsa_p256_test::run_ecdsa_p256();
//! ```

use capsules_core::test::capsule_test::{CapsuleTest, CapsuleTestClient};
use core::ptr::addr_of;
use core::ptr::addr_of_mut;
use ecdsa_sw::p256_signer::EcdsaP256SignatureSigner;
use ecdsa_sw::test::p256::TestEcdsaP256Sign;
use kernel::static_init;

pub static mut HBUF: [u8; 32] = [0; 32]; // TODO: SET
pub static mut SKEY: [u8; 32] = [0; 32]; // TODO: SET
pub static mut HSIG: [u8; 64] = [0; 64];
pub static mut CSIG: [u8; 64] = [0; 64]; // TODO: SET

pub unsafe fn run_ecdsa_p256(client: &'static dyn CapsuleTestClient) {
    let t = static_init_test_ecdsa_p256(client);
    t.run();
}

unsafe fn static_init_test_ecdsa_p256(
    client: &'static dyn CapsuleTestClient,
) -> &'static TestEcdsaP256Sign {
    let ecdsa = static_init!(
        EcdsaP256SignatureSigner<'static>,
        EcdsaP256SignatureSigner::new(&*addr_of!(SKEY)),
    );
    kernel::deferred_call::DeferredCallClient::register(ecdsa);

    let test = static_init!(
        TestEcdsaP256Sign,
        TestEcdsaP256Sign::new(
            ecdsa,
            &mut *addr_of_mut!(HBUF),
            &mut *addr_of_mut!(HSIG),
            &mut *addr_of_mut!(CSIG)
        )
    );

    test.set_client(client);

    test
}
