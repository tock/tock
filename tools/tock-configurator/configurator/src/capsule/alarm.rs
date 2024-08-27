// Copyright OxidOS Automotive 2024.

use std::rc::Rc;

use crate::menu::capsule_popup;
use crate::state::Data;
use parse::peripherals::{Chip, DefaultPeripherals};

const PERIPHERAL: &str = "TIMER";

/// Menu for configuring the Alarm capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    chip: Rc<C>,
    previous_state: Option<
        Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Timer>,
    >,
) -> cursive::views::LinearLayout {
    match previous_state {
        // If there isn't an Alarm already configured, we switch to another menu.
        None => config_none(chip),
        Some(inner) => match chip.peripherals().timer() {
            // If we have at least one timer peripheral, we make a list with it.
            Ok(timer_peripherals) => {
                capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                    Vec::from(timer_peripherals),
                    on_timer_submit::<C>,
                    inner,
                ))
            }
            // If we don't have any timer peripheral, we show a popup 
            // with an error describing this.
            Err(_) => capsule_popup::<C, _>(crate::menu::no_support(PERIPHERAL)),
        },
    }
}

/// Menu for configuring the Alarm capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    match chip.peripherals().timer() {
        Ok(timer_peripherals) => capsule_popup::<C, _>(crate::views::radio_group_with_null(
            Vec::from(timer_peripherals),
            on_timer_submit::<C>,
        )),
        Err(_) => capsule_popup::<C, _>(crate::menu::no_support(PERIPHERAL)),
    }
}

/// Configure an Alarm capsule based on the submitted Timer peripheral.
fn on_timer_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<C::Peripherals as DefaultPeripherals>::Timer>>,
) {
    if let Some(data) = siv.user_data::<Data<C>>() {
        match submit {
            Some(timer) => data.platform.update_alarm(Rc::clone(timer)),
            None => data.platform.remove_alarm(),
        }
    }
}
