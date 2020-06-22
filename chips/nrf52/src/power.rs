//! Power management

use kernel::common::cells::OptionalCell;
use kernel::common::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;

const POWER_BASE: StaticRef<PowerRegisters> =
    unsafe { StaticRef::new(0x40000000 as *const PowerRegisters) };

// Note: only the nrf52833+ have 9 banks, but we create all of them to avoid
// gating this code by a feature.
const NUM_RAM_BANKS: usize = 9;

register_structs! {
    PowerRegisters {
        (0x000 => _reserved0),
        /// Enable Constant Latency mode
        (0x078 => task_constlat: WriteOnly<u32, Task::Register>),
        /// Enable Low-power mode (variable latency)
        (0x07C => task_lowpwr: WriteOnly<u32, Task::Register>),
        (0x080 => _reserved1),
        /// Power failure warning
        (0x108 => event_pofwarn: ReadWrite<u32, Event::Register>),
        (0x10C => _reserved2),
        /// CPU entered WFI/WFE sleep
        (0x114 => event_sleepenter: ReadWrite<u32, Event::Register>),
        /// CPU exited WFI/WFE sleep
        (0x118 => event_sleepexit: ReadWrite<u32, Event::Register>),
        /// Voltage supply detected on VBUS
        (0x11C => event_usbdetected: ReadWrite<u32, Event::Register>),
        /// Voltage supply removed from VBUS
        (0x120 => event_usbremoved: ReadWrite<u32, Event::Register>),
        /// USB 3.3V supply ready
        (0x124 => event_usbpwrrdy: ReadWrite<u32, Event::Register>),
        (0x128 => _reserved3),
        /// Enable interrupt
        (0x304 => intenset: ReadWrite<u32, Interrupt::Register>),
        /// Disable interrupt
        (0x308 => intenclr: ReadWrite<u32, Interrupt::Register>),
        (0x30C => _reserved4),
        /// Reset reason
        (0x400 => resetreas: ReadWrite<u32, ResetReason::Register>),
        (0x404 => _reserved5),
        /// USB supply status
        (0x438 => usbregstatus: ReadOnly<u32, UsbRegStatus::Register>),
        (0x43C => _reserved6),
        /// System OFF register
        (0x500 => systemoff: WriteOnly<u32, Task::Register>),
        (0x504 => _reserved7),
        /// Power failure comparator configuration
        (0x510 => pofcon: ReadWrite<u32, PowerFailure::Register>),
        (0x514 => _reserved8),
        /// General purpose retention register
        (0x51C => gpregret: ReadWrite<u32, Byte::Register>),
        /// General purpose retention register
        (0x520 => gpregret2: ReadWrite<u32, Byte::Register>),
        (0x524 => _reserved9),
        /// Enable DC/DC converter for REG1 stage
        (0x578 => dcdcen: ReadWrite<u32, Task::Register>),
        (0x57C => _reserved10),
        /// Enable DC/DC converter for REG0 stage
        (0x580 => dcdcen0: ReadWrite<u32, Task::Register>),
        (0x584 => _reserved11),
        /// Main supply status
        (0x640 => mainregstatus: ReadOnly<u32, MainSupply::Register>),
        (0x644 => _reserved12),
        /// RAMx power control registers
        /// - Address: 0x900 - 0x980 (<= nRF52832)
        /// - Address: 0x900 - 0x990 (>= nRF52833)
        (0x900 => ram: [RamPowerRegisters; NUM_RAM_BANKS]),
        (0x990 => @END),
    },

    RamPowerRegisters {
        /// RAMn power control register.
        /// The RAM size will vary depending on product variant, and the
        /// RAMn register will only be present if the corresponding RAM AHB
        /// slave is present on the device.
        (0x000 => power: ReadWrite<u32, RamPower::Register>),
        /// RAMn power control set register
        (0x004 => powerset: WriteOnly<u32, RamPower::Register>),
        /// RAMn power control clear register
        (0x008 => powerclr: WriteOnly<u32, RamPower::Register>),
        (0x00C => _reserved),
        (0x010 => @END),
    }
}

