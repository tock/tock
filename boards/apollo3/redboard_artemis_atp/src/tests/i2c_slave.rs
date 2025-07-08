// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::tests::run_kernel_op;
use crate::PERIPHERALS;
use core::cell::Cell;
use kernel::debug;
use kernel::hil::i2c::I2CHwSlaveClient;
use kernel::hil::i2c::I2CSlave;
use kernel::hil::i2c::SlaveTransmissionType;
use kernel::static_init;
use kernel::utilities::cells::TakeCell;

struct I2CSlaveCallback {
    master_write_done: Cell<bool>,
    send_data: TakeCell<'static, [u8]>,
    master_read_done: Cell<bool>,
    received_data: TakeCell<'static, [u8]>,
}

impl<'a> I2CSlaveCallback {
    fn new(send_data: &'static mut [u8], received_data: &'static mut [u8]) -> Self {
        I2CSlaveCallback {
            master_write_done: Cell::new(false),
            send_data: TakeCell::new(send_data),
            master_read_done: Cell::new(false),
            received_data: TakeCell::new(received_data),
        }
    }

    fn reset(&self) {
        self.master_write_done.set(false);
        self.master_read_done.set(false);
    }
}

impl<'a> I2CHwSlaveClient for I2CSlaveCallback {
    fn command_complete(
        &self,
        buffer: &'static mut [u8],
        length: usize,
        transmission_type: SlaveTransmissionType,
    ) {
        match transmission_type {
            SlaveTransmissionType::Write => {
                self.master_write_done.set(true);
                debug!("Was Sent: {buffer:x?}");
            }
            SlaveTransmissionType::Read => {
                self.master_read_done.set(true);
            }
        }
    }

    fn read_expected(&self) {
        unimplemented!()
    }

    fn write_expected(&self) {
        unimplemented!()
    }
}

unsafe fn static_init_test_cb() -> &'static I2CSlaveCallback {
    let received_data = static_init!([u8; 8], [0xdc, 0x55, 0x51, 0x5e, 0x30, 0xac, 0x50, 0xc7]);
    let send_data = static_init!([u8; 8], [0xdc, 0x55, 0x51, 0x5e, 0x30, 0xac, 0x50, 0xc7]);

    static_init!(
        I2CSlaveCallback,
        I2CSlaveCallback::new(send_data, received_data)
    )
}

#[test_case]
fn i2c_slave_receive() {
    let perf = unsafe { PERIPHERALS.unwrap() };
    let i2c_slave = &perf.ios;
    let cb = unsafe { static_init_test_cb() };
    let received_data = cb.received_data.take().unwrap();
    let send_data = cb.send_data.take().unwrap();

    debug!("[I2C] Setup ios to receive... ");
    run_kernel_op(100);

    i2c_slave.set_slave_client(cb);
    cb.reset();

    debug!("    [I2C] Enable... ");
    i2c_slave.enable();
    run_kernel_op(100);

    debug!("    [I2C] Set address... ");
    assert_eq!(i2c_slave.set_address(0x41), Ok(()));
    run_kernel_op(100);

    debug!("    [I2C] read_send... ");
    i2c_slave.read_send(send_data, send_data.len()).unwrap();
    run_kernel_op(100);

    debug!("    [I2C] Starting listen... ");
    i2c_slave.listen();
    run_kernel_op(100);

    debug!("    [I2C] Run... ");
    run_kernel_op(5000);

    debug!("    [I2C] write_receive... ");
    i2c_slave
        .write_receive(received_data, received_data.len())
        .unwrap();
    run_kernel_op(5000_00);

    // If there is an I2C master device you can uncomment this to
    // ensure we recieve the data.
    // assert_eq!(cb.master_write_done.get(), true);

    run_kernel_op(100);
    debug!("    [ok]");
    run_kernel_op(100);
}
