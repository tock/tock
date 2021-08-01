use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {

    WatchdogRegisters {
        /// Watchdog control
        /// The rst_wdsel register determines which subsystems are reset when th
        /// The watchdog can be triggered in software.
        (0x000 => ctrl: ReadWrite<u32, CTRL::Register>),
        /// Load the watchdog timer. The maximum setting is 0xffffff which corresponds to 0x
        (0x004 => load: ReadWrite<u32>),
        /// Logs the reason for the last reset. Both bits are zero for the case of a hardwar
        (0x008 => reason: ReadWrite<u32, REASON::Register>),
        /// Scratch register. Information persists through soft reset of the chip.
        (0x00C => scratch0: ReadWrite<u32, SCRATCH0::Register>),
        /// Scratch register. Information persists through soft reset of the chip.
        (0x010 => scratch1: ReadWrite<u32, SCRATCH1::Register>),
        /// Scratch register. Information persists through soft reset of the chip.
        (0x014 => scratch2: ReadWrite<u32, SCRATCH2::Register>),
        /// Scratch register. Information persists through soft reset of the chip.
        (0x018 => scratch3: ReadWrite<u32, SCRATCH3::Register>),
        /// Scratch register. Information persists through soft reset of the chip.
        (0x01C => scratch4: ReadWrite<u32, SCRATCH4::Register>),
        /// Scratch register. Information persists through soft reset of the chip.
        (0x020 => scratch5: ReadWrite<u32, SCRATCH5::Register>),
        /// Scratch register. Information persists through soft reset of the chip.
        (0x024 => scratch6: ReadWrite<u32, SCRATCH6::Register>),
        /// Scratch register. Information persists through soft reset of the chip.
        (0x028 => scratch7: ReadWrite<u32, SCRATCH7::Register>),
        /// Controls the tick generator
        (0x02C => tick: ReadWrite<u32, TICK::Register>),
        (0x030 => @END),
    }
}
register_bitfields![u32,
    CTRL [
        /// Trigger a watchdog reset
        TRIGGER OFFSET(31) NUMBITS(1) [],
        /// When not enabled the watchdog timer is paused
        ENABLE OFFSET(30) NUMBITS(1) [],
        /// Pause the watchdog timer when processor 1 is in debug mode
        PAUSE_DBG1 OFFSET(26) NUMBITS(1) [],
        /// Pause the watchdog timer when processor 0 is in debug mode
        PAUSE_DBG0 OFFSET(25) NUMBITS(1) [],
        /// Pause the watchdog timer when JTAG is accessing the bus fabric
        PAUSE_JTAG OFFSET(24) NUMBITS(1) [],
        /// Indicates the number of ticks / 2 (see errata RP2040-E1) before a watchdog reset
        TIME OFFSET(0) NUMBITS(24) []
    ],
    LOAD [

        LOAD OFFSET(0) NUMBITS(24) []
    ],
    REASON [

        FORCE OFFSET(1) NUMBITS(1) [],

        TIMER OFFSET(0) NUMBITS(1) []
    ],
    SCRATCH0 [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    SCRATCH1 [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    SCRATCH2 [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    SCRATCH3 [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    SCRATCH4 [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    SCRATCH5 [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    SCRATCH6 [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    SCRATCH7 [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    TICK [
        /// Count down timer: the remaining number clk_tick cycles before the next tick is g
        COUNT OFFSET(11) NUMBITS(9) [],
        /// Is the tick generator running?
        RUNNING OFFSET(10) NUMBITS(1) [],
        /// start / stop tick generation
        ENABLE OFFSET(9) NUMBITS(1) [],
        /// Total number of clk_tick cycles before the next tick.
        CYCLES OFFSET(0) NUMBITS(9) []
    ]
];
const WATCHDOG_BASE: StaticRef<WatchdogRegisters> =
    unsafe { StaticRef::new(0x40058000 as *const WatchdogRegisters) };

pub struct Watchdog {
    registers: StaticRef<WatchdogRegisters>,
}

impl Watchdog {
    pub const fn new() -> Watchdog {
        Watchdog {
            registers: WATCHDOG_BASE,
        }
    }

    pub fn start_tick(&self, cycles_in_mhz: u32) {
        self.registers
            .tick
            .modify(TICK::CYCLES.val(cycles_in_mhz) + TICK::ENABLE::SET);
    }
}
