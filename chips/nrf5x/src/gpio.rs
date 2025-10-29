// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! GPIO and GPIOTE (task and events), nRF5x-family
//!
//! ### Author
//! * Philip Levis <pal@cs.stanford.edu>
//! * Date: August 18, 2016

use core::ops::{Index, IndexMut};
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::debug;
use kernel::hil;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadWrite};
use kernel::utilities::StaticRef;

#[cfg(feature = "nrf51")]
const NUM_GPIOTE: usize = 4;
#[cfg(feature = "nrf52")]
const NUM_GPIOTE: usize = 8;
// Dummy value for testing on Travis-CI.
#[cfg(all(
    not(all(target_arch = "arm", target_os = "none")),
    not(feature = "nrf51"),
    not(feature = "nrf52"),
))]
const NUM_GPIOTE: usize = 4;

const GPIO_PER_PORT: usize = 32;

pub const GPIO_BASE_ADDRESS: usize = 0x50000000;
pub const GPIO_SIZE: usize = 0x300;

/// nRF5x GPIOTE Registers
///
/// The nRF5x doesn't automatically provide GPIO interrupts. Instead, to receive
/// interrupts from a GPIO line, you must allocate a GPIOTE (GPIO Task and
/// Event) channel, and bind the channel to the desired pin. There are 4
/// channels for the nrf51 and 8 channels for the nrf52. This means that
/// requesting an interrupt can fail, if they are all already allocated.
#[repr(C)]
pub struct GpioteRegisters {
    /// Task for writing to pin specified in CONFIG\[n\].PSEL.
    /// Action on pin is configured in CONFIG\[n\].POLARITY
    ///
    /// - Address: 0x000 - 0x010 (nRF51)
    /// - Address: 0x000 - 0x020 (nRF52)
    task_out: [ReadWrite<u32, TasksOut::Register>; NUM_GPIOTE],
    /// Reserved
    // task_set and task_clear are not used on nRF52
    _reserved0: [u8; 0x100 - (0x0 + NUM_GPIOTE * 4)],
    /// Event generated from pin specified in CONFIG\[n\].PSEL
    ///
    /// - Address: 0x100 - 0x110 (nRF51)
    /// - Address: 0x100 - 0x120 (nRF52)
    event_in: [ReadWrite<u32, EventsIn::Register>; NUM_GPIOTE],
    /// Reserved
    _reserved1: [u8; 0x17C - (0x100 + NUM_GPIOTE * 4)],
    /// Event generated from multiple input GPIO pins
    /// - Address: 0x17C - 0x180
    event_port: ReadWrite<u32, EventsPort::Register>,
    /// Reserved
    // inten on nRF51 is ignored because intenset and intenclr provides the same functionality
    _reserved2: [u8; 0x184],
    /// Enable interrupt
    /// - Address: 0x304 - 0x308
    intenset: ReadWrite<u32, Intenset::Register>,
    /// Disable interrupt
    /// - Address: 0x308 - 0x30C
    intenclr: ReadWrite<u32, Intenclr::Register>,
    /// Reserved
    _reserved3: [u8; 0x204],
    /// Configuration for OUT\[n\], SET\[n\] and CLR\[n\] tasks and IN\[n\] event
    ///
    /// - Adress: 0x510 - 0x520 (nRF51)
    /// - Adress: 0x510 - 0x530 (nRF52)
    // Note, only IN\[n\] and OUT\[n\] are used in Tock
    config: [ReadWrite<u32, Config::Register>; NUM_GPIOTE],
}

