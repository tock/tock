// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! A SPI test which read/writes and expects MOSI to
//! be loopbacked to MISO. It checks that what it writes
//! is what it reads. The values put in the buffer are
//! a circular ring of 8-bit values, starting with an
//! initial value and incrementing by 1 on each write.
//! So if the first write is [33, 34, ... , 32],
//! the next write will be [34, 35, ..., 34]. You can set
//! the speed of the operation to check that configurations
//! are being set correctly: running two tests in parallel
//! with different bit rates should see different clock
//! frequencies.

use capsules_core::virtualizers::virtual_spi::MuxSpiMaster;
use components::spi::SpiComponent;
use core::cell::Cell;
use core::ptr::addr_of_mut;
use kernel::component::Component;
use kernel::debug;
use kernel::hil::spi::{self, SpiMasterDevice};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

#[allow(unused_variables, dead_code)]
pub struct SpiLoopback {
    cs: Cell<u8>,
    val: Cell<u8>,
    spi: &'static dyn SpiMasterDevice<'static>,
}

impl SpiLoopback {
    pub fn new(spi: &'static dyn SpiMasterDevice<'static>, cs: u8, counter: u8) -> Self {
        Self {
            val: Cell::new(counter),
            cs: Cell::new(cs),
            spi,
        }
    }
}

pub static mut WBUF: [u8; 256] = [0; 256];
pub static mut RBUF: [u8; 256] = [0; 256];
pub static mut WBUF2: [u8; 256] = [0; 256];
pub static mut RBUF2: [u8; 256] = [0; 256];

impl spi::SpiMasterClient for SpiLoopback {
    #[allow(unused_variables, dead_code)]
    fn read_write_done(
        &self,
        mut write: SubSliceMut<'static, u8>,
        read: Option<SubSliceMut<'static, u8>>,
        status: Result<usize, ErrorCode>,
    ) {
        let mut good = true;
        let read = read.unwrap();
        for (c, v) in write[..].iter().enumerate() {
            if read[c] != *v {
                debug!(
                    "SPI test error at index {}: wrote {} but read {}",
                    c, v, read[c]
                );
                good = false;
            }
        }
        if good {
            debug!("SPI CS={} test passed.", self.cs.get());
        }
        self.val.set(self.val.get() + 1);
        let counter = self.val.get();

        for i in 0..write.len() {
            write[i] = counter.wrapping_add(i as u8);
        }

        if let Err((e, _, _)) = self.spi.read_write_bytes(write, Some(read)) {
            panic!(
                "Could not continue SPI test, error on read_write_bytes is {:?}",
                e
            );
        }
    }
}

#[inline(never)]
#[allow(unused_variables, dead_code)]
pub unsafe fn spi_loopback_test(
    spi: &'static dyn SpiMasterDevice<'static>,
    counter: u8,
    speed: u32,
) {
    let spicb = kernel::static_init!(SpiLoopback, SpiLoopback::new(spi, 0, counter));
    spi.set_client(spicb);
    spi.set_rate(speed)
        .expect("Failed to set SPI speed in SPI loopback test.");

    let wbuf = &mut *addr_of_mut!(WBUF);
    let len = wbuf.len();
    if let Err((e, _, _)) = spi.read_write_bytes(
        (wbuf as &mut [u8]).into(),
        Some((&mut *addr_of_mut!(RBUF) as &mut [u8]).into()),
    ) {
        panic!(
            "Could not start SPI test, error on read_write_bytes is {:?}",
            e
        );
    }
}

#[inline(never)]
#[allow(unused_variables, dead_code)]
pub unsafe fn spi_two_loopback_test(mux: &'static MuxSpiMaster<'static, sam4l::spi::SpiHw>) {
    let spi_fast = SpiComponent::new(mux, sam4l::spi::Peripheral::Peripheral0)
        .finalize(components::spi_component_static!(sam4l::spi::SpiHw));
    let spi_slow = SpiComponent::new(mux, sam4l::spi::Peripheral::Peripheral1)
        .finalize(components::spi_component_static!(sam4l::spi::SpiHw));

    let spicb_fast = kernel::static_init!(SpiLoopback, SpiLoopback::new(spi_fast, 0, 0x80));
    let spicb_slow = kernel::static_init!(SpiLoopback, SpiLoopback::new(spi_slow, 1, 0x00));
    spi_fast
        .set_rate(1000000)
        .expect("Failed to set SPI speed in SPI loopback test.");
    spi_slow
        .set_rate(250000)
        .expect("Failed to set SPI speed in SPI loopback test.");
    spi_fast.set_client(spicb_fast);
    spi_slow.set_client(spicb_slow);

    let wbuf = &mut *addr_of_mut!(WBUF);
    let len = wbuf.len();
    if let Err((e, _, _)) = spi_fast.read_write_bytes(
        (wbuf as &mut [u8]).into(),
        Some((&mut *addr_of_mut!(RBUF) as &mut [u8]).into()),
    ) {
        panic!(
            "Could not start SPI test, error on read_write_bytes is {:?}",
            e
        );
    }

    let wbuf = &mut *addr_of_mut!(WBUF);
    let len = wbuf.len();
    if let Err((e, _, _)) = spi_slow.read_write_bytes(
        (wbuf as &mut [u8]).into(),
        Some((&mut *addr_of_mut!(RBUF2) as &mut [u8]).into()),
    ) {
        panic!(
            "Could not start SPI test, error on read_write_bytes is {:?}",
            e
        );
    }
}
