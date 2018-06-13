//! GPIO and GPIOTE (task and events), nRF5x-family
//!
//! ### Author
//! * Philip Levis <pal@cs.stanford.edu>
//! * Date: August 18, 2016

use core::{cell::Cell,
           ops::{Index, IndexMut}};
use kernel::{common::regs::{FieldValue, ReadWrite},
             hil};

#[cfg(feature = "nrf51")]
const NUM_GPIOTE: usize = 4;
#[cfg(feature = "nrf52")]
const NUM_GPIOTE: usize = 8;

const GPIOTE_BASE: usize = 0x40006000;
const GPIO_BASE: usize = 0x50000000;

/// The nRF5x doesn't automatically provide GPIO interrupts. Instead, to receive
/// interrupts from a GPIO line, you must allocate a GPIOTE (GPIO Task and
/// Event) channel, and bind the channel to the desired pin. There are 4
/// channels for the nrf51 and 8 channels for the nrf52. This means that
/// requesting an interrupt can fail, if they are all already allocated.
#[repr(C)]
struct GpioteRegisters {
    /// Task for writing to pin specified in CONFIG[n].PSEL.
    /// Action on pin is configured in CONFIG[n].POLARITY
    ///
    /// - Address: 0x000 - 0x010 (nRF51)
    /// - Address: 0x000 - 0x020 (nRF52)
    task_out: [ReadWrite<u32, TasksOut::Register>; NUM_GPIOTE],
    /// Reserved
    // task_set and task_clear are not used on nRF52
    _reserved0: [u8; 0x100 - (0x0 + NUM_GPIOTE * 4)],
    /// Event generated from pin specified in CONFIG[n].PSEL
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
    /// Configuration for OUT[n], SET[n] and CLR[n] tasks and IN[n] event
    ///
    /// - Adress: 0x510 - 0x520 (nRF51)
    /// - Adress: 0x510 - 0x530 (nRF52)
    // Note, only IN[n] and OUT[n] are used in Tock
    config: [ReadWrite<u32, Config::Register>; NUM_GPIOTE],
}

#[repr(C)]
struct GpioRegisters {
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
    #[cfg(feature = "nrf52")]
    /// Latch register indicating what GPIO pins that have met the criteria set in the
    /// PIN_CNF[n].SENSE
    /// - Address: 0x520 - 0x524
    #[cfg(feature = "nrf52")]
    latch: ReadWrite<u32, Latch::Register>,
    /// Select between default DETECT signal behaviour and LDETECT mode
    /// - Address: 0x524 - 0x528
    #[cfg(feature = "nrf52")]
    detect_mode: ReadWrite<u32, DetectMode::Register>,
    #[cfg(feature = "nrf52")]
    /// Reserved
    _reserved2: [u32; 118],
    /// Configuration of GPIO pins
    pin_cnf: [ReadWrite<u32, PinConfig::Register>; 32],
}

