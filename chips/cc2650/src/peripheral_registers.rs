use kernel::common::VolatileCell;

pub const IOC_BASE: usize = 0x4008_1000;

pub const GPIO_BASE: usize = 0x4002_2000;

#[repr(C)]
pub struct GPIO {
    _reserved0: [u8; 0x90],
    pub dout_set: VolatileCell<u32>,
    _reserved1: [u8; 0xC],
    pub dout_clr: VolatileCell<u32>,
    _reserved2: [u8; 0xC],
    pub dout_tgl: VolatileCell<u32>,
    _reserved3: [u8; 0xC],
    pub din: VolatileCell<u32>,
    _reserved4: [u8; 0xC],
    pub doe: VolatileCell<u32>,
    _reserved5: [u8; 0xC],
    pub evflags: VolatileCell<u32>,
}

pub const PRCM_BASE: usize = 0x4008_2000;

#[repr(C)]
pub struct PRCM {
    _reserved0: [VolatileCell<u8>; 0x28],

    // Write 1 in order to load settings
    pub clk_load_ctl: VolatileCell<u32>,

    _reserved1: [VolatileCell<u8>; 0x1C],

    pub gpio_clk_gate_run: VolatileCell<u32>,
    pub gpio_clk_gate_sleep: VolatileCell<u32>,
    pub gpio_clk_gate_deep_sleep: VolatileCell<u32>,

    _reserved2: [VolatileCell<u8>; 0xD8],

    // Power domain control 0
    pub pd_ctl0: VolatileCell<u32>,
    pub pd_ctl0_rfc: VolatileCell<u32>,
    pub pd_ctl0_serial: VolatileCell<u32>,
    pub pd_ctl0_peripheral: VolatileCell<u32>,

    _reserved3: [VolatileCell<u8>; 0x04],

    // Power domain status 0
    pub pd_stat0: VolatileCell<u32>,
    pub pd_stat0_rfc: VolatileCell<u32>,
    pub pd_stat0_serial: VolatileCell<u32>,
    pub pd_stat0_periph: VolatileCell<u32>,
}
