// Copyright OxidOS Automotive 2024.

use parse::{constants::PERIPHERALS, peripheral, Ident};
use quote::quote;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum UartType {
    Uart0,
}

#[derive(Debug, PartialEq)]
#[peripheral(serde, ident = ".nrf52.uarte0")]
pub struct Uart(UartType);

impl Default for Uart {
    fn default() -> Self {
        Self::new(UartType::Uart0)
    }
}

impl parse::Component for Uart {
    fn before_usage(&self) -> Option<parse::proc_macro2::TokenStream> {
        let ident: proc_macro2::TokenStream = self.ident().ok()?.parse().unwrap();
        Some(quote! {
        #ident.initialize(
               nrf52::pinmux::Pinmux::new(nrf52833::gpio::Pin::P0_06 as u32),
               nrf52::pinmux::Pinmux::new(nrf52833::gpio::Pin::P1_08 as u32),
               None,
               None,
           );
        })
    }
}

impl parse::Uart for Uart {}
impl std::fmt::Display for Uart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "uarte0")
    }
}
