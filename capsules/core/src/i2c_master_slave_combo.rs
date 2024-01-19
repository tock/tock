// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Combines two hardware devices into a single I2C master/slave device.
//!
//! If a chip provides a seperate hardware implementation for a I2C master and
//! slave device, like the Apollo3 for example, this capsule can be used to
//! combine them into a single `I2CMasterSlave` compatible implementation.
//!
//! This allows the `I2CMasterSlaveDriver` capsule to be implemented on more
//! types of hardware.

use kernel::hil::i2c::{
    Error, I2CHwMasterClient, I2CHwSlaveClient, I2CMaster, I2CMasterSlave, I2CSlave,
};

pub struct I2CMasterSlaveCombo<'a, M: I2CMaster<'a>, S: I2CSlave<'a>> {
    i2c_master: &'a M,
    i2c_slave: &'a S,
}

impl<'a, M: I2CMaster<'a>, S: I2CSlave<'a>> I2CMasterSlaveCombo<'a, M, S> {
    pub fn new(i2c_master: &'a M, i2c_slave: &'a S) -> I2CMasterSlaveCombo<'a, M, S> {
        I2CMasterSlaveCombo {
            i2c_master,
            i2c_slave,
        }
    }
}

impl<'a, M: I2CMaster<'a>, S: I2CSlave<'a>> I2CMaster<'a> for I2CMasterSlaveCombo<'a, M, S> {
    fn set_master_client(&self, master_client: &'a dyn I2CHwMasterClient) {
        self.i2c_master.set_master_client(master_client)
    }

    fn enable(&self) {
        self.i2c_master.enable()
    }

    fn disable(&self) {
        self.i2c_master.disable()
    }

    fn write_read(
        &self,
        addr: u8,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        self.i2c_master.write_read(addr, data, write_len, read_len)
    }

    fn write(
        &self,
        addr: u8,
        data: &'static mut [u8],
        len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        self.i2c_master.write(addr, data, len)
    }

    fn read(
        &self,
        addr: u8,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        self.i2c_master.read(addr, buffer, len)
    }
}

impl<'a, M: I2CMaster<'a>, S: I2CSlave<'a>> I2CSlave<'a> for I2CMasterSlaveCombo<'a, M, S> {
    fn set_slave_client(&self, slave_client: &'a dyn I2CHwSlaveClient) {
        self.i2c_slave.set_slave_client(slave_client);
    }

    fn enable(&self) {
        self.i2c_slave.enable()
    }

    fn disable(&self) {
        self.i2c_slave.disable()
    }

    fn set_address(&self, addr: u8) -> Result<(), Error> {
        self.i2c_slave.set_address(addr)
    }

    fn write_receive(
        &self,
        data: &'static mut [u8],
        max_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        self.i2c_slave.write_receive(data, max_len)
    }

    fn read_send(
        &self,
        data: &'static mut [u8],
        max_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        self.i2c_slave.read_send(data, max_len)
    }

    fn listen(&self) {
        self.i2c_slave.listen()
    }
}

impl<'a, M: I2CMaster<'a>, S: I2CSlave<'a>> I2CMasterSlave<'a> for I2CMasterSlaveCombo<'a, M, S> {}
