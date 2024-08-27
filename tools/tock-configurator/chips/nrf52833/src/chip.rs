// Copyright OxidOS Automotive 2024.

use crate::Peripherals;
use common::cortex_m4::Systick;
use parse::{Component, Ident};
use quote::{format_ident, quote};
use std::rc::Rc;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Chip {
    #[serde(skip)]
    systick: Rc<Systick>,
    peripherals: Rc<Peripherals>,
}

impl Default for Chip {
    fn default() -> Self {
        Self {
            peripherals: Rc::new(Peripherals::new()),
            systick: Rc::new(Systick::new()),
        }
    }
}

impl Chip {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Ident for Chip {
    fn ident(&self) -> Result<String, parse::error::Error> {
        Ok(parse::constants::CHIP.clone())
    }
}

impl Component for Chip {
    fn ty(&self) -> Result<parse::proc_macro2::TokenStream, parse::Error> {
        Ok(quote!(
            nrf52833::chip::NRF52<
                'static,
                nrf52833::interrupt_service::Nrf52833DefaultPeripherals<'static>,
            >
        ))
    }

    fn dependencies(&self) -> Option<Vec<Rc<dyn Component>>> {
        Some(vec![self.peripherals.clone()])
    }

    fn init_expr(&self) -> Result<parse::proc_macro2::TokenStream, parse::Error> {
        let peripherals_ident = format_ident!("{}", self.peripherals.ident()?);
        Ok(quote! {
            kernel::static_init!(
                nrf52833::chip::NRF52<nrf52833::interrupt_service::Nrf52833DefaultPeripherals>,
                nrf52833::chip::NRF52::new(#peripherals_ident)
            );
        })
    }

    fn after_init(&self) -> Option<parse::proc_macro2::TokenStream> {
        let ident = format_ident!("{}", self.ident().ok()?);
        Some(quote!(CHIP = Some(#ident);))
    }

    fn before_init(&self) -> Option<parse::proc_macro2::TokenStream> {
        let peripherals = format_ident!("{}", self.peripherals.ident().ok()?);

        Some(quote! {
            let __base_peripherals = &#peripherals.nrf52;
            __base_peripherals.clock.low_stop();
            __base_peripherals.clock.high_stop();
            __base_peripherals.clock.low_start();
            __base_peripherals.clock.high_start();
            while !__base_peripherals.clock.low_started() {}
            while !__base_peripherals.clock.high_started() {}
        })
    }
}

impl parse::Chip for Chip {
    type Peripherals = Peripherals;
    type Systick = Systick;

    fn peripherals(&self) -> Rc<Self::Peripherals> {
        self.peripherals.clone()
    }

    fn systick(&self) -> Result<Rc<Self::Systick>, parse::Error> {
        Ok(self.systick.clone())
    }
}
