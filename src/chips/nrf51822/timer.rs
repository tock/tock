use core::mem;
use common::VolatileCell;
use common::take_cell::TakeCell;
use peripheral_interrupts::NvicIdx;
use nvic;
use chip;
use hil;
// The nRF51822 timer system operates off of the high frequency clock 
// (HFCLK) and provides three timers from the clock. Timer0 is tied
// to the radio through some hard-coded peripheral linkages (e.g., there
// are dedicated PPI connections between Timer0's compare events and
// radio tasks, its capture tasks and radio events).
// 
// This implementation provides a full-fledged Timer interface to
// timers 0 and 2, and exposes Timer1 as an HIL Alarm, for a Tock 
// timer system. It may be that the Tock timer system should be ultimately
// placed on top of the RTC (from the low frequency clock). It's currently
// implemented this way as a demonstration that it can be and because
// the full RTC/clock interface hasn't been finalized yet.

#[repr(C, packed)]
struct Registers {
    pub task_start:      VolatileCell<u32>,
    pub task_stop:       VolatileCell<u32>,
    pub task_count:      VolatileCell<u32>,
    pub task_clear:      VolatileCell<u32>,
    pub task_shutdown:   VolatileCell<u32>,
    _reserved0:        [VolatileCell<u32>;  11],
    pub task_capture:  [VolatileCell<u32>;   4],  // 0x40
    _reserved1:        [VolatileCell<u32>;  60],  // 0x140
    pub event_compare: [VolatileCell<u32>;   4],
    _reserved2:        [VolatileCell<u32>;  44],  // 0x150 
    pub shorts:          VolatileCell<u32>,       // 0x200 
    _reserved3:        [VolatileCell<u32>;  64],  // 0x204
    pub intenset:        VolatileCell<u32>,       // 0x304 
    pub intenclr:        VolatileCell<u32>,       // 0x308
    _reserved4:        [VolatileCell<u32>; 126],  // 0x30C
    pub mode:            VolatileCell<u32>,       // 0x504
    pub bitmode:         VolatileCell<u32>,       // 0x508
    _reserved5:          VolatileCell<u32>,
    pub prescaler:       VolatileCell<u32>,       // 0x510
    _reserved6:        [VolatileCell<u32>;  11],  // 0x514
    pub cc:            [VolatileCell<u32>;   4],  // 0x540
}

const SIZE:       usize = 0x1000;
const TIMER_BASE: usize = 0x40008000;

#[derive(Copy,Clone)]
pub enum Location {
    TIMER0, TIMER1, TIMER2
}

pub static mut TIMER0 : Timer = Timer {
    which: Location::TIMER0,
    nvic: NvicIdx::TIMER0,
    client: TakeCell::empty()
};

pub static mut ALARM1 : TimerAlarm = TimerAlarm {
    which:  Location::TIMER1,
    nvic:   NvicIdx::TIMER1,
    client: TakeCell::empty(),
};

pub static mut TIMER2 : Timer = Timer {
    which: Location::TIMER2,
    nvic: NvicIdx::TIMER2,
    client: TakeCell::empty()
};

#[allow(non_snake_case)]
fn TIMER(location: Location) -> &'static Registers {
    let ptr = TIMER_BASE + (location as usize) * SIZE;
    unsafe { mem::transmute(ptr) }
}

pub trait CompareClient {
    // Passes a bitmask of which compares/captures fired
    fn compare(&self, bitmask: u8);
}

pub struct Timer {
    which: Location,
    nvic: NvicIdx,
    client: TakeCell<&'static CompareClient>,
}

impl Timer {
    fn timer(&self) -> &'static Registers { TIMER(self.which) }

    pub const fn new(location: Location, nvic: NvicIdx) -> Timer {
        Timer {
            which: location,
            nvic: nvic,
            client: TakeCell::empty(),
        }
    }

    pub fn set_client(&self, client: &'static CompareClient) {
        self.client.replace(client);
    }

    pub fn start(&self) {
        self.timer().task_start.set(1); 
    }
    // Stops the timer and keeps the value
    pub fn stop(&self) {
        self.timer().task_stop.set(1); 
    }
    // Stops the timer and clears the value
    pub fn shutdown(&self) {
        self.timer().task_shutdown.set(1); 
    }
    // Clear the value
    pub fn clear(&self) {
        self.timer().task_clear.set(1); 
    }

    pub fn capture(&self, which: u8) -> u32 {
        match which {
            0 => {
                self.timer().task_capture[0].set(1);
                self.timer().cc[0].get()
            }
            1 => {
                self.timer().task_capture[1].set(1);
                self.timer().cc[1].get()
            }
            2 => {
                self.timer().task_capture[2].set(1);
                self.timer().cc[2].get()
            }
            _ => {
                self.timer().task_capture[3].set(1);
                self.timer().cc[3].get()
            }
        }
    }

    pub fn capture_to(&self, which: u8) {
        let _ = self.capture(which);
    }

    pub fn get_shortcuts(&self) -> u32 {
        self.timer().shorts.get() 
    }
    pub fn set_shortcuts(&self, shortcut: u32) {
        self.timer().shorts.set(shortcut); 
    }

