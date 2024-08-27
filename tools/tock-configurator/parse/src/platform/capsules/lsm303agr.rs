// Copyright OxidOS Automotive 2024.

use parse_macros::component;

use crate::{peripherals::i2c, Component};
use std::rc::Rc;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum Lsm303AccelDataRate {
    Off,
    DataRate1Hz,
    DataRate10Hz,
    DataRate25Hz,
    DataRate50Hz,
    DataRate100Hz,
    DataRate200Hz,
    DataRate400Hz,
    LowPower1620Hz,
    Normal1344LowPower5376Hz,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Lsm303Scale {
    Scale2G,
    Scale4G,
    Scale8G,
    Scale16G,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Lsm303MagnetoDataRate {
    DataRate0_75Hz,
    DataRate1_5Hz,
    DataRate3_0Hz,
    DataRate7_5Hz,
    DataRate15_0Hz,
    DataRate30_0Hz,
    DataRate75_0Hz,
    DataRate220_0Hz,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Lsm303Range {
    Range1G,
    Range1_3G,
    Range1_9G,
    Range2_5G,
    Range4_0G,
    Range4_7G,
    Range5_6G,
    Range8_1,
}

#[component(curr, ident = "lsm303agr")]
pub struct Lsm303agr<I: i2c::I2c> {
    pub inner: Rc<I>,
    pub accel_data_rate: Lsm303AccelDataRate,
    pub accel_scale: Lsm303Scale,
    pub mag_data_rate: Lsm303MagnetoDataRate,
    pub mag_range: Lsm303Range,
}

impl<I: i2c::I2c + 'static> Component for Lsm303agr<I> {}