#[repr(C)]
pub struct GpioRegisters {
    /// Reserved
    _reserved1: [u32; 321],
    /// Write GPIO port
    /// - Address: 0x504 - 0x508
    out: ReadWrite<u32, Out::Register>,
    /// Set individual bits in GPIO port
    /// - Address: 0x508 - 0x50C
    outset: ReadWrite<u32, OutSet::Register>,
    /// Clear individual bits in GPIO port
    /// - Address: 0x50C - 0x510
    outclr: ReadWrite<u32, OutClr::Register>,
    /// Read GPIO Port
    /// - Address: 0x510 - 0x514
    in_: ReadWrite<u32, In::Register>,
    /// Direction of GPIO pins
    /// - Address: 0x514 - 0x518
    dir: ReadWrite<u32, Dir::Register>,
    /// DIR set register
    /// - Address: 0x518 - 0x51C
    dirset: ReadWrite<u32, DirSet::Register>,
    /// DIR clear register
    /// - Address: 0x51C - 0x520
    dirclr: ReadWrite<u32, DirClr::Register>,
    #[cfg(feature = "nrf51")]
    /// Reserved
    _reserved2: [u32; 120],
    /// Latch register indicating what GPIO pins that have met the criteria set in the
    /// PIN_CNF\[n\].SENSE
    /// - Address: 0x520 - 0x524
    #[cfg(feature = "nrf52")]
    latch: ReadWrite<u32, Latch::Register>,
    /// Select between default DETECT signal behaviour and LDETECT mode
    /// - Address: 0x524 - 0x528
    #[cfg(feature = "nrf52")]
    detect_mode: ReadWrite<u32, DetectMode::Register>,
    /// Reserved
    #[cfg(feature = "nrf52")]
    _reserved2: [u32; 118],
    /// Configuration of GPIO pins
    pin_cnf: [ReadWrite<u32, PinConfig::Register>; 32],
}

// Gpio
register_bitfields! [u32,
    /// Write GPIO port
    Out [
        /// Pin\[n\], each bit correspond to a pin 0 to 31
        /// 0 - Low, Pin driver is low
        /// 1 - High, Pin driver is high
        PIN OFFSET(0) NUMBITS(32)
    ],
    /// Set individual bits in GPIO port
    OutSet [
        /// Pin\[n\], each bit correspond to a pin 0 to 31
        /// 0 - Low
        /// 1 - High
        /// Writing a '1' sets the pin high
        /// Writing a '0' has no effect
        PIN OFFSET(0) NUMBITS(32)
    ],
    /// Clear individual bits in GPIO port
    OutClr [
        /// Pin\[n\], each bit correspond to a pin 0 to 31
        /// 0 - Low
        /// 1 - High
        /// Writing a '1' sets the pin low
        /// Writing a '0' has no effect
        PIN OFFSET(0) NUMBITS(32)
    ],
    /// Read GPIO port
    In [
        /// Pin\[n\], each bit correspond to a pin 0 to 31
        /// 0 - Low
        /// 1 - High
        PIN OFFSET(0) NUMBITS(32)
    ],
    /// Direction of GPIO pins
    Dir [
        /// 0 - Pin set as input
        /// 1 - Pin set as output
        PIN OFFSET(0) NUMBITS(32)
    ],
    /// Configure direction of individual GPIO pins as output
    DirSet [
        /// Pin\[n\], each bit correspond to a pin 0 to 31
        /// 0 - Pin set as input
        /// 1 - Pin set as output
        /// Write: writing a '1' sets pin to output
        /// Writing a '0' has no effect
        PIN OFFSET(0) NUMBITS(32)
    ],
    /// Configure direction of individual GPIO pins as input
    DirClr [
        /// Pin\[n\], each bit correspond to a pin 0 to 31
        /// 0 - Pin set as input
        /// 1 - Pin set as output
        /// Write: writing a '1' sets pin to input
        /// Writing a '0' has no effect
        PIN OFFSET(0) NUMBITS(32)
    ],
    /// Latch register indicating what GPIO pins that have met the criteria set in the
    /// PIN_CNF\[n\].SENSE registers
    Latch [
        /// Pin\[n\], each bit correspond to a pin 0 to 31
        /// 0 - NotLatched
        /// 1 - Latched
        PIN OFFSET(0) NUMBITS(32)
    ],
    /// Select between default DETECT signal behaviour and LDETECT mode
    DetectMode [
        /// 0 - NotLatched
        /// 1 - Latched
        DETECTMODE OFFSET(0) NUMBITS(1) [
            DEFAULT = 0,
            LDDETECT = 1
        ]
    ],
    /// Configuration of GPIO pins
    /// Pin\[n\], each bit correspond to a pin 0 to 31
    PinConfig [
        /// Pin direction. Same physical register as DIR register
        DIR OFFSET(0) NUMBITS(1) [
            Input = 0,
            Output = 1
        ],
        /// Connect or disconnect input buffer
        INPUT OFFSET(1) NUMBITS(1) [
            Connect = 0,
            Disconnect = 1
        ],
        /// Pull configuration
        PULL OFFSET(2) NUMBITS(2) [
            Disabled = 0,
            Pulldown = 1,
            Pullup = 3
        ],
        /// Drive configuration
        DRIVE OFFSET(8) NUMBITS(3) [
            /// Standard '0', standard '1'
            S0S1 = 0,
            /// High drive '0', standard '1'
            H0S1 = 1,
            /// Standard '0', high drive '1
            S0H1 = 2,
            /// High drive '0', high 'drive '1'
            H0H1 = 3,
            /// Disconnect '0' standard '1' (normally used for wired-or connections)
            D0S1 = 4,
            /// Disconnect '0', high drive '1' (normally used for wired-or connections)
            D0H1 = 5,
            /// Standard '0'. disconnect '1' (normally used for wired-and connections)
            S0D1 = 6,
            /// High drive '0', disconnect '1' (normally used for wired-and connections)
            H0D1 = 7
        ],
        /// Pin sensing mechanism
        SENSE OFFSET(16) NUMBITS(2) [
            /// Disabled
            Disabled = 0,
            /// Sense for high level
            High = 2,
            /// Sense for low level
            Low = 3
        ]
    ]
];

