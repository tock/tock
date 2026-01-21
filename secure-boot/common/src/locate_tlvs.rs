// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Kernel and TLVs location discovery by scanning backwards from application start

use crate::error::BootError;
use crate::BootloaderIO;
use crate::attributes_parser;

const TOCK: [u8; 4] = [84, 79, 67, 75];

/// Potential kernel found in flash
#[derive(Debug, Clone, Copy)]
pub struct PotentialKernel {
    pub start_address: usize,
    pub size: usize,
    pub attributes_start: usize,
    pub attributes_end: usize,
}


/// Scan flash region for potential kernels
/// 
/// Searches forward through the given flash range looking for `TOCK` sentinels.
/// Returns up to 8 potential kernels.
pub fn scan_for_potential_kernels<IO: BootloaderIO>(
    scan_start: usize,
    scan_end: usize,
    _io: &IO,
) -> Result<[Option<PotentialKernel>; 8], BootError> {
    let mut kernels: [Option<PotentialKernel>; 8] = [None; 8];
    let mut kernel_count = 0;
    // let mut buf = [0u8; 32];

    // _io.debug("scanning");
    // _io.debug("scan_start:");
    // _io.format(scan_start, &mut buf);
    // _io.debug("scan_end:");
    // _io.format(scan_end, &mut buf);
    
    // Align to word boundary
    let mut current_addr = (scan_start + 3) & !3;

    // _io.debug("current_address:");
    // _io.format(current_addr, &mut buf);
    
    while current_addr < scan_end && kernel_count < 8 {
        // Look for next TOCK sentinel
        if let Some(sentinel_addr) = find_tock_sentinel(current_addr, scan_end, _io) {
            // Try to parse basic kernel info
            match parse_kernel_info(sentinel_addr, current_addr, _io) {
                Ok(kernel) => {
                    // _io.debug("Found a kernel");
                    kernels[kernel_count] = Some(kernel);
                    kernel_count += 1;
                    
                    // Skip past this kernel to continue scanning
                    current_addr = kernel.start_address + kernel.attributes_end;
                    // _io.debug("current address:");
                    // _io.format(current_addr, &mut buf);
                }
                Err(_) => {
                    // Couldn't parse this one, skip this sentinel
                    current_addr = sentinel_addr + 4;
                }
            }
        } else {
            // No more sentinels found
            // _io.debug("no more sentinels");
            break;
        }
    }
    
    Ok(kernels)
}

/// Find next sentinel in flash range
fn find_tock_sentinel<IO: BootloaderIO>(start: usize, end: usize, _io:&IO) -> Option<usize> {
    // Align to word boundary
    let mut addr = (start + 3) & !3;
    // let mut buf = [0u8; 32];
    
    while addr + 4 <= end {
        let bytes = unsafe { 
            core::slice::from_raw_parts(addr as *const u8, 4) 
        };
        
        if bytes == TOCK {
            // _io.debug("tock sentinel found:");
            // _io.format(addr, &mut buf);
            return Some(addr);
        }
        
        addr += 4;
    }
    
    None
}


/// Parse basic kernel info from a sentinel location
/// 
/// This extracts kernel boundaries and location.
fn parse_kernel_info<IO: BootloaderIO>(
    sentinel_addr: usize,
    _kernel_start: usize,
    _io: &IO,
) -> Result<PotentialKernel, BootError> {

    // let mut buf = [0u8; 32];
    // Find start of attributes (walk backward through TLV chain)
    let attributes_start = scan_tlvs_backward(sentinel_addr, _io)?;
    let attributes_end = sentinel_addr + 4;
    
    // Parse attributes to get kernel boundaries
    let attributes = attributes_parser::parse_attributes(attributes_start, attributes_end, _io)?;

    // _io.debug("parsed attributes");
    
    // Get kernel flash TLV
    let (_kernel_start, kernel_len) = attributes.kernel_flash
        .ok_or(BootError::InvalidTLV)?;
    
    // // let kernel_start = kernel_start as usize;
    let kernel_size = kernel_len as usize;
    // let kernel_size = attributes_end - kernel_start;
    let actual_kernel_start = attributes_start.checked_sub(kernel_size)
        .ok_or(BootError::InvalidKernelRegion)?;

    // _io.debug("Kernel start and size:");
    // _io.format(kernel_start, &mut buf);
    // _io.debug("kernel size: ");
    // _io.format(kernel_size, &mut buf);
    
    // Sanity checks
    if actual_kernel_start >= attributes_start {
        return Err(BootError::InvalidKernelRegion);
    }

    // _io.debug("kernel start sanity check passed");

    Ok(PotentialKernel {
        start_address: actual_kernel_start,
        size: kernel_size,
        attributes_start,
        attributes_end,
    })
}


/// Scan TLVs backward from sentinel to find start of attributes
/// 
/// Layout in flash: [...kernel code...] [TLVs...] [Version/Reserved] [TOCK]
fn scan_tlvs_backward<IO: BootloaderIO>(sentinel_address: usize, _io: &IO) -> Result<usize, BootError> {
    let mut pos = sentinel_address;
    // let mut buf = [0u8; 32];
    // _io.debug("Scanning for TLVs");
    // Skip past TOCK sentinel (4 bytes)
    if pos < 4 {
        // _io.debug("Invalid TLV1");
        return Err(BootError::InvalidTLV);
    }
    pos -= 4; // Now at Version/Reserved (end of TLV chain)

    const VALID_TLV_TYPES: [u16; 4] = [
        0x0101, // App Memory
        0x0102, // Kernel Flash
        0x0103, // Version
        0x0104, // Signature
    ];
    
    // Walk backward through TLV chain
    loop {
        if pos < 8 {
            return Err(BootError::InvalidTLV);
        }
        
        // Read TLV header
        let header = unsafe { 
            core::slice::from_raw_parts((pos - 4) as *const u8, 4) 
        };
        let tlv_type = u16::from_le_bytes([header[0], header[1]]);
        let tlv_len = u16::from_le_bytes([header[2], header[3]]) as usize;

        // _io.debug("TLV Length");
        // _io.format(tlv_len, &mut buf);

        if !VALID_TLV_TYPES.contains(&tlv_type) {
            // Hit garbage data - we've gone past the start
            return Ok(pos);
        }
        
        // Sanity check
        if pos < (4 + tlv_len) {
            return Err(BootError::InvalidTLV);
        }
        
        // Move to start of this TLV's value
        pos -= 4 + tlv_len;

        // _io.debug("size of attributes:");
        // _io.format((sentinel_address - pos), &mut buf);
        
        // Check if this is the signature TLV (type 0x0105)
        // If so, we've reached the start of attributes
        if tlv_type == 0x0104 {
            // _io.debug("found start addr of attributes");
            // _io.format(pos, &mut buf);

            // _io.debug("size of attributes:");
            // _io.format(sentinel_address - pos, &mut buf);
            // pos -= 4 + tlv_len;
            return Ok(pos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_word_alignment() {
        assert_eq!((0x9000 + 3) & !3, 0x9000);
        assert_eq!((0x9001 + 3) & !3, 0x9004);
        assert_eq!((0x9003 + 3) & !3, 0x9004);
    }
}