    pub fn get_cc0(&self) -> u32    { self.timer().cc[0].get() }
    pub fn set_cc0(&self, val: u32) { self.timer().cc[0].set(val); }
    pub fn get_cc1(&self) -> u32    { self.timer().cc[1].get() }
    pub fn set_cc1(&self, val: u32) { self.timer().cc[0].set(val); }
    pub fn get_cc2(&self) -> u32    { self.timer().cc[2].get() }
    pub fn set_cc2(&self, val: u32) { self.timer().cc[0].set(val); }
    pub fn get_cc3(&self) -> u32    { self.timer().cc[3].get() }
    pub fn set_cc3(&self, val: u32) { self.timer().cc[0].set(val); }

    pub fn enable_interrupts(&self, interrupts: u32) {
        self.timer().intenset.set(interrupts); 
    }
    pub fn disable_interrupts(&self, interrupts: u32) {
        self.timer().intenclr.set(interrupts); 
    }

    pub fn enable_nvic(&self) {
        nvic::enable(self.nvic);
    }

    pub fn disable_nvic(&self) {
        nvic::disable(self.nvic);
    }

    pub fn set_prescaler(&self, val: u8) {
        // Only bottom 4 bits are valid, so mask them
        // nRF51822 reference manual, page 102
        self.timer().prescaler.set((val & 0xf) as u32); 
    }
    pub fn get_prescaler(&self) -> u8 {
        self.timer().prescaler.get() as u8
    }

    pub fn handle_interrupt(&self) {
        nvic::clear_pending(self.nvic);
        self.client.map(|client| {
            let mut val = 0;
            // For each of 4 possible compare events, if it's happened,
            // clear it and store its bit in val to pass in callback.
            for i in 0..4 {
                if self.timer().event_compare[i].get() != 0 {
                    val = val | 1 << i;
                    self.timer().event_compare[i].set(0);
                    self.disable_interrupts(1 << (i + 16));
                }
            }
            client.compare(val as u8);
        });
    }
}

pub struct TimerAlarm {
    which: Location,
    nvic: NvicIdx,
    client: TakeCell<&'static hil::alarm::AlarmClient>,
}

impl TimerAlarm {
    fn timer(&self) -> &'static Registers { TIMER(self.which) }

    pub const fn new(location: Location, nvic: NvicIdx) -> TimerAlarm {
        TimerAlarm {
            which: location,
            nvic: nvic,
            client: TakeCell::empty(),
        }
    }

    pub fn clear(&self) {
        self.timer().task_clear.set(1);
    }

    pub fn set_client(&self, client: &'static hil::alarm::AlarmClient) {
        self.client.replace(client);
    }

    pub fn start(&self) {
        // Clock is 16MHz, so scale down by 2^10 to 16KHz
        self.timer().prescaler.set(10);
        self.timer().task_start.set(1);
    }

    pub fn stop(&self) {
        self.timer().task_stop.set(1);
    }

    pub fn handle_interrupt(&self) {
        nvic::clear_pending(self.nvic);
        self.disable_interrupts(0b1111 << 16);
        self.timer().event_compare[0].set(0);
        self.timer().event_compare[1].set(0);

        self.client.map(|client| {
            client.fired();
        });
    }

    pub fn enable_interrupts(&self, interrupts: u32) {
        self.timer().intenset.set(interrupts); 
    }

    pub fn disable_interrupts(&self, interrupts: u32) {
        self.timer().intenclr.set(interrupts); 
    }

    pub fn enable_nvic(&self) {
        nvic::enable(self.nvic);
    }

    pub fn disable_nvic(&self) {
        nvic::disable(self.nvic);
    }

    pub fn value(&self) -> u32 {
        self.timer().task_capture[0].set(1);
        self.timer().cc[0].get()
    }
}

impl hil::alarm::Alarm for TimerAlarm {
    type Frequency = hil::alarm::Freq16KHz;

    fn now(&self) -> u32 {
        self.timer().task_capture[0].set(1);
        self.timer().cc[0].get()
    }
    fn set_alarm(&self, tics: u32) {
        self.disable_alarm();
        self.enable_nvic();
        // Enable interrupt on cc1
        self.enable_interrupts(1 << 1);
        self.timer().cc[1].set(tics);
        self.timer().shorts.set(0b10);
    }
    fn disable_alarm(&self) {
        // Disable interrupt on cc1
        self.disable_interrupts(1 << 1);
        self.timer().cc[1].set(0);
    }
    fn is_armed(&self) -> bool {
        self.timer().cc[1].get() != 0
    }
    fn get_alarm(&self) -> u32 {
        self.timer().cc[1].get()
    }
}


#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn TIMER0_Handler() {
    use common::Queue;

    nvic::disable(NvicIdx::TIMER0);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(NvicIdx::TIMER0);
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn TIMER1_Handler() {
    use common::Queue;

    nvic::disable(NvicIdx::TIMER1);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(NvicIdx::TIMER1);
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn TIMER2_Handler() {
    use common::Queue;

    nvic::disable(NvicIdx::TIMER2);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(NvicIdx::TIMER2);
}


