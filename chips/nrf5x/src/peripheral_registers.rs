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
