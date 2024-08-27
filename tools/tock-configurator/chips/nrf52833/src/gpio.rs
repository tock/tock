// Copyright OxidOS Automotive 2024.

use parse::peripheral;
use std::rc::Rc;

use parse::constants::PERIPHERALS;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinIds {
    P0_00,
    P0_01,
    P0_02,
    P0_03,
    P0_04,
    P0_05,
    P0_06,
    P0_07,
    P0_08,
    P0_09,
    P0_10,
    P0_11,
    P0_12,
    P0_13,
    P0_14,
    P0_15,
    P0_16,
    P0_17,
    P0_18,
    P0_19,
    P0_20,
    P0_21,
    P0_22,
    P0_23,
    P0_24,
    P0_25,
    P0_26,
    P0_27,
    P0_28,
    P0_29,
    P0_30,
    P0_31,
    P1_00,
    P1_01,
    P1_02,
    P1_03,
    P1_04,
    P1_05,
    P1_06,
    P1_07,
    P1_08,
    P1_09,
}

impl std::fmt::Display for PinIds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum GpioType {
    Gpio0,
}

#[derive(Debug, PartialEq)]
#[peripheral(serde, ident = "gpio")]
pub struct Gpio {}

impl parse::Gpio for Gpio {
    type PinId = PinIds;

    fn pins(&self) -> Option<std::rc::Rc<[Self::PinId]>> {
        Some(Rc::new([
            PinIds::P0_00,
            PinIds::P0_01,
            PinIds::P0_02,
            PinIds::P0_03,
            PinIds::P0_04,
            PinIds::P0_05,
            PinIds::P0_06,
            PinIds::P0_07,
            PinIds::P0_08,
            PinIds::P0_09,
            PinIds::P0_10,
            PinIds::P0_11,
            PinIds::P0_12,
            PinIds::P0_13,
            PinIds::P0_14,
            PinIds::P0_15,
            PinIds::P0_16,
            PinIds::P0_17,
            PinIds::P0_18,
            PinIds::P0_19,
            PinIds::P0_20,
            PinIds::P0_21,
            PinIds::P0_22,
            PinIds::P0_23,
            PinIds::P0_24,
            PinIds::P0_25,
            PinIds::P0_26,
            PinIds::P0_27,
            PinIds::P0_28,
            PinIds::P0_29,
            PinIds::P0_30,
            PinIds::P0_31,
            PinIds::P1_00,
            PinIds::P1_01,
            PinIds::P1_02,
            PinIds::P1_03,
            PinIds::P1_04,
            PinIds::P1_05,
            PinIds::P1_06,
            PinIds::P1_07,
            PinIds::P1_08,
            PinIds::P1_09,
        ]))
    }
}

impl std::fmt::Display for Gpio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "gpio")
    }
}
