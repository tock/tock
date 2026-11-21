// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! A dummy SPI client to test the SPI implementation

use kernel::ErrorCode;
use kernel::hil::gpio;
use kernel::hil::gpio::Pin;
use kernel::hil::spi::{self, SpiSlave};
use sam4l::spi::SPI as SPI_SLAVE;

#[allow(unused_variables, dead_code)]
pub struct SlaveCB {
    val: u8,
}

impl SlaveCB {
    fn new() -> Self {
        SlaveCB { val: 0x55_u8 }
    }
}

impl spi::SpiSlaveClient for SlaveCB {
    #[allow(unused_variables, dead_code)]
    fn read_write_done(
        &self,
        write_buffer: Option<&'static mut [u8]>,
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
        status: Result<(), ErrorCode>,
    ) {
        // `write_buffer` is the same buffer handed to the previous
        // `read_write_bytes` call (initially in `spi_slave_dummy_test`, and
        // on every later call, the one below), returned to us here by the
        // driver.
        unsafe {
            SPI_SLAVE.read_write_bytes(write_buffer, None, 8);
        }
    }

    #[allow(unused_variables, dead_code)]
    fn chip_selected(&self) {
        unsafe {
            SPI_SLAVE.set_write_byte(0x05);
        }
    }
}

#[inline(never)]
#[allow(unused_variables, dead_code)]
pub unsafe fn spi_slave_dummy_test() {
    // set the LED to mark that we've programmed.
    // TODO: This doesn't do anything? We always blink...
    sam4l::gpio::PC[10].make_output();
    &sam4l::gpio::PC[10].set();

    let pin2: &mut gpio::Pin = &mut sam4l::gpio::PC[31]; // It's on D2 of the IMIX
    pin2.make_output();
    pin2.set();

    let buf1 = kernel::static_init!([u8; 8], [0, 0, 0, 0, 0, 0, 0, 0]);
    let buf2 = kernel::static_init!([u8; 8], [8, 7, 6, 5, 4, 3, 2, 1]);
    let slave_cb = kernel::static_init!(SlaveCB, SlaveCB::new());

    //sam4l::spi::SPI_SLAVE.set_active_peripheral(sam4l::spi::Peripheral::Peripheral0);
    SPI_SLAVE.set_client(Some(slave_cb));
    SPI_SLAVE.init(); // SpiSlave::init
    SPI_SLAVE.read_write_bytes(Some(buf2), Some(buf1), 8);
    SPI_SLAVE.enable();

    // Hint: Temporarily, when switching between master and slave dummy code,
    // - uncomment the right line at the end of main in main.rs
    // - uncomment the right client at the end of transfer_done in spi.rs
    // - uncomment 240-242 in main.rs for slave and comment it for master

    // YES interrupts are up, prints 0x07 all the way
    // SPI_SLAVE.set_write_byte(SPI_SLAVE.are_interrupts_up());
    //sam4l::spi::SPI_SLAVE.set_baud_rate(1000000);

    // pin2.clear();

    // TODO: We clear this for the trigger, set it permanently to behave as NSS
}