// GpioTe
register_bitfields! [u32,
    /// Task for writing to pin specified in CONFIG\[n\].PSEL.
    /// Action on pin is configured in CONFIG\[n\].POLARITY
    TasksOut [
        TASK OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ]
    ],

    /// Event generated from pin specified in CONFIG\[n\].PSEL
    EventsIn [
        EVENT OFFSET(0) NUMBITS(1) [
            NotReady = 0,
            Ready = 1
        ]
    ],

    /// Event generated from multiple input pins
    EventsPort [
        PINS OFFSET(0) NUMBITS(1) [
            NotReady = 0,
            Ready = 1
        ]
    ],

    /// Enable interrupt
    Intenset [
        // nRF51 has only 4 inputs
        IN OFFSET(0) NUMBITS(8),
        PORT OFFSET(31) NUMBITS(1)
    ],

    /// Disable interrupt
    Intenclr [
        // nRF51 has only 4 inputs
        IN OFFSET(0) NUMBITS(8),
        PORT OFFSET(31) NUMBITS(1)
    ],

    /// Configuration for OUT\[n\], SET\[n\] and CLR\[n\] tasks and IN\[n\] event
    Config [
        /// Mode
        MODE OFFSET(0) NUMBITS(2) [
            /// Disabled. Pin specified by PSEL will not be acquired by the
            /// GPIOTE module
            Disabled = 0,
            /// The pin specified by PSEL will be configured as an input and the
            /// IN\[n\] event will be generated if operation specified in POLARITY
            /// occurs on the pin.
            Event = 1,
            ///The GPIO specified by PSEL will be configured as an output and
            /// triggering the SET\[n\], CLR\[n\] or OUT\[n\] task will perform the
            /// operation specified by POLARITY on the pin. When enabled as a
            /// task the GPIOTE module will acquire the pin and the pin can no
            /// longer be written as a regular output pin from the GPIO module.
            Task = 3
        ],
        /// GPIO number associated with SET\[n\], CLR\[n\] and OUT\[n\] tasks
        /// and IN\[n\] event. Only 5 bits are used but they are followed by 1 bit
        /// indicating the port. This allows us to abstract the port away as each port
        /// is defined for 32 pins.
        PSEL OFFSET(8) NUMBITS(6) [],
        /// When In task mode: Operation to be performed on output
        /// when OUT\[n\] task is triggered. When In event mode: Operation
        /// on input that shall trigger IN\[n\] event
        POLARITY OFFSET(16) NUMBITS(2) [
            /// Task mode: No effect on pin from OUT\[n\] task. Event mode: no
            /// IN\[n\] event generated on pin activity
            Disabled = 0,
            /// Task mode: Set pin from OUT\[n\] task. Event mode: Generate
            /// IN\[n\] event when rising edge on pin
            LoToHi = 1,
            /// Task mode: Clear pin from OUT\[n\] task. Event mode: Generate
            /// IN\[n\] event when falling edge on pin
            HiToLo = 2,
            /// Task mode: Toggle pin from OUT\[n\]. Event mode: Generate
            /// IN\[n\] when any change on pin
            Toggle = 3
        ],
        /// When in task mode: Initial value of the output when the GPIOTE
        /// channel is configured. When in event mode: No effect
        OUTINIT OFFSET(20) NUMBITS(1) [
            /// Task mode: Initial value of pin before task triggering is low
            Low = 0,
            /// Task mode: Initial value of pin before task triggering is high
            High = 1
        ]
    ]
];

