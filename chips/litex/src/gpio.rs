// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! LiteX Tristate GPIO controller
//!
//! Hardware source and documentation available at
//! [`litex/soc/cores/gpio.py`](https://github.com/enjoy-digital/litex/blob/master/litex/soc/cores/gpio.py).

use core::cell::Cell;
use core::mem;
use kernel::hil;
use kernel::utilities::cells::MapCell;
use kernel::utilities::StaticRef;

use crate::event_manager::LiteXEventManager;
use crate::litex_registers::{LiteXSoCRegisterConfiguration, Read, Write};

// TODO: Make the register width adaptable, perhaps by another trait
// with the integer type as an associated type?

type LiteXGPIOEV<'a, R> = LiteXEventManager<
    'a,
    u32,
    <R as LiteXSoCRegisterConfiguration>::ReadOnly32,
    <R as LiteXSoCRegisterConfiguration>::ReadWrite32,
    <R as LiteXSoCRegisterConfiguration>::ReadWrite32,
>;

/// [`LiteXGPIOController`] register layout
#[repr(C)]
pub struct LiteXGPIORegisters<R: LiteXSoCRegisterConfiguration> {
    gpio_output_enable: R::ReadWrite32,
    gpio_input: R::ReadOnly32,
    gpio_output: R::ReadWrite32,
    gpio_mode: R::ReadWrite32,
    gpio_edge: R::ReadWrite32,
    gpio_ev_status: R::ReadOnly32,
    gpio_ev_pending: R::ReadWrite32,
    gpio_ev_enable: R::ReadWrite32,
}

impl<R: LiteXSoCRegisterConfiguration> LiteXGPIORegisters<R> {
    fn ev(&self) -> LiteXGPIOEV<'_, R> {
        LiteXGPIOEV::<R>::new(
            &self.gpio_ev_status,
            &self.gpio_ev_pending,
            &self.gpio_ev_enable,
        )
    }
}

/// LiteX Tristate GPIO controller core
pub struct LiteXGPIOController<'client, R: LiteXSoCRegisterConfiguration> {
    regs: StaticRef<LiteXGPIORegisters<R>>,
    gpio_count: usize,
    gpio_references: Cell<u32>,
    // We can't reasonably put this field only on the GPIOPin
    // instances, as then the controller would need to have a way to
    // call out to them. Thus, allocate space for a client for every
    // pin in the controller.
    gpio_clients: MapCell<[Option<&'client dyn hil::gpio::Client>; 32]>,
}

impl<'client, R: LiteXSoCRegisterConfiguration> LiteXGPIOController<'client, R> {
    pub fn new(
        base: StaticRef<LiteXGPIORegisters<R>>,
        gpio_count: usize,
    ) -> LiteXGPIOController<'client, R> {
        // The number of GPIOs may not be larger than the bit width of
        // the supplied register layout
        //
        // TODO: Automatically determine based on the type
        assert!(
            gpio_count <= 32,
            "LiteXGPIOController register width insufficient to support the requested GPIO count"
        );

