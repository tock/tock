// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Secure Boot Common Library
//! 
//! This library provides board-agnostic secure boot functionality for Tock kernels.
//! It handles kernel signature verification using ECDSA P-256 and manages the
//! Binaries Discovery Table (BDT) for dynamic kernel placement.

#![no_std]

pub mod attributes_parser;
pub mod error;
pub mod compute_hash;
pub mod locate_tlvs;
pub mod types;
pub mod signature_verifier;
// pub mod binary_discovery_table;
// pub mod table_writer;
// pub mod flash_hal;
// pub mod kernel_relocator;

use crate::error::BootError;
use crate::types::{KernelVersion, KernelRegion};
// use crate::binary_discovery_table::BinaryEntry;

/// Trait that boards must implement for bootloader I/O operations
pub trait BootloaderIO {
    /// Signal successful verification: LED1
    fn signal_success(&self);
    
    /// Signal verification failure: LED4 blink
    fn signal_failure(&self);
    
    /// Optional: Write debug message to UART
    fn debug(&self, _msg: &str) {}

    /// Optional: Blink a board LED `count` times
    fn debug_blink(&self, _pin: u32, _count: usize) {}

    fn format(&self, _value: usize, _buf: &mut [u8; 32]) {}
}

/// Board-specific configuration that must be provided
pub trait BoardConfig {
    // /// Applications start address (_sapps)
    // const APP_START: usize;
    
    // /// Kernel start address
    // const KERNEL_START: usize;

    /// Available flash start address (after the discovery table)
    const AVAILABLE_FLASH_START: usize;

    /// Available flash end address (board specific)
    const AVAILABLE_FLASH_END: usize;
    
    /// ECDSA P-256 public key (64 bytes)
    const PUBLIC_KEY: [u8; 64];
    
    /// Minimum required kernel version
    const MIN_KERNEL_VERSION: KernelVersion;
}

/// Secure bootloader verification flow
/// 
/// This function verifies the kernel image:
/// 1. Locates the kernel region by scanning backwards from TOCK sentinel
/// 2. Parses kernel attributes to extract signature and version
/// 3. Checks kernel version against minimum required version
/// 4. Computes hash of kernel image
/// 5. Verifies signature
/// 
/// Returns the kernel entry point address and length on success.

/// Kernel candidate stores metadata about potential kernels
#[derive(Copy, Clone)]
struct KernelCandidate {
    kernel: locate_tlvs::PotentialKernel,
    version: KernelVersion,
}

pub fn verify_and_boot<C: BoardConfig, IO: BootloaderIO>(
    io: &IO,
) -> Result<(usize, usize), BootError> {
    // let mut buf = [0u8; 32];

    // io.debug("checking for potential candidates");
    // Scan flash for all kernels
    let potential_kernels = locate_tlvs::scan_for_potential_kernels(
        C::AVAILABLE_FLASH_START,
        C::AVAILABLE_FLASH_END,
        io,
    )?;
    
    
    // Extract versions from each kernel
    let mut candidates: [Option<KernelCandidate>; 8] = [None; 8];
    let mut candidate_count = 0;
    
    for maybe_kernel in &potential_kernels {
        if let Some(kernel) = maybe_kernel {
            // Parse attributes to get version
            if let Ok(version) = extract_version::<C, IO>(&kernel, io) {
                candidates[candidate_count] = Some(KernelCandidate {
                    kernel: *kernel,
                    version,
                });
                candidate_count += 1;
                // io.debug("candidate count:");
                // io.format(candidate_count, &mut buf);
            }
        }
    }
    
    if candidate_count == 0 {
        // io.debug("No candidates found");
        return Err(BootError::NoValidKernel);
    }

    // io.debug("sorting candidates by version");
    
    // Sort candidates by version
    sort_candidates_by_version(&mut candidates, candidate_count);
    
    // Try to verify and boot kernels in order
    let mut selected_kernel = None;

    for i in 0..candidate_count {
        if let Some(candidate) = &candidates[i] {
            // Try to the kernel's signature
            // io.debug("verifying single kernel");
            if verify_single_kernel::<C, IO>(io, &candidate.kernel).is_ok() {
                // Verification succeeded. Use this kernel
                // io.debug("verification successful");
                selected_kernel = Some(candidate.kernel);
                break;
            }
            // Verification failed, try next candidate
        }
    }

    let selected_kernel = selected_kernel.ok_or(BootError::NoValidKernel)?;

    // // Check if kernel requires to be relocated
    // let _ = attributes_parser::parse_attributes(
    //     selected_kernel.attributes_start,
    //     selected_kernel.attributes_end,
    //     io
    // )?;
    
    // if let Some(reloc_info) = attributes.relocation {
    //     let link_addr = reloc_info.link_address as usize;
    //     let current_addr = selected_kernel.start_address;
        
    //     if current_addr != link_addr {
    //         // Kernel needs relocation!
    //         // io.debug("Kernel requires relocation");
            
    //         kernel_relocator::relocate_kernel_in_place(
    //             &selected_kernel,
    //             &reloc_info,
    //             io,
    //         )?;
            
    //         // io.debug("Relocation complete");
    //     } else {
    //         // io.debug("No relocation needed");
    //     }
    // } else {
    //     // No relocation TLV found - kernel might be old format or doesn't need it
    //     // io.debug("No relocation TLV found (old kernel?)");
    // }
    
    // // Build discovery table with found kernels
    // let mut kernel_entries = [BinaryEntry::empty(); 8];
    // for i in 0..candidate_count {
    //     if let Some(candidate) = &candidates[i] {
    //         kernel_entries[i] = BinaryEntry {
    //             start_address: candidate.kernel.start_address as u32,
    //             size: (candidate.kernel.attributes_end - candidate.kernel.start_address) as u32,
    //             version: [
    //                 candidate.version.major as u8,
    //                 candidate.version.minor as u8,
    //                 candidate.version.patch as u8,
    //             ],
    //             binary_type: binary_discovery_table::binary_type::KERNEL,
    //             reserved: [0; 4],
    //         };
    //     }
    // }

    // // Write discovery table to flash
    // table_writer::write_bdt_to_flash(&kernel_entries, candidate_count)?;

    // Return to main

    // io.debug("selected kernel start address: ");
    // io.format(selected_kernel.start_address, &mut buf);
    
    Ok((selected_kernel.start_address, selected_kernel.attributes_end))
}

