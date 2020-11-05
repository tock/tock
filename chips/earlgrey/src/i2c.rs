use kernel::common::StaticRef;
use lowrisc::i2c::I2cRegisters;

// This is a placeholder address as the I2C MMIO interface isn't avaliable yet
pub const I2C_BASE: StaticRef<I2cRegisters> =
    unsafe { StaticRef::new(0x4005_0000 as *const I2cRegisters) };
