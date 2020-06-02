//! Component for starting up nrf52 platforms.

use kernel::component::Component;
use nrf52::gpio::Pin;
use nrf52::uicr::Regulator0Output;

pub struct NrfStartupComponent {
    nfc_as_gpios: bool,
    button_rst_pin: Pin,
    reg_vout: Regulator0Output,
}

impl NrfStartupComponent {
    pub fn new(nfc_as_gpios: bool, button_rst_pin: Pin, reg_vout: Regulator0Output) -> Self {
        Self {
            nfc_as_gpios,
            button_rst_pin,
            reg_vout,
        }
    }
}

impl Component for NrfStartupComponent {
    type StaticInput = ();
    type Output = ();
    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        // Make non-volatile memory writable and activate the reset button
        let uicr = nrf52::uicr::Uicr::new();

        // Check if we need to erase UICR memory to re-program it
        // This only needs to be done when a bit needs to be flipped from 0 to 1.
        let psel0_reset: u32 = uicr.get_psel0_reset_pin().map_or(0, |pin| pin as u32);
        let psel1_reset: u32 = uicr.get_psel1_reset_pin().map_or(0, |pin| pin as u32);
        let mut erase_uicr = ((!psel0_reset & (self.button_rst_pin as u32))
            | (!psel1_reset & (self.button_rst_pin as u32))
            | (!(uicr.get_vout() as u32) & (self.reg_vout as u32)))
            != 0;

        // Only enabling the NFC pin protection requires an erase.
        if self.nfc_as_gpios {
            erase_uicr |= !uicr.is_nfc_pins_protection_enabled();
        }

        if erase_uicr {
            nrf52::nvmc::NVMC.erase_uicr();
        }

        nrf52::nvmc::NVMC.configure_writeable();
        while !nrf52::nvmc::NVMC.is_ready() {}

        let mut needs_soft_reset: bool = false;

        // Configure reset pins
        if uicr
            .get_psel0_reset_pin()
            .map_or(true, |pin| pin != self.button_rst_pin)
        {
            uicr.set_psel0_reset_pin(self.button_rst_pin);
            while !nrf52::nvmc::NVMC.is_ready() {}
            needs_soft_reset = true;
        }
        if uicr
            .get_psel1_reset_pin()
            .map_or(true, |pin| pin != self.button_rst_pin)
        {
            uicr.set_psel1_reset_pin(self.button_rst_pin);
            while !nrf52::nvmc::NVMC.is_ready() {}
            needs_soft_reset = true;
        }

        // Configure voltage regulator output
        if uicr.get_vout() != self.reg_vout {
            uicr.set_vout(self.reg_vout);
            while !nrf52::nvmc::NVMC.is_ready() {}
            needs_soft_reset = true;
        }

        // Check if we need to free the NFC pins for GPIO
        if self.nfc_as_gpios {
            uicr.set_nfc_pins_protection(true);
            while !nrf52::nvmc::NVMC.is_ready() {}
            needs_soft_reset = true;
        }

        // Any modification of UICR needs a soft reset for the changes to be taken into account.
        if needs_soft_reset {
            cortexm4::scb::reset();
        }
    }
}

pub struct NrfClockComponent {}

impl NrfClockComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for NrfClockComponent {
    type StaticInput = ();
    type Output = ();
    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        // Start all of the clocks. Low power operation will require a better
        // approach than this.
        nrf52::clock::CLOCK.low_stop();
        nrf52::clock::CLOCK.high_stop();

        nrf52::clock::CLOCK.low_set_source(nrf52::clock::LowClockSource::XTAL);
        nrf52::clock::CLOCK.low_start();
        nrf52::clock::CLOCK.high_start();
        while !nrf52::clock::CLOCK.low_started() {}
        while !nrf52::clock::CLOCK.high_started() {}
    }
}
