// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! SHA-256 hash computation for kernel image verification

use crate::error::BootError;
use sha2::{Digest, Sha256};
use crate::types::{KernelRegion, SignatureAttribute};

#[inline(always)]
fn in_flash(addr: usize) -> bool {
    // Assuming 1MB flash size
    addr < 0x0010_0000
}

#[inline(always)]
fn range_ok(start: usize, end: usize) -> bool {
    start <= end && in_flash(start) && in_flash(end)
}

/// Computes hash over kernel binary and attributes regions separately
/// 
/// - Kernel binary: [kernel_start .. kernel_end)
/// - Attributes: [attr_start .. attr_end)
/// 
/// This hashes the kernel binary, then the attributes section,
/// zeroing out the 64-byte signature during hashing.
pub fn compute_kernel_hash(
    region: &KernelRegion,
    signature: &SignatureAttribute,
    attributes_end: usize,
) -> Result<[u8; 32], BootError> {
    let kernel_start = region.start;
    let attributes_start = region.attributes_start;
    let attributes_end = attributes_end;
    let (signature_start, signature_end) = signature.location;

    // Basic checks
    if !range_ok(kernel_start, attributes_start) {
        return Err(BootError::HashError);
    }
    if !range_ok(attributes_start, attributes_end) {
        return Err(BootError::HashError);
    }
    if signature_end.checked_sub(signature_start).unwrap_or(0) != 64 {
        return Err(BootError::InvalidSignature);
    }
    
    // Signature must be within attributes section
    if signature_start < attributes_start || signature_end > attributes_end {
        return Err(BootError::InvalidSignature);
    }

    let mut hasher = Sha256::new();

    unsafe {
        // Hash entire kernel binary region [kernel_start .. kernel_end)
        let kernel_end = region.end;
        if kernel_end > kernel_start {
            if !range_ok(kernel_start, kernel_end) {
                return Err(BootError::HashError);
            }
            let kernel_data = core::slice::from_raw_parts(
                kernel_start as *const u8, 
                kernel_end - kernel_start
            );
            hasher.update(kernel_data);
        }
        
        // Hash [attributes_start .. signature_start)
        if signature_start > attributes_start {
            if !range_ok(attributes_start, signature_start) {
                return Err(BootError::HashError);
            }
            let pre_signature = core::slice::from_raw_parts(
                attributes_start as *const u8, 
                signature_start - attributes_start
            );
            hasher.update(pre_signature);
        }

        // Hash 64 zeros instead of the signature
        hasher.update(&[0u8; 64]);

        // Hash [signature_end .. attributes_end)
        if attributes_end > signature_end {
            if !range_ok(signature_end, attributes_end) {
                return Err(BootError::HashError);
            }
            let post_signature = core::slice::from_raw_parts(
                signature_end as *const u8, 
                attributes_end - signature_end
            );
            hasher.update(post_signature);
        }
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&hasher.finalize());
    Ok(out)
}