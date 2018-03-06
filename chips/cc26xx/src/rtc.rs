//! RTC driver, sensortag family

use core::cell::Cell;
use kernel::common::VolatileCell;
use kernel::hil::time::{self, Alarm, Freq32KHz, Time};

#[repr(C)]
pub struct RtcRegisters {
    ctl: VolatileCell<u32>,

    // Event flags
    evflags: VolatileCell<u32>,

    // Integer part
    sec: VolatileCell<u32>,
    // Fractional part (1/32kHz parts of a second)
    subsec: VolatileCell<u32>,

    _subsec_inc: VolatileCell<u32>,
    channel_ctl: VolatileCell<u32>,
    _channel0_cmp: VolatileCell<u32>,
    channel1_cmp: VolatileCell<u32>,
    _channel2_cmp: VolatileCell<u32>,
    _channel2_cmp_inc: VolatileCell<u32>,
    _channel1_capture: VolatileCell<u32>,

    // A read request to the sync register will not return
    // until all outstanding writes have properly propagated to the RTC domain
    sync: VolatileCell<u32>,
}

const RTC_BASE: *mut RtcRegisters = 0x4009_2000 as *mut RtcRegisters;

pub struct Rtc {
    regs: *mut RtcRegisters,
    callback: Cell<Option<&'static time::Client>>,
}

pub static mut RTC: Rtc = Rtc::new();

// const RTC_CTL_RESET: u32 = (1 << 7);
const RTC_CTL_CHANNEL1: u32 = (1 << 17);
const RTC_CTL_ENABLE: u32 = 0x01;

const RTC_EVENT_CHANNEL1: u32 = (1 << 8);
const RTC_CHANNEL1_ENABLE: u32 = (1 << 8);

impl Rtc {
    const fn new() -> Rtc {
        Rtc {
            regs: RTC_BASE,
            callback: Cell::new(None),
        }
    }

    pub fn start(&self) {
        let regs: &RtcRegisters = unsafe { &*self.regs };
        regs.ctl.set(regs.ctl.get() | RTC_CTL_ENABLE);

        regs.sync.get();
    }

    pub fn stop(&self) {
        let regs: &RtcRegisters = unsafe { &*self.regs };
        regs.ctl.set(regs.ctl.get() & !RTC_CTL_ENABLE);
        regs.sync.get();
    }

    fn read_counter(&self) -> u32 {
        let regs: &RtcRegisters = unsafe { &*self.regs };

        /*
            SEC can change during the SUBSEC read, so we need to be certain
            that the SUBSEC we read belong to the correct SEC counterpart.
        */
        let mut current_sec: u32 = 0;
        let mut current_subsec: u32 = 0;
        let mut after_subsec_read: u32 = 1;
        while current_sec != after_subsec_read {
            current_sec = regs.sec.get();
            current_subsec = regs.subsec.get();
            after_subsec_read = regs.sec.get();
        }

        return (current_sec << 16) | (current_subsec >> 16);
    }

    pub fn is_running(&self) -> bool {
        let regs: &RtcRegisters = unsafe { &*self.regs };
        (regs.ctl.get() & RTC_CTL_CHANNEL1) != 0
    }

    pub fn handle_interrupt(&self) {
        let regs: &RtcRegisters = unsafe { &*self.regs };

        // Clear the event flag
        regs.evflags.set(regs.evflags.get() | RTC_EVENT_CHANNEL1);
        regs.ctl.set(regs.ctl.get() & !RTC_CTL_CHANNEL1);
        regs.channel_ctl
            .set(regs.channel_ctl.get() & !RTC_CHANNEL1_ENABLE);
        regs.sync.get();

        self.callback.get().map(|cb| cb.fired());
    }

    pub fn set_client(&self, client: &'static time::Client) {
        self.callback.set(Some(client));
    }
}

impl Time for Rtc {
    type Frequency = Freq32KHz;

    fn disable(&self) {
        let regs: &RtcRegisters = unsafe { &*self.regs };

        regs.ctl.set(regs.ctl.get() & !RTC_CTL_CHANNEL1);
        regs.channel_ctl
            .set(regs.channel_ctl.get() & !RTC_CHANNEL1_ENABLE);
        regs.sync.get();
    }

    fn is_armed(&self) -> bool {
        self.is_running()
    }
}

impl Alarm for Rtc {
    fn now(&self) -> u32 {
        self.read_counter()
    }

    fn set_alarm(&self, tics: u32) {
        let regs: &RtcRegisters = unsafe { &*self.regs };

        regs.ctl.set(regs.ctl.get() | RTC_CTL_CHANNEL1);
        regs.channel1_cmp.set(tics);
        regs.channel_ctl
            .set(regs.channel_ctl.get() | RTC_CHANNEL1_ENABLE);

        regs.sync.get();
    }

    fn get_alarm(&self) -> u32 {
        self.read_counter()
    }
}
