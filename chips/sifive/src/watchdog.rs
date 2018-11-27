//! Watchdog

use kernel::common::registers::{ReadWrite, WriteOnly};
use kernel::common::StaticRef;

#[repr(C)]
pub struct WatchdogRegisters {
    /// Watchdog Configuration Register
    wdogcfg: ReadWrite<u32, cfg::Register>,
    _reserved0: [u8; 4],
    /// Watchdog Counter Register
    wdogcount: ReadWrite<u32>,
    _reserved1: [u8; 4],
    /// Watchdog Scaled Counter Register
    wdogs: ReadWrite<u32>,
    _reserved2: [u8; 4],
    /// Watchdog Feed Register
    wdogfeed: ReadWrite<u32, feed::Register>,
    /// Watchdog Key Register
    wdogkey: WriteOnly<u32, key::Register>,
    /// Watchdog Compare Register
    wdogcmp: ReadWrite<u32>,
}

register_bitfields![u32,
	cfg [
	    cmpip OFFSET(28) NUMBITS(1) [],
	    encoreawake OFFSET(13) NUMBITS(1) [],
	    enalways OFFSET(12) NUMBITS(1) [],
	    zerocmp OFFSET(9) NUMBITS(1) [],
	    rsten OFFSET(8) NUMBITS(1) [],
	    scale OFFSET(0) NUMBITS(4) []
	],
	key [
		key OFFSET(0) NUMBITS(32) []
	],
	feed [
		feed OFFSET(0) NUMBITS(32) []
	]
];

pub struct Watchdog {
    registers: StaticRef<WatchdogRegisters>,
}

impl Watchdog {
    pub const fn new(base: StaticRef<WatchdogRegisters>) -> Watchdog {
        Watchdog { registers: base }
    }

    fn unlock(&self) {
        let regs = &*self.registers;
        regs.wdogkey.write(key::key.val(0x51F15E));
    }

    fn feed(&self) {
        let regs = &*self.registers;

        self.unlock();
        regs.wdogfeed.write(feed::feed.val(0xD09F00D));
    }

    pub fn disable(&self) {
        let regs = &*self.registers;

        self.unlock();
        regs.wdogcfg.write(
            cfg::scale.val(0)
                + cfg::rsten::CLEAR
                + cfg::zerocmp::CLEAR
                + cfg::enalways::CLEAR
                + cfg::encoreawake::CLEAR,
        );
        self.feed();
    }
}
