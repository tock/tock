// Copyright OxidOS Automotive 2024.

use parse::constants::PERIPHERALS;
use parse::{peripheral, Component};
use quote::quote;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum TimerType {
    Rtc,
}

#[derive(Debug, PartialEq)]
#[peripheral(serde, ident = ".nrf52.rtc")]
pub struct Timer(TimerType);

impl Component for Timer {
    fn ty(&self) -> Result<parse::proc_macro2::TokenStream, parse::Error> {
        Ok(quote!(nrf52::rtc::Rtc<'static>))
    }
}

impl parse::Timer for Timer {
    fn frequency(&self) -> usize {
        0
    }
}

impl std::fmt::Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rtc")
    }
}