enum_from_primitive! {
    #[derive(Copy, Clone, Debug, PartialEq)]
    #[rustfmt::skip]
    pub enum Pin {
        P0_00, P0_01, P0_02, P0_03, P0_04, P0_05, P0_06, P0_07,
        P0_08, P0_09, P0_10, P0_11, P0_12, P0_13, P0_14, P0_15,
        P0_16, P0_17, P0_18, P0_19, P0_20, P0_21, P0_22, P0_23,
        P0_24, P0_25, P0_26, P0_27, P0_28, P0_29, P0_30, P0_31,
        // Pins only on nrf52840.
        P1_00, P1_01, P1_02, P1_03, P1_04, P1_05, P1_06, P1_07,
        P1_08, P1_09, P1_10, P1_11, P1_12, P1_13, P1_14, P1_15,
    }
}

pub struct GPIOPin<'a> {
    pin: u8,
    port: u8,
    client: OptionalCell<&'a dyn hil::gpio::Client>,
    gpiote_registers: StaticRef<GpioteRegisters>,
    gpio_registers: StaticRef<GpioRegisters>,
    allocated_channel: OptionalCell<usize>,
}

impl<'a> GPIOPin<'a> {
    pub const fn new(
        pin: Pin,
        gpiote_registers: StaticRef<GpioteRegisters>,
        gpio_registers: StaticRef<GpioRegisters>,
    ) -> GPIOPin<'a> {
        GPIOPin {
            pin: ((pin as usize) % GPIO_PER_PORT) as u8,
            port: ((pin as usize) / GPIO_PER_PORT) as u8,
            client: OptionalCell::empty(),
            gpio_registers,
            gpiote_registers,
            allocated_channel: OptionalCell::empty(),
        }
    }

    pub fn set_high_drive(&self, high_drive: bool) {
        self.gpio_registers.pin_cnf[self.pin as usize].modify(if high_drive {
            PinConfig::DRIVE::H0H1
        } else {
            PinConfig::DRIVE::S0S1
        });
    }

    // This sets the specified pin cfg as per the TRM for i2c pin usage.
    pub fn set_i2c_pin_cfg(&self) {
        self.gpio_registers.pin_cnf[self.pin as usize].modify(
            PinConfig::DIR::Input
                + PinConfig::INPUT::Disconnect
                + PinConfig::DRIVE::S0D1
                + PinConfig::SENSE::Disabled,
        );
    }
}

