use crate::chip;
use kernel::common::StaticRef;
use lowrisc::i2c::{I2c, I2cRegisters};

pub static mut I2C: I2c = I2c::new(I2C_BASE, (1 / chip::CHIP_FREQ) * 1000 * 1000);

// This is a placeholder address as the I2C MMIO interface isn't avaliable yet
const I2C_BASE: StaticRef<I2cRegisters> =
    unsafe { StaticRef::new(0x4005_0000 as *const I2cRegisters) };
