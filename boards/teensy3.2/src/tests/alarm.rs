#![allow(unused)]

use mk20::{gpio, clock, pit};
use kernel::hil::time::{Time, Alarm, Frequency, Client};
use tests::blink;

static mut INTERVAL: u32 = 18_000_000;
static mut UP: bool = false;

static mut LAST_TIME: u32 = 0;

struct LedClient;
impl Client for LedClient {
    fn fired(&self) {
        unsafe {
            let now = pit::PIT.now();
            let gap = now - LAST_TIME;
            let wasted = gap - INTERVAL;
            blink::led_toggle();
            if INTERVAL > 18_000_000 || INTERVAL < 4_000_000 {
                UP = !UP;
            }
            INTERVAL = if UP {INTERVAL + 1_000_000} else { INTERVAL - 1_000_000 };
            LAST_TIME = now;
            pit::PIT.set_alarm(INTERVAL);
            println!("Interval: {}, Time: {}, Gap: {}, Overhead: {}", INTERVAL, now, gap, wasted);
        }
    }
}

static LED: LedClient = LedClient;

pub fn alarm_test() {
    unsafe {
        pit::PIT.set_client(&LED);
        pit::PIT.set_alarm(pit::PitFrequency::frequency() / 2);
    }
}

struct LoopClient {
    callback: Option<fn()>
}
static mut LOOP: LoopClient = LoopClient { callback: None };

impl Client for LoopClient {
    fn fired(&self) {
        unsafe {
            self.callback.map(|cb| cb() );
            pit::PIT.set_alarm(pit::PitFrequency::frequency() / 2);
        }
    }
}

pub fn loop_500ms(client: fn()) {
    unsafe {
        LOOP.callback = Some(client);
        pit::PIT.set_client(&LOOP);
        pit::PIT.set_alarm(pit::PitFrequency::frequency() / 2);
    }
}
