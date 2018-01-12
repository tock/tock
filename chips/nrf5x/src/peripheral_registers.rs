use kernel::common::VolatileCell;

pub const RTC1_BASE: usize = 0x40011000;
#[repr(C)]
pub struct RTC1 {
    pub tasks_start: VolatileCell<u32>,
    pub tasks_stop: VolatileCell<u32>,
    pub tasks_clear: VolatileCell<u32>,
    pub tasks_trigovrflw: VolatileCell<u32>,
    _reserved1: [u32; 60],
    pub events_tick: VolatileCell<u32>,
    pub events_ovrflw: VolatileCell<u32>,
    _reserved2: [u32; 14],
    pub events_compare: [VolatileCell<u32>; 4],
    _reserved3: [u32; 109],
    pub intenset: VolatileCell<u32>,
    pub intenclr: VolatileCell<u32>,
    _reserved4: [u32; 13],
    pub evten: VolatileCell<u32>,
    pub evtenset: VolatileCell<u32>,
    pub evtenclr: VolatileCell<u32>,
    _reserved5: [u32; 110],
    pub counter: VolatileCell<u32>,
    pub prescaler: VolatileCell<u32>,
    _reserved6: [u32; 13],
    pub cc: [VolatileCell<u32>; 4],
}

// FIXME: check registers and add TIMER3 and TIMER4
pub const TIMER_SIZE: usize = 0x1000;
pub const TIMER_BASE: usize = 0x40008000;
#[repr(C)]
pub struct TIMER {
    pub task_start: VolatileCell<u32>,
    pub task_stop: VolatileCell<u32>,
    pub task_count: VolatileCell<u32>,
    pub task_clear: VolatileCell<u32>,
    pub task_shutdown: VolatileCell<u32>,
    _reserved0: [VolatileCell<u32>; 11],
    pub task_capture: [VolatileCell<u32>; 4], // 0x40
    _reserved1: [VolatileCell<u32>; 60],      // 0x140
    pub event_compare: [VolatileCell<u32>; 4],
    _reserved2: [VolatileCell<u32>; 44],  // 0x150
    pub shorts: VolatileCell<u32>,        // 0x200
    _reserved3: [VolatileCell<u32>; 64],  // 0x204
    pub intenset: VolatileCell<u32>,      // 0x304
    pub intenclr: VolatileCell<u32>,      // 0x308
    _reserved4: [VolatileCell<u32>; 126], // 0x30C
    pub mode: VolatileCell<u32>,          // 0x504
    pub bitmode: VolatileCell<u32>,       // 0x508
    _reserved5: VolatileCell<u32>,
    pub prescaler: VolatileCell<u32>,    // 0x510
    _reserved6: [VolatileCell<u32>; 11], // 0x514
    pub cc: [VolatileCell<u32>; 4],      // 0x540
}

pub const AESECB_BASE: usize = 0x4000E000;
#[repr(C)]
pub struct AESECB_REGS {
    pub task_startecb: VolatileCell<u32>,  // 0x000 - 0x004
    pub task_stopecb: VolatileCell<u32>,   // 0x004 - 0x008
    pub _reserved1: [u32; 62],             // 0x008 - 0x100
    pub event_endecb: VolatileCell<u32>,   // 0x100 - 0x104
    pub event_errorecb: VolatileCell<u32>, // 0x104 - 0x108
    pub _reserved2: [u32; 127],            // 0x108 - 0x304
    pub intenset: VolatileCell<u32>,       // 0x304 - 0x308
    pub intenclr: VolatileCell<u32>,       // 0x308 - 0x30c
    pub _reserved3: [u32; 126],            // 0x30c - 0x504
    pub ecbdataptr: VolatileCell<u32>,     // 0x504 - 0x508
}

pub const GPIO_BASE: usize = 0x50000000;
#[repr(C)]
pub struct GPIO {
    _reserved1: [u32; 321],
    pub out: VolatileCell<u32>,
    pub outset: VolatileCell<u32>,
    pub outclr: VolatileCell<u32>,
    pub in_: VolatileCell<u32>,
    pub dir: VolatileCell<u32>,
    pub dirset: VolatileCell<u32>,
    pub dirclr: VolatileCell<u32>,
    _reserved2: [u32; 120],
    pub pin_cnf: [VolatileCell<u32>; 32],
}

pub const TEMP_BASE: usize = 0x4000C000;
#[repr(C)]
pub struct TEMP_REGS {
    pub task_start: VolatileCell<u32>,    // 0x000 - 0x004
    pub task_stop: VolatileCell<u32>,     // 0x004 - 0x008
    pub _reserved1: [u32; 62],            // 0x008 - 0x100
    pub event_datardy: VolatileCell<u32>, // 0x100 - 0x104
    pub _reserved2: [u32; 127],           // 0x104 - 0x300
    pub inten: VolatileCell<u32>,         // 0x300 - 0x304
    pub intenset: VolatileCell<u32>,      // 0x304 - 0x308
    pub intenclr: VolatileCell<u32>,      // 0x308 - 0x30c
    pub _reserved3: [u32; 127],           // 0x30c - 0x508
    pub temp: VolatileCell<u32>,          // 0x508 - 0x50c
}

pub const RNG_BASE: usize = 0x4000D000;
#[repr(C)]
pub struct RNG_REGS {
    pub task_start: VolatileCell<u32>,   // 0x000 - 0x004
    pub task_stop: VolatileCell<u32>,    // 0x004 - 0x008
    pub _reserved1: [u32; 62],           // 0x008 - 0x100
    pub event_valrdy: VolatileCell<u32>, // 0x100 - 0x104
    pub _reserved2: [u32; 63],           // 0x104 - 0x200
    pub shorts: VolatileCell<u32>,       // 0x200 - 0x204
    pub _reserved3: [u32; 63],           // 0x204 - 0x300
    pub inten: VolatileCell<u32>,        // 0x300 - 0x304
    pub intenset: VolatileCell<u32>,     // 0x304 - 0x308
    pub intenclr: VolatileCell<u32>,     // 0x308 - 0x30c
    pub _reserved4: [u32; 126],          // 0x30c - 0x504
    pub config: VolatileCell<u32>,       // 0x504 - 0x508
    pub value: VolatileCell<u32>,        // 0x508 - 0x50c
}
