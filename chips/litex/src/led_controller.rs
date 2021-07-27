//! LiteX led controller (`LedChaser` core)
//!
//! Hardware source and documentation available at
//! [`litex/soc/cores/led.py`](https://github.com/enjoy-digital/litex/blob/master/litex/soc/cores/led.py).

use core::cell::Cell;
use core::mem;
use kernel::hil;
use kernel::utilities::StaticRef;

use crate::litex_registers::{LiteXSoCRegisterConfiguration, Read, Write};

// TODO: Make the register width adaptable, perhaps by another trait
// with the integer type as an associated type?

/// [`LiteXLedController`] register layout
#[repr(C)]
pub struct LiteXLedRegisters<R: LiteXSoCRegisterConfiguration> {
    leds_out: R::ReadWrite8,
}

/// LiteX led controller core
pub struct LiteXLedController<R: LiteXSoCRegisterConfiguration> {
    regs: StaticRef<LiteXLedRegisters<R>>,
    led_count: usize,
    led_references: Cell<u8>,
}

impl<R: LiteXSoCRegisterConfiguration> LiteXLedController<R> {
    pub fn new(base: StaticRef<LiteXLedRegisters<R>>, led_count: usize) -> LiteXLedController<R> {
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
        }
    }

    /// Initialize the [`LiteXLedController`]
    ///
    /// This will turn all LEDs off, thus disabling the *LED Chaser*
    /// hardware-pattern of the LiteX core and switching to explicit
    /// software control.
    pub fn initialize(&self) {
        self.regs.leds_out.set(0);
    }

    /// Returns the number of LEDs managed by the
    /// [`LiteXLedController`]
    pub fn led_count(&self) -> usize {
        self.led_count
    }

    /// Create a [`LiteXLed`] instance
    ///
    /// To avoid duplicate use of a LED, this will return `None` if an
    /// instance for the requested LED already exists. Call
    /// [`LiteXLed::destroy`] (or drop the [`LiteXLed`]) to be create
    /// a new instance for this LED.
    pub fn get_led<'a>(&'a self, index: usize) -> Option<LiteXLed<'a, R>> {
        if index < self.led_count() && (self.led_references.get() & (1 << index)) == 0 {
            self.led_references
                .set(self.led_references.get() | (1 << index));
            Some(LiteXLed::new(self, index))
        } else {
            None
        }
    }

    /// Create a [`LiteXLed`] without checking for duplicates
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

    /// Internal method to mark a [`LiteXLed`] instance as destroyed
    pub(self) fn destroy_led(&self, index: usize) {
        self.led_references
            .set(self.led_references.get() & !(1 << index));
    }

    /// Internal method to set a LED output
    pub(self) fn set_led(&self, index: usize, val: bool) {
        if val {
            self.regs
                .leds_out
                .set(self.regs.leds_out.get() | (1 << index));
        } else {
            self.regs
                .leds_out
                .set(self.regs.leds_out.get() & !(1 << index));
        }
    }

    /// Internal method to read the current state of a LED
    pub(self) fn read_led(&self, index: usize) -> bool {
        (self.regs.leds_out.get() & (1 << index)) != 0
    }
}

/// Single LED of a [`LiteXLedController`]
///
/// Can be obtained by calling [`LiteXLedController::get_led`].
///
/// Only one [`LiteXLed`] instance may exist per LED. To deregister
/// this instance, call [`LiteXLed::destroy`] (or drop it).
pub struct LiteXLed<'a, R: LiteXSoCRegisterConfiguration> {
    controller: &'a LiteXLedController<R>,
    index: usize,
}

impl<'a, R: LiteXSoCRegisterConfiguration> LiteXLed<'a, R> {
    fn new(controller: &'a LiteXLedController<R>, index: usize) -> LiteXLed<'a, R> {
        LiteXLed { controller, index }
    }

    /// Index of this LED in the [`LiteXLedController`] LED array
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns a reference to the [`LiteXLedController`] of this LED
    pub fn controller(&self) -> &'a LiteXLedController<R> {
        self.controller
    }

    /// Destroy (deregister & consume) the [`LiteXLed`]
    pub fn destroy(self) {
        mem::drop(self);
    }
}

impl<'a, R: LiteXSoCRegisterConfiguration> hil::led::Led for LiteXLed<'a, R> {
    fn init(&self) {
        self.controller.set_led(self.index, false);
    }

    fn on(&self) {
        self.controller.set_led(self.index, true);
    }

    fn off(&self) {
        self.controller.set_led(self.index, false);
    }

    fn toggle(&self) {
        self.controller
            .set_led(self.index, !self.controller.read_led(self.index));
    }

    fn read(&self) -> bool {
        self.controller.read_led(self.index)
    }
}

impl<'a, R: LiteXSoCRegisterConfiguration> Drop for LiteXLed<'a, R> {
    /// Deregister the LED with the controller
    fn drop(&mut self) {
        self.controller.destroy_led(self.index);
    }
}