register_bitfields! [u32,
    /// Start task
    Task [
        ENABLE OFFSET(0) NUMBITS(1)
    ],

    /// Read event
    Event [
        READY OFFSET(0) NUMBITS(1)
    ],

    /// Power management Interrupts
    Interrupt [
        POFWARN OFFSET(2) NUMBITS(1),
        SLEEPENTER OFFSET(5) NUMBITS(1),
        SLEEPEXIT OFFSET(6) NUMBITS(1),
        USBDETECTED OFFSET(7) NUMBITS(1),
        USBREMOVED OFFSET(8) NUMBITS(1),
        USBPWRRDY OFFSET(9) NUMBITS(1)
    ],

    ResetReason [
        RESETPIN OFFSET(0) NUMBITS(1) [
            Detected = 1
        ],
        DOG OFFSET(1) NUMBITS(1) [
            Detected = 1
        ],
        SREQ OFFSET(2) NUMBITS(1) [
            Detected = 1
        ],
        LOCKUP OFFSET(3) NUMBITS(1) [
            Detected = 1
        ],
        OFF OFFSET(16) NUMBITS(1) [
            Detected = 1
        ],
        LPCOMP OFFSET(17) NUMBITS(1) [
            Detected = 1
        ],
        DIF OFFSET(18) NUMBITS(1) [
            Detected = 1
        ],
        NFC OFFSET(19) NUMBITS(1) [
            Detected = 1
        ],
        VBUS OFFSET(20) NUMBITS(1) [
            Detected = 1
        ]
    ],

    PowerFailure [
        POF OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        THRESHOLD OFFSET(1) NUMBITS(4) [
            V17 = 4,
            V18 = 5,
            V19 = 6,
            V20 = 7,
            V21 = 8,
            V22 = 9,
            V23 = 10,
            V24 = 11,
            V25 = 12,
            V26 = 13,
            V27 = 14,
            V28 = 15
        ],
        THRESHOLDVDDH OFFSET(8) NUMBITS(4) [
            V27 = 0,
            V28 = 1,
            V29 = 2,
            V30 = 3,
            V31 = 4,
            V32 = 5,
            V33 = 6,
            V34 = 7,
            V35 = 8,
            V36 = 9,
            V37 = 10,
            V38 = 11,
            V39 = 12,
            V40 = 13,
            V41 = 14,
            V42 = 15
        ]
    ],

    Byte [
        VALUE OFFSET(0) NUMBITS(8)
    ],

    UsbRegStatus [
        VBUSDETECT OFFSET(0) NUMBITS(1),
        OUTPUTRDY OFFSET(1) NUMBITS(1)
    ],

    MainSupply [
        MAINREGSTATUS OFFSET(0) NUMBITS(1) [
            Normal = 0,
            High = 1
        ]
    ],

    RamPower [
        S0POWER OFFSET(0) NUMBITS(1),
        S1POWER OFFSET(1) NUMBITS(1),
        S2POWER OFFSET(2) NUMBITS(1),
        S3POWER OFFSET(3) NUMBITS(1),
        S4POWER OFFSET(4) NUMBITS(1),
        S5POWER OFFSET(5) NUMBITS(1),
        S6POWER OFFSET(6) NUMBITS(1),
        S7POWER OFFSET(7) NUMBITS(1),
        S8POWER OFFSET(8) NUMBITS(1),
        S9POWER OFFSET(9) NUMBITS(1),
        S10POWER OFFSET(10) NUMBITS(1),
        S11POWER OFFSET(11) NUMBITS(1),
        S12POWER OFFSET(12) NUMBITS(1),
        S13POWER OFFSET(13) NUMBITS(1),
        S14POWER OFFSET(14) NUMBITS(1),
        S15POWER OFFSET(15) NUMBITS(1),
        S0RETENTION OFFSET(16) NUMBITS(1),
        S1RETENTION OFFSET(17) NUMBITS(1),
        S2RETENTION OFFSET(18) NUMBITS(1),
        S3RETENTION OFFSET(19) NUMBITS(1),
        S4RETENTION OFFSET(20) NUMBITS(1),
        S5RETENTION OFFSET(21) NUMBITS(1),
        S6RETENTION OFFSET(22) NUMBITS(1),
        S7RETENTION OFFSET(23) NUMBITS(1),
        S8RETENTION OFFSET(24) NUMBITS(1),
        S9RETENTION OFFSET(25) NUMBITS(1),
        S10RETENTION OFFSET(26) NUMBITS(1),
        S11RETENTION OFFSET(27) NUMBITS(1),
        S12RETENTION OFFSET(28) NUMBITS(1),
        S13RETENTION OFFSET(29) NUMBITS(1),
        S14RETENTION OFFSET(30) NUMBITS(1),
        S15RETENTION OFFSET(31) NUMBITS(1)
    ]
];