        LiteXGPIOController {
            regs: base,
            gpio_count,
            gpio_references: Cell::new(0),
            gpio_clients: MapCell::new([None; 32]),
        }
    }

    /// Initialize the [`LiteXGPIOController`]
    ///
    /// This will set all GPIOs to be inputs.
    pub fn initialize(&self) {
        self.regs.gpio_output_enable.set(0);
        self.regs.ev().disable_all();
        self.regs.ev().clear_all();
    }

    /// Returns the number of GPIOs managed by the
    /// [`LiteXGPIOController`]
    pub fn gpio_count(&self) -> usize {
        self.gpio_count
    }

    /// Create a [`LiteXGPIOPin`] instance
    ///
    /// To avoid duplicate use of a GPIO, this will return `None` if
    /// an instance for the requested GPIO already exists. Call
    /// [`LiteXGPIOPin::destroy`] (or drop the [`LiteXGPIOPin`]) to be
    /// able to create a new instance for this GPIO.
    pub fn get_gpio_pin<'controller>(
        &'controller self,
        index: usize,
    ) -> Option<LiteXGPIOPin<'controller, 'client, R>> {
        if index < self.gpio_count() && (self.gpio_references.get() & (1 << index)) == 0 {
            self.gpio_references
                .set(self.gpio_references.get() | (1 << index));
            Some(LiteXGPIOPin::new(self, index))
        } else {
            None
        }
    }

    /// Internal method to mark a [`LiteXGPIOPin`] instance as destroyed
    pub(self) fn destroy_gpio_pin(&self, index: usize) {
        self.gpio_clients.map(|clients| clients[index] = None);
        self.gpio_references
            .set(self.gpio_references.get() & !(1 << index));
    }

    /// Internal method to set a GPIO output enable configuration
    pub(self) fn set_gpio_output_enable(&self, index: usize, oe: bool) {
        if oe {
            self.regs
                .gpio_output_enable
                .set(self.regs.gpio_output_enable.get() | (1 << index));
        } else {
            self.regs
                .gpio_output_enable
                .set(self.regs.gpio_output_enable.get() & !(1 << index));
        }
    }

    /// Internal method to set a GPIO output
    pub(self) fn set_gpio_output(&self, index: usize, output: bool) {
        if output {
            self.regs
                .gpio_output
                .set(self.regs.gpio_output.get() | (1 << index));
        } else {
            self.regs
                .gpio_output
                .set(self.regs.gpio_output.get() & !(1 << index));
        }
    }

    /// Internal method to read the current state of a GPIO
    ///
    /// Returns a tuple of (oe, out, in).
    pub(self) fn read_gpio(&self, index: usize) -> (bool, bool, bool) {
        (
            (self.regs.gpio_output_enable.get() & (1 << index)) != 0,
            (self.regs.gpio_output.get() & (1 << index)) != 0,
            (self.regs.gpio_input.get() & (1 << index)) != 0,
        )
    }

    /// Internal method to set a GPIO pins' interrupt client
    fn set_gpio_client(&self, index: usize, client: &'client dyn hil::gpio::Client) {
        self.gpio_clients
            .map(|clients| clients[index] = Some(client));
    }

    /// Internal method to check whether an interrupt of a GPIO pin is
    /// pending.
    ///
    /// Only GPIO pins which are in an input state will be
    /// reported as having pending interrupts.
    pub(self) fn gpio_interrupt_pending(&self, index: usize) -> bool {
        self.regs.ev().event_asserted(index)
    }

    /// Internal method to configure a GPIO pin's interrupts, or
    /// disable them.
    pub(self) fn configure_gpio_interrupt(
        &self,
        index: usize,
        edge: Option<hil::gpio::InterruptEdge>,
    ) {
        if let Some(e) = edge {
            // To make sure we don't cause any CPU interrupts just
            // because of reconfiguration, disable the event first.
            self.regs.ev().disable_event(index);

            // Now, set the configuration. Interrupts are configured
            // in two bits:
            // - mode: 0 for a specific edge, 1 for every edge
            // - edge: if mode == 1, 0 for rising edge, 1 for falling egde
            match e {
                hil::gpio::InterruptEdge::RisingEdge => {
                    self.regs
                        .gpio_mode
                        .set(self.regs.gpio_mode.get() & !(1 << index));
                    self.regs
                        .gpio_edge
                        .set(self.regs.gpio_edge.get() & !(1 << index));
                }
                hil::gpio::InterruptEdge::FallingEdge => {
                    self.regs
                        .gpio_mode
                        .set(self.regs.gpio_mode.get() & !(1 << index));
                    self.regs
                        .gpio_edge
                        .set(self.regs.gpio_edge.get() & !(1 << index));
                }
                hil::gpio::InterruptEdge::EitherEdge => {
                    self.regs
                        .gpio_mode
                        .set(self.regs.gpio_mode.get() & !(1 << index));
                }
            }

            // (Re)enable the event associated with the GPIO pin
            self.regs.ev().enable_event(index);
        } else {
            // Simply disable the interrupts in the EV. This will
            // prevent the source from asserting the event manager's
            // CPU interrupt.
            self.regs.ev().disable_event(index);
        }
    }

    pub fn service_interrupt(&self) {
        while let Some(event_index) = self.regs.ev().next_asserted() {
            self.regs.ev().clear_event(event_index);
            self.gpio_clients
                .map(|clients| clients[event_index].map(|client| client.fired()));
        }
    }
}

/// Single GPIO pin of a [`LiteXGPIOController`]
///
/// Can be obtained by calling [`LiteXGPIOController::get_gpio_pin`].
///
/// Only one [`LiteXGPIOPin`] instance may exist per GPIO pin. To
/// deregister this instance, call [`LiteXGPIOPin::destroy`] (or drop it).
pub struct LiteXGPIOPin<'controller, 'client, R: LiteXSoCRegisterConfiguration> {
    controller: &'controller LiteXGPIOController<'client, R>,
    index: usize,
}

