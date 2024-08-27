// Copyright OxidOS Automotive 2024.

use std::rc::Rc;

use crate::menu;
use crate::state::Data;
use crate::{menu::capsule_popup, views};
use parse::peripherals::{Chip, DefaultPeripherals};

const PERIPHERAL: &str = "I2C";

/// Menu for configuring the Lsm303agr capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    chip: Rc<C>,
    previous_state: Option<
        Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::I2c>,
    >,
) -> cursive::views::LinearLayout {
    match previous_state {
        // If there isn't a LSM303AGR already configured, we switch to another menu.
        None => config_none(chip),
        Some(inner) => match chip.peripherals().i2c() {
            // If we have at least one I2C peripheral, we make a list with it.
            Ok(uart_peripherals) => {
                capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                    Vec::from(uart_peripherals),
                    move |siv, submit| on_bus_submit::<C>(siv, submit),
                    inner,
                ))
            }
            // If we don't have any I2C peripheral, we show a popup 
            // with an error describing this.
            Err(_) => capsule_popup::<C, _>(crate::menu::no_support(PERIPHERAL)),
        },
    }
}

/// Menu for configuring the LSM303AGR capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    match chip.peripherals().i2c() {
        Ok(i2c_peripherals) => crate::menu::capsule_popup::<C, _>(
            crate::views::radio_group_with_null(Vec::from(i2c_peripherals), |siv, submit| {
                on_bus_submit::<C>(siv, submit)
            }),
        ),
        Err(_) => crate::menu::capsule_popup::<C, _>(crate::menu::no_support(PERIPHERAL)),
    }
}

/// After choosing an I2C, go to the Acceleration Rate choice.
fn on_bus_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::I2c>>,
) {
    siv.pop_layer();
    if let Some(data) = siv.user_data::<Data<C>>() {
        if let Some(bus) = submit {
            siv.add_layer(accel_rate_popup::<C>(Rc::clone(bus)));
        } else {
            data.platform.remove_lsm303agr();
        }
    }
}

/// Acceleration Rate choice popup.
fn accel_rate_popup<C: Chip + 'static + serde::ser::Serialize>(
    bus: Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::I2c>,
) -> cursive::views::LinearLayout {
    capsule_popup::<C, _>(views::select_menu(
        //  TODO: This can be a macro?
        vec![
            ("Off", parse::capsules::lsm303agr::Lsm303AccelDataRate::Off),
            (
                "DataRate1Hz",
                parse::capsules::lsm303agr::Lsm303AccelDataRate::DataRate1Hz,
            ),
            (
                "DataRate10Hz",
                parse::capsules::lsm303agr::Lsm303AccelDataRate::DataRate10Hz,
            ),
            (
                "DataRate25Hz",
                parse::capsules::lsm303agr::Lsm303AccelDataRate::DataRate25Hz,
            ),
            (
                "DataRate50Hz",
                parse::capsules::lsm303agr::Lsm303AccelDataRate::DataRate50Hz,
            ),
            (
                "DataRate100Hz",
                parse::capsules::lsm303agr::Lsm303AccelDataRate::DataRate100Hz,
            ),
            (
                "DataRate200Hz",
                parse::capsules::lsm303agr::Lsm303AccelDataRate::DataRate200Hz,
            ),
            (
                "DataRate400Hz",
                parse::capsules::lsm303agr::Lsm303AccelDataRate::DataRate400Hz,
            ),
            (
                "LowPower1620Hz",
                parse::capsules::lsm303agr::Lsm303AccelDataRate::LowPower1620Hz,
            ),
            (
                "Normal1344LowPower5376Hz",
                parse::capsules::lsm303agr::Lsm303AccelDataRate::Normal1344LowPower5376Hz,
            ),
        ],
        move |siv, choice| on_accel_rate_submit::<C>(siv, Rc::clone(&bus), *choice),
    ))
}

/// After choosing an acceleration rate, go to the Acceleration Scale choice.
fn on_accel_rate_submit<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    bus: Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::I2c>,
    accel_rate: parse::capsules::lsm303agr::Lsm303AccelDataRate,
) {
    siv.pop_layer();
    siv.add_layer(accel_scale_popup::<C>(Rc::clone(&bus), accel_rate));
}

/// Acceleration Scale choice popup.
fn accel_scale_popup<C: Chip + 'static + serde::ser::Serialize>(
    bus: Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::I2c>,
    accel_rate: parse::capsules::lsm303agr::Lsm303AccelDataRate,
) -> cursive::views::LinearLayout {
    capsule_popup::<C, _>(views::select_menu(
        vec![
            ("Scale2G", parse::capsules::lsm303agr::Lsm303Scale::Scale2G),
            ("Scale4G", parse::capsules::lsm303agr::Lsm303Scale::Scale4G),
            ("Scale8G", parse::capsules::lsm303agr::Lsm303Scale::Scale8G),
            (
                "Scale16G",
                parse::capsules::lsm303agr::Lsm303Scale::Scale16G,
            ),
        ],
        move |siv, choice| on_accel_scale_submit::<C>(siv, Rc::clone(&bus), accel_rate, *choice),
    ))
}

