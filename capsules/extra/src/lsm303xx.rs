//! LSM303xx Sensors
//!

#![allow(non_camel_case_types)]

use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;

use kernel::utilities::registers::register_bitfields;

pub const ACCELEROMETER_BASE_ADDRESS: u8 = 0x19;
pub const MAGNETOMETER_BASE_ADDRESS: u8 = 0x1e;

// Manual page Table 20, page 25
enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum Lsm303AccelDataRate {
        Off = 0,
        DataRate1Hz = 1,
        DataRate10Hz = 2,
        DataRate25Hz = 3,
        DataRate50Hz = 4,
        DataRate100Hz = 5,
        DataRate200Hz = 6,
        DataRate400Hz = 7,
        LowPower1620Hz = 8,
        Normal1344LowPower5376Hz = 9,
    }
}

// Manual table 72, page 25
enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum Lsm303MagnetoDataRate {
        DataRate0_75Hz = 0,
        DataRate1_5Hz = 1,
        DataRate3_0Hz = 2,
        DataRate7_5Hz = 3,
        DataRate15_0Hz = 4,
        DataRate30_0Hz = 5,
        DataRate75_0Hz = 6,
        DataRate220_0Hz = 7,
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum Lsm303Scale {
        Scale2G = 0,
        Scale4G = 1,
        Scale8G = 2,
        Scale16G = 3
    }
}

// Manual table 27, page 27
pub(crate) const SCALE_FACTOR: [u8; 4] = [2, 4, 8, 16];

// Manual table 75, page 38
enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum Lsm303Range {
        Range1G = 0,
        Range1_3G = 1,
        Range1_9G = 2,
        Range2_5G = 3,
        Range4_0G = 4,
        Range4_7G = 5,
        Range5_6G = 7,
        Range8_1 = 8,
    }
}

// Manual table 75, page 38
pub(crate) const RANGE_FACTOR_X_Y: [i16; 8] = [
    1000, // placeholder
    1100, 855, 670, 450, 400, 330, 230,
];

// Manual table 75, page 38
pub(crate) const RANGE_FACTOR_Z: [i16; 8] = [
    1000, // placeholder
    980, 760, 600, 400, 355, 295, 205,
];

register_bitfields![u8,
    pub (crate) CTRL_REG1 [
        /// Output data rate
        ODR OFFSET(4) NUMBITS(4) [],
        /// Low Power enable
        LPEN OFFSET(3) NUMBITS(1) [],
        /// Z enable
        ZEN OFFSET(2) NUMBITS(1) [],
        /// Y enable
        YEN OFFSET(1) NUMBITS(1) [],
        /// X enable
        XEN OFFSET(0) NUMBITS(1) []
    ],
    pub (crate) CTRL_REG4 [
        /// Block Data update
        BDU OFFSET(7) NUMBITS(2) [],
        /// Big Little Endian
        BLE OFFSET(6) NUMBITS(1) [],
        /// Full Scale selection
        FS OFFSET(4) NUMBITS(2) [],
        /// High Resolution
        HR OFFSET(3) NUMBITS(1) [],
        /// SPI Serial Interface
        SIM OFFSET(0) NUMBITS(1) []
    ]
];

enum_from_primitive! {
    pub enum AccelerometerRegisters {
        CTRL_REG1 = 0x20,
        CTRL_REG4 = 0x23,
        OUT_X_L_A = 0x28,
        OUT_X_H_A = 0x29,
        OUT_Y_L_A = 0x2A,
        OUT_Y_H_A = 0x2B,
        OUT_Z_L_A = 0x2C,
        OUT_Z_H_A = 0x2D,
    }
}