impl hil::gpio::Configure for GPIOPin<'_> {
    fn set_floating_state(&self, mode: hil::gpio::FloatingState) {
        let pin_config = match mode {
            hil::gpio::FloatingState::PullUp => PinConfig::PULL::Pullup,
            hil::gpio::FloatingState::PullDown => PinConfig::PULL::Pulldown,
            hil::gpio::FloatingState::PullNone => PinConfig::PULL::Disabled,
        };
        // PIN_CNF also holds the direction and the pin driving mode, settings we don't
        // want to overwrite!
        self.gpio_registers.pin_cnf[self.pin as usize].modify(pin_config);
    }

    fn floating_state(&self) -> hil::gpio::FloatingState {
        match self.gpio_registers.pin_cnf[self.pin as usize].read_as_enum(PinConfig::PULL) {
            Some(PinConfig::PULL::Value::Pullup) => hil::gpio::FloatingState::PullUp,
            Some(PinConfig::PULL::Value::Pulldown) => hil::gpio::FloatingState::PullDown,
            Some(PinConfig::PULL::Value::Disabled) => hil::gpio::FloatingState::PullNone,
            None => hil::gpio::FloatingState::PullNone,
        }
    }

    fn make_output(&self) -> hil::gpio::Configuration {
        self.gpio_registers.pin_cnf[self.pin as usize].modify(PinConfig::DIR::Output);
        hil::gpio::Configuration::Output
    }

    fn disable_output(&self) -> hil::gpio::Configuration {
        self.make_input()
    }

    fn make_input(&self) -> hil::gpio::Configuration {
        self.gpio_registers.pin_cnf[self.pin as usize]
            .modify(PinConfig::DIR::Input + PinConfig::INPUT::Connect);
        hil::gpio::Configuration::Input
    }

    fn disable_input(&self) -> hil::gpio::Configuration {
        // GPIOs are either inputs or outputs on this chip. To "disable" input
        // would cause this pin to start driving, which is likely undesired, so
        // this function is a no-op.
        self.configuration()
    }

    fn configuration(&self) -> hil::gpio::Configuration {
        let dir = self.gpio_registers.pin_cnf[self.pin as usize].read_as_enum(PinConfig::DIR);
        let connected =
            self.gpio_registers.pin_cnf[self.pin as usize].read_as_enum(PinConfig::INPUT);
        match (dir, connected) {
            (Some(PinConfig::DIR::Value::Input), Some(PinConfig::INPUT::Value::Connect)) => {
                hil::gpio::Configuration::Input
            }
            (Some(PinConfig::DIR::Value::Input), Some(PinConfig::INPUT::Value::Disconnect)) => {
                hil::gpio::Configuration::LowPower
            }
            (Some(PinConfig::DIR::Value::Output), _) => hil::gpio::Configuration::Output,
            _ => hil::gpio::Configuration::Other,
        }
    }

    fn deactivate_to_low_power(&self) {
        self.gpio_registers.pin_cnf[self.pin as usize].write(
            PinConfig::DIR::Input + PinConfig::INPUT::Disconnect + PinConfig::PULL::Disabled,
        );
    }
}

impl hil::gpio::Input for GPIOPin<'_> {
    fn read(&self) -> bool {
        self.gpio_registers.in_.get() & (1 << self.pin) != 0
    }
}

impl hil::gpio::Output for GPIOPin<'_> {
    fn set(&self) {
        self.gpio_registers.outset.set(1 << self.pin);
    }

    fn clear(&self) {
        self.gpio_registers.outclr.set(1 << self.pin);
    }

    fn toggle(&self) -> bool {
        let result = (1 << self.pin) ^ self.gpio_registers.out.get();
        self.gpio_registers.out.set(result);
        result & (1 << self.pin) != 0
    }
}

