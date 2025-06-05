// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

pub static KEY: &[u8; kernel::hil::symmetric_encryption::AES128_KEY_SIZE] = b"InsecureAESKey12";

pub struct EncryptionOracleDriver {}

impl EncryptionOracleDriver {
    /// Create a new instance of our encryption oracle userspace driver:
    pub fn new() -> Self {
        EncryptionOracleDriver {}
    }
}
