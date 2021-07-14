use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    /// Controls the crystal oscillator
    XoscRegisters {
        /// Crystal Oscillator Control
        (0x000 => ctrl: ReadWrite<u32, CTRL::Register>),
        /// Crystal Oscillator Status
        (0x004 => status: ReadWrite<u32, STATUS::Register>),
        /// Crystal Oscillator pause control\n
        /// This is used to save power by pausing the XOSC\n
        /// On power-up this field is initialised to WAKE\n
        /// An invalid write will also select WAKE\n
        /// WARNING: stop the PLLs before selecting dormant mode\n
        /// WARNING: setup the irq before selecting dormant mode
        (0x008 => dormant: ReadWrite<u32, DORMANT::Register>),
        /// Controls the startup delay
        (0x00C => startup: ReadWrite<u32, STARTUP::Register>),
        (0x010 => _reserved0),
        /// A down counter running at the xosc frequency which counts to zero and stops.\n
        /// To start the counter write a non-zero value.\n
        /// Can be used for short software pauses when setting up time sensitive
        (0x01C => count: ReadWrite<u32>),
        (0x020 => @END),
    }
}

register_bitfields![u32,
    CTRL [
        /// On power-up this field is initialised to DISABLE and the chip runs from the ROSC
        /// If the chip has subsequently been programmed to run from the XOS
        /// The 12-bit code is intended to give some protection against acci
        ENABLE OFFSET(12) NUMBITS(12) [
            ENABLE = 0xfab,
            DISABLE = 0xd1e
        ],
        /// Frequency range. This resets to 0xAA0 and cannot be changed.
        FREQ_RANGE OFFSET(0) NUMBITS(12) [

            _1_15MHZ = 0xaa0
        ]
    ],
    STATUS [
        /// Oscillator is running and stable
        STABLE OFFSET(31) NUMBITS(1) [],
        /// An invalid value has been written to CTRL_ENABLE or CTRL_FREQ_RANGE or DORMANT
        BADWRITE OFFSET(24) NUMBITS(1) [],
        /// Oscillator is enabled but not necessarily running and stable, resets to 0
        ENABLED OFFSET(12) NUMBITS(1) [],
        /// The current frequency range setting, always reads 0
        FREQ_RANGE OFFSET(0) NUMBITS(2) [

            _1_15MHZ = 0
        ]
    ],
    DORMANT [
        VALUE OFFSET (0) NUMBITS (32) [
            DORMANT = 0x636f6d61,
            WAKE = 0x77616b65
        ]
    ],
    STARTUP [
        /// Multiplies the startup_delay by 4. This is of little value to the user given tha
        X4 OFFSET(20) NUMBITS(1) [],
        /// in multiples of 256*xtal_period
        DELAY OFFSET(0) NUMBITS(14) []
    ],
    COUNT [

        COUNT OFFSET(0) NUMBITS(8) []
    ]
];

const XOSC_BASE: StaticRef<XoscRegisters> =
    unsafe { StaticRef::new(0x40024000 as *const XoscRegisters) };

pub struct Xosc {
    registers: StaticRef<XoscRegisters>,
}

impl Xosc {
    pub const fn new() -> Self {
        Self {
            registers: XOSC_BASE,
        }
    }

    pub fn init(&self) {
        // there is only one frequency range available
        // RP2040 Manual https://datasheets.raspberrypi.org/rp2040/rp2040-datasheet.pdf section 2.16.7
        self.registers.ctrl.modify(CTRL::FREQ_RANGE::_1_15MHZ);
        let startup_delay = (((12 * 1000000) / 1000) + 128) / 256;
        self.registers
            .startup
            .modify(STARTUP::DELAY.val(startup_delay));
        self.registers.ctrl.modify(CTRL::ENABLE::ENABLE);
        // wait for the oscillator to become stable
        while !self.registers.status.is_set(STATUS::STABLE) {}
    }

    pub fn disable(&self) {
        self.registers.ctrl.modify(CTRL::ENABLE::DISABLE);
    }

    /// disable the oscillator until an interrupt arrives
    pub fn dormant(&self) {
        self.registers.dormant.modify(DORMANT::VALUE::DORMANT);
        // wait for the oscillator to become stable
        while !self.registers.status.is_set(STATUS::STABLE) {}
    }
}
