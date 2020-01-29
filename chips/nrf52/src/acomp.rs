//! Analog Comparator Peripheral Driver, for nrf52
//!
//! Partially based on sam4l implementation of an analog comparator.
//!
//! The comparator (COMP) compares an input voltage (VIN+) against a second input voltage (VIN-). VIN+ can
//! be derived from an analog input pin (AIN0-AIN7). VIN- can be derived from multiple sources depending on
//! the operation mode of the comparator.
//!
//! Main features of the comparator are:
//! - Input range from 0 V to VDD
//! - Single-ended mode
//!     - Fully flexible hysteresis using a 64-level reference ladder
//! - Differential mode
//!     - Configurable 50 mV hysteresis
//! - Reference inputs (VREF):
//!     - VDD
//!     - External reference from AIN0 to AIN7 (between 0 V and VDD)
//!     - Internal references 1.2 V, 1.8 V and 2.4 V
//! - Three speed/power consumption modes: low-power, normal and high-speed
//! - Single-pin capacitive sensor support
//! - Event generation on output changes

use kernel::common::cells::OptionalCell;
use kernel::common::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;
use kernel::hil::analog_comparator;
use kernel::ReturnCode;

/// The nrf52840 only has one analog comparator, so it does need channels
/// However, the HIL was designed to support having multiple comparators, each
/// one with a separate channel. So we create a dummy channel with only
/// one possible value to represent this.
/// Code for channels is based on Sam4l implementation
pub struct Channel {
    _chan_num: u32,
}

/// Only one channel
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum ChannelNumber {
    AC0 = 0x00,
}

/// Initialization of an AC channel.
impl Channel {
    /// Create a new AC channel.
    ///
    /// - `channel`: Channel enum representing the channel number
    const fn new(channel: ChannelNumber) -> Channel {
        Channel {
            _chan_num: (channel as u32) & 0x0F,
        }
    }
}

/// Uses only comparator, with VIN+=AIN5 and VIN-=AIN0
pub static mut CHANNEL_AC0: Channel = Channel::new(ChannelNumber::AC0);

register_structs! {
    CompRegisters {
        /// TASK REGISTERS
        /// Trigger task by writing 1
        /// start comparator. Needed before comparator does anything.
        /// After start, triggers events_ready
        (0x000 => tasks_start: WriteOnly<u32>),
        /// stop comparator
        (0x004 => tasks_stop: WriteOnly<u32>),
        /// sample comparator value
        /// This triggers an update to RESULT register
        /// This task doesn't do anything if comparator hasn't
        /// been started yet.
        (0x008 => tasks_sample: WriteOnly<u32>),
        (0x00c => _reserved0),

        /// EVENT REGISTERS
        /// EVENTS: Can clear by writing 0, occured by 1
        /// COMP is ready to go after start task
        (0x100 => events_ready: ReadWrite<u32>),
        /// Cross from high to low
        (0x104 => events_down: ReadWrite<u32>),
        /// cross from low to high
        (0x108 => events_up: ReadWrite<u32>),
        /// either events_up or events_down
        (0x10c => events_cross: ReadWrite<u32>),
        (0x110 => _reserved1),

        /// Used to link tasks to events in hardware itself
        /// Pretty much unused.
        (0x200 => shorts: ReadWrite<u32>),
        (0x204 => _reserved2),

        /// Used to enable and disable interrupts
        (0x300 => inten: ReadWrite<u32, InterruptEnable::Register>),
        /// An alternate way to enable and disable interrupts
        (0x304 => intenset: ReadWrite<u32>),
        (0x308 => intenclr: ReadWrite<u32>),
        (0x30c => _reserved3),

        /// holds result after sampling comparison
        (0x400 => result: ReadOnly<u32, ComparisonResult::Register>),
        (0x404 => _reserved4),

        /// Write to enable comparator. Do this after you've set all other settings
        (0x500 => enable: ReadWrite<u32, Enable::Register>),
        /// select VIN+
        (0x504 => psel: ReadWrite<u32, PinSelect::Register>),
        /// choose where you get VIN- from
        (0x508 => refsel: ReadWrite<u32, ReferenceSelect::Register>),
        /// choose which pin to use from VIN-
        (0x50c => extrefsel: ReadWrite<u32, ExternalRefSelect::Register>),
        (0x510 => _reserved5),

        /// Hysteresis configuration (single-ended mode)
        (0x530 => th: ReadWrite<u32>),
        /// Choose between single ended and differential, and also speed/power
        (0x534 => mode: ReadWrite<u32, Mode::Register>),
        /// Hysteresis configuration (differential mode)
        (0x538 => hyst: ReadWrite<u32, Hysteresis::Register>),
        /// nrf52832 has one more register that was not included here because it's not used
        /// and it doesn't exist on nrf52840
        (0x53c => @END),
    }
}

