// Copyright OxidOS Automotive 2024.

use std::rc::Rc;

use crate::menu::capsule_popup;
use crate::state::Data;
use parse::peripherals::{Chip, DefaultPeripherals};

const PERIPHERAL: &str = "I2C";

/// Menu for configuring the I2C capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    chip: Rc<C>,
    choice: Option<Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::I2c>>,
) -> cursive::views::LinearLayout {
    match choice {
        // If there isn't an I2C already configured, we switch to another menu.
        None => config_none(chip),
        Some(inner) => match chip.peripherals().i2c() {
            // If we have at least one I2C peripheral, we make a list with it.
            Ok(i2c_peripherals) => {
                capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                    Vec::from(i2c_peripherals),
                    on_i2c_submit::<C>,
                    inner,
                ))
            }
            // If we don't have any timer peripheral, we show a popup 
            // with an error describing this.
            Err(_) => capsule_popup::<C, _>(crate::menu::no_support(PERIPHERAL)),
        },
    }
}

/// Menu for configuring the I2C capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    match chip.peripherals().i2c() {
        Ok(i2c_peripherals) => capsule_popup::<C, _>(crate::views::radio_group_with_null(
            Vec::from(i2c_peripherals),
            on_i2c_submit::<C>,
        )),
        Err(_) => capsule_popup::<C, _>(crate::menu::no_support(PERIPHERAL)),
    }
}

/// Configure an I2C capsule based on the submitted I2C peripheral.
fn on_i2c_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<C::Peripherals as DefaultPeripherals>::I2c>>,
) {
    if let Some(data) = siv.user_data::<Data<C>>() {
        if let Some(i2c) = submit {
            data.platform.update_i2c(Rc::clone(i2c));
        } else {
            data.platform.remove_i2c();
        }
    }
}