/// The USB state machine needs to be notified of power events (USB detected, USB
/// removed, USB power ready) in order to be initialized and shut down properly.
/// These events come from the power management registers of this module; that's
/// this has a USB client to notify.
pub struct Power<'a> {
    registers: StaticRef<PowerRegisters>,
    /// A client to which to notify USB plug-in/plug-out/power-ready events.
    usb_client: OptionalCell<&'a dyn PowerClient>,
}

pub enum MainVoltage {
    /// Normal voltage mode, when supply voltage is connected to both the VDD and
    /// VDDH pins (so that VDD equals VDDH).
    Normal = 0,
    /// High voltage mode, when supply voltage is only connected to the VDDH pin,
    /// and the VDD pin is not connected to any voltage supply.
    High = 1,
}

pub enum PowerEvent {
    PowerFailure,
    EnterSleep,
    ExitSleep,
    UsbPluggedIn,
    UsbPluggedOut,
    UsbPowerReady,
}

pub trait PowerClient {
    fn handle_power_event(&self, event: PowerEvent);
}

impl<'a> Power<'a> {
    const fn new() -> Self {
        Power {
            registers: POWER_BASE,
            usb_client: OptionalCell::empty(),
        }
    }

    pub fn set_usb_client(&self, client: &'a dyn PowerClient) {
        self.usb_client.set(client);
    }

    pub fn handle_interrupt(&self) {
        self.disable_all_interrupts();

        if self.registers.event_usbdetected.is_set(Event::READY) {
            self.registers.event_usbdetected.write(Event::READY::CLEAR);
            self.usb_client
                .map(|client| client.handle_power_event(PowerEvent::UsbPluggedIn));
        }

        if self.registers.event_usbremoved.is_set(Event::READY) {
            self.registers.event_usbremoved.write(Event::READY::CLEAR);
            self.usb_client
                .map(|client| client.handle_power_event(PowerEvent::UsbPluggedOut));
        }

        if self.registers.event_usbpwrrdy.is_set(Event::READY) {
            self.registers.event_usbpwrrdy.write(Event::READY::CLEAR);
            self.usb_client
                .map(|client| client.handle_power_event(PowerEvent::UsbPowerReady));
        }

        // Clearing unused events
        self.registers.event_pofwarn.write(Event::READY::CLEAR);
        self.registers.event_sleepenter.write(Event::READY::CLEAR);
        self.registers.event_sleepexit.write(Event::READY::CLEAR);

        self.enable_interrupts();
    }

    pub fn enable_interrupts(&self) {
        self.registers.intenset.write(
            Interrupt::USBDETECTED::SET + Interrupt::USBREMOVED::SET + Interrupt::USBPWRRDY::SET,
        );
    }

    pub fn enable_interrupt(&self, intr: u32) {
        self.registers.intenset.set(intr);
    }

    pub fn clear_interrupt(&self, intr: u32) {
        self.registers.intenclr.set(intr);
    }

    pub fn disable_all_interrupts(&self) {
        // disable all possible interrupts
        self.registers.intenclr.set(0xffffffff);
    }

    pub fn get_main_supply_status(&self) -> MainVoltage {
        match self
            .registers
            .mainregstatus
            .read_as_enum(MainSupply::MAINREGSTATUS)
        {
            Some(MainSupply::MAINREGSTATUS::Value::Normal) => MainVoltage::Normal,
            Some(MainSupply::MAINREGSTATUS::Value::High) => MainVoltage::High,
            // This case shouldn't happen as the register only holds 1 bit.
            None => unreachable!(),
        }
    }

    pub fn is_vbus_present(&self) -> bool {
        self.registers.usbregstatus.is_set(UsbRegStatus::VBUSDETECT)
    }

    pub fn is_usb_power_ready(&self) -> bool {
        self.registers.usbregstatus.is_set(UsbRegStatus::OUTPUTRDY)
    }
}

pub static mut POWER: Power<'static> = Power::new();
