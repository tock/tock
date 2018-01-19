#[repr(C)]
struct PrcmRegisters {
    _r0: [VolatileCell<u8>; 0x28],

    // Write 1 in order to load settings
    clk_load_ctl: VolatileCell<u32>,

    _r1: [VolatileCell<u8>; 0x1C],

    gpio_clk_gate_run: VolatileCell<u32>,
    gpio_clk_gate_sleep: VolatileCell<u32>,
    gpio_clk_gate_deep_sleep: VolatileCell<u32>,

    _r2: [VolatileCell<u8>; 0xD8],

    // Power domain control 0
    pd_ctl0: VolatileCell<u32>,
    _pd_ctl0_rfc: VolatileCell<u32>,
    _pd_ctl0_serial: VolatileCell<u32>,
    _pd_ctl0_peripheral: VolatileCell<u32>,

    _r3: [VolatileCell<u8>; 0x04],

    // Power domain status 0
    _pd_stat0: VolatileCell<u32>,
    _pd_stat0_rfc: VolatileCell<u32>,
    _pd_stat0_serial: VolatileCell<u32>,
    pd_stat0_periph: VolatileCell<u32>,
}

const PRCM_BASE: usize = 0x4008_2000;
