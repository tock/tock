// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! AppID assigner based on name and credential check metadata.
//!
//! This assigns a short ID where the most significant four bits are from the
//! credential checking metadata and the remaining bits are a CRC of the name.
//!
//! ```text
//! 32         28                       0 bits
//! +----------+------------------------+
//! | metadata | CRC(name)              |
//! +----------+------------------------+
//! ```
//!
//! The intention is that the CRC makes the short ID generally unique, and the
//! 4-bit metadata indicates the key that was used to verify the app's signing
//! credential.

use kernel::process::{Process, ProcessBinary, ShortId};
use kernel::process_checker::Compress;

pub struct AppIdAssignerNameMetadata {}

impl AppIdAssignerNameMetadata {
    pub fn new() -> Self {
        Self {}
    }
}

impl kernel::process_checker::Compress for AppIdAssignerNameMetadata {
    fn to_short_id(&self, process: &ProcessBinary) -> ShortId {
        // Get the stored metadata returned when this process had its credential
        // checked.
        let metadata = process.credential.get().map_or(0xF, |accepted_credential| {
            accepted_credential
                .metadata
                .map_or(0xF, |metadata| metadata.metadata) as u32
        });

        let name = process.header.get_package_name().unwrap_or("");
        let sum = kernel::utilities::helpers::crc32_posix(name.as_bytes());

        // Combine the metadata and CRC into the short id.
        let sid = ((metadata & 0xF) << 28) | (sum & 0xFFFFFFF);

        core::num::NonZeroU32::new(sid).into()
    }
}

// We just use the generic version which compares Short IDs.
impl kernel::process_checker::AppUniqueness for AppIdAssignerNameMetadata {
    fn different_identifier(&self, process_a: &ProcessBinary, process_b: &ProcessBinary) -> bool {
        self.to_short_id(process_a) != self.to_short_id(process_b)
    }

    fn different_identifier_process(
        &self,
        process_a: &ProcessBinary,
        process_b: &dyn Process,
    ) -> bool {
        self.to_short_id(process_a) != process_b.short_app_id()
    }

    fn different_identifier_processes(
        &self,
        process_a: &dyn Process,
        process_b: &dyn Process,
    ) -> bool {
        process_a.short_app_id() != process_b.short_app_id()
    }
}