// Extract version from 
fn extract_version<C: BoardConfig, IO: BootloaderIO>(
    kernel: &locate_tlvs::PotentialKernel,
    io: &IO,
) -> Result<KernelVersion, BootError> {
    // let mut buf = [0u8; 32];

    // Parse attributes
    let attributes = attributes_parser::parse_attributes(
        kernel.attributes_start,
        kernel.attributes_end,
        io,
    )?;

    // Get version
    let version = attributes.kernel_version.ok_or(BootError::InvalidTLV)?;
    
    // Check minimum version
    if version < C::MIN_KERNEL_VERSION {
        return Err(BootError::VersionTooOld);
    }

    Ok(version)
}

pub fn verify_single_kernel<C: BoardConfig, IO: BootloaderIO>(
    io: &IO,
    kernel: &locate_tlvs::PotentialKernel,
) -> Result<(), BootError> {
    // Verify and Boot entered
    // let mut buf = [0u8; 32];

    // Parsing attributes
    let attributes = attributes_parser::parse_attributes(
        kernel.attributes_start,
        kernel.attributes_end,
        io,
    )?;

    // Finding signature
    // io.debug_blink(15, 4);
    // io.debug("checking for signature");
    let signature = attributes.signature.ok_or(BootError::SignatureMissing)?;
    // io.debug("signature tlv present");

    // Check flash TLV validity
    let _ = attributes.kernel_flash.ok_or(BootError::InvalidTLV)?;
    

    // let (flash_start, _flash_len) = attributes.kernel_flash.ok_or(BootError::InvalidTLV)?;

    // io.debug("flash start:");
    // io.format(flash_start as usize, &mut buf);

    let region = KernelRegion {
        start: kernel.start_address,
        end:   kernel.attributes_end,
        entry_point: kernel.start_address,
        attributes_start: kernel.attributes_start,
    };

    // io.debug("Kernel Region:");
    // io.debug("region.start:");
    // io.format(region.start, &mut buf);
    // io.debug("region.end:");
    // io.format(region.end, &mut buf);
    // let (sig_start, sig_end) = signature.location;
    // io.debug("sig flash_addr:");
    // io.format(sig_start, &mut buf);

    // Compute hash
    let hash = compute_hash::compute_kernel_hash(
        &region,
        &signature,
        kernel.attributes_end,
    )?;

    // Verify signature
    signature_verifier::verify_signature::<C, IO>(&hash, &signature, io)?;

    // Verification success
    Ok(())
}

/// Sort kernel candidates by version
fn sort_candidates_by_version(
    candidates: &mut [Option<KernelCandidate>; 8],
    count: usize,
) {
    for i in 0..count {
        for j in 0..count - i - 1 {
            let should_swap = match (&candidates[j], &candidates[j + 1]) {
                (Some(a), Some(b)) => {
                    if a.version < b.version {
                        true
                        // TBD Policy
                    // } else if a.version == b.version {
                    //     // If two kernels have same version 
                    //     // (even though it should not happen in practice),
                    //     // we choose the kernel that was written most recently
                    //     a.kernel.start_address < b.kernel.start_address
                    } else {
                        false
                    }
                }
                _ => false,
            };
            
            if should_swap {
                candidates.swap(j, j + 1);
            }
        }
    }
}