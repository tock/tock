// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::utilities::registers::register_bitfields;

// mtval contains the address of an exception
register_bitfields![usize,
    pub mtval [
        exception_addr OFFSET(0) NUMBITS(crate::XLEN) []
    ]
];
