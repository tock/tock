// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! A dummy SPI client to test the SPI implementation

use kernel::ErrorCode;
use kernel::hil::gpio::Configure;
use kernel::hil::spi::{self, SpiMaster};
use kernel::utilities::cells::TakeCell;
use kernel::utilities::leasable_buffer::SubSliceMut;

#[allow(unused_variables, dead_code)]
pub struct DummyCB {
    val: u8,
    spi: &'static sam4l::spi::SpiHw<'static>,
    // The buffer used for the continuous echo transfer started in the first
    // `read_write_done` callback. It is handed to the SPI driver and comes
    // back to us as the `write` parameter of every later callback.
    a5: TakeCell<'static, [u8]>,
}

impl DummyCB {
    pub fn new(spi: &'static sam4l::spi::SpiHw<'static>, a5: &'static mut [u8]) -> Self {
        Self {
            val: 0x55_u8,
            spi,
            a5: TakeCell::new(a5),
        }
    }
}

impl spi::SpiMasterClient for DummyCB {
    #[allow(unused_variables, dead_code)]
    fn read_write_done(
        &self,
        write: SubSliceMut<'static, u8>,
        read: Option<SubSliceMut<'static, u8>>,
        status: Result<usize, ErrorCode>,
    ) {
        // do actual stuff
        // TODO verify SPI return value
        //
        // On the first callback (triggered by the initial BUF1/BUF2
        // transfer in `spi_dummy_test`), switch over to the dedicated A5
        // buffer. On every later callback, `write` already is that same
        // buffer being returned to us by the driver.
        let buf = self.a5.take().unwrap_or_else(|| write.take());
        let _ = self.spi.read_write_bytes(buf.into(), None);
    }
}

// This test first turns on the Imix's User led, asserts pin D2 and then
// initiates a continuous SPI transfer of 8 bytes.
//
// If the SPI transfer of multiple bytes fail, then the test will loop writing
// 0xA5.
//
// The first SPI transfer outputs [8, 7, 6, 5, 4, 3, 2, 1] then echoes whatever
// input it recieves from the slave on peripheral 1 continuously.
//
// To test with a logic analyzer, connect probes to pin D2 on the Imix, and
// the SPI MOSI and CLK pins (exposed on the Imix's 20-pin header). Setup
// the logic analyzer to trigger sampling on assertion of pin 2, then restart
// the board.
#[inline(never)]
#[allow(unused_variables, dead_code)]
pub unsafe fn spi_dummy_test(spi: &'static sam4l::spi::SpiHw<'static>) {
    // set the LED to mark that we've programmed.
    let pin = sam4l::gpio::GPIOPin::new(sam4l::gpio::Pin::PC10);
    pin.make_output();
    pin.set();

    let pin2 = sam4l::gpio::GPIOPin::new(sam4l::gpio::Pin::PC31); // It's on D2 of the IMIX
    pin2.make_output();
    pin2.set();

    let a5_buf: &'static mut [u8] = kernel::static_init!([u8; 16], [0xA5; 16]);
    let spicb = kernel::static_init!(DummyCB, DummyCB::new(spi, a5_buf));
    spi.specify_chip_select(sam4l::spi::Peripheral::Peripheral0)
        .unwrap();
    spi.set_client(spicb);

    spi.init().unwrap();
    spi.set_baud_rate(200000);

    let buf1 = kernel::static_init!([u8; 8], [0, 0, 0, 0, 0, 0, 0, 0]);
    let buf2 = kernel::static_init!([u8; 8], [8, 7, 6, 5, 4, 3, 2, 1]);
    let len = buf2.len();
    if spi.read_write_bytes((buf2 as &mut [u8]).into(), Some((buf1 as &mut [u8]).into())) != Ok(())
    {
        loop {
            spi.write_byte(0xA5).unwrap();
        }
    }

    pin2.clear();
}
