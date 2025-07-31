// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Low-level 8042 (i8042) controller bring-up.
//! Does not touch keyboard protocol settings.

use crate::ps2::{read_data, wait_output_ready, write_command, write_data, Ps2Controller};
use kernel::errorcode::ErrorCode;

/// Run once, before any device-level init

pub fn init_controller() -> Result<(), ErrorCode> {
    // Disable keyboard (port 1) and aux (port 2)
    write_command(0xAD);
    write_command(0xA7);

    // Self-test: 0xAA - expect 0x55
    write_command(0xAA);
    wait_output_ready();
    if read_data() != 0x55 {
        return Err(ErrorCode::FAIL);
    }

    // Enable IRQ1 in config byte
    write_command(0x20);
    wait_output_ready();
    let mut cfg = read_data();
    cfg |= 1 << 0; //bit0 = IRQ1 enable
    write_command(0x60);
    write_data(cfg);

    // Port-1 interface test 0xAB - expect 0x00
    write_command(0xAB);
    wait_output_ready();
    if read_data() != 0x00 {
        return Err(ErrorCode::FAIL);
    }
    Ok(())
}
