// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

use tock_registers::Mmio32;

/// Returns the addresses of QEMU's VirtIO instances on simulated riscv32
/// systems. Unsafe code may rely on this being correct.
pub fn addresses() -> [Mmio32; 8] {
    [
        0x1000_1000,
        0x1000_2000,
        0x1000_3000,
        0x1000_4000,
        0x1000_5000,
        0x1000_6000,
        0x1000_7000,
        0x1000_8000,
    ]
    .map(Mmio32::from_addr)
}
