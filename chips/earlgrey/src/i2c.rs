use kernel::utilities::StaticRef;
use lowrisc::i2c::I2cRegisters;

pub const I2C0_BASE: StaticRef<I2cRegisters> =
    unsafe { StaticRef::new(0x4008_0000 as *const I2cRegisters) };