impl<'controller, 'client, R: LiteXSoCRegisterConfiguration> LiteXGPIOPin<'controller, 'client, R> {
    fn new(
        controller: &'controller LiteXGPIOController<'client, R>,
        index: usize,
    ) -> LiteXGPIOPin<'controller, 'client, R> {
        LiteXGPIOPin { controller, index }
    }

    /// Index of this GPIO pin in the [`LiteXGPIOController`] GPIO array
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns a reference to the [`LiteXGPIOController`] of this GPIO
    pub fn controller(&self) -> &'controller LiteXGPIOController<'client, R> {
        self.controller
    }

    /// Destroy (deregister & consume) the [`LiteXGPIOPin`]
    pub fn destroy(self) {
        mem::drop(self);
    }
}

impl<R: LiteXSoCRegisterConfiguration> hil::gpio::Configure for LiteXGPIOPin<'_, '_, R> {
    fn configuration(&self) -> hil::gpio::Configuration {
        let (output_enable, _, _) = self.controller.read_gpio(self.index);
        if output_enable {
            hil::gpio::Configuration::Output
        } else {
            hil::gpio::Configuration::Input
        }
    }

    fn make_output(&self) -> hil::gpio::Configuration {
        self.controller.set_gpio_output_enable(self.index, true);
        hil::gpio::Configuration::Output
    }

    fn disable_output(&self) -> hil::gpio::Configuration {
        // Only meaningful thing to do here is to switch to being an
        // input.
        self.make_input()
    }

    fn make_input(&self) -> hil::gpio::Configuration {
        self.controller.set_gpio_output_enable(self.index, false);
        hil::gpio::Configuration::Input
    }

    fn disable_input(&self) -> hil::gpio::Configuration {
        // The GPIO tristate pin has to be in either output or
        // input. We can't not be in input, but also not in
        // output. However, switching to an output when one wants to
        // "disable_input" is pretty dangerous. We can however remain
        // an output if we are one. Thus, do nothing and return the
        // current configuration.
        self.configuration()
    }

    fn deactivate_to_low_power(&self) {
        self.make_input();
    }

    fn set_floating_state(&self, _state: hil::gpio::FloatingState) {
        // Do nothing, we don't have any pullups we could reasonably
        // use.
    }

    fn floating_state(&self) -> hil::gpio::FloatingState {
        hil::gpio::FloatingState::PullNone
    }
}

impl<R: LiteXSoCRegisterConfiguration> hil::gpio::Output for LiteXGPIOPin<'_, '_, R> {
    fn set(&self) {
        self.controller.set_gpio_output(self.index, true);
    }

    fn clear(&self) {
        self.controller.set_gpio_output(self.index, false);
    }

    fn toggle(&self) -> bool {
        let (_, current, _) = self.controller.read_gpio(self.index);
        self.controller.set_gpio_output(self.index, !current);
        !current
    }
}

impl<R: LiteXSoCRegisterConfiguration> hil::gpio::Input for LiteXGPIOPin<'_, '_, R> {
    fn read(&self) -> bool {
        // For a proper tristate, we could probably just read it and
        // if the pin is an output, retrieve the current output value
        // directly. However, the simulation behaves a litte
        // different. Thus check the pin state and either return the
        // current input or output state, depending on output_enable.
        let (output_enable, output, input) = self.controller.read_gpio(self.index);
        if output_enable {
            output
        } else {
            input
        }
    }
}

impl<'client, R: LiteXSoCRegisterConfiguration> hil::gpio::Interrupt<'client>
    for LiteXGPIOPin<'_, 'client, R>
{
    fn set_client(&self, client: &'client dyn hil::gpio::Client) {
        self.controller.set_gpio_client(self.index, client);
    }

    fn is_pending(&self) -> bool {
        self.controller.gpio_interrupt_pending(self.index)
    }

    fn enable_interrupts(&self, mode: hil::gpio::InterruptEdge) {
        self.controller
            .configure_gpio_interrupt(self.index, Some(mode));
    }

    fn disable_interrupts(&self) {
        self.controller.configure_gpio_interrupt(self.index, None);
    }
}

impl<R: LiteXSoCRegisterConfiguration> Drop for LiteXGPIOPin<'_, '_, R> {
    /// Deregister the GPIO with the controller
    fn drop(&mut self) {
        self.controller.destroy_gpio_pin(self.index);
    }
}
