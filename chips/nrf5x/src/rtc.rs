//! RTC driver, nRF5X-family

use core::cell::Cell;
use core::mem;
use kernel::hil::Controller;
use kernel::hil::time::{self, Alarm, Freq32KHz, Time};
use nvic;
use peripheral_interrupts::NvicIdx;
use peripheral_registers::{RTC1_BASE, RTC1};

fn rtc1() -> &'static RTC1 {
    unsafe { mem::transmute(RTC1_BASE as usize) }
}

pub struct Rtc {
    callback: Cell<Option<&'static time::Client>>,
}

pub static mut RTC: Rtc = Rtc { callback: Cell::new(None) };

impl Controller for Rtc {
    type Config = &'static time::Client;

    fn configure(&self, client: &'static time::Client) {
        self.callback.set(Some(client));

        // FIXME: what to do here?
        // self.start();
        // Set counter incrementing frequency to 16KHz
        // rtc1().prescaler.set(1);
    }
}

const COMPARE0_EVENT: u32 = 1 << 16;

impl Rtc {
    pub fn start(&self) {
        // This function takes a nontrivial amount of time
        // So it should only be called during initialization, not each tick
        rtc1().prescaler.set(0);
        rtc1().tasks_start.set(1);
        self.enable_interrupts();
    }

    pub fn disable_interrupts(&self) {
        nvic::disable(NvicIdx::RTC1);
    }

    pub fn enable_interrupts(&self) {
        nvic::enable(NvicIdx::RTC1);
    }

    pub fn stop(&self) {
        self.disable_interrupts();
        rtc1().cc[0].set(0);
        rtc1().tasks_stop.set(1);
    }

    fn is_running(&self) -> bool {
        rtc1().evten.get() & (COMPARE0_EVENT) == (COMPARE0_EVENT)
    }

    pub fn handle_interrupt(&self) {
        rtc1().events_compare[0].set(0);
        rtc1().intenclr.set(COMPARE0_EVENT);
        self.callback.get().map(|cb| { cb.fired(); });
    }

    pub fn set_client(&self, client: &'static time::Client) {
        self.callback.set(Some(client));
    }
}

impl Time for Rtc {
    type Frequency = Freq32KHz;

    fn disable(&self) {
        rtc1().intenclr.set(COMPARE0_EVENT);
    }

    fn is_armed(&self) -> bool {
        self.is_running()
    }
}

impl Alarm for Rtc {
    fn now(&self) -> u32 {
        rtc1().counter.get()
    }

    fn set_alarm(&self, tics: u32) {
        // Similarly to the disable function, here we don't restart the timer
        // Instead, we just listen for it again
        rtc1().cc[0].set(tics);
        rtc1().intenset.set(COMPARE0_EVENT);
    }

    fn get_alarm(&self) -> u32 {
        rtc1().cc[0].get()
    }
}
