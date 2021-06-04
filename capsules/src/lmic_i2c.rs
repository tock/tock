use kernel::ErrorCode;
use kernel::{
    debug,
    hil::{i2c, lmic},
};

pub struct LmicI2c<'a> {
    i2c: &'a dyn i2c::I2CDevice,
}

impl<'a> LmicI2c<'a> {
    pub fn new(i2c: &'a dyn i2c::I2CDevice) -> LmicI2c<'a> {
        LmicI2c { i2c }
    }
}

impl<'a> lmic::LMIC for LmicI2c<'a> {
    fn set_tx_data(&self, tx_data: &'static mut [u8], len: u8) -> Result<(), ErrorCode> {
        debug!("lmic_i2c call to i2c write");
        // turn on i2c to send commands
        self.i2c.enable();
        self.i2c.write(tx_data, len);

        Ok(())
    }
}

impl<'a> i2c::I2CClient for LmicI2c<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], error: i2c::Error) {
        debug!("I2C command complete!");
        self.i2c.disable();
    }
}
