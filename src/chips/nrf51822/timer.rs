use core::mem;
use common::VolatileCell;
use peripheral_interrupts::NvicIdx;
use nvic;
use chip;

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
    client: Option<&'static CompareClient>,
}

impl Timer {
    pub const fn new(location: Location, nvic: NvicIdx) -> Timer {
        Timer {
            which: location,
            nvic: nvic,
            client: None,
        }
    }

    fn timer(&self) -> &'static Registers {
        TIMER(self.which)
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
        self.client.map(|client| {
            let val = self.timer().event_compare[0].get() |
                      self.timer().event_compare[1].get() << 1 |
                      self.timer().event_compare[2].get() << 2 |
                      self.timer().event_compare[3].get() << 3;
            self.timer().event_compare[0].set(0);
            self.timer().event_compare[1].set(0);
            self.timer().event_compare[2].set(0);
            self.timer().event_compare[3].set(0);
            client.compare(val as u8);
        });
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


