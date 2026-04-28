// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Mechanism for verifying signatures with public keys stored in kernel attributes.
//!
//! This capsule should be used when a system wants to verify signatures using
//! public keys stored as kernel attributes in flash. Keys are read from the
//! kernel attributes section at runtime, so no additional key storage buffers
//! are needed beyond a single working buffer for the active key.
//!
//! The capsule searches the provided kernel attributes slice for entries with
//! type `0x0104` (public key). Each such entry must have a value of `KL` bytes.
//!
//! The intended layering with this capsule looks like this:
//!
//! ```text
//!   ┌───────────────────────────────────────┐
//!   │                                       │
//!   │         Signature User                │
//!   │ (e.g., `AppCheckerSignature`)         │
//!   │                                       │
//!   └───────────────────────────────────────┘
//!     SignatureVerify + SelectKey         ^
//!   ┌───────────────────────────────────┐ │
//!   │                                   │ │
//!   │ SignatureVerifyKernelAttributeKeys│ │SignatureVerifyClient
//!   │    (this module)                  │ │
//!   │                                   │ │
//!   └───────────────────────────────────┘ │
//!     SignatureVerify + SetKeyBySlice     │
//!   ┌───────────────────────────────────────┐
//!   │                                       │
//!   │         Signature Verifier            │
//!   │  (e.g., `EcdsaP256SignatureVerifier`) │
//!   │                                       │
//!   └───────────────────────────────────────┘
//! ```

use kernel::ErrorCode;
use kernel::hil;
use kernel::utilities::cells::MapCell;
use kernel::utilities::cells::OptionalCell;

/// Kernel attribute type value for public keys.
const KERNEL_ATTR_PUBLIC_KEY_TYPE: u16 = 0x0104;

/// Size of the attributes section trailer: 4-byte sentinel ("TOCK") + 1-byte
/// version + 3 reserved bytes.
const KERNEL_ATTR_TRAILER_LEN: usize = 8;

/// Iterator over public key entries (type `0x0104`) in the kernel attributes
/// slice.
struct PublicKeyAttrIter<'a> {
    remaining: &'a [u8],
}

impl<'a> PublicKeyAttrIter<'a> {
    fn new(attributes: &'a [u8]) -> Self {
        let remaining = attributes
            .len()
            .checked_sub(KERNEL_ATTR_TRAILER_LEN)
            .map_or(&[][..], |end| &attributes[..end]);
        Self { remaining }
    }
}

impl<'a> Iterator for PublicKeyAttrIter<'a> {
    /// `(algorithm_id, key_metadata, key_bytes)`
    type Item = (u16, u32, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Check if there is room for one TLV header.
            if self.remaining.len() < 4 {
                return None;
            }
            let end = self.remaining.len();
            let tlv_len =
                u16::from_le_bytes([self.remaining[end - 2], self.remaining[end - 1]]) as usize;
            let tlv_type = u16::from_le_bytes([self.remaining[end - 4], self.remaining[end - 3]]);

            // Check if there is room for the TLV data.
            let total_size = tlv_len + 4;
            if self.remaining.len() < total_size {
                return None;
            }

            // Collect this TLV and skip over it for the next loop iteration.
            let value_end = end - 4;
            let value = &self.remaining[value_end - tlv_len..value_end];
            self.remaining = &self.remaining[..self.remaining.len() - total_size];

            // Check if this is a valid public key TLV.
            if tlv_type == KERNEL_ATTR_PUBLIC_KEY_TYPE && value.len() >= 8 {
                // The public key TLV looks like:
                //
                // ```
                // [ key data ...                         ]
                // [ metadata (u32)                       ]
                // [ reserved (u16) ][ algorithm ID (u16) ]

                let end = value.len();
                let algorithm_id = u16::from_le_bytes([value[end - 2], value[end - 1]]);
                let key_use = u32::from_le_bytes([
                    value[end - 8],
                    value[end - 7],
                    value[end - 6],
                    value[end - 5],
                ]);
                return Some((algorithm_id, key_use, &value[0..end - 8]));
            }
        }
    }
}

pub struct SignatureVerifyKernelAttributeKeys<
    'a,
    S: hil::public_key_crypto::signature::SignatureVerify<'a, HL, SL>
        + hil::public_key_crypto::keys::SetKeyBySlice<'a, KL>,
    const ALGORITHM_ID: u16,
    const KL: usize,
    const HL: usize,
    const SL: usize,
> {
    verifier: &'a S,

    /// Kernel attributes section in flash, searched for type-0x0104 entries.
    attributes: &'static [u8],

    /// Single working buffer used to copy a key from flash before passing it
    /// to the underlying verifier.
    key_buffer: MapCell<&'static mut [u8; KL]>,

    /// Store the index of the key we selected and the key use identifier for
    /// after the deferred call.
    active_key: OptionalCell<(usize, usize)>,

    /// Client for select key done callback.
    select_key_client: OptionalCell<&'a dyn hil::public_key_crypto::keys::SelectKeyClient>,

    /// Needed for number keys callback.
    deferred_call: kernel::deferred_call::DeferredCall,
}

impl<
    'a,
    S: hil::public_key_crypto::signature::SignatureVerify<'a, HL, SL>
        + hil::public_key_crypto::keys::SetKeyBySlice<'a, KL>,
    const ALGORITHM_ID: u16,
    const KL: usize,
    const HL: usize,
    const SL: usize,
