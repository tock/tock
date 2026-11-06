// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use tock_registers_v0::register_bitfields;

register_bitfields![usize,
    pub mscratch [
        scratch OFFSET(0) NUMBITS(crate::XLEN) []
    ]
];