/// Gpio
register_bitfields! [u32,
    /// Write GPIO port
    Out [
        /// Pin[n], each bit correspond to a pin 0 to 31
        /// 0 - Low, Pin driver is low
        /// 1 - High, Pin driver is high
        PIN OFFSET(0) NUMBITS(32)
    ],
    /// Set individual bits in GPIO port
    OutSet [
        /// Pin[n], each bit correspond to a pin 0 to 31
        /// 0 - Low
        /// 1 - High
        /// Writing a '1' sets the pin high
        /// Writing a '0' has no effect
        PIN OFFSET(0) NUMBITS(32)
    ],
    /// Clear individual bits in GPIO port
    OutClr [
        /// Pin[n], each bit correspond to a pin 0 to 31
        /// 0 - Low
        /// 1 - High
        /// Writing a '1' sets the pin low
        /// Writing a '0' has no effect
        PIN OFFSET(0) NUMBITS(32)
    ],
    /// Read GPIO port
    In [
        /// Pin[n], each bit correspond to a pin 0 to 31
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
        /// Pin[n], each bit correspond to a pin 0 to 31
        /// 0 - Pin set as input
        /// 1 - Pin set as output
        /// Write: writing a '1' sets pin to output
        /// Writing a '0' has no effect
        PIN OFFSET(0) NUMBITS(32)
    ],
    /// Configure direction of individual GPIO pins as input
    DirClr [
        /// Pin[n], each bit correspond to a pin 0 to 31
        /// 0 - Pin set as input
        /// 1 - Pin set as output
        /// Write: writing a '1' sets pin to input
        /// Writing a '0' has no effect
        PIN OFFSET(0) NUMBITS(32)
    ],
    /// Latch register indicating what GPIO pins that have met the criteria set in the
    /// PIN_CNF[n].SENSE registers
    Latch [
        /// Pin[n], each bit correspond to a pin 0 to 31
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
    /// Pin[n], each bit correspond to a pin 0 to 31
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

/// GpioTe
register_bitfields! [u32,
    /// Task for writing to pin specified in CONFIG[n].PSEL.
    /// Action on pin is configured in CONFIG[n].POLARITY
    TasksOut [
        TASK OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ]
    ],

    /// Event generated from pin specified in CONFIG[n].PSEL
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

    /// Configuration for OUT[n], SET[n] and CLR[n] tasks and IN[n] event
    Config [
        /// Mode
        MODE OFFSET(0) NUMBITS(2) [
            /// Disabled. Pin specified by PSEL will not be acquired by the
            /// GPIOTE module
            Disabled = 0,
            /// The pin specified by PSEL will be configured as an input and the
            /// IN[n] event will be generated if operation specified in POLARITY
            /// occurs on the pin.
            Event = 1,
            ///The GPIO specified by PSEL will be configured as an output and
            /// triggering the SET[n], CLR[n] or OUT[n] task will perform the
            /// operation specified by POLARITY on the pin. When enabled as a
            /// task the GPIOTE module will acquire the pin and the pin can no
            /// longer be written as a regular output pin from the GPIO module.
            Task = 3
        ],
        /// GPIO number associated with SET[n], CLR[n] and OUT[n] tasks
        /// and IN[n] event
        PSEL OFFSET(8) NUMBITS(5) [],
        /// When In task mode: Operation to be performed on output
        /// when OUT[n] task is triggered. When In event mode: Operation
        /// on input that shall trigger IN[n] event
        POLARITY OFFSET(16) NUMBITS(2) [
            /// Task mode: No effect on pin from OUT[n] task. Event mode: no
            /// IN[n] event generated on pin activity
            Disabled = 0,
            /// Task mode: Set pin from OUT[n] task. Event mode: Generate
            /// IN[n] event when rising edge on pin
            LoToHi = 1,
            /// Task mode: Clear pin from OUT[n] task. Event mode: Generate
            /// IN[n] event when falling edge on pin
            HiToLo = 2,
            /// Task mode: Toggle pin from OUT[n]. Event mode: Generate
            /// IN[n] when any change on pin
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

pub struct GPIOPin {
    pin: u8,
    client_data: Cell<usize>,
    client: Cell<Option<&'static hil::gpio::Client>>,
    gpiote_register: *const GpioteRegisters,
    gpio_register: *const GpioRegisters,
}

impl GPIOPin {
    const fn new(pin: u8) -> GPIOPin {
        GPIOPin {
            pin: pin,
            client_data: Cell::new(0),
            client: Cell::new(None),
            gpio_register: GPIO_BASE as *const GpioRegisters,
            gpiote_register: GPIOTE_BASE as *const GpioteRegisters,
        }
    }

    pub fn set_client<C: hil::gpio::Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }

    pub fn write_config(&self, config: FieldValue<u32, PinConfig::Register>) {
        let gpio_regs = unsafe { &*self.gpio_register };
        gpio_regs.pin_cnf[self.pin as usize].write(config);
    }
}

impl hil::gpio::PinCtl for GPIOPin {
    fn set_input_mode(&self, mode: hil::gpio::InputMode) {
        let pin_config = match mode {
            hil::gpio::InputMode::PullUp => PinConfig::PULL::Pullup,
            hil::gpio::InputMode::PullDown => PinConfig::PULL::Pulldown,
            hil::gpio::InputMode::PullNone => PinConfig::PULL::Disabled,
        };
        self.write_config(pin_config);
    }
}

impl hil::gpio::Pin for GPIOPin {
    fn make_output(&self) {
        unsafe { (&*self.gpio_register).dirset.set(1 << self.pin) };
    }

    // Configuration constants stolen from
    // mynewt/hw/mcu/nordic/nrf51xxx/include/mcu/nrf51_bitfields.h
    fn make_input(&self) {
        unsafe { (&*self.gpio_register).dirclr.set(1 << self.pin) };
    }

    // Not clk
    fn disable(&self) {
        hil::gpio::PinCtl::set_input_mode(self, hil::gpio::InputMode::PullNone);
    }

    fn set(&self) {
        unsafe { (&*self.gpio_register).outset.set(1 << self.pin) };
    }

    fn clear(&self) {
        unsafe { (&*self.gpio_register).outclr.set(1 << self.pin) };
    }

    fn toggle(&self) {
        let gpio_regs = unsafe { &*self.gpio_register };
        gpio_regs.out.set((1 << self.pin) ^ gpio_regs.out.get());
    }

    fn read(&self) -> bool {
        let gpio_regs = unsafe { &*self.gpio_register };
        gpio_regs.in_.get() & (1 << self.pin) != 0
    }

    fn enable_interrupt(&self, client_data: usize, mode: hil::gpio::InterruptMode) {
        if let Ok(channel) = self.allocate_channel() {
            self.client_data.set(client_data);
            let polarity = match mode {
                hil::gpio::InterruptMode::EitherEdge => Config::POLARITY::Toggle,
                hil::gpio::InterruptMode::RisingEdge => Config::POLARITY::LoToHi,
                hil::gpio::InterruptMode::FallingEdge => Config::POLARITY::HiToLo,
            };
            let regs = unsafe { &*self.gpiote_register };
            regs.config[channel]
                .write(Config::MODE::Event + Config::PSEL.val(self.pin as u32) + polarity);
            regs.intenset.set(1 << channel);
        } else {
            debug!("No available GPIOTE interrupt channels");
        }
    }

    fn disable_interrupt(&self) {
        if let Ok(channel) = self.find_channel(self.pin) {
            let regs = unsafe { &*self.gpiote_register };
            regs.config[channel]
                .write(Config::MODE::CLEAR + Config::PSEL::CLEAR + Config::POLARITY::CLEAR);
            regs.intenclr.set(1 << channel);
        }
    }
}

impl GPIOPin {
    /// Allocate a GPIOTE channel
    /// If the channel couldn't be allocated return error instead
    fn allocate_channel(&self) -> Result<usize, ()> {
        let regs = unsafe { &*self.gpiote_register };
        for (i, ch) in regs.config.iter().enumerate() {
            if ch.matches_all(Config::MODE::Disabled) {
                return Ok(i);
            }
        }
        Err(())
    }

    /// Return which channel is allocated to a pin,
    /// If the channel is not found return an error instead
    fn find_channel(&self, pin: u8) -> Result<usize, ()> {
        let regs = unsafe { &*self.gpiote_register };
        for (i, ch) in regs.config.iter().enumerate() {
            if ch.matches_all(Config::PSEL.val(pin as u32)) {
                return Ok(i);
            }
        }
        return Err(());
    }

    fn handle_interrupt(&self) {
        self.client.get().map(|client| {
            client.fired(self.client_data.get());
        });
    }
}

pub struct Port {
    pins: [GPIOPin; 32],
}

impl Index<usize> for Port {
    type Output = GPIOPin;

    fn index(&self, index: usize) -> &GPIOPin {
        &self.pins[index]
    }
}

impl IndexMut<usize> for Port {
    fn index_mut(&mut self, index: usize) -> &mut GPIOPin {
        &mut self.pins[index]
    }
}

impl Port {
    /// GPIOTE interrupt: check each GPIOTE channel, if any has
    /// fired then trigger its corresponding pin's interrupt handler.
    pub fn handle_interrupt(&self) {
        // do this just to get a pointer the memory map
        // doesn't matter which pin is used because it is the same
        let regs = unsafe { &*self.pins[0].gpiote_register };

        for (i, ev) in regs.event_in.iter().enumerate() {
            if ev.matches_any(EventsIn::EVENT::Ready) {
                ev.write(EventsIn::EVENT::NotReady);
                // Get pin number for the event and `trigger` an interrupt manually on that pin
                let pin = regs.config[i].read(Config::PSEL) as usize;
                self.pins[pin].handle_interrupt();
            }
        }
    }
}

pub static mut PORT: Port = Port {
    pins: [
        GPIOPin::new(0),
        GPIOPin::new(1),
        GPIOPin::new(2),
        GPIOPin::new(3),
        GPIOPin::new(4),
        GPIOPin::new(5),
        GPIOPin::new(6),
        GPIOPin::new(7),
        GPIOPin::new(8),
        GPIOPin::new(9),
        GPIOPin::new(10),
        GPIOPin::new(11),
        GPIOPin::new(12),
        GPIOPin::new(13),
        GPIOPin::new(14),
        GPIOPin::new(15),
        GPIOPin::new(16),
        GPIOPin::new(17),
        GPIOPin::new(18),
        GPIOPin::new(19),
        GPIOPin::new(20),
        GPIOPin::new(21),
        GPIOPin::new(22),
        GPIOPin::new(23),
        GPIOPin::new(24),
        GPIOPin::new(25),
        GPIOPin::new(26),
        GPIOPin::new(27),
        GPIOPin::new(28),
        GPIOPin::new(29),
        GPIOPin::new(30),
        GPIOPin::new(31),
    ],
};
