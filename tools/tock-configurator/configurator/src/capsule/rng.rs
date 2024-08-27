// Copyright OxidOS Automotive 2024.

use std::rc::Rc;

use crate::menu::capsule_popup;
use crate::state::Data;
use parse::peripherals::{Chip, DefaultPeripherals};

const PERIPHERAL: &str = "RNG";

/// Menu for configuring the Rng capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    chip: Rc<C>,
    choice: Option<Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Rng>>,
) -> cursive::views::LinearLayout {
    match choice {
        // If there isn't a RNG already configured, we switch to another menu.
        None => config_none(chip),
        Some(inner) => match chip.peripherals().rng() {
            // If we have at least one RNG peripheral, we make a list with it.
            Ok(rng_peripherals) => {
                capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                    Vec::from(rng_peripherals),
                    on_rng_submit::<C>,
                    inner,
                ))
            }
            // If we don't have any RNG peripheral, we show a popup 
            // with an error describing this.
            Err(_) => capsule_popup::<C, _>(crate::menu::no_support(PERIPHERAL)),
        },
    }
}

/// Menu for configuring the RNG capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    match chip.peripherals().rng() {
        Ok(rng_peripherals) => capsule_popup::<C, _>(crate::views::radio_group_with_null(
            Vec::from(rng_peripherals),
            on_rng_submit::<C>,
        )),
        Err(_) => capsule_popup::<C, _>(crate::menu::no_support(PERIPHERAL)),
    }
}

/// Configure a RNG based on the submitted RNG.
fn on_rng_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<C::Peripherals as DefaultPeripherals>::Rng>>,
) {
    if let Some(data) = siv.user_data::<Data<C>>() {
        if let Some(rng) = submit {
            data.platform.update_rng(Rc::clone(rng));
        } else {
            data.platform.remove_rng();
        }
    }
}