register_bitfields! [
    u32, 
    InterruptEnable [
        /// enable / disable interrupts for each event
        READY OFFSET(0) NUMBITS(1) [],
        DOWN OFFSET(1) NUMBITS(1) [],
        UP OFFSET(2) NUMBITS(1) [],
        CROSS OFFSET(3) NUMBITS(1) []
    ],
    ComparisonResult [
        RESULT OFFSET(1) NUMBITS(1) [
            /// VIN+ < VIN- 
            Below = 0,
            /// VIN+ > VIN-
            Above = 1
        ]
    ],
    Enable [
        ENABLE OFFSET(0) NUMBITS(2) [
            Disabled = 0,
            Enabled = 2
        ]
    ],
    /// Select VIN+ input pin
    PinSelect [
        PinSelect OFFSET(0) NUMBITS(3) [
            AnalogInput0 = 0,
            AnalogInput1 = 1,
            AnalogInput2 = 2,
            AnalogInput3 = 3,
            AnalogInput4 = 4,
            AnalogInput5 = 5,
            AnalogInput6 = 6,
            AnalogInput7 = 7
        ]
    ],
    /// Select reference source if in Single-Ended mode
    ReferenceSelect [
        ReferenceSelect OFFSET(0) NUMBITS(3) [
            /// Reference voltage = 1.2V VDD >= 1.7
            Internal1V2 = 0,
            /// VREF = 1.8V (VDD >= VREF + .2V)
            Internal1V8 = 1,
            /// VREF = 2.4V (VDD >= VREF + .2V)
            Internal2V4 = 2,
            /// VREF = VDD
            VDD = 4,
            /// Select another pin as a reference
            /// (VDD >= VREF >= AREFMIN)
            AnalogReference = 5
        ]
    ],
    /// If in diff mode, or single-ended mode with an analog reference,
    /// use the pin specified here for VIN-
    ExternalRefSelect[
        ExternalRefSelect OFFSET(0) NUMBITS(3) [
            AnalogRef0 = 0,
            AnalogRef1 = 1,
            /// The last six values are only valid on nrf52840, not nrf52832
            AnalogRef2 = 2,
            AnalogRef3 = 3,
            AnalogRef4 = 4,
            AnalogRef5 = 5,
            AnalogRef6 = 6,
            AnalogRef7 = 7
        ]
    ],
    Mode[
        /// Controls power usage and correspondingly speed of comparator
        SpeedAndPower OFFSET(0) NUMBITS(3) [
            Low = 0,
            Normal = 1,
            High = 2
        ], 
        OperatingMode OFFSET(8) NUMBITS(1) [
            SingleEnded = 0,
            Differential = 1
        ]
    ],
    /// HYST register for hysteresis in diff mode
    /// Turns on 50mV hysteresis
    Hysteresis [
        Hysteresis OFFSET(0) NUMBITS(1)[]
    ]
];

pub struct Comparator {
    registers: StaticRef<CompRegisters>,
    client: OptionalCell<&'static dyn analog_comparator::Client>,
}

impl Comparator {
    const fn new(registers: StaticRef<CompRegisters>) -> Comparator {
        Comparator {
            registers: registers,
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'static dyn analog_comparator::Client) {
        self.client.set(client);
    }

    /// Enables comparator
    /// Uses differential mode, with no hysteresis, and normal speed and power
    /// VIN+ = AIN5 and VIN- = AIN0
    fn enable(&self) {
        let regs = &*self.registers;
        // Checks if it's already enabled
        // Assumes no one else is writing to comp regs directly
        if regs.enable.matches_any(Enable::ENABLE::Enabled) {
            return;
        }

        // Set mode to Differential
        // Differential and single ended are pretty much the same,
        // except single-ended gives more options for input and
        // uses a ref ladder for hysteresis instead of a set voltage
        regs.mode
            .write(Mode::OperatingMode::Differential + Mode::SpeedAndPower::Normal);
        // VIN+ = Pin 0
        regs.psel.write(PinSelect::PinSelect::AnalogInput5);
        // VIN- = Pin 1
        regs.extrefsel
            .write(ExternalRefSelect::ExternalRefSelect::AnalogRef0);
        // Disable hysteresis
        regs.hyst.write(Hysteresis::Hysteresis::CLEAR);

        regs.enable.write(Enable::ENABLE::Enabled);
        // start comparator
        regs.events_ready.set(0);
        regs.tasks_start.set(1);
        // wait for comparator to be ready
        // delay is on order of 3 microseconds so spin wait is OK
        while regs.events_ready.get() == 0 {}
    }

    fn disable(&self) {
        let regs = &*self.registers;
        // stop comparator
        regs.tasks_stop.set(1);
        // completely turn comparator off
        regs.enable.write(Enable::ENABLE::Disabled);
    }

    /// Handles upward crossing events (when VIN+ becomes greater than VIN-)
    pub fn handle_interrupt(&self) {
        // HIL only cares about upward crossing interrupts
        let regs = &*self.registers;
        // VIN+ crossed VIN-
        if regs.events_up.get() == 1 {
            // Clear event
            regs.events_up.set(0);
            self.client.map(|client| {
                // Only one channel (0)
                client.fired(0);
            });
        }
    }
}

impl analog_comparator::AnalogComparator for Comparator {
    type Channel = Channel;

    /// Starts comparison on only channel
    /// This enables comparator and interrupts
    fn start_comparing(&self, _: &Self::Channel) -> ReturnCode {
        let regs = &*self.registers;
        self.enable();

        // Enable only up interrupt (If VIN+ crosses VIN-)
        regs.inten.write(InterruptEnable::UP::SET);

        ReturnCode::SUCCESS
    }

    /// Stops comparing and disables comparator
    fn stop_comparing(&self, _: &Self::Channel) -> ReturnCode {
        let regs = &*self.registers;
        // Disables interrupts
        regs.inten.set(0);
        // Stops comparison
        regs.tasks_stop.set(1);

        self.disable();
        ReturnCode::SUCCESS
    }

    /// Performs a single comparison between VIN+ and VIN-
    /// Returns true if vin+ > vin-
    /// Enables comparator if not enabled, to disable call stop comparing
    fn comparison(&self, _: &Self::Channel) -> bool {
        let regs = &*self.registers;
        self.enable();

        // Signals to update Result register
        regs.tasks_sample.set(1);

        // Returns 1 (true) if vin+ > vin-
        regs.result.get() == 1
    }
}

const ACOMP_BASE: StaticRef<CompRegisters> =
    unsafe { StaticRef::new(0x40013000 as *const CompRegisters) };

pub static mut ACOMP: Comparator = Comparator::new(ACOMP_BASE);