> SignatureVerifyKernelAttributeKeys<'a, S, ALGORITHM_ID, KL, HL, SL>
{
    pub fn new(
        verifier: &'a S,
        attributes: &'static [u8],
        key_buffer: &'static mut [u8; KL],
    ) -> Self {
        Self {
            verifier,
            attributes,
            key_buffer: MapCell::new(key_buffer),
            active_key: OptionalCell::empty(),
            select_key_client: OptionalCell::empty(),
            deferred_call: kernel::deferred_call::DeferredCall::new(),
        }
    }

    /// Count the number of public key attributes with a key of exactly `KL` bytes.
    fn count_keys(&self) -> usize {
        PublicKeyAttrIter::new(self.attributes)
            .filter(|(algorithm_id, _, key_bytes)| {
                *algorithm_id == ALGORITHM_ID && key_bytes.len() == KL
            })
            .count()
    }

    /// Return the key bytes for the `index`-th public key attribute with a key of
    /// exactly `KL` bytes, or `None` if no such entry exists at that index.
    fn find_key(&self, index: usize) -> Option<(u32, &'a [u8])> {
        PublicKeyAttrIter::new(self.attributes)
            .filter(|(algorithm_id, _, key_bytes)| {
                *algorithm_id == ALGORITHM_ID && key_bytes.len() == KL
            })
            .nth(index)
            .map(|(_, key_metadata, key_bytes)| (key_metadata, key_bytes))
    }
}

impl<
    'a,
    S: hil::public_key_crypto::signature::SignatureVerify<'a, HL, SL>
        + hil::public_key_crypto::keys::SetKeyBySlice<'a, KL>,
    const ALGORITHM_ID: u16,
    const KL: usize,
    const HL: usize,
    const SL: usize,
> hil::public_key_crypto::signature::SignatureVerify<'a, HL, SL>
    for SignatureVerifyKernelAttributeKeys<'a, S, ALGORITHM_ID, KL, HL, SL>
{
    fn set_verify_client(
        &self,
        client: &'a dyn hil::public_key_crypto::signature::ClientVerify<HL, SL>,
    ) {
        self.verifier.set_verify_client(client);
    }

    fn verify(
        &self,
        hash: &'static mut [u8; HL],
        signature: &'static mut [u8; SL],
    ) -> Result<
        (),
        (
            kernel::ErrorCode,
            &'static mut [u8; HL],
            &'static mut [u8; SL],
        ),
    > {
        self.verifier.verify(hash, signature)
    }
}

impl<
    'a,
    S: hil::public_key_crypto::signature::SignatureVerify<'a, HL, SL>
        + hil::public_key_crypto::keys::SetKeyBySlice<'a, KL>,
    const ALGORITHM_ID: u16,
    const KL: usize,
    const HL: usize,
    const SL: usize,
> hil::public_key_crypto::keys::SelectKey<'a>
    for SignatureVerifyKernelAttributeKeys<'a, S, ALGORITHM_ID, KL, HL, SL>
{
    fn get_key_count(&self) -> Result<(), ErrorCode> {
        self.deferred_call.set();
        Ok(())
    }

    fn select_key(&self, index: usize) -> Result<(), ErrorCode> {
        let key_buf = self.key_buffer.take().ok_or(ErrorCode::NOMEM)?;

        match self.find_key(index) {
            Some((key_metadata, key_data)) => {
                key_buf.copy_from_slice(key_data);
                self.active_key.set((index, key_metadata as usize));

                self.verifier.set_key(key_buf).map_err(|(e, k)| {
                    self.key_buffer.replace(k);
                    self.active_key.clear();
                    e
                })
            }
            None => {
                self.key_buffer.replace(key_buf);
                Err(ErrorCode::INVAL)
            }
        }
    }

    fn set_client(&self, client: &'a dyn hil::public_key_crypto::keys::SelectKeyClient) {
        self.select_key_client.replace(client);
    }
}

impl<
    'a,
    S: hil::public_key_crypto::signature::SignatureVerify<'a, HL, SL>
        + hil::public_key_crypto::keys::SetKeyBySlice<'a, KL>,
    const ALGORITHM_ID: u16,
    const KL: usize,
    const HL: usize,
    const SL: usize,
> hil::public_key_crypto::keys::SetKeyBySliceClient<KL>
    for SignatureVerifyKernelAttributeKeys<'a, S, ALGORITHM_ID, KL, HL, SL>
{
    fn set_key_done(&self, key: &'static mut [u8; KL], error: Result<(), ErrorCode>) {
        self.key_buffer.replace(key);

        self.select_key_client.map(|client| {
            let (key_index, key_metadata) = self.active_key.get().unwrap_or((0, 0));
            client.select_key_done(key_index, key_metadata, error);
        });
    }
}

impl<
    'a,
    S: hil::public_key_crypto::signature::SignatureVerify<'a, HL, SL>
        + hil::public_key_crypto::keys::SetKeyBySlice<'a, KL>,
    const ALGORITHM_ID: u16,
    const KL: usize,
    const HL: usize,
    const SL: usize,
> kernel::deferred_call::DeferredCallClient
    for SignatureVerifyKernelAttributeKeys<'a, S, ALGORITHM_ID, KL, HL, SL>
{
    fn handle_deferred_call(&self) {
        self.select_key_client.map(|client| {
            client.get_key_count_done(self.count_keys());
        });
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
