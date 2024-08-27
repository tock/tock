// Copyright OxidOS Automotive 2024.

use std::rc::Rc;

use crate::menu::capsule_popup;
use crate::state::Data;
use parse::peripherals::{Chip, DefaultPeripherals};

const BLE_PERIPHERAL: &str = "BLE";
const TIMER_PERIPHERAL: &str = "TIMER";

type BleRadioPeripherals<C> = (
    Rc<<<C as Chip>::Peripherals as DefaultPeripherals>::Timer>,
    Rc<<<C as Chip>::Peripherals as DefaultPeripherals>::BleAdvertisement>,
);

/// Menu for configuring the Ble Radio capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    chip: Rc<C>,
    choice: Option<BleRadioPeripherals<C>>,
) -> cursive::views::LinearLayout {
    match choice {
        // If there isn't a Ble Radio already configured, we switch to another menu.
        None => config_none(chip),
        Some(inner) => {
            let inner_ble = inner.1;
            match chip.peripherals().ble() {
                // If we have at least one Ble peripheral, we make a list with it.
                Ok(ble_peripherals) => {
                    capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                        Vec::from(ble_peripherals),
                        move |siv, submit| {
                            on_ble_submit::<C>(
                                Rc::clone(&chip),
                                siv,
                                submit,
                                Some(Rc::clone(&inner.0)),
                            )
                        },
                        inner_ble,
                    ))
                }
                // If we don't have any Ble peripheral, we show a popup
                // with an error describing this.
                Err(_) => capsule_popup::<C, _>(crate::menu::no_support(BLE_PERIPHERAL)),
            }
        }
    }
}

/// Menu for configuring the Ble Radio capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    match chip.peripherals().ble() {
        Ok(ble_peripherals) => capsule_popup::<C, _>(crate::views::radio_group_with_null(
            Vec::from(ble_peripherals),
            move |siv, submit| on_ble_submit::<C>(Rc::clone(&chip), siv, submit, None),
        )),
        Err(_) => capsule_popup::<C, _>(crate::menu::no_support(BLE_PERIPHERAL)),
    }
}

/// Continue configuring the Ble Radio with the selected Ble.
fn on_ble_submit<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<C::Peripherals as DefaultPeripherals>::BleAdvertisement>>,
    previous_timer: Option<
        Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Timer>,
    >,
) {
    siv.pop_layer();
    if let Some(data) = siv.user_data::<Data<C>>() {
        if let Some(ble) = submit {
            siv.add_layer(timer_popup::<C>(chip, Rc::clone(ble), previous_timer))
        } else {
            data.platform.remove_ble();
        }
    }
}

/// Menu for choosing the timer peripheral.
fn timer_popup<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
    submit: Rc<<C::Peripherals as DefaultPeripherals>::BleAdvertisement>,
    previous_timer: Option<
        Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Timer>,
    >,
) -> cursive::views::LinearLayout {
    match previous_timer {
        // If there was a timer already chosen, show this in the menu.
        Some(prev) => {
            let inner = prev;
            match chip.peripherals().timer() {
                // If we have at least one timer peripheral, we make a list with it.
                Ok(timer_peripherals) => {
                    capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                        Vec::from(timer_peripherals),
                        move |siv, submit_timer| {
                            on_timer_submit::<C>(siv, submit_timer, Rc::clone(&submit))
                        },
                        inner,
                    ))
                }
                // If we had a timer selected, then the chip should have timers.
                Err(_) => unreachable!(),
            }
        }
        // If there wasn't a timer already chosen, show the default menu.
        None => match chip.peripherals().timer() {
            // If we have at least one timer peripheral, we make a list with it.
            Ok(timer_peripherals) => capsule_popup::<C, _>(crate::views::radio_group_with_null(
                Vec::from(timer_peripherals),
                move |siv, submit_timer| {
                    on_timer_submit::<C>(siv, submit_timer, Rc::clone(&submit))
                },
            )),
            // If we don't have any timer peripheral, we show a popup
            // with an error describing this.
            Err(_) => capsule_popup::<C, _>(crate::menu::no_support(TIMER_PERIPHERAL)),
        },
    }
}

/// Configure a Ble capsule based on the submitted Ble and Timer peripherals.
fn on_timer_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit_timer: &Option<Rc<<C::Peripherals as DefaultPeripherals>::Timer>>,
    submit_ble: Rc<<C::Peripherals as DefaultPeripherals>::BleAdvertisement>,
) {
    if let Some(data) = siv.user_data::<Data<C>>() {
        if let Some(timer) = submit_timer {
            data.platform.update_ble(submit_ble, Rc::clone(timer));
        } else {
            data.platform.remove_ble();
        }
    }
}