/// After choosing an acceleration scale, go to the Magnetometer Data Rate choice.
fn on_accel_scale_submit<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    bus: Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::I2c>,
    accel_rate: parse::capsules::lsm303agr::Lsm303AccelDataRate,
    accel_scale: parse::capsules::lsm303agr::Lsm303Scale,
) {
    siv.pop_layer();
    siv.add_layer(mag_data_rate_popup::<C>(
        Rc::clone(&bus),
        accel_rate,
        accel_scale,
    ));
}

/// Magnetometer Data Rate choice popup.
fn mag_data_rate_popup<C: Chip + 'static + serde::ser::Serialize>(
    bus: Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::I2c>,
    accel_rate: parse::capsules::lsm303agr::Lsm303AccelDataRate,
    accel_scale: parse::capsules::lsm303agr::Lsm303Scale,
) -> cursive::views::LinearLayout {
    capsule_popup::<C, _>(views::select_menu(
        vec![
            (
                "DataRate0_75Hz",
                parse::capsules::lsm303agr::Lsm303MagnetoDataRate::DataRate0_75Hz,
            ),
            (
                "DataRate1_5Hz",
                parse::capsules::lsm303agr::Lsm303MagnetoDataRate::DataRate1_5Hz,
            ),
            (
                "DataRate3_0Hz",
                parse::capsules::lsm303agr::Lsm303MagnetoDataRate::DataRate3_0Hz,
            ),
            (
                "DataRate7_5Hz",
                parse::capsules::lsm303agr::Lsm303MagnetoDataRate::DataRate7_5Hz,
            ),
            (
                "DataRate15_0Hz",
                parse::capsules::lsm303agr::Lsm303MagnetoDataRate::DataRate15_0Hz,
            ),
            (
                "DataRate30_0Hz",
                parse::capsules::lsm303agr::Lsm303MagnetoDataRate::DataRate30_0Hz,
            ),
            (
                "DataRate75_0Hz",
                parse::capsules::lsm303agr::Lsm303MagnetoDataRate::DataRate75_0Hz,
            ),
            (
                "DataRate220_0Hz",
                parse::capsules::lsm303agr::Lsm303MagnetoDataRate::DataRate220_0Hz,
            ),
        ],
        move |siv, choice| {
            on_mag_data_rate_submit::<C>(siv, Rc::clone(&bus), accel_rate, accel_scale, *choice)
        },
    ))
}

/// After choosing an magnetometer data rate, go to the Magnetometer Range choice.
fn on_mag_data_rate_submit<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    bus: Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::I2c>,
    accel_rate: parse::capsules::lsm303agr::Lsm303AccelDataRate,
    accel_scale: parse::capsules::lsm303agr::Lsm303Scale,
    mag_data_rate: parse::capsules::lsm303agr::Lsm303MagnetoDataRate,
) {
    siv.pop_layer();
    siv.add_layer(mag_range_popup::<C>(
        Rc::clone(&bus),
        accel_rate,
        accel_scale,
        mag_data_rate,
    ));
}

/// Magnetometer Range choice popup.
fn mag_range_popup<C: Chip + 'static + serde::ser::Serialize>(
    bus: Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::I2c>,
    accel_rate: parse::capsules::lsm303agr::Lsm303AccelDataRate,
    accel_scale: parse::capsules::lsm303agr::Lsm303Scale,
    mag_data_rate: parse::capsules::lsm303agr::Lsm303MagnetoDataRate,
) -> cursive::views::LinearLayout {
    capsule_popup::<C, _>(views::select_menu(
        vec![
            ("Range1G", parse::capsules::lsm303agr::Lsm303Range::Range1G),
            (
                "Range1_3G",
                parse::capsules::lsm303agr::Lsm303Range::Range1_3G,
            ),
            (
                "Range1_9G",
                parse::capsules::lsm303agr::Lsm303Range::Range1_9G,
            ),
            (
                "Range2_5G",
                parse::capsules::lsm303agr::Lsm303Range::Range2_5G,
            ),
            (
                "Range4_0G",
                parse::capsules::lsm303agr::Lsm303Range::Range4_0G,
            ),
            (
                "Range4_7G",
                parse::capsules::lsm303agr::Lsm303Range::Range4_7G,
            ),
            (
                "Range5_6G",
                parse::capsules::lsm303agr::Lsm303Range::Range5_6G,
            ),
            (
                "Range8_1",
                parse::capsules::lsm303agr::Lsm303Range::Range8_1,
            ),
        ],
        move |siv, choice| {
            on_mag_range_submit::<C>(
                siv,
                Rc::clone(&bus),
                accel_rate,
                accel_scale,
                mag_data_rate,
                *choice,
            )
        },
    ))
}

/// After choosing the parameters, configure a LSM303AGR with them.
fn on_mag_range_submit<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    bus: Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::I2c>,
    accel_rate: parse::capsules::lsm303agr::Lsm303AccelDataRate,
    accel_scale: parse::capsules::lsm303agr::Lsm303Scale,
    mag_data_rate: parse::capsules::lsm303agr::Lsm303MagnetoDataRate,
    mag_range: parse::capsules::lsm303agr::Lsm303Range,
) {
    siv.pop_layer();
    if let Some(data) = siv.user_data::<Data<C>>() {
        data.platform.update_lsm303agr(
            bus,
            accel_rate,
            false,
            accel_scale,
            false,
            false,
            mag_data_rate,
            mag_range,
        );

        siv.add_layer(menu::capsules_menu::<C>())
    }
}
