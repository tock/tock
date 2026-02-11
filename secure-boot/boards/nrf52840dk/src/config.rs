// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Board-specific configuration for nRF52840DK

use secure_boot_common::{BoardConfig, types::KernelVersion};

/// Configuration for nRF52840DK secure bootloader
pub struct Nrf52840Config;

impl BoardConfig for Nrf52840Config {
    const AVAILABLE_FLASH_START: usize = 0x8000; // After Bootloader

    const AVAILABLE_FLASH_END: usize = 0x10_0000; // End of flash
    
    /// ECDSA P-256 public key (uncompressed format: 64 bytes)
    const PUBLIC_KEY: [u8; 64] = [
        0xca, 0x53, 0x3e, 0xf4, 0xc6, 0x08, 0xe6, 0x83,
        0x11, 0xd1, 0xf9, 0xd4, 0xd5, 0x50, 0x1d, 0x7d,
        0xaf, 0xcb, 0xf0, 0x15, 0x16, 0x4e, 0x8d, 0x29,
        0x00, 0xcd, 0x1c, 0x63, 0x30, 0xa7, 0xfc, 0x22,
        0x1f, 0x9c, 0xeb, 0xd5, 0xd1, 0xe9, 0x1f, 0x6b,
        0x81, 0x03, 0xae, 0xe0, 0x32, 0x22, 0xff, 0xbc,
        0xd2, 0x76, 0x08, 0x5c, 0x01, 0x28, 0x6b, 0xff,
        0xca, 0x24, 0xc6, 0x16, 0x69, 0xd4, 0xd1, 0x1d,
    ];
    
    /// Minimum required kernel version
    /// Set to 2.3.0 to match current Tock kernel version
    const MIN_KERNEL_VERSION: KernelVersion = KernelVersion {
        major: 2,
        minor: 3,
        patch: 0,
    };
}