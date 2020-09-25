use core::cell::Cell;
use kernel::common::StaticRef;
use kernel::hil;

use crate::litex_registers::{LiteXSoCRegisterConfiguration, Read, Write};

// TODO: Make the register width adaptable, perhaps by another trait
// with the integer type as an associated type?

#[repr(C)]
pub struct LiteXLedRegisters<R: LiteXSoCRegisterConfiguration> {
    leds_out: R::ReadWrite8,
}

pub struct LiteXLedController<R: LiteXSoCRegisterConfiguration> {
    regs: StaticRef<LiteXLedRegisters<R>>,
    led_count: usize,
    led_references: Cell<u8>,
    active_low: bool,
}

impl<R: LiteXSoCRegisterConfiguration> LiteXLedController<R> {
    pub const fn new(
        base: StaticRef<LiteXLedRegisters<R>>,
        led_count: usize,
        active_low: bool,
    ) -> LiteXLedController<R> {
        // The number of leds may not be larger than the bit width of
        // the supplied register layout
        //
        // TODO: Automatically determine based on the type
        assert!(
            led_count <= 8,
            "LiteXLedController register width insufficient to support the requested LED count"
        );

        LiteXLedController {
            regs: base,
            led_count,
            led_references: Cell::new(0),
            active_low,
        }
    }

    pub fn initialize(&self) {
        self.regs.leds_out.set(if self.active_low {
            ((1 << self.led_count() as isize) - 1) as u8
        } else {
            0
        });
    }

    pub fn led_count(&self) -> usize {
        self.led_count
    }

    pub fn get_led<'a>(&'a self, index: usize) -> Option<LiteXLed<'a, R>> {
        if index < self.led_count() && (self.led_references.get() & (1 << index)) == 0 {
            self.led_references
                .set(self.led_references.get() | (1 << index));
            Some(LiteXLed::new(self, index))
        } else {
            None
        }
    }

    /// Create a LiteXLed instance referencing a specific LED on the
    /// controller
    ///
    /// This function must only be used in a panic handler, if no
    /// other code will be running afterwards, in order to guarantee
    /// consistency between ownership of the LiteXLed instance and
    /// control over the LED state
    ///
    /// This function only checks whether the requested LEDs is within
    /// the controller's range of available LEDs, but *NOT* whether
    /// there already is a different reference to the same LED.
    pub unsafe fn panic_led<'a>(&'a self, index: usize) -> Option<LiteXLed<'a, R>> {
        if index < self.led_count() {
            Some(LiteXLed::new(self, index))
        } else {
            None
        }
    }

    fn destroy_led(&self, index: usize) {
        self.led_references
            .set(self.led_references.get() & !(1 << index));
    }

    fn set_led(&self, index: usize, val: bool) {
        if val ^ self.active_low {
            self.regs
                .leds_out
                .set(self.regs.leds_out.get() | (1 << index));
        } else {
            self.regs
                .leds_out
                .set(self.regs.leds_out.get() & !(1 << index));
        }
    }

    fn read_led(&self, index: usize) -> bool {
        ((self.regs.leds_out.get() & (1 << index)) != 0) ^ self.active_low
    }
}

pub struct LiteXLed<'a, R: LiteXSoCRegisterConfiguration> {
    controller: &'a LiteXLedController<R>,
    index: usize,
}

impl<'a, R: LiteXSoCRegisterConfiguration> LiteXLed<'a, R> {
    fn new(controller: &'a LiteXLedController<R>, index: usize) -> LiteXLed<'a, R> {
        LiteXLed { controller, index }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn controller(&self) -> &'a LiteXLedController<R> {
        self.controller
    }

    pub fn destroy(self) {
        self.controller.destroy_led(self.index);
    }
}

impl<'a, R: LiteXSoCRegisterConfiguration> hil::led::Led for LiteXLed<'a, R> {
    fn init(&mut self) {
        self.controller.set_led(self.index, false);
    }

    fn on(&mut self) {
        self.controller.set_led(self.index, true);
    }

    fn off(&mut self) {
        self.controller.set_led(self.index, false);
    }

    fn toggle(&mut self) {
        self.controller
            .set_led(self.index, !self.controller.read_led(self.index));
    }

    fn read(&self) -> bool {
        self.controller.read_led(self.index)
    }
}
