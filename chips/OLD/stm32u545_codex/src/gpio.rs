// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::cell::Cell;

use kernel::hil::gpio;
use kernel::utilities::cells::OptionalCell;

pub struct Pin<'a> {
    value: Cell<bool>,
    configuration: Cell<gpio::Configuration>,
    floating: Cell<gpio::FloatingState>,
    interrupt_enabled: Cell<bool>,
    interrupt_pending: Cell<bool>,
    interrupt_edge: Cell<gpio::InterruptEdge>,
    client: OptionalCell<&'a dyn gpio::Client>,
}

impl<'a> Pin<'a> {
    pub const fn new() -> Self {
        Self {
            value: Cell::new(false),
            configuration: Cell::new(gpio::Configuration::LowPower),
            floating: Cell::new(gpio::FloatingState::PullNone),
            interrupt_enabled: Cell::new(false),
            interrupt_pending: Cell::new(false),
            interrupt_edge: Cell::new(gpio::InterruptEdge::EitherEdge),
            client: OptionalCell::empty(),
        }
    }

    pub fn handle_interrupt(&self) {
        if self.interrupt_enabled.get() {
            self.interrupt_pending.set(false);
            self.client.map(|client| client.fired());
        }
    }
}

impl gpio::Configure for Pin<'_> {
    fn configuration(&self) -> gpio::Configuration {
        self.configuration.get()
    }

    fn make_output(&self) -> gpio::Configuration {
        self.configuration.set(gpio::Configuration::Output);
        self.configuration.get()
    }

    fn disable_output(&self) -> gpio::Configuration {
        self.configuration.set(gpio::Configuration::LowPower);
        self.configuration.get()
    }

    fn make_input(&self) -> gpio::Configuration {
        self.configuration.set(gpio::Configuration::Input);
        self.configuration.get()
    }

    fn disable_input(&self) -> gpio::Configuration {
        self.configuration.set(gpio::Configuration::LowPower);
        self.configuration.get()
    }

    fn deactivate_to_low_power(&self) {
        self.configuration.set(gpio::Configuration::LowPower);
    }

    fn set_floating_state(&self, state: gpio::FloatingState) {
        self.floating.set(state);
    }

    fn floating_state(&self) -> gpio::FloatingState {
        self.floating.get()
    }
}

impl gpio::Output for Pin<'_> {
    fn set(&self) {
        self.value.set(true);
    }

    fn clear(&self) {
        self.value.set(false);
    }

    fn toggle(&self) -> bool {
        let next = !self.value.get();
        self.value.set(next);
        next
    }
}

impl gpio::Input for Pin<'_> {
    fn read(&self) -> bool {
        self.value.get()
    }
}

impl<'a> gpio::Interrupt<'a> for Pin<'a> {
    fn set_client(&self, client: &'a dyn gpio::Client) {
        self.client.set(client);
    }

    fn enable_interrupts(&self, mode: gpio::InterruptEdge) {
        self.interrupt_edge.set(mode);
        self.interrupt_enabled.set(true);
        self.interrupt_pending.set(false);
    }

    fn disable_interrupts(&self) {
        self.interrupt_enabled.set(false);
    }

    fn is_pending(&self) -> bool {
        self.interrupt_pending.get()
    }
}

pub struct Pins<'a> {
    pub led: Pin<'a>,
    pub button: Pin<'a>,
}

impl<'a> Pins<'a> {
    pub const fn new() -> Self {
        Self {
            led: Pin::new(),
            button: Pin::new(),
        }
    }
}
