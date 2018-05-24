use kernel::common::cells::VolatileCell;

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
