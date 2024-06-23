// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! AppID mechanisms based on TBF headers.

use kernel::process::{Process, ProcessBinary, ShortId};
use kernel::process_checker::{AppUniqueness, Compress};

/// Assign AppIDs based on fields in the app's TBF header.
///
/// This uses the ShortId TBF header to assign short IDs to applications. If the
/// header is not present the application will be assigned a
/// `ShortID::LocallyUnique` ID.
///
/// This assigner uses ShortIds as the AppID, so the built-in check for ShortId
/// uniqueness is sufficient.
pub struct AppIdAssignerTbfHeader {}

impl AppUniqueness for AppIdAssignerTbfHeader {
    fn different_identifier(&self, _process_a: &ProcessBinary, _process_b: &ProcessBinary) -> bool {
        true
    }

    fn different_identifier_process(
        &self,
        _process_binary: &ProcessBinary,
        _process: &dyn Process,
    ) -> bool {
        true
    }

    fn different_identifier_processes(
        &self,
        _process_a: &dyn Process,
        _process_b: &dyn Process,
    ) -> bool {
        true
    }
}

impl Compress for AppIdAssignerTbfHeader {
    fn to_short_id(&self, process: &ProcessBinary) -> ShortId {
        process.header.get_fixed_short_id().into()
    }
}
