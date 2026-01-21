// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! ECDSA P-256 signature verification using the p256 crate

use crate::error::BootError;
use crate::types::SignatureAttribute;
use crate::BoardConfig;
use p256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};
use p256::EncodedPoint;
use crate::BootloaderIO;

/// Verifies an ECDSA P-256 signature against a SHA-256 hash
/// 
/// Uses the public key from the BoardConfig to verify the signature.
pub fn verify_signature<C: BoardConfig, IO: BootloaderIO>(
    hash: &[u8; 32],
    signature: &SignatureAttribute,
    _io: &IO,
) -> Result<(), BootError> {
    // _io.debug("verifying signature");
    // Check algorithm ID (0x00000001 = ECDSA P-256 and SHA-256)
    if signature.algorithm_id != 0x00000001 {
        // _io.debug("bad signature algo id");
        return Err(BootError::UnsupportedAlgorithm);
    }
    
    // Get the public key from board config
    let public_key_bytes = &C::PUBLIC_KEY;
    
    // Parse the public key (64 bytes r || s )
    let encoded_point = EncodedPoint::from_untagged_bytes(public_key_bytes.into());
    let verifying_key = VerifyingKey::from_encoded_point(&encoded_point)
        .map_err(|_| BootError::VerificationFailed)?;
    
    // Construct signature
    let mut signature_bytes = [0u8; 64];
    signature_bytes[0..32].copy_from_slice(&signature.r);
    signature_bytes[32..64].copy_from_slice(&signature.s);
    
    let signature = Signature::from_bytes(&signature_bytes.into())
        .map_err(|_| BootError::InvalidSignature)?;
    
    // Verify the signature against the hash
    verifying_key
        .verify_prehash(hash, &signature)
        .map_err(|_| BootError::VerificationFailed)?;
    
    Ok(())
}