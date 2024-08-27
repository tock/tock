// Copyright OxidOS Automotive 2024.

use std::rc::Rc;

use crate::menu::capsule_popup;
use crate::state::Data;
use parse::peripherals::{Chip, DefaultPeripherals};

const PERIPHERAL: &str = "SPI";

/// Menu for configuring the SPI Controller capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    chip: Rc<C>,
    choice: Option<Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Spi>>,
) -> cursive::views::LinearLayout {
    match choice {
        // If there isn't a SPI Controller already configured, we switch to another menu.
        None => config_none(chip),
        Some(inner) => match chip.peripherals().spi() {
            // If we have at least one SPI peripheral, we make a list with it.
            Ok(spi_peripherals) => {
                capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                    Vec::from(spi_peripherals),
                    on_spi_submit::<C>,
                    inner,
                ))
            }
            // If we don't have any SPI peripheral, we show a popup 
            // with an error describing this.
            Err(_) => capsule_popup::<C, _>(crate::menu::no_support(PERIPHERAL)),
        },
    }
}

/// Menu for configuring the SPI capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    match chip.peripherals().spi() {
        Ok(spi_peripherals) => capsule_popup::<C, _>(crate::views::radio_group_with_null(
            Vec::from(spi_peripherals),
            on_spi_submit::<C>,
        )),
        Err(_) => capsule_popup::<C, _>(crate::menu::no_support(PERIPHERAL)),
    }
}

/// Configure a SPI controller based on the submitted SPI.
fn on_spi_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<C::Peripherals as DefaultPeripherals>::Spi>>,
) {
    if let Some(data) = siv.user_data::<Data<C>>() {
        if let Some(spi) = submit {
            data.platform.update_spi(Rc::clone(spi));
        } else {
            data.platform.remove_spi();
        }
    }
}
