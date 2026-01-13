// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! A dummy I2C client

use core::cell::Cell;
use core::ptr::addr_of_mut;
use kernel::debug;
use kernel::hil;
use kernel::hil::i2c::{Error, I2CMaster};

// ===========================================
// Scan for I2C Slaves
// ===========================================

struct ScanClient {
    dev_id: Cell<u8>,
    i2c_master: &'static dyn I2CMaster<'static>,
}

impl ScanClient {
    pub fn new(i2c_master: &'static dyn I2CMaster<'static>) -> Self {
        Self {
            dev_id: Cell::new(1),
            i2c_master,
        }
    }
}

impl hil::i2c::I2CHwMasterClient for ScanClient {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), Error>) {
        debug!("I2C command complete");

        debug!("Status: {:?}", status);

        debug!("Buffer: {:x?}", buffer);
    }
}

/// This test should be called with I2C2, specifically
pub fn i2c_scan_slaves(i2c_master: &'static dyn I2CMaster<'static>) {
    let dev = i2c_master;

    let i2c_client = unsafe { kernel::static_init!(ScanClient, ScanClient::new(dev)) };
    dev.set_master_client(i2c_client);

    dev.enable();

    //debug!("Resetting ADS1219");
    //// reset command
    //static mut RESET: [u8; 1] = [0b000_0110];
    //dev.write(0x40 << 1, unsafe { &mut *addr_of_mut!(RESET) }, 1).unwrap();

    debug!("Reading register 0x0 from ADS1219");
    static mut REG0: [u8; 1] = [0b0010_0000];
    dev.write_read(0x40 << 1, unsafe { &mut *addr_of_mut!(REG0) }, 1, 1)
        .unwrap();
}
