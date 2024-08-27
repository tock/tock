// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

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
