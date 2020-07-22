//! Implements a helper function for enabling/disabling power on the imix
//! submodules.
//  On imix, submodules are powered on/off via power gate ICs. The MCU has an
//  enable pin connected to each power gate. Setting an enable pin high or low
//  turns on or off the corresponding submodule.
//
//  For the RF233, NRF, and sensors, there is an additional IC which electrically
//  connects/disconnects the I2C and SPI busses to the appropriate submodules,
//  depending on their power state. This is because the busses can inadvertently
//  supply power to the connected submodules. The correct behavior is all handled
//  in hardware via the enable pins.
//
//  For the RF233 and NRF, there are still some remaining GPIO pins that need to
//  be configured when these submodules are turned off. These pins are not gated
//  by a switch like the I2C and SPI busses are, so if the kernel does not
//  explicitly pull them to ground, the submodule will be powered through the GPIO.
//
//  This file exports `configure_submodules`, which hides the complexity
//  of correctly turning the submodules on and off. It allows the caller to
//  conveniently disable and enable the individual submodules at will.

use kernel::hil::Controller;
use sam4l::gpio::GPIOPin;
use sam4l::gpio::PeripheralFunction;
use sam4l::gpio::PeripheralFunction::{A, B, E};
use sam4l::gpio::{PA, PB, PC};

struct DetachablePin {
    pin: &'static GPIOPin<'static>,
    function: Option<PeripheralFunction>,
}

impl DetachablePin {
    fn detach(&self) {
        self.pin.configure(None);
        self.pin.enable_output();
        self.pin.clear();
    }

    fn restore(&self) {
        self.pin.configure(self.function);
    }
}

struct Submodule<'a> {
    gate_pin: &'static GPIOPin<'static>,
    detachable_pins: &'a [DetachablePin],
}

impl<'a> Submodule<'a> {
    fn power(&self, state: bool) {
        self.gate_pin.enable_output();
        match state {
            true => {
                for d in self.detachable_pins.iter() {
                    d.restore();
                }
                self.gate_pin.set();
            }
            false => {
                self.gate_pin.clear();
                for d in self.detachable_pins.iter() {
                    d.detach();
                }
            }
        }
    }
}

pub struct SubmoduleConfig {
    pub rf233: bool,
    pub nrf51422: bool,
    pub sensors: bool,
    pub trng: bool,
}

pub unsafe fn configure_submodules(enabled_submodules: SubmoduleConfig) {
    let rf233_detachable_pins = [
        DetachablePin {
            pin: &PA[08],
            function: None,
        },
        DetachablePin {
            pin: &PA[09],
            function: None,
        },
        DetachablePin {
            pin: &PA[10],
            function: None,
        },
    ];
    let rf233 = Submodule {
        gate_pin: &PC[18],
        detachable_pins: &rf233_detachable_pins,
    };

    let nrf_detachable_pins = [
        DetachablePin {
            pin: &PB[07],
            function: None,
        },
        DetachablePin {
            pin: &PA[17],
            function: None,
        },
        DetachablePin {
            pin: &PA[18],
            function: Some(A),
        },
        DetachablePin {
            pin: &PC[07],
            function: Some(B),
        },
        DetachablePin {
            pin: &PC[08],
            function: Some(E),
        },
        DetachablePin {
            pin: &PC[09],
            function: None,
        },
    ];
    let nrf = Submodule {
        gate_pin: &PC[17],
        detachable_pins: &nrf_detachable_pins,
    };

    let sensors = Submodule {
        gate_pin: &PC[16],
        detachable_pins: &[],
    };

    let trng = Submodule {
        gate_pin: &PC[19],
        detachable_pins: &[],
    };

    rf233.power(enabled_submodules.rf233);
    nrf.power(enabled_submodules.nrf51422);
    sensors.power(enabled_submodules.sensors);
    trng.power(enabled_submodules.trng);
}
