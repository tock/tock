use core::mem;
use core::cell::Cell;
use hil::Controller;
use hil::alarm::{Alarm, AlarmClient, Freq16Khz};
use peripheral_registers::{RTC1_BASE, RTC1};
use peripheral_interrupts::NvicIdx;
use chip;
use nvic;

fn rtc1() -> &'static RTC1 {
    unsafe { mem::transmute(RTC1_BASE as usize) }
}

pub struct Rtc {
    callback: Cell<Option<&'static AlarmClient>>
}

pub static mut RTC : Rtc = Rtc {
    callback: Cell::new(None),
};

impl Controller for Rtc {
    type Config = &'static AlarmClient;

    fn configure(&self, client: &'static AlarmClient) {
        self.callback.set(Some(client));

        // FIXME: what to do here?
        //self.start();
        // Set counter incrementing frequency to 16KHz
        rtc1().prescaler.set(1);
    }
}

// CLEAR, STOP, START and TRIGOVRFLW tasks have up to 46us delay to process.
// See 18.1.8 "TASK and EVENT jitter/delay" on the nRF51 Reference Manual.
//
// wait_task() will delay approximately this time. For a 16 MHz CPU,
// 1us == 16 instructions (assuming each instruction takes one cycle).
#[inline(never)]
fn wait_task() {
    // The inner loop instructions are: 14 NOPs + 1 SUBS + 1 CMP
    unsafe {
        asm!(
            "movs r0, #47\n\
            1:\n\
            nop\n\
            nop\n\
            nop\n\
            nop\n\
            nop\n\
            nop\n\
            nop\n\
            nop\n\
            nop\n\
            nop\n\
            nop\n\
            nop\n\
            nop\n\
            nop\n\
            subs r0, #1\n\
            cmp r0, #0\n\
            bne.n 1b"
            : /* no output */
            : /* no input */
            : "{r0}"
        );
   }
}

impl Rtc {
    fn start(&self) {
        rtc1().evtenset.set(1 << 16);
        rtc1().intenset.set(1 << 16);
        nvic::clear_pending(NvicIdx::RTC1);
        nvic::enable(NvicIdx::RTC1);
        rtc1().tasks_start.set(1);
        wait_task();
    }

    fn stop(&self) {
        nvic::disable(NvicIdx::RTC1);
        rtc1().evtenclr.set(1 << 16);
        rtc1().intenclr.set(1 << 16);
        rtc1().tasks_stop.set(1);
        wait_task();
    }

    fn is_running(&self) -> bool {
        rtc1().evten.get() & (1 << 16) == (1 << 16)
    }

    pub fn handle_interrupt(&self) {
        self.callback.get().map(|cb| {
            cb.fired();
        });
    }
}

impl Alarm for Rtc {
    type Frequency = Freq16Khz;

    fn now(&self) -> u32 {
        rtc1().counter.get()
    }

    fn disable_alarm(&self) {
        self.stop();
    }

    fn set_alarm(&self, tics: u32) {
        rtc1().cc[0].set(tics);
        self.start();
    }

    fn get_alarm(&self) -> u32 {
        rtc1().cc[0].get()
    }

    fn is_armed(&self) -> bool {
       self.is_running()
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn RTC1_Handler() {
    use common::Queue;

    rtc1().events_compare[0].set(0);
    rtc1().events_compare[1].set(0);
    rtc1().events_compare[2].set(0);
    rtc1().events_compare[3].set(0);
    rtc1().events_tick.set(0);
    rtc1().events_ovrflw.set(0);

    nvic::disable(NvicIdx::RTC1);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(NvicIdx::RTC1);
}