impl<'a> hil::gpio::Interrupt<'a> for GPIOPin<'a> {
    fn set_client(&self, client: &'a dyn hil::gpio::Client) {
        self.client.set(client);
    }

    fn is_pending(&self) -> bool {
        if let Some(channel) = self.allocated_channel.get() {
            let ev = &self.gpiote_registers.event_in[channel];
            ev.any_matching_bits_set(EventsIn::EVENT::Ready)
        } else {
            false
        }
    }

    fn enable_interrupts(&self, mode: hil::gpio::InterruptEdge) {
        let channel = if let Some(chan) = self.allocated_channel.get() {
            // We only support one interrupt mode per pin, despite the
            // hardware supporting multiple. This is to comply with
            // expectations in other Tock components, such as the button
            // driver which re-registers interrupts for a restarted app,
            // assuming the old ones to be overwritten.
            chan
        } else if let Ok(chan) = self.allocate_channel() {
            // Don't have a channel yet, got a new one:
            chan
        } else {
            debug!("No available GPIOTE interrupt channels");
            return;
        };

        // Remember that we have allocated this channel for this pin:
        self.allocated_channel.set(channel);

        let polarity = match mode {
            hil::gpio::InterruptEdge::EitherEdge => Config::POLARITY::Toggle,
            hil::gpio::InterruptEdge::RisingEdge => Config::POLARITY::LoToHi,
            hil::gpio::InterruptEdge::FallingEdge => Config::POLARITY::HiToLo,
        };
        let pin: u32 = (GPIO_PER_PORT as u32 * self.port as u32) + self.pin as u32;
        self.gpiote_registers.config[channel]
            .write(Config::MODE::Event + Config::PSEL.val(pin) + polarity);
        self.gpiote_registers.intenset.set(1 << channel);
    }

    fn disable_interrupts(&self) {
        if let Some(channel) = self.allocated_channel.get() {
            self.gpiote_registers.config[channel]
                .write(Config::MODE::CLEAR + Config::PSEL::CLEAR + Config::POLARITY::CLEAR);
            self.gpiote_registers.intenclr.set(1 << channel);
            self.allocated_channel.clear();
        }
    }
}

impl GPIOPin<'_> {
    /// Allocate a GPIOTE channel
    /// If the channel couldn't be allocated return error instead
    fn allocate_channel(&self) -> Result<usize, ()> {
        for (i, ch) in self.gpiote_registers.config.iter().enumerate() {
            if ch.matches_all(Config::MODE::Disabled) {
                return Ok(i);
            }
        }
        Err(())
    }

    fn handle_interrupt(&self) {
        self.client.map(|client| {
            client.fired();
        });
    }
}

pub struct Port<'a, const N: usize> {
    pub pins: [GPIOPin<'a>; N],
}

impl<'a, const N: usize> Index<Pin> for Port<'a, N> {
    type Output = GPIOPin<'a>;

    fn index(&self, index: Pin) -> &GPIOPin<'a> {
        &self.pins[index as usize]
    }
}

impl<'a, const N: usize> IndexMut<Pin> for Port<'a, N> {
    fn index_mut(&mut self, index: Pin) -> &mut GPIOPin<'a> {
        &mut self.pins[index as usize]
    }
}

impl<'a, const N: usize> Port<'a, N> {
    pub const fn new(pins: [GPIOPin<'a>; N]) -> Self {
        Self { pins }
    }

    /// GPIOTE interrupt: check each GPIOTE channel, if any has
    /// fired then trigger its corresponding pin's interrupt handler.
    pub fn handle_interrupt(&self) {
        // do this just to get a pointer the memory map
        // doesn't matter which pin is used because it is the same
        let pin_registers = self.pins[0].gpiote_registers;

        for (i, ev) in pin_registers.event_in.iter().enumerate() {
            if ev.any_matching_bits_set(EventsIn::EVENT::Ready) {
                ev.write(EventsIn::EVENT::NotReady);
                // Get pin number for the event and `trigger` an interrupt manually on that pin
                let pin = pin_registers.config[i].read(Config::PSEL) as usize;
                self.pins[pin].handle_interrupt();
            }
        }
    }
}
