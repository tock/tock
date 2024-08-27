// Copyright OxidOS Automotive 2024.

use cursive::views::LinearLayout;
use parse::peripherals::Chip;
use std::rc::Rc;

pub(crate) mod alarm;
pub(crate) mod ble;
pub(crate) mod console;
pub(crate) mod flash;
pub(crate) mod gpio;
pub(crate) mod i2c;
pub(crate) mod lsm303agr;
pub(crate) mod rng;
pub(crate) mod spi;
pub(crate) mod temperature;

pub trait ConfigMenu: std::fmt::Debug {
    fn config<C: Chip + 'static + serde::ser::Serialize>(chip: Rc<C>) -> LinearLayout;
}
