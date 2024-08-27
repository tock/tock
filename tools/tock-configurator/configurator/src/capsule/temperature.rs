// Copyright OxidOS Automotive 2024.

use std::rc::Rc;

use crate::menu::capsule_popup;
use crate::state::Data;
use parse::peripherals::{Chip, DefaultPeripherals};

const PERIPHERAL: &str = "TEMPERATURE";

/// Menu for configuring the Temperature capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    chip: Rc<C>,
    choice: Option<
        Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Temperature>,
    >,
) -> cursive::views::LinearLayout {
    match choice {
        // If there isn't a Temperature already configured, we switch to another menu.
        None => config_none(chip),
        Some(inner) => match chip.peripherals().temp() {
            // If we have at least one Temperature peripheral, we make a list with it.
            Ok(temp_peripherals) => {
                capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                    Vec::from(temp_peripherals),
                    on_temp_submit::<C>,
                    inner,
                ))
            }
            // If we don't have any temperature peripheral, we show a popup 
            // with an error describing this.
            Err(_) => capsule_popup::<C, _>(crate::menu::no_support(PERIPHERAL)),
        },
    }
}

/// Menu for configuring the Temperature capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    match chip.peripherals().temp() {
        Ok(temp_peripherals) => capsule_popup::<C, _>(crate::views::radio_group_with_null(
            Vec::from(temp_peripherals),
            on_temp_submit::<C>,
        )),
        Err(_) => capsule_popup::<C, _>(crate::menu::no_support(PERIPHERAL)),
    }
}

/// Configure a Temperature capsule based on the submitted Temperature peripheral.
fn on_temp_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<C::Peripherals as DefaultPeripherals>::Temperature>>,
) {
    if let Some(data) = siv.user_data::<Data<C>>() {
        if let Some(temp) = submit {
            data.platform.update_temp(Rc::clone(temp));
        } else {
            data.platform.remove_temp();
        }
    }
}
