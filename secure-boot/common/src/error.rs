// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Error types for the secure bootloader

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BootError {
    /// "TOCK" sentinel not found in expected location
    SentinelNotFound,
    
    /// Kernel signature attribute (0x0104) not found
    SignatureMissing,
    
    /// Invalid TLV structure or length
    InvalidTLV,
    
    /// Signature data is invalid or malformed
    InvalidSignature,
    
    /// Signature verification failed
    VerificationFailed,
    
    /// Kernel version is older than minimum required version
    VersionTooOld,
    
    /// Algorithm ID in signature doesn't match expected
    UnsupportedAlgorithm,
    
    /// Kernel region is invalid or corrupted
    InvalidKernelRegion,
    
    /// Hash computation failed
    HashError,

    /// BDT invalid or corrupted
    InvalidBDT,
    
    /// BDT checksum failed
    BDTChecksumFailed,
    
    /// No valid kernel found in BDT
    NoValidKernel,

    /// Flash Operation Failed
    FlashOperationFailed,
}