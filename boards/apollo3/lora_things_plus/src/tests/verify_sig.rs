// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::tests::run_kernel_op;
use crate::ATECC508A;
use core::cell::Cell;
use kernel::hil::public_key_crypto::signature::ClientVerify;
use kernel::hil::public_key_crypto::signature::SignatureVerify;
use kernel::static_init;
use kernel::utilities::cells::TakeCell;
use kernel::{debug, ErrorCode};

struct HmacTestCallback {
    verify_done: Cell<bool>,
    message_buffer: TakeCell<'static, [u8; 32]>,
    signature_buffer: TakeCell<'static, [u8; 64]>,
    pub_key_buffer: TakeCell<'static, [u8; 64]>,
}

impl<'a> HmacTestCallback {
    fn new(
        message_buffer: &'static mut [u8; 32],
        signature_buffer: &'static mut [u8; 64],
        pub_key_buffer: &'static mut [u8; 64],
    ) -> Self {
        HmacTestCallback {
            verify_done: Cell::new(false),
            message_buffer: TakeCell::new(message_buffer),
            signature_buffer: TakeCell::new(signature_buffer),
            pub_key_buffer: TakeCell::new(pub_key_buffer),
        }
    }

    fn reset(&self) {
        self.verify_done.set(false);
    }
}

impl<'a> ClientVerify<32, 64> for HmacTestCallback {
    fn verification_done(
        &self,
        result: Result<bool, ErrorCode>,
        hash: &'static mut [u8; 32],
        signature: &'static mut [u8; 64],
    ) {
        debug!("Verification Complete");

        self.message_buffer.replace(hash);
        self.signature_buffer.replace(signature);
        assert_eq!(result, Ok(true));
        self.verify_done.set(true);
    }
}

/// The below values are generated from the following Python code
///
/// ```python
/// import ecdsa
/// from ecdsa import SigningKey, NIST256p
/// from hashlib import sha256
///
/// sk = ecdsa.SigningKey.generate(curve=NIST256p, hashfunc=sha256)
///
/// # Write the keys in PEM for future reference
/// with open("priv_key.pem", "wb") as f:
///     f.write(sk.to_pem(format="pkcs8"))
/// with open("pub_key.pem", "wb") as f:
///     f.write(public_key.to_pem())
///
/// # Public Key with X and Y values in a single 64-byte hex array
/// sk.verifying_key.to_string().hex()
///
/// # Dump the Private Key so we have it in hex
/// sk.to_string().hex()
///
/// # Prints the R and the S value in a single 64-byte hex array
/// sk.sign_deterministic(b"This is a test message to sign!!").hex()
///
/// # The ATECC508A operates on a hash of the message, so calculate that
/// sha = sha256()
/// sha.update(b"This is a test message to sign!!")
/// sha.digest().hex()
/// ```
///
// These are the generated test keys used below, please do not use them
// for anything important!!!!
//
// These keys are not leaked, they are only used for this test case.
//
// -----BEGIN PRIVATE KEY-----
// MIGHAgEBMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgWClhguWHtAK85Kqc
// /BucDBQMGQw6R2PEQkyISHkn5xWhRANCAAQUFMTFoNL9oFpGmg6Cp351hQMq9hol
// KpEdQfjP1nYF1jxqz52YjPpFHvudkK/fFsik5Rd0AevNkQqjBdWEqmpW
// -----END PRIVATE KEY-----
//
// -----BEGIN PUBLIC KEY-----
// MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEFBTExaDS/aBaRpoOgqd+dYUDKvYa
// JSqRHUH4z9Z2BdY8as+dmIz6RR77nZCv3xbIpOUXdAHrzZEKowXVhKpqVg==
// -----END PUBLIC KEY-----
macro_rules! static_init_test_cb {
    () => {{
        let message_data = static_init!(
            [u8; 32],
            [
            // The SHA256 of the message: 5468697320697320612074657374206d65737361676520746f207369676e2121
                0x61, 0xff, 0x79, 0x61, 0x27, 0xe5, 0xf8, 0xe4, 0x61, 0x8d, 0xde, 0x14, 0x4f,
                0x5b, 0x91, 0xcc, 0xa4, 0x47, 0x16, 0xda, 0xc8, 0x75, 0x8b, 0xe2, 0x85, 0x9e,
                0xbf, 0x1d, 0xb1, 0x2f, 0xe2, 0xc7,
            ]
        );
        let signature_data = static_init!(
            [u8; 64],
            [
                0xd7, 0x09, 0xd8, 0x2a, 0xdc, 0x15, 0x3c, 0xc4, 0x2e, 0x37, 0xd3, 0x91, 0x92,
                0xe2, 0x0d, 0x6a, 0xa9, 0x68, 0xf7, 0x10, 0xbb, 0x38, 0xc2, 0x16, 0xf3, 0x4f,
                0x59, 0xdc, 0x69, 0x72, 0x59, 0xc2, 0xe3, 0x9c, 0x27, 0x7f, 0x32, 0x63, 0xc8,
                0xbf, 0x27, 0x26, 0x5b, 0x8a, 0x11, 0x68, 0x90, 0x02, 0xa6, 0x7b, 0x3e, 0x72,
                0x59, 0x9e, 0x6c, 0x85, 0xda, 0x00, 0xc8, 0xca, 0x87, 0x37, 0x7d, 0x1a
            ]
        );
        // Note that the private key is: 58296182e587b402bce4aa9cfc1b9c0c140c190c3a4763c4424c88487927e715
        let public_key = static_init!(
            [u8; 64],
            [
                0x14, 0x14, 0xc4, 0xc5, 0xa0, 0xd2, 0xfd, 0xa0, 0x5a, 0x46, 0x9a, 0x0e, 0x82,
                0xa7, 0x7e, 0x75, 0x85, 0x03, 0x2a, 0xf6, 0x1a, 0x25, 0x2a, 0x91, 0x1d, 0x41,
                0xf8, 0xcf, 0xd6, 0x76, 0x05, 0xd6, 0x3c, 0x6a, 0xcf, 0x9d, 0x98, 0x8c, 0xfa,
                0x45, 0x1e, 0xfb, 0x9d, 0x90, 0xaf, 0xdf, 0x16, 0xc8, 0xa4, 0xe5, 0x17, 0x74,
                0x01, 0xeb, 0xcd, 0x91, 0x0a, 0xa3, 0x05, 0xd5, 0x84, 0xaa, 0x6a, 0x56

            ]
        );

        static_init!(
            HmacTestCallback,
            HmacTestCallback::new(message_data, signature_data, public_key)
        )
    }};
}

#[test_case]
fn hmac_check_load_binary() {
    let atecc508a = unsafe { ATECC508A.unwrap() };

    let callback = unsafe { static_init_test_cb!() };

    debug!("check signature verify... ");
    run_kernel_op(100);

    SignatureVerify::set_verify_client(atecc508a, callback);
    callback.reset();

    atecc508a.set_public_key(Some(callback.pub_key_buffer.take().unwrap()));

    assert_eq!(
        atecc508a.verify(
            callback.message_buffer.take().unwrap(),
            callback.signature_buffer.take().unwrap()
        ),
        Ok(())
    );

    run_kernel_op(20_000);
    assert_eq!(callback.verify_done.get(), true);

    debug!("    [ok]");
    run_kernel_op(100);
}
