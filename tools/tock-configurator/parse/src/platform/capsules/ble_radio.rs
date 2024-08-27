// Copyright OxidOS Automotive 2024.

use crate::{ble, timer, Component, MuxAlarm};
use parse_macros::component;
use std::rc::Rc;

#[component(curr, ident = "ble_radio")]
pub struct BleRadio<T: timer::Timer + 'static, B: ble::BleAdvertisement> {
    /// Ble used by the capsule.
    _inner_ble: Rc<B>,
    /// Alarm used by the capsule.
    _inner_alarm: Rc<MuxAlarm<T>>,
}

impl<T: timer::Timer + 'static, B: ble::BleAdvertisement + 'static> Component for BleRadio<T, B> {}